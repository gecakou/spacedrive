use crate::{
	invalidate_query,
	job::{job_without_data, Job, JobReport, JobStatus, Jobs},
	library::Library,
	location::{find_location, LocationError},
	object::{
		file_identifier::file_identifier_job::FileIdentifierJobInit, media::MediaProcessorJobInit,
		validation::validator_job::ObjectValidatorJobInit,
	},
	prisma::{job, location, SortOrder},
};

use std::{
	collections::{hash_map::Entry, BTreeMap, HashMap, VecDeque},
	path::PathBuf,
	time::Instant,
};

use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use prisma_client_rust::or;
use rspc::alpha::AlphaRouter;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::time::Duration;
use tracing::{info, trace};
use uuid::Uuid;

use super::{utils::library, CoreEvent, Ctx, R};

pub(crate) fn mount() -> AlphaRouter<Ctx> {
	R.router()
		.procedure("progress", {
			// Listen for updates from the job manager
			// - the client listens for events containing an updated JobReport
			// - the client replaces its local copy of the JobReport using the index provided by the reports procedure
			// - this should be used with the ephemeral sync engine
			R.with2(library())
				.subscription(|(node, _), _: ()| async move {
					let mut event_bus_rx = node.event_bus.0.subscribe();
					// debounce per-job
					let mut intervals = BTreeMap::<Uuid, Instant>::new();

					async_stream::stream! {
						loop {
							let progress_event = loop {
								if let Ok(CoreEvent::JobProgress(progress_event)) = event_bus_rx.recv().await {
									break progress_event;
								}
							};

							let instant = intervals.entry(progress_event.id).or_insert_with(
								Instant::now
							);

							if instant.elapsed() <= Duration::from_secs_f64(1.0 / 30.0) {
								continue;
							}

							yield progress_event;

							*instant = Instant::now();
						}
					}
				})
		})
		.procedure("reports", {
			// Reports provides the client with a list of JobReports
			// - we query with a custom select! to avoid returning paused job cache `job.data`
			// - results must include running jobs, and be combined with the in-memory state
			//	  this is to ensure the client will always get the correct initial state
			// - jobs are sorted in to groups by their action
			// - TODO: refactor grouping system to a many-to-many table
			#[derive(Debug, Clone, Serialize, Deserialize, Type)]
			pub struct JobGroup {
				id: Uuid,
				action: Option<String>,
				status: JobStatus,
				created_at: DateTime<Utc>,
				jobs: VecDeque<JobReport>,
			}

			async fn group_jobs_by_library(
				library: &Library,
				active_job_reports_by_id: &HashMap<Uuid, JobReport>,
			) -> Result<Vec<JobGroup>, rspc::Error> {
				let mut groups: HashMap<String, JobGroup> = HashMap::new();

				let job_reports: Vec<JobReport> = library
					.db
					.job()
					.find_many(vec![])
					.order_by(job::date_created::order(SortOrder::Desc))
					.take(100)
					.select(job_without_data::select())
					.exec()
					.await?
					.into_iter()
					.flat_map(JobReport::try_from)
					.collect();

				for job in job_reports {
					// action name and group key are computed from the job data
					let (action_name, group_key) = job.get_meta();

					trace!(
						"job {:#?}, action_name {}, group_key {:?}",
						job,
						action_name,
						group_key
					);

					// if the job is running, use the in-memory report
					let report = active_job_reports_by_id.get(&job.id).unwrap_or(&job);

					// if we have a group key, handle grouping
					if let Some(group_key) = group_key {
						match groups.entry(group_key) {
							// Create new job group with metadata
							Entry::Vacant(entry) => {
								entry.insert(JobGroup {
									id: job.parent_id.unwrap_or(job.id),
									action: Some(action_name.clone()),
									status: job.status,
									jobs: [report.clone()].into_iter().collect(),
									created_at: job.created_at.unwrap_or(Utc::now()),
								});
							}
							// Add to existing job group
							Entry::Occupied(mut entry) => {
								let group = entry.get_mut();

								// protect paused status from being overwritten
								if report.status != JobStatus::Paused {
									group.status = report.status;
								}

								// if group.status.is_finished() && !report.status.is_finished() {
								// }
								group.jobs.push_front(report.clone());
							}
						}
					} else {
						// insert individual job as group
						groups.insert(
							job.id.to_string(),
							JobGroup {
								id: job.id,
								action: None,
								status: job.status,
								jobs: [report.clone()].into_iter().collect(),
								created_at: job.created_at.unwrap_or(Utc::now()),
							},
						);
					}
				}

				let mut groups_vec = groups.into_values().collect::<Vec<_>>();
				groups_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));

				Ok(groups_vec)
			}

			R.query(|node, _: ()| async move {
				// WARN: We really need the borrow in this line, this way each async move in the map below
				// received a copy of the reference to the active job reports,
				// I feel like I'm conquering the borrow checker
				let active_job_reports_by_id = &node.jobs.get_active_reports_with_id().await;

				try_join_all(
					node.libraries
						.get_all()
						.await
						.into_iter()
						.map(|library| async move {
							group_jobs_by_library(&library, active_job_reports_by_id)
								.await
								.map(|groups| (library.id, groups))
						}),
				)
				.await
				.map(|groups_by_library_id| {
					groups_by_library_id.into_iter().collect::<HashMap<_, _>>()
				})
			})
		})
		.procedure("isActive", {
			R.with2(library())
				.query(|(node, _), _: ()| async move { Ok(node.jobs.has_active_workers().await) })
		})
		.procedure("clear", {
			R.with2(library())
				.mutation(|(_, library), id: Uuid| async move {
					library
						.db
						.job()
						.delete(job::id::equals(id.as_bytes().to_vec()))
						.exec()
						.await?;

					invalidate_query!(library, "jobs.reports");
					Ok(())
				})
		})
		.procedure("clearAll", {
			R.with2(library())
				.mutation(|(_, library), _: ()| async move {
					info!("Clearing all jobs");
					library
						.db
						.job()
						.delete_many(vec![or![
							job::status::equals(Some(JobStatus::Canceled as i32)),
							job::status::equals(Some(JobStatus::Failed as i32)),
							job::status::equals(Some(JobStatus::Completed as i32)),
							job::status::equals(Some(JobStatus::CompletedWithErrors as i32)),
						]])
						.exec()
						.await?;

					invalidate_query!(library, "jobs.reports");
					Ok(())
				})
		})
		// pause job
		.procedure("pause", {
			R.with2(library())
				.mutation(|(node, library), id: Uuid| async move {
					let ret = Jobs::pause(&node.jobs, id).await.map_err(Into::into);
					invalidate_query!(library, "jobs.reports");
					ret
				})
		})
		.procedure("resume", {
			R.with2(library())
				.mutation(|(node, library), id: Uuid| async move {
					let ret = Jobs::resume(&node.jobs, id).await.map_err(Into::into);
					invalidate_query!(library, "jobs.reports");
					ret
				})
		})
		.procedure("cancel", {
			R.with2(library())
				.mutation(|(node, library), id: Uuid| async move {
					let ret = Jobs::cancel(&node.jobs, id).await.map_err(Into::into);
					invalidate_query!(library, "jobs.reports");
					ret
				})
		})
		.procedure("generateThumbsForLocation", {
			#[derive(Type, Deserialize)]
			pub struct GenerateThumbsForLocationArgs {
				pub id: location::id::Type,
				pub path: PathBuf,
				#[serde(default)]
				pub regenerate: bool,
			}

			R.with2(library()).mutation(
				|(node, library),
				 GenerateThumbsForLocationArgs {
				     id,
				     path,
				     regenerate,
				 }: GenerateThumbsForLocationArgs| async move {
					let Some(location) = find_location(&library, id).exec().await? else {
						return Err(LocationError::IdNotFound(id).into());
					};

					Job::new(MediaProcessorJobInit {
						location,
						sub_path: Some(path),
						regenerate_thumbnails: regenerate,
					})
					.spawn(&node, &library)
					.await
					.map_err(Into::into)
				},
			)
		})
		.procedure("objectValidator", {
			#[derive(Type, Deserialize)]
			pub struct ObjectValidatorArgs {
				pub id: location::id::Type,
				pub path: PathBuf,
			}

			R.with2(library())
				.mutation(|(node, library), args: ObjectValidatorArgs| async move {
					let Some(location) = find_location(&library, args.id).exec().await? else {
						return Err(LocationError::IdNotFound(args.id).into());
					};

					Job::new(ObjectValidatorJobInit {
						location,
						sub_path: Some(args.path),
					})
					.spawn(&node, &library)
					.await
					.map_err(Into::into)
				})
		})
		.procedure("identifyUniqueFiles", {
			#[derive(Type, Deserialize)]
			pub struct IdentifyUniqueFilesArgs {
				pub id: location::id::Type,
				pub path: PathBuf,
			}

			R.with2(library()).mutation(
				|(node, library), args: IdentifyUniqueFilesArgs| async move {
					let Some(location) = find_location(&library, args.id).exec().await? else {
						return Err(LocationError::IdNotFound(args.id).into());
					};

					Job::new(FileIdentifierJobInit {
						location,
						sub_path: Some(args.path),
					})
					.spawn(&node, &library)
					.await
					.map_err(Into::into)
				},
			)
		})
		.procedure("newThumbnail", {
			R.with2(library())
				.subscription(|(node, _), _: ()| async move {
					// TODO: Only return event for the library that was subscribed to

					let mut event_bus_rx = node.event_bus.0.subscribe();
					async_stream::stream! {
						while let Ok(event) = event_bus_rx.recv().await {
							match event {
								CoreEvent::NewThumbnail { thumb_key } => yield thumb_key,
								_ => {}
							}
						}
					}
				})
		})
}
