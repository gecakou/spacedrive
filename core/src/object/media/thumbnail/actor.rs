use crate::{
	api::CoreEvent,
	library::{Libraries, LibraryId, LibraryManagerEvent},
	util::error::{FileIOError, NonUtf8PathError},
};

use sd_prisma::prisma::{location, PrismaClient};

use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use async_channel as chan;
use once_cell::sync::OnceCell;
use thiserror::Error;
use tokio::{
	fs, spawn,
	sync::{broadcast, oneshot, Mutex},
	time::{sleep, Instant},
};
use tracing::{debug, error};
use uuid::Uuid;

use super::{
	directory::init_thumbnail_dir,
	process::{generate_thumbnail, ThumbData},
	state::RegisterReporter,
	worker::{worker, WorkerChannels},
	BatchToProcess, ThumbnailKind, ThumbnailerError, ONE_SEC, THUMBNAIL_CACHE_DIR_NAME,
};

static BATCH_SIZE: OnceCell<usize> = OnceCell::new();

#[derive(Error, Debug)]
pub(super) enum ActorError {
	#[error("database error")]
	Database(#[from] prisma_client_rust::QueryError),
	#[error(transparent)]
	FileIO(#[from] FileIOError),
	#[error(transparent)]
	NonUtf8Path(#[from] NonUtf8PathError),
}

#[derive(Debug)]
pub(super) enum DatabaseMessage {
	Add(Uuid, Arc<PrismaClient>),
	Update(Uuid, Arc<PrismaClient>),
	Remove(Uuid),
}

// Thumbnails directory have the following structure:
// thumbnails/
// ├── version.txt
// ├── thumbs_to_process.bin # processing save state
// ├── ephemeral/ # ephemeral ones have it's own directory
// │  └── <cas_id>[0..3]/ # sharding
// │     └── <cas_id>.webp
// └── <library_id>/ # we segregate thumbnails by library
//    └── <cas_id>[0..3]/ # sharding
//       └── <cas_id>.webp
pub struct Thumbnailer {
	thumbnails_directory: Arc<PathBuf>,
	cas_ids_to_delete_tx: chan::Sender<(Vec<String>, ThumbnailKind)>,
	thumbnails_to_generate_tx: chan::Sender<(BatchToProcess, ThumbnailKind)>,
	progress_reporter_tx: chan::Sender<RegisterReporter>,
	last_single_thumb_generated: Mutex<Instant>,
	reporter: broadcast::Sender<CoreEvent>,
	cancel_tx: chan::Sender<oneshot::Sender<()>>,
}

impl Thumbnailer {
	pub async fn new(
		data_dir: PathBuf,
		libraries_manager: Arc<Libraries>,
		reporter: broadcast::Sender<CoreEvent>,
	) -> Self {
		let thumbnails_directory = Arc::new(
			init_thumbnail_dir(&data_dir, Arc::clone(&libraries_manager))
				.await
				.unwrap_or_else(|e| {
					error!("Failed to initialize thumbnail directory: {e:#?}");
					let mut thumbnails_directory = data_dir;
					thumbnails_directory.push(THUMBNAIL_CACHE_DIR_NAME);
					thumbnails_directory
				}),
		);

		let (progress_management_tx, progress_management_rx) = chan::bounded(16);

		let (databases_tx, databases_rx) = chan::bounded(4);
		let (thumbnails_to_generate_tx, ephemeral_thumbnails_to_generate_rx) = chan::unbounded();
		let (cas_ids_to_delete_tx, cas_ids_to_delete_rx) = chan::bounded(16);
		let (cancel_tx, cancel_rx) = chan::bounded(1);

		BATCH_SIZE
			.set(std::thread::available_parallelism().map_or_else(
				|e| {
					error!("Failed to get available parallelism: {e:#?}");
					4
				},
				|non_zero| {
					let count = non_zero.get();
					debug!("Thumbnailer will process batches of {count} thumbnails in parallel.");
					count
				},
			))
			.ok();

		spawn({
			let progress_management_rx = progress_management_rx.clone();
			let cancel_rx = cancel_rx.clone();
			let thumbnails_directory = Arc::clone(&thumbnails_directory);
			let reporter = reporter.clone();

			async move {
				while let Err(e) = spawn(worker(
					*BATCH_SIZE
						.get()
						.expect("BATCH_SIZE is set at thumbnailer new method"),
					reporter.clone(),
					thumbnails_directory.clone(),
					WorkerChannels {
						progress_management_rx: progress_management_rx.clone(),
						databases_rx: databases_rx.clone(),
						cas_ids_to_delete_rx: cas_ids_to_delete_rx.clone(),
						thumbnails_to_generate_rx: ephemeral_thumbnails_to_generate_rx.clone(),
						cancel_rx: cancel_rx.clone(),
					},
				))
				.await
				{
					error!(
						"Error on Thumbnail Remover Actor; \
						Error: {e}; \
						Restarting the worker loop...",
					);
				}
			}
		});

		spawn({
			let rx = libraries_manager.rx.clone();
			let thumbnails_directory = Arc::clone(&thumbnails_directory);

			async move {
				let subscribe_res = rx
					.subscribe(|event| {
						let databases_tx = databases_tx.clone();

						let thumbnails_directory = &thumbnails_directory;

						async move {
							match event {
								LibraryManagerEvent::Load(library) => {
									let library_dir =
										thumbnails_directory.join(library.id.to_string());

									if let Err(e) = fs::create_dir_all(&library_dir).await {
										error!(
											"Failed to create library dir for thumbnails: {:#?}",
											FileIOError::from((library_dir, e))
										);
									}

									databases_tx
										.send(DatabaseMessage::Add(
											library.id,
											Arc::clone(&library.db),
										))
										.await
										.expect("critical thumbnailer error: databases channel closed on send add")
								}

								LibraryManagerEvent::Edit(library)
								| LibraryManagerEvent::InstancesModified(library) => databases_tx
									.send(DatabaseMessage::Update(
										library.id,
										Arc::clone(&library.db),
									))
									.await
									.expect("critical thumbnailer error: databases channel closed on send update"),

								LibraryManagerEvent::Delete(library) => databases_tx
									.send(DatabaseMessage::Remove(library.id))
									.await
									.expect("critical thumbnailer error: databases channel closed on send delete"),
							}
						}
					})
					.await;

				if subscribe_res.is_err() {
					error!("Thumbnailer actor has crashed...")
				}
			}
		});

		Self {
			thumbnails_directory,
			cas_ids_to_delete_tx,
			thumbnails_to_generate_tx,
			progress_reporter_tx: progress_management_tx,
			last_single_thumb_generated: Mutex::new(Instant::now()),
			reporter,
			cancel_tx,
		}
	}

	#[inline]
	async fn new_batch(&self, batch: BatchToProcess, kind: ThumbnailKind) {
		self.thumbnails_to_generate_tx
			.send((batch, kind))
			.await
			.expect("critical thumbnailer error: failed to send new batch");
	}

	#[inline]
	pub async fn new_ephemeral_thumbnails_batch(&self, batch: BatchToProcess) {
		self.new_batch(batch, ThumbnailKind::Ephemeral).await
	}

	#[inline]
	pub async fn new_indexed_thumbnails_batch(&self, batch: BatchToProcess, library_id: LibraryId) {
		self.new_batch(batch, ThumbnailKind::Indexed(library_id))
			.await
	}

	#[inline]
	pub async fn new_indexed_thumbnails_batch_with_ticket(
		&self,
		mut batch: BatchToProcess,
		library_id: LibraryId,
		location_id: location::id::Type,
	) {
		batch.location_id = Some(location_id);

		self.new_batch(batch, ThumbnailKind::Indexed(library_id))
			.await;
	}

	#[inline]
	pub async fn register_reporter(
		&self,
		location_id: location::id::Type,
		progress_tx: chan::Sender<(u32, u32)>,
	) {
		self.progress_reporter_tx
			.send((location_id, progress_tx))
			.await
			.expect("critical thumbnailer error: failed to send register reporter fn");
	}

	#[inline]
	async fn remove_cas_ids(&self, cas_ids: Vec<String>, kind: ThumbnailKind) {
		self.cas_ids_to_delete_tx
			.send((cas_ids, kind))
			.await
			.expect("critical thumbnailer error: failed to send cas ids to delete");
	}

	#[inline]
	pub async fn remove_ephemeral_cas_ids(&self, cas_ids: Vec<String>) {
		self.remove_cas_ids(cas_ids, ThumbnailKind::Ephemeral).await
	}

	#[inline]
	pub async fn remove_indexed_cas_ids(&self, cas_ids: Vec<String>, library_id: LibraryId) {
		self.remove_cas_ids(cas_ids, ThumbnailKind::Indexed(library_id))
			.await
	}

	#[inline]
	pub async fn shutdown(&self) {
		let (tx, rx) = oneshot::channel();
		self.cancel_tx
			.send(tx)
			.await
			.expect("critical thumbnailer error: failed to send shutdown signal");

		rx.await
			.expect("critical thumbnailer error: failed to receive shutdown signal response");
	}

	/// WARNING!!!! DON'T USE THIS METHOD IN A LOOP!!!!!!!!!!!!! It will be pretty slow on purpose!
	pub async fn generate_single_indexed_thumbnail(
		&self,
		extension: &str,
		cas_id: String,
		path: impl AsRef<Path>,
		library_id: LibraryId,
	) -> Result<(), ThumbnailerError> {
		self.generate_single_thumbnail(extension, cas_id, path, ThumbnailKind::Indexed(library_id))
			.await
	}

	async fn generate_single_thumbnail(
		&self,
		extension: &str,
		cas_id: String,
		path: impl AsRef<Path>,
		kind: ThumbnailKind,
	) -> Result<(), ThumbnailerError> {
		let mut last_single_thumb_generated_guard = self.last_single_thumb_generated.lock().await;

		let elapsed = Instant::now() - *last_single_thumb_generated_guard;
		if elapsed < ONE_SEC {
			// This will choke up in case someone try to use this method in a loop, otherwise
			// it will consume all the machine resources like a gluton monster from hell
			sleep(ONE_SEC - elapsed).await;
		}

		let res = generate_thumbnail(
			self.thumbnails_directory.as_ref().clone(),
			ThumbData {
				extension,
				cas_id,
				path,
				in_background: false,
				should_regenerate: false,
				kind,
			},
			self.reporter.clone(),
		)
		.await
		.map(|_| ());

		*last_single_thumb_generated_guard = Instant::now();

		res
	}
}
