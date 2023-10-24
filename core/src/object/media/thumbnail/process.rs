use crate::{api::CoreEvent, util::error::FileIOError};

use sd_file_ext::extensions::{DocumentExtension, ImageExtension};
use sd_images::{format_image, scale_dimensions};
use sd_media_metadata::image::Orientation;
use sd_prisma::prisma::location;

use std::{
	collections::VecDeque,
	ffi::OsString,
	ops::Deref,
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
};

use async_channel as chan;
use futures_concurrency::future::{Join, Race};
use image::{self, imageops, DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use tokio::{
	fs, io,
	sync::{broadcast, oneshot},
	task::{spawn, spawn_blocking},
	time::timeout,
};
use tracing::{error, trace, warn};
use webp::Encoder;

use super::{
	can_generate_thumbnail_for_document, can_generate_thumbnail_for_image, get_thumb_key,
	shard::get_shard_hex, ThumbnailKind, ThumbnailerError, EPHEMERAL_DIR, TARGET_PX,
	TARGET_QUALITY, THIRTY_SECS, WEBP_EXTENSION,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateThumbnailArgs {
	pub extension: String,
	pub cas_id: String,
	pub path: PathBuf,
}

impl GenerateThumbnailArgs {
	pub fn new(extension: String, cas_id: String, path: PathBuf) -> Self {
		Self {
			extension,
			cas_id,
			path,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchToProcess {
	pub(super) batch: Vec<GenerateThumbnailArgs>,
	pub(super) should_regenerate: bool,
	pub(super) in_background: bool,
	pub(super) location_id: Option<location::id::Type>,
}

impl BatchToProcess {
	pub fn new(
		batch: Vec<GenerateThumbnailArgs>,
		should_regenerate: bool,
		in_background: bool,
	) -> Self {
		Self {
			batch,
			should_regenerate,
			in_background,
			location_id: None,
		}
	}
}

pub(super) struct ProcessorControlChannels {
	pub stop_rx: chan::Receiver<oneshot::Sender<()>>,
	pub done_tx: oneshot::Sender<()>,
	pub batch_report_progress_tx: chan::Sender<(location::id::Type, u32)>,
}

pub(super) async fn batch_processor(
	thumbnails_directory: Arc<PathBuf>,
	(
		BatchToProcess {
			batch,
			should_regenerate,
			in_background,
			location_id,
		},
		kind,
	): (BatchToProcess, ThumbnailKind),
	generated_ephemeral_thumbs_file_names_tx: chan::Sender<Vec<OsString>>,
	ProcessorControlChannels {
		stop_rx,
		done_tx,
		batch_report_progress_tx,
	}: ProcessorControlChannels,
	leftovers_tx: chan::Sender<(BatchToProcess, ThumbnailKind)>,
	reporter: broadcast::Sender<CoreEvent>,
	batch_size: usize,
) {
	trace!(
		"Processing thumbnails batch of kind {kind:?} with size {} in {}",
		batch.len(),
		if in_background {
			"background"
		} else {
			"foreground"
		},
	);

	// Tranforming to `VecDeque` so we don't need to move anything as we consume from the beginning
	// This from is guaranteed to be O(1)
	let mut queue = VecDeque::from(batch);

	enum RaceOutputs {
		Processed,
		Stop(oneshot::Sender<()>),
	}

	while !queue.is_empty() {
		let chunk = (0..batch_size)
			.filter_map(|_| queue.pop_front())
			.map(
				|GenerateThumbnailArgs {
				     extension,
				     cas_id,
				     path,
				 }| {
					let reporter = reporter.clone();
					let thumbnails_directory = thumbnails_directory.as_ref().clone();
					spawn(async move {
						timeout(
							THIRTY_SECS,
							generate_thumbnail(
								thumbnails_directory,
								ThumbData {
									extension: &extension,
									cas_id,
									path: &path,
									in_background,
									should_regenerate,
									kind,
								},
								reporter,
							),
						)
						.await
						.unwrap_or_else(|_| Err(ThumbnailerError::TimedOut(path.into_boxed_path())))
					})
				},
			)
			.collect::<Vec<_>>();

		let generated_ephemeral_thumbs_file_names_tx =
			generated_ephemeral_thumbs_file_names_tx.clone();
		let report_progress_tx = batch_report_progress_tx.clone();
		let chunk_len = chunk.len() as u32;

		if let RaceOutputs::Stop(stopped_tx) = (
			async move {
				if let Err(e) = spawn(async move {
					let cas_ids = chunk
						.join()
						.await
						.into_iter()
						.filter_map(|join_result| {
							join_result
								.map_err(|e| {
									error!("Failed to join thumbnail generation task: {e:#?}")
								})
								.ok()
						})
						.filter_map(|result| {
							result
								.map_err(|e| {
									error!(
										"Failed to generate thumbnail for {} location: {e:#?}",
										if let ThumbnailKind::Ephemeral = kind {
											"ephemeral"
										} else {
											"indexed"
										}
									)
								})
								.ok()
						})
						.map(|cas_id| OsString::from(format!("{}.webp", cas_id)))
						.collect();

					if kind == ThumbnailKind::Ephemeral
						&& generated_ephemeral_thumbs_file_names_tx
							.send(cas_ids)
							.await
							.is_err()
					{
						error!("Thumbnail actor is dead: Failed to send generated cas ids")
					}

					if let Some(location_id) = location_id {
						report_progress_tx.send((location_id, chunk_len)).await.ok();
					}
				})
				.await
				{
					error!("Failed to join spawned task to process thumbnails chunk on a batch: {e:#?}");
				}

				trace!("Processed chunk with {chunk_len} thumbnails");
				RaceOutputs::Processed
			},
			async {
				let tx = stop_rx
					.recv()
					.await
					.expect("Critical error on thumbnails actor");
				trace!("Received a stop signal");
				RaceOutputs::Stop(tx)
			},
		)
			.race()
			.await
		{
			// Our queue is always contiguous, so this `from` is free
			let leftovers = Vec::from(queue);

			trace!(
				"Stopped with {} thumbnails left to process",
				leftovers.len()
			);
			if !leftovers.is_empty()
				&& leftovers_tx
					.send((
						BatchToProcess {
							batch: leftovers,
							should_regenerate,
							in_background: true, // Leftovers should always be in background
							location_id,
						},
						kind,
					))
					.await
					.is_err()
			{
				error!("Thumbnail actor is dead: Failed to send leftovers")
			}

			done_tx.send(()).ok();
			stopped_tx.send(()).ok();

			return;
		}
	}

	trace!("Finished batch!");

	done_tx.send(()).ok();
}

pub(super) struct ThumbData<'ext, P: AsRef<Path>> {
	pub extension: &'ext str,
	pub cas_id: String,
	pub path: P,
	pub in_background: bool,
	pub should_regenerate: bool,
	pub kind: ThumbnailKind,
}

pub(super) async fn generate_thumbnail(
	thumbnails_directory: PathBuf,
	ThumbData {
		extension,
		cas_id,
		path,
		in_background,
		should_regenerate,
		kind,
	}: ThumbData<'_, impl AsRef<Path>>,
	reporter: broadcast::Sender<CoreEvent>,
) -> Result<String, ThumbnailerError> {
	let path = path.as_ref();
	trace!("Generating thumbnail for {}", path.display());

	let mut output_path = thumbnails_directory;
	match kind {
		ThumbnailKind::Ephemeral => output_path.push(EPHEMERAL_DIR),
		ThumbnailKind::Indexed(library_id) => output_path.push(library_id.to_string()),
	};
	output_path.push(get_shard_hex(&cas_id));
	output_path.push(&cas_id);
	output_path.set_extension(WEBP_EXTENSION);

	if let Err(e) = fs::metadata(&output_path).await {
		if e.kind() != io::ErrorKind::NotFound {
			error!(
				"Failed to check if thumbnail exists, but we will try to generate it anyway: {e:#?}"
			);
		}
	// Otherwise we good, thumbnail doesn't exist so we can generate it
	} else if !should_regenerate {
		trace!(
			"Skipping thumbnail generation for {} because it already exists",
			path.display()
		);
		return Ok(cas_id);
	}

	if let Ok(extension) = ImageExtension::from_str(extension) {
		if can_generate_thumbnail_for_image(&extension) {
			generate_image_thumbnail(&path, &output_path).await?;
		}
	} else if let Ok(extension) = DocumentExtension::from_str(extension) {
		if can_generate_thumbnail_for_document(&extension) {
			generate_image_thumbnail(&path, &output_path).await?;
		}
	}

	#[cfg(feature = "ffmpeg")]
	{
		use crate::object::media::thumbnail::can_generate_thumbnail_for_video;
		use sd_file_ext::extensions::VideoExtension;

		if let Ok(extension) = VideoExtension::from_str(extension) {
			if can_generate_thumbnail_for_video(&extension) {
				generate_video_thumbnail(&path, &output_path).await?;
			}
		}
	}

	if !in_background {
		trace!("Emitting new thumbnail event");
		if reporter
			.send(CoreEvent::NewThumbnail {
				thumb_key: get_thumb_key(&cas_id, kind),
			})
			.is_err()
		{
			warn!("Error sending event to Node's event bus");
		}
	}

	trace!("Generated thumbnail for {}", path.display());

	Ok(cas_id)
}

async fn generate_image_thumbnail(
	file_path: impl AsRef<Path>,
	output_path: impl AsRef<Path>,
) -> Result<(), ThumbnailerError> {
	let file_path = file_path.as_ref().to_path_buf();

	let webp = spawn_blocking(move || -> Result<_, ThumbnailerError> {
		let img = format_image(&file_path).map_err(|e| ThumbnailerError::SdImages {
			path: file_path.clone().into_boxed_path(),
			error: e,
		})?;

		let (w, h) = img.dimensions();
		let (w_scaled, h_scaled) = scale_dimensions(w as f32, h as f32, TARGET_PX);

		// Optionally, resize the existing photo and convert back into DynamicImage
		let mut img = DynamicImage::ImageRgba8(imageops::resize(
			&img,
			w_scaled as u32,
			h_scaled as u32,
			imageops::FilterType::Triangle,
		));

		// this corrects the rotation/flip of the image based on the *available* exif data
		// not all images have exif data, so we don't error
		if let Some(orientation) = Orientation::from_path(&file_path) {
			img = orientation.correct_thumbnail(img);
		}

		// Create the WebP encoder for the above image
		let encoder =
			Encoder::from_image(&img).map_err(|reason| ThumbnailerError::WebPEncoding {
				path: file_path.into_boxed_path(),
				reason: reason.to_string(),
			})?;

		// Type WebPMemory is !Send, which makes the Future in this function !Send,
		// this make us `deref` to have a `&[u8]` and then `to_owned` to make a Vec<u8>
		// which implies on a unwanted clone...
		Ok(encoder.encode(TARGET_QUALITY).deref().to_owned())
	})
	.await??;

	let output_path = output_path.as_ref();

	if let Some(shard_dir) = output_path.parent() {
		fs::create_dir_all(shard_dir)
			.await
			.map_err(|e| FileIOError::from((shard_dir, e)))?;
	} else {
		error!(
			"Failed to get parent directory of '{}' for sharding parent directory",
			output_path.display()
		);
	}

	fs::write(output_path, &webp)
		.await
		.map_err(|e| FileIOError::from((output_path, e)))
		.map_err(Into::into)
}

#[cfg(feature = "ffmpeg")]
async fn generate_video_thumbnail(
	file_path: impl AsRef<Path>,
	output_path: impl AsRef<Path>,
) -> Result<(), ThumbnailerError> {
	use sd_ffmpeg::to_thumbnail;

	to_thumbnail(file_path, output_path, 256, TARGET_QUALITY)
		.await
		.map_err(Into::into)
}
