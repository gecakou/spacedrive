use crate::{
	api::{utils::InvalidateOperationEvent, CoreEvent},
	library::Library,
	object::media::thumbnail::WEBP_EXTENSION,
	p2p::operations,
	util::InfallibleResponse,
	Node,
};

use hyper::{header, upgrade::OnUpgrade};
use sd_file_ext::text::is_text;
use sd_file_path_helper::{file_path_to_handle_custom_uri, IsolatedFilePathData};
use sd_p2p2::{IdentityOrRemoteIdentity, RemoteIdentity};
use sd_p2p_block::Range;
use sd_prisma::prisma::{file_path, location};
use sd_utils::db::maybe_missing;

use std::{
	cmp::min,
	ffi::OsStr,
	fmt::Debug,
	fs::Metadata,
	path::{Path, PathBuf},
	str::FromStr,
	sync::{atomic::Ordering, Arc},
};

use async_stream::stream;
use axum::{
	body::{self, Body, BoxBody, Full, StreamBody},
	extract::{self, State},
	http::{HeaderMap, HeaderValue, Request, Response, StatusCode},
	middleware,
	response::IntoResponse,
	routing::get,
	Router,
};
use bytes::Bytes;
use mini_moka::sync::Cache;
use tokio::{
	fs::{self, File},
	io::{self, copy_bidirectional, AsyncReadExt, AsyncSeekExt, SeekFrom},
};
use tokio_util::sync::PollSender;
use tracing::{error, warn};
use uuid::Uuid;

use self::{mpsc_to_async_write::MpscToAsyncWrite, serve_file::serve_file, utils::*};

mod async_read_body;
mod mpsc_to_async_write;
mod serve_file;
mod utils;

type CacheKey = (Uuid, file_path::id::Type);

#[derive(Debug, Clone)]
struct CacheValue {
	name: PathBuf,
	ext: String,
	file_path_pub_id: Uuid,
	serve_from: ServeFrom,
}

const MAX_TEXT_READ_LENGTH: usize = 10 * 1024; // 10KB

#[derive(Debug, Clone)]
pub enum ServeFrom {
	/// Serve from the local filesystem
	Local,
	/// Serve from a specific instance
	Remote(RemoteIdentity),
}

#[derive(Clone)]
pub struct LocalState {
	node: Arc<Node>,

	// This LRU cache allows us to avoid doing a DB lookup on every request.
	// The main advantage of this LRU Cache is for video files. Video files are fetch in multiple chunks and the cache prevents a DB lookup on every chunk reducing the request time from 15-25ms to 1-10ms.
	// TODO: We should listen to events when deleting or moving a location and evict the cache accordingly.
	file_metadata_cache: Arc<Cache<CacheKey, CacheValue>>,
}

type ExtractedPath = extract::Path<(String, String, String)>;

async fn get_or_init_lru_entry(
	state: &LocalState,
	extract::Path((lib_id, loc_id, path_id)): ExtractedPath,
) -> Result<(CacheValue, Arc<Library>), Response<BoxBody>> {
	let library_id = Uuid::from_str(&lib_id).map_err(bad_request)?;
	let location_id = loc_id.parse::<location::id::Type>().map_err(bad_request)?;
	let file_path_id = path_id
		.parse::<file_path::id::Type>()
		.map_err(bad_request)?;

	let lru_cache_key = (library_id, file_path_id);
	let library = state
		.node
		.libraries
		.get_library(&library_id)
		.await
		.ok_or_else(|| internal_server_error(()))?;

	if let Some(entry) = state.file_metadata_cache.get(&lru_cache_key) {
		Ok((entry, library))
	} else {
		let file_path = library
			.db
			.file_path()
			.find_unique(file_path::id::equals(file_path_id))
			// TODO: This query could be seen as a security issue as it could load the private key (`identity`) when we 100% don't need it. We are gonna wanna fix that!
			.select(file_path_to_handle_custom_uri::select())
			.exec()
			.await
			.map_err(internal_server_error)?
			.ok_or_else(|| not_found(()))?;

		let location = maybe_missing(&file_path.location, "file_path.location")
			.map_err(internal_server_error)?;
		let path = maybe_missing(&location.path, "file_path.location.path")
			.map_err(internal_server_error)?;
		let instance = maybe_missing(&location.instance, "file_path.location.instance")
			.map_err(internal_server_error)?;

		let path = Path::new(path)
			.join(IsolatedFilePathData::try_from((location_id, &file_path)).map_err(not_found)?);

		let identity = IdentityOrRemoteIdentity::from_bytes(&instance.identity)
			.map_err(internal_server_error)?
			.remote_identity();

		let lru_entry = CacheValue {
			name: path,
			ext: maybe_missing(file_path.extension, "extension").map_err(not_found)?,
			file_path_pub_id: Uuid::from_slice(&file_path.pub_id).map_err(internal_server_error)?,
			serve_from: if identity == library.identity.to_remote_identity() {
				ServeFrom::Local
			} else {
				ServeFrom::Remote(identity)
			},
		};

		state
			.file_metadata_cache
			.insert(lru_cache_key, lru_entry.clone());

		Ok((lru_entry, library))
	}
}

