use crate::jobs::JobId;

use prisma_client_rust::QueryError;
use sd_core_prisma_helpers::job_without_data;

use sd_prisma::prisma::{job, PrismaClient};
use sd_utils::db::{maybe_missing, MissingFieldError};

use std::{
	collections::HashMap,
	fmt::{Display, Formatter},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum ReportError {
	#[error("failed to create job report in database: {0}")]
	Create(QueryError),
	#[error("failed to update job report in database: {0}")]
	Update(QueryError),
	#[error("invalid job status integer: {0}")]
	InvalidJobStatusInt(i32),
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct Report {
	pub id: JobId,
	pub name: String,
	pub action: Option<String>,
	pub data: Option<Vec<u8>>,
	// In Typescript `any | null` is just `any` so we don't get prompted for null checks
	// TODO(@Oscar): This will be fixed
	#[specta(type = Option<HashMap<String, serde_json::Value>>)]
	pub metadata: Option<serde_json::Value>,
	pub errors_text: Vec<String>,

	pub created_at: Option<DateTime<Utc>>,
	pub started_at: Option<DateTime<Utc>>,
	pub completed_at: Option<DateTime<Utc>>,

	pub parent_id: Option<JobId>,

	pub status: Status,
	pub task_count: i32,
	pub completed_task_count: i32,

	pub phase: String,
	pub message: String,
	pub estimated_completion: DateTime<Utc>,
}

impl Display for Report {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Job <name='{}', uuid='{}'> {:#?}",
			self.name, self.id, self.status
		)
	}
}

// convert database struct into a resource struct
impl TryFrom<job::Data> for Report {
	type Error = MissingFieldError;

	fn try_from(data: job::Data) -> Result<Self, Self::Error> {
		Ok(Self {
			id: JobId::from_slice(&data.id).expect("corrupted database"),
			name: maybe_missing(data.name, "job.name")?,
			action: data.action,
			data: data.data,
			metadata: data.metadata.and_then(|m| {
				serde_json::from_slice(&m).unwrap_or_else(|e| -> Option<serde_json::Value> {
					error!("Failed to deserialize job metadata: {}", e);
					None
				})
			}),
			errors_text: data
				.errors_text
				.map(|errors_str| errors_str.split("\n\n").map(str::to_string).collect())
				.unwrap_or_default(),
			created_at: data.date_created.map(DateTime::into),
			started_at: data.date_started.map(DateTime::into),
			completed_at: data.date_completed.map(DateTime::into),
			parent_id: data
				.parent_id
				.map(|id| JobId::from_slice(&id).expect("corrupted database")),
			status: Status::try_from(maybe_missing(data.status, "job.status")?)
				.expect("corrupted database"),
			task_count: data.task_count.unwrap_or(0),
			completed_task_count: data.completed_task_count.unwrap_or(0),
			phase: String::new(),
			message: String::new(),
			estimated_completion: data
				.date_estimated_completion
				.map_or(Utc::now(), DateTime::into),
		})
	}
}

// I despise having to write this twice, but it seems to be the only way to
// remove the data field from the struct
// would love to get this DRY'd up
impl TryFrom<job_without_data::Data> for Report {
	type Error = MissingFieldError;

	fn try_from(data: job_without_data::Data) -> Result<Self, Self::Error> {
		Ok(Self {
			id: JobId::from_slice(&data.id).expect("corrupted database"),
			name: maybe_missing(data.name, "job.name")?,
			action: data.action,
			data: None,
			metadata: data.metadata.and_then(|m| {
				serde_json::from_slice(&m).unwrap_or_else(|e| -> Option<serde_json::Value> {
					error!("Failed to deserialize job metadata: {}", e);
					None
				})
			}),
			errors_text: data
				.errors_text
				.map(|errors_str| errors_str.split("\n\n").map(str::to_string).collect())
				.unwrap_or_default(),
			created_at: data.date_created.map(DateTime::into),
			started_at: data.date_started.map(DateTime::into),
			completed_at: data.date_completed.map(DateTime::into),
			parent_id: data
				.parent_id
				.map(|id| JobId::from_slice(&id).expect("corrupted database")),
			status: Status::try_from(maybe_missing(data.status, "job.status")?)
				.expect("corrupted database"),
			task_count: data.task_count.unwrap_or(0),
			completed_task_count: data.completed_task_count.unwrap_or(0),

			phase: String::new(),
			message: String::new(),
			estimated_completion: data
				.date_estimated_completion
				.map_or(Utc::now(), DateTime::into),
		})
	}
}

impl Report {
	pub fn new(uuid: JobId, name: String) -> Self {
		Self {
			id: uuid,
			name,
			action: None,
			created_at: None,
			started_at: None,
			completed_at: None,
			status: Status::Queued,
			errors_text: vec![],
			task_count: 0,
			data: None,
			metadata: None,
			parent_id: None,
			completed_task_count: 0,
			phase: String::new(),
			message: String::new(),
			estimated_completion: Utc::now(),
		}
	}

	pub fn get_meta(&self) -> (String, Option<String>) {
		// actions are formatted like "added_location" or "added_location-1"
		let Some(action_name) = self.action.as_ref().map(|action| {
			action
				.split('-')
				.next()
				.map(str::to_string)
				.unwrap_or_default()
		}) else {
			return (self.id.to_string(), None);
		};
		// create a unique group_key, EG: "added_location-<location_id>"
		let group_key = self.parent_id.map_or_else(
			|| format!("{}-{}", action_name, &self.id),
			|parent_id| format!("{}-{}", action_name, parent_id),
		);

		(action_name, Some(group_key))
	}

	pub async fn create(&mut self, db: &PrismaClient) -> Result<(), ReportError> {
		let now = Utc::now();

		db.job()
			.create(
				self.id.as_bytes().to_vec(),
				sd_utils::chain_optional_iter(
					[
						job::name::set(Some(self.name.clone())),
						job::action::set(self.action.clone()),
						job::data::set(self.data.clone()),
						job::date_created::set(Some(now.into())),
						job::status::set(Some(self.status as i32)),
						job::date_started::set(self.started_at.map(|d| d.into())),
						job::task_count::set(Some(1)),
						job::completed_task_count::set(Some(0)),
					],
					[self
						.parent_id
						.map(|id| job::parent::connect(job::id::equals(id.as_bytes().to_vec())))],
				),
			)
			.exec()
			.await
			.map_err(ReportError::Create)?;

		// Only setting created_at after we successfully created the job in DB
		self.created_at = Some(now);

		Ok(())
	}

	pub async fn update(&mut self, db: &PrismaClient) -> Result<(), ReportError> {
		db.job()
			.update(
				job::id::equals(self.id.as_bytes().to_vec()),
				vec![
					job::status::set(Some(self.status as i32)),
					job::errors_text::set(
						(!self.errors_text.is_empty()).then(|| self.errors_text.join("\n\n")),
					),
					job::data::set(self.data.clone()),
					job::metadata::set(serde_json::to_vec(&self.metadata).ok()),
					job::task_count::set(Some(self.task_count)),
					job::completed_task_count::set(Some(self.completed_task_count)),
					job::date_started::set(self.started_at.map(Into::into)),
					job::date_completed::set(self.completed_at.map(Into::into)),
				],
			)
			.exec()
			.await
			.map_err(ReportError::Update)?;

		Ok(())
	}
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Eq, PartialEq)]
pub enum Status {
	Queued = 0,
	Running = 1,
	Completed = 2,
	Canceled = 3,
	Failed = 4,
	Paused = 5,
	CompletedWithErrors = 6,
}

