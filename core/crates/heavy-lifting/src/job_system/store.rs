use crate::indexer::IndexerJob;

use sd_prisma::prisma::{job, location};
use sd_utils::uuid_to_bytes;

use std::{
	collections::{HashMap, VecDeque},
	future::Future,
	iter,
	marker::PhantomData,
};

use futures_concurrency::future::TryJoin;
use serde::{Deserialize, Serialize};

use super::{
	job::{DynJob, Job, JobContext, JobHolder, JobName},
	report::{Report, ReportError},
	JobId, JobSystemError,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedTasks(pub Vec<u8>);

pub trait SerializableJob: 'static
where
	Self: Sized,
{
	fn serialize(
		self,
	) -> impl Future<Output = Result<Option<Vec<u8>>, rmp_serde::encode::Error>> + Send {
		async move { Ok(None) }
	}

	#[allow(unused_variables)]
	fn deserialize(
		serialized_job: &[u8],
		ctx: &impl JobContext,
	) -> impl Future<
		Output = Result<Option<(Self, Option<SerializedTasks>)>, rmp_serde::decode::Error>,
	> + Send {
		async move { Ok(None) }
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredJob {
	pub(super) id: JobId,
	pub(super) name: JobName,
	pub(super) serialized_job: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredJobEntry {
	pub(super) location_id: location::id::Type,
	pub(super) root_job: StoredJob,
	pub(super) next_jobs: Vec<StoredJob>,
}

pub async fn load_jobs<Ctx: JobContext>(
	entries: Vec<StoredJobEntry>,
	job_ctx: &Ctx,
) -> Result<
	Vec<(
		location::id::Type,
		Box<dyn DynJob<Ctx>>,
		Option<SerializedTasks>,
	)>,
	JobSystemError,
> {
	let mut reports = job_ctx
		.db()
		.job()
		.find_many(vec![job::id::in_vec(
			entries
				.iter()
				.flat_map(
					|StoredJobEntry {
					     root_job: StoredJob { id, .. },
					     next_jobs,
					     ..
					 }| { iter::once(*id).chain(next_jobs.iter().map(|StoredJob { id, .. }| *id)) },
				)
				.map(uuid_to_bytes)
				.collect::<Vec<_>>(),
		)])
		.exec()
		.await
		.map_err(JobSystemError::LoadReportsForResume)?
		.into_iter()
		.map(Report::try_from)
		.map(|report_res| report_res.map(|report| (report.id, report)))
		.collect::<Result<HashMap<_, _>, _>>()?;

	entries
		.into_iter()
		.map(
			|StoredJobEntry {
			     location_id,
			     root_job,
			     next_jobs,
			 }| {
				let report = reports
					.remove(&root_job.id)
					.ok_or(ReportError::MissingReport(root_job.id))?;

				Ok(async move {
					load_job(root_job, report, job_ctx)
						.await
						.map(|maybe_loaded_job| {
							maybe_loaded_job
								.map(|(dyn_job, tasks)| (location_id, dyn_job, tasks, next_jobs))
						})
				})
			},
		)
		.collect::<Result<Vec<_>, JobSystemError>>()?
		.try_join()
		.await?
		.into_iter()
		.flatten()
		.map(|(location_id, mut dyn_job, tasks, next_jobs)| {
			let next_jobs_and_reports = next_jobs
				.into_iter()
				.map(|next_job| {
					let next_job_id = next_job.id;
					reports
						.remove(&next_job.id)
						.map(|report| (next_job, report))
						.ok_or(ReportError::MissingReport(next_job_id))
				})
				.collect::<Result<Vec<_>, _>>()?;

			Ok(async move {
				next_jobs_and_reports
					.into_iter()
					.map(|(next_job, report)| async move {
						load_job(next_job, report, job_ctx)
							.await
							.map(|maybe_loaded_next_job| {
								maybe_loaded_next_job.map(|(next_dyn_job, next_tasks)| {
									assert!(
										next_tasks.is_none(),
										"Next jobs must not have tasks as they haven't run yet"
									);
									assert!(
										next_dyn_job.next_jobs().is_empty(),
										"Next jobs must not have next jobs"
									);
									next_dyn_job
								})
							})
					})
					.collect::<Vec<_>>()
					.try_join()
					.await
					.map(|maybe_next_dyn_jobs| {
						dyn_job.set_next_jobs(maybe_next_dyn_jobs.into_iter().flatten().collect());
						(location_id, dyn_job, tasks)
					})
			})
		})
		.collect::<Result<Vec<_>, JobSystemError>>()?
		.try_join()
		.await
}

macro_rules! match_deserialize_job {
	($stored_job:ident, $report:ident, $job_ctx:ident, $ctx_type:ty, [$($job_type:ty),+ $(,)?]) => {{
		let StoredJob {
			id,
			name,
			serialized_job,
		} = $stored_job;


		match name {
			$(<$job_type as Job>::NAME => <$job_type as SerializableJob>::deserialize(
					&serialized_job,
					$job_ctx,
				).await
					.map(|maybe_job| maybe_job.map(|(job, tasks)| -> (
							Box<dyn DynJob<$ctx_type>>,
							Option<SerializedTasks>
						) {
							(
								Box::new(JobHolder {
									id,
									job,
									report: $report,
									next_jobs: VecDeque::new(),
									_ctx: PhantomData,
								}),
								tasks,
							)
						}
					))
					.map_err(Into::into),)+
		}
	}};
}

async fn load_job<Ctx: JobContext>(
	stored_job: StoredJob,
	report: Report,
	job_ctx: &Ctx,
) -> Result<Option<(Box<dyn DynJob<Ctx>>, Option<SerializedTasks>)>, JobSystemError> {
	match_deserialize_job!(
		stored_job,
		report,
		job_ctx,
		Ctx,
		[
			IndexerJob,
			// TODO: Add more jobs here
			// e.g.: FileIdentifierJob, MediaProcessorJob, etc.,
		]
	)
}