pub fn base_router() -> Router<LocalState> {
	Router::new()
		.route(
			"/thumbnail/*path",
			get(
				|State(state): State<LocalState>,
				 extract::Path(path): extract::Path<String>,
				 request: Request<Body>| async move {
					let thumbnail_path = state.node.config.data_directory().join("thumbnails");
					let path = thumbnail_path.join(path);

					// Prevent directory traversal attacks (Eg. requesting `../../../etc/passwd`)
					// For now we only support `webp` thumbnails.
					(path.starts_with(&thumbnail_path)
						&& path.extension() == Some(WEBP_EXTENSION.as_ref()))
					.then_some(())
					.ok_or_else(|| not_found(()))?;

					let file = File::open(&path).await.map_err(|err| {
						InfallibleResponse::builder()
							.status(if err.kind() == io::ErrorKind::NotFound {
								StatusCode::NOT_FOUND
							} else {
								StatusCode::INTERNAL_SERVER_ERROR
							})
							.body(body::boxed(Full::from("")))
					})?;
					let metadata = file.metadata().await;
					serve_file(
						file,
						metadata,
						request.into_parts().0,
						InfallibleResponse::builder()
							.header("Content-Type", HeaderValue::from_static("image/webp")),
					)
					.await
				},
			),
		)
		.route(
			"/file/:lib_id/:loc_id/:path_id",
			get(
				|State(state): State<LocalState>, path: ExtractedPath, request: Request<Body>| async move {
					let (
						CacheValue {
							name: file_path_full_path,
							ext: extension,
							file_path_pub_id,
							serve_from,
							..
						},
						library,
					) = get_or_init_lru_entry(&state, path).await?;

					match serve_from {
						ServeFrom::Local => {
							let metadata = fs::metadata(&file_path_full_path)
								.await
								.map_err(internal_server_error)?;
							(!metadata.is_dir())
								.then_some(())
								.ok_or_else(|| not_found(()))?;

							let mut file =
								File::open(&file_path_full_path).await.map_err(|err| {
									InfallibleResponse::builder()
										.status(if err.kind() == io::ErrorKind::NotFound {
											StatusCode::NOT_FOUND
										} else {
											StatusCode::INTERNAL_SERVER_ERROR
										})
										.body(body::boxed(Full::from("")))
								})?;

							let resp = InfallibleResponse::builder().header(
								"Content-Type",
								HeaderValue::from_str(
									&infer_the_mime_type(&extension, &mut file, &metadata).await?,
								)
								.map_err(|err| {
									error!("Error converting mime-type into header value: {}", err);
									internal_server_error(())
								})?,
							);

							serve_file(file, Ok(metadata), request.into_parts().0, resp).await
						}
						ServeFrom::Remote(identity) => {
							if !state.node.files_over_p2p_flag.load(Ordering::Relaxed) {
								return Ok(not_found(()));
							}

							// TODO: Support `Range` requests and `ETag` headers
							let stream = state
								.node
								.p2p
								.get_instance(&library.id, identity)
								.ok_or_else(|| {
									not_found(format!("Error connecting to {identity}: no connection method available"))
								})?
								.new_stream()
								.await
								.map_err(|err| {
									not_found(format!("Error connecting to {identity}: {err:?}"))
								})?;

							let (tx, mut rx) = tokio::sync::mpsc::channel::<io::Result<Bytes>>(150);
							// TODO: We only start a thread because of stupid `ManagerStreamAction2` and libp2p's `!Send/!Sync` bounds on a stream.
							tokio::spawn(async move {
								let Ok(()) = operations::request_file(
									stream,
									&library,
									file_path_pub_id,
									Range::Full,
									MpscToAsyncWrite::new(PollSender::new(tx)),
								)
								.await
								else {
									return;
								};
							});

							// TODO: Content Type
							Ok(InfallibleResponse::builder().status(StatusCode::OK).body(
								body::boxed(StreamBody::new(stream! {
									while let Some(item) = rx.recv().await {
										yield item;
									}
								})),
							))
						}
					}
				},
			),
		)
		.route(
			"/local-file-by-path/:path",
			get(
				|extract::Path(path): extract::Path<String>, request: Request<Body>| async move {
					let path = PathBuf::from(path);

					let metadata = fs::metadata(&path).await.map_err(internal_server_error)?;
					(!metadata.is_dir())
						.then_some(())
						.ok_or_else(|| not_found(()))?;

					let mut file = File::open(&path).await.map_err(|err| {
						InfallibleResponse::builder()
							.status(if err.kind() == io::ErrorKind::NotFound {
								StatusCode::NOT_FOUND
							} else {
								StatusCode::INTERNAL_SERVER_ERROR
							})
							.body(body::boxed(Full::from("")))
					})?;

					let resp = InfallibleResponse::builder().header(
						"Content-Type",
						HeaderValue::from_str(&match path.extension().and_then(OsStr::to_str) {
							None => "text/plain".to_string(),
							Some(ext) => infer_the_mime_type(ext, &mut file, &metadata).await?,
						})
						.map_err(|err| {
							error!("Error converting mime-type into header value: {}", err);
							internal_server_error(())
						})?,
					);

					serve_file(file, Ok(metadata), request.into_parts().0, resp).await
				},
			),
		)
}

