use crate::{
  file::indexer::IndexerJob, prisma::location, state::client, ClientQuery, CoreContext, CoreEvent,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, io, io::Write, path::Path};
use thiserror::Error;
use ts_rs::TS;

use super::SysError;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LocationResource {
  pub id: i32,
  pub name: Option<String>,
  pub path: Option<String>,
  pub total_capacity: Option<i32>,
  pub available_capacity: Option<i32>,
  pub is_removable: Option<bool>,
  pub is_online: bool,
  #[ts(type = "string")]
  pub date_created: chrono::DateTime<chrono::Utc>,
}

impl Into<LocationResource> for location::Data {
  fn into(self) -> LocationResource {
    LocationResource {
      id: self.id,
      name: self.name,
      path: self.local_path,
      total_capacity: self.total_capacity,
      available_capacity: self.available_capacity,
      is_removable: self.is_removable,
      is_online: self.is_online,
      date_created: self.date_created,
    }
  }
}

#[derive(Serialize, Deserialize, Default)]
pub struct DotSpacedrive {
  pub location_uuid: String,
  pub library_uuid: String,
}

static DOTFILE_NAME: &str = ".spacedrive";

// checks to see if a location is:
// - accessible on from the local filesystem
// - already exists in the database
pub async fn check_location(path: &str) -> Result<DotSpacedrive, LocationError> {
  let dotfile: DotSpacedrive = match fs::File::open(format!("{}/{}", path.clone(), DOTFILE_NAME)) {
    Ok(file) => serde_json::from_reader(file).unwrap_or(DotSpacedrive::default()),
    Err(e) => return Err(LocationError::DotfileReadFailure(e)),
  };

  Ok(dotfile)
}

pub async fn get_location(
  ctx: &CoreContext,
  location_id: i32,
) -> Result<LocationResource, SysError> {
  let db = &ctx.database;

  // get location by location_id from db and include location_paths
  let location = match db
    .location()
    .find_unique(location::id::equals(location_id))
    .exec()
    .await?
  {
    Some(location) => location,
    None => Err(LocationError::NotFound(location_id.to_string()))?,
  };

  println!("Retrieved location: {:?}", location);

  Ok(location.into())
}

pub async fn new_location_and_scan(
  ctx: &CoreContext,
  path: &str,
) -> Result<LocationResource, SysError> {
  let location = create_location(&ctx, path).await?;

  ctx.spawn_job(Box::new(IndexerJob {
    path: path.to_string(),
  }));

  Ok(location)
}

pub async fn get_locations(ctx: &CoreContext) -> Result<Vec<LocationResource>, SysError> {
  let db = &ctx.database;

  let locations = db.location().find_many(vec![]).exec().await?;

  // turn locations into LocationResource
  let locations: Vec<LocationResource> = locations
    .into_iter()
    .map(|location| location.into())
    .collect();

  Ok(locations)
}

pub async fn create_location(ctx: &CoreContext, path: &str) -> Result<LocationResource, SysError> {
  let db = &ctx.database;
  let config = client::get();

  // check if we have access to this location
  match fs::File::open(&path) {
    Ok(_) => println!("Path is valid, creating location for '{}'", &path),
    Err(e) => Err(LocationError::FileReadError(e))?,
  }
  // check if location already exists
  let location = match db
    .location()
    .find_first(vec![location::local_path::equals(Some(path.to_string()))])
    .exec()
    .await?
  {
    Some(location) => location,
    None => {
      println!(
        "Location does not exist, creating new location for '{}'",
        &path
      );
      let uuid = uuid::Uuid::new_v4();

      let p = Path::new(&path);

      let location = db
        .location()
        .create(
          location::pub_id::set(uuid.to_string()),
          vec![
            location::name::set(Some(p.file_name().unwrap().to_string_lossy().to_string())),
            location::is_online::set(true),
            location::local_path::set(Some(path.to_string())),
          ],
        )
        .exec()
        .await?;

      println!("Created location: {:?}", location);

      // write a file called .spacedrive to path containing the location id in JSON format
      let mut dotfile = match fs::File::create(format!("{}/{}", path.clone(), DOTFILE_NAME)) {
        Ok(file) => file,
        Err(e) => Err(LocationError::DotfileWriteFailure(e, path.to_string()))?,
      };

      let data = DotSpacedrive {
        location_uuid: uuid.to_string(),
        library_uuid: config.current_library_uuid,
      };

      let json = match serde_json::to_string(&data) {
        Ok(json) => json,
        Err(e) => Err(LocationError::DotfileSerializeFailure(e, path.to_string()))?,
      };

      match dotfile.write_all(json.as_bytes()) {
        Ok(_) => (),
        Err(e) => Err(LocationError::DotfileWriteFailure(e, path.to_string()))?,
      }

      ctx
        .emit(CoreEvent::InvalidateQuery(ClientQuery::SysGetLocations))
        .await;

      location
    }
  };

  Ok(location.into())
}

#[derive(Error, Debug)]
pub enum LocationError {
  #[error("Failed to create location (uuid {uuid:?})")]
  CreateFailure { uuid: String },
  #[error("Failed to read location dotfile")]
  DotfileReadFailure(io::Error),
  #[error("Failed to serialize dotfile for location (at path: {1:?})")]
  DotfileSerializeFailure(serde_json::Error, String),
  #[error("Location not found (uuid: {1:?})")]
  DotfileWriteFailure(io::Error, String),
  #[error("Location not found (uuid: {0:?})")]
  NotFound(String),
  #[error("Failed to open file from local os")]
  FileReadError(io::Error),
  #[error("Failed to read mounted volumes from local os")]
  VolumeReadError(String),
  #[error("Failed to connect to database (error: {0:?})")]
  IOError(io::Error),
}