impl Status {
	pub fn is_finished(self) -> bool {
		matches!(
			self,
			Self::Completed
				| Self::Canceled | Self::Paused
				| Self::Failed | Self::CompletedWithErrors
		)
	}
}

impl TryFrom<i32> for Status {
	type Error = ReportError;

	fn try_from(value: i32) -> Result<Self, Self::Error> {
		let s = match value {
			0 => Self::Queued,
			1 => Self::Running,
			2 => Self::Completed,
			3 => Self::Canceled,
			4 => Self::Failed,
			5 => Self::Paused,
			6 => Self::CompletedWithErrors,
			_ => return Err(Self::Error::InvalidJobStatusInt(value)),
		};

		Ok(s)
	}
}

pub struct ReportBuilder {
	pub id: JobId,
	pub name: String,
	pub action: Option<String>,
	pub metadata: Option<serde_json::Value>,
	pub parent_id: Option<JobId>,
}

impl ReportBuilder {
	pub fn build(self) -> Report {
		Report {
			id: self.id,
			name: self.name,
			action: self.action,
			created_at: None,
			started_at: None,
			completed_at: None,
			status: Status::Queued,
			errors_text: vec![],
			task_count: 0,
			data: None,
			metadata: self.metadata,
			parent_id: self.parent_id,
			completed_task_count: 0,
			phase: String::new(),
			message: String::new(),
			estimated_completion: Utc::now(),
		}
	}

	pub fn new(id: JobId, name: impl Into<String>) -> Self {
		Self {
			id,
			name: name.into(),
			action: None,
			metadata: None,
			parent_id: None,
		}
	}

	pub fn with_action(mut self, action: impl Into<String>) -> Self {
		self.action = Some(action.into());
		self
	}

	pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
		self.metadata = Some(metadata);
		self
	}

	pub fn with_parent_id(mut self, parent_id: JobId) -> Self {
		self.parent_id = Some(parent_id);
		self
	}
}