pub fn with_state(node: Arc<Node>) -> LocalState {
	let file_metadata_cache = Arc::new(Cache::new(150));

	tokio::spawn({
		let file_metadata_cache = file_metadata_cache.clone();
		let mut tx = node.event_bus.0.subscribe();
		async move {
			while let Ok(event) = tx.recv().await {
				if let CoreEvent::InvalidateOperation(e) = event {
					match e {
						InvalidateOperationEvent::Single(event) => {
							// TODO: This is inefficent as any change will invalidate who cache. We need the new invalidation system!!!
							// TODO: It's also error prone and a fine-grained resource based invalidation system would avoid that.
							if event.key == "search.objects" || event.key == "search.paths" {
								file_metadata_cache.invalidate_all();
							}
						}
						InvalidateOperationEvent::All => {
							file_metadata_cache.invalidate_all();
						}
					}
				}
			}
		}
	});

	LocalState {
		node,
		file_metadata_cache,
	}
}

// We are using Axum on all platforms because Tauri's custom URI protocols can't be async!
pub fn router(node: Arc<Node>) -> Router<()> {
	Router::new()
		.route(
			"/remote/:identity/*path",
			get(
				|State(state): State<LocalState>,
				 extract::Path((identity, rest)): extract::Path<(String, String)>,
				 mut request: Request<Body>| async move {
					let identity = match RemoteIdentity::from_str(&identity) {
						Ok(identity) => identity,
						Err(err) => {
							warn!("Error parsing identity '{}': {}", identity, err);
							return (StatusCode::BAD_REQUEST, HeaderMap::new(), vec![])
								.into_response();
						}
					};

					*request.uri_mut() = format!("/{rest}")
						.parse()
						.expect("url was validated by Axum");

					let request_upgrade_header =
						request.headers().get(header::UPGRADE).map(Clone::clone);
					let maybe_client_upgrade = request.extensions_mut().remove::<OnUpgrade>();

					let mut response = match operations::remote_rspc(
						state.node.p2p.p2p.clone(),
						identity,
						request,
					)
					.await
					{
						Ok(v) => v,
						Err(err) => {
							warn!("Error doing remote rspc query with '{identity}': {err:?}");
							return StatusCode::BAD_GATEWAY.into_response();
						}
					};
					if response.status() == StatusCode::SWITCHING_PROTOCOLS {
						if response.headers().get(header::UPGRADE)
							!= request_upgrade_header.as_ref()
						{
							return StatusCode::BAD_REQUEST.into_response();
						}

						let Some(request_upgraded) = maybe_client_upgrade else {
							return StatusCode::BAD_REQUEST.into_response();
						};
						let Some(response_upgraded) =
							response.extensions_mut().remove::<OnUpgrade>()
						else {
							return StatusCode::BAD_REQUEST.into_response();
						};

						tokio::spawn(async move {
							let Ok(mut request_upgraded) = request_upgraded.await.map_err(|err| {
								warn!("Error upgrading websocket request: {err}");
							}) else {
								return;
							};
							let Ok(mut response_upgraded) =
								response_upgraded.await.map_err(|err| {
									warn!("Error upgrading websocket response: {err}");
								})
							else {
								return;
							};

							copy_bidirectional(&mut request_upgraded, &mut response_upgraded)
								.await
								.map_err(|err| {
									warn!("Error upgrading websocket response: {err}");
								})
								.ok();
						});
					}

					response.into_response()
				},
			),
		)
		.merge(base_router())
		.route_layer(middleware::from_fn(cors_middleware))
		.with_state(with_state(node))
}

// TODO: This should possibly be determined from magic bytes when the file is indexed and stored it in the DB on the file path
async fn infer_the_mime_type(
	ext: &str,
	file: &mut File,
	metadata: &Metadata,
) -> Result<String, Response<BoxBody>> {
	let ext = ext.to_lowercase();
	let mime_type = match ext.as_str() {
		// AAC audio
		"aac" => "audio/aac",
		// Musical Instrument Digital Interface (MIDI)
		"mid" | "midi" => "audio/midi, audio/x-midi",
		// MP3 audio
		"mp3" => "audio/mpeg",
		// MP4 audio
		"m4a" => "audio/mp4",
		// OGG audio
		"oga" => "audio/ogg",
		// Opus audio
		"opus" => "audio/opus",
		// Waveform Audio Format
		"wav" => "audio/wav",
		// WEBM audio
		"weba" => "audio/webm",
		// AVI: Audio Video Interleave
		"avi" => "video/x-msvideo",
		// MP4 video
		"mp4" | "m4v" => "video/mp4",
		// TODO: Bruh
		#[cfg(not(target_os = "macos"))]
		// TODO: Bruh
		// FIX-ME: This media types break macOS video rendering
		// MPEG transport stream
		"ts" => "video/mp2t",
		// TODO: Bruh
		#[cfg(not(target_os = "macos"))]
		// FIX-ME: This media types break macOS video rendering
		// MPEG Video
		"mpeg" => "video/mpeg",
		// OGG video
		"ogv" => "video/ogg",
		// WEBM video
		"webm" => "video/webm",
		// 3GPP audio/video container (TODO: audio/3gpp if it doesn't contain video)
		"3gp" => "video/3gpp",
		// 3GPP2 audio/video container (TODO: audio/3gpp2 if it doesn't contain video)
		"3g2" => "video/3gpp2",
		// Quicktime movies
		"mov" => "video/quicktime",
		// Windows OS/2 Bitmap Graphics
		"bmp" => "image/bmp",
		// Graphics Interchange Format (GIF)
		"gif" => "image/gif",
		// Icon format
		"ico" => "image/vnd.microsoft.icon",
		// JPEG images
		"jpeg" | "jpg" => "image/jpeg",
		// Portable Network Graphics
		"png" => "image/png",
		// Scalable Vector Graphics (SVG)
		"svg" => "image/svg+xml",
		// Tagged Image File Format (TIFF)
		"tif" | "tiff" => "image/tiff",
		// WEBP image
		"webp" => "image/webp",
		// PDF document
		"pdf" => "application/pdf",
		// HEIF images
		"heif" => "image/heif",
		// HEIF images sequence (animated)
		"heifs" => "image/heif-sequence",
		// HEIC images
		"heic" | "hif" => "image/heic",
		// HEIC images sequence (animated)
		"heics" => "image/heic-sequence",
		// AV1 in HEIF images
		"avif" => "image/avif",
		// AV1 in HEIF images sequence (DEPRECATED: https://github.com/AOMediaCodec/av1-avif/pull/86/files)
		"avifs" => "image/avif-sequence",
		// AVC in HEIF images
		"avci" => "image/avci",
		// AVC in HEIF images sequence (animated)
		"avcs" => "image/avcs",
		_ => "text/plain",
	};

	Ok(if mime_type == "text/plain" {
		let mut text_buf = vec![
			0;
			min(
				metadata.len().try_into().unwrap_or(usize::MAX),
				MAX_TEXT_READ_LENGTH
			)
		];
		if !text_buf.is_empty() {
			file.read_exact(&mut text_buf)
				.await
				.map_err(internal_server_error)?;
			file.seek(SeekFrom::Start(0))
				.await
				.map_err(internal_server_error)?;
		}

		let charset = is_text(&text_buf, text_buf.len() == (metadata.len() as usize)).unwrap_or("");

		// Only browser recognized types, everything else should be text/plain
		// https://www.iana.org/assignments/media-types/media-types.xhtml#table-text
		let mime_type = match ext.as_str() {
			// HyperText Markup Language
			"html" | "htm" => "text/html",
			// Cascading Style Sheets
			"css" => "text/css",
			// Javascript
			"js" | "mjs" => "text/javascript",
			// Comma-separated values
			"csv" => "text/csv",
			// Markdown
			"md" | "markdown" => "text/markdown",
			// Rich text format
			"rtf" => "text/rtf",
			// Web Video Text Tracks
			"vtt" => "text/vtt",
			// Extensible Markup Language
			"xml" => "text/xml",
			// Text
			"txt" => "text/plain",
			_ => {
				if charset.is_empty() {
					// "TODO: This filetype is not supported because of the missing mime type!",
					return Err(not_implemented(()));
				};
				mime_type
			}
		};

		format!("{mime_type}; charset={charset}")
	} else {
		mime_type.to_string()
	})
}
