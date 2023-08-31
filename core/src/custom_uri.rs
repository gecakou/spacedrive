use crate::{
	location::file_path_helper::{file_path_to_handle_custom_uri, IsolatedFilePathData},
	prisma::{file_path, location},
	util::{db::*, InfallibleResponse},
	Node,
};

use std::{
	cmp::min,
	ffi::OsStr,
	fmt::Debug,
	fs::Metadata,
	io::{self, SeekFrom},
	panic::Location,
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
	time::UNIX_EPOCH,
};

use axum::{
	body::{self, Body, BoxBody, Full, StreamBody},
	extract::{self, State},
	http::{self, request, HeaderValue, Method, Request, Response, StatusCode},
	middleware::{self, Next},
	routing::get,
	Router,
};
use http_range::HttpRange;
use mini_moka::sync::Cache;
use sd_file_ext::text::is_text;
use tokio::{
	fs::File,
	io::{AsyncReadExt, AsyncSeekExt},
};
use tokio_util::io::ReaderStream;
use tracing::{debug, error};
use uuid::Uuid;

type MetadataCacheKey = (Uuid, file_path::id::Type);
type NameAndExtension = (PathBuf, String);

const MAX_TEXT_READ_LENGTH: usize = 10 * 1024; // 10KB

#[derive(Clone)]
struct LocalState {
	node: Arc<Node>,

	// This LRU cache allows us to avoid doing a DB lookup on every request.
	// The main advantage of this LRU Cache is for video files. Video files are fetch in multiple chunks and the cache prevents a DB lookup on every chunk reducing the request time from 15-25ms to 1-10ms.
	// TODO: We should listen to events when deleting or moving a location and evict the cache accordingly.
	file_metadata_cache: Cache<MetadataCacheKey, NameAndExtension>,
}

// We are using Axum on all platforms because Tauri's custom URI protocols can't be async!
// TODO(@Oscar): Long-term hopefully this can be moved into rspc but streaming files is a hard thing for rspc to solve (Eg. how does batching work, dyn-safe handler, etc).
pub fn router(node: Arc<Node>) -> Router<()> {
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
					(path.starts_with(&thumbnail_path) && path.extension() == Some(OsStr::new("webp"))).then_some(()).ok_or_else(|| not_found(()))?;

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
						InfallibleResponse::builder().header("Content-Type", HeaderValue::from_static("image/webp")),
					)
					.await
				},
			),
		)
		.route(
			"/file/:lib_id/:loc_id/:path_id",
			get(
				|State(state): State<LocalState>,
				 extract::Path((lib_id, loc_id, path_id)): extract::Path<(
					String,
					String,
					String,
				)>,
				 request: Request<Body>| async move {
					let library_id = Uuid::from_str(&lib_id).map_err(bad_request)?;
					let location_id = loc_id.parse::<location::id::Type>().map_err(bad_request)?;
					let file_path_id = path_id.parse::<file_path::id::Type>().map_err(bad_request)?;

					let lru_cache_key = (library_id, file_path_id);

					let (file_path_full_path, extension) = if let Some(entry) =
						state.file_metadata_cache.get(&lru_cache_key)
					{
						entry
					} else {
						let library = state.node.libraries.get_library(&library_id).await.ok_or_else(|| internal_server_error(()))?;

						let file_path = library
							.db
							.file_path()
							.find_unique(file_path::id::equals(file_path_id))
							.select(file_path_to_handle_custom_uri::select())
							.exec()
							.await
							.map_err(internal_server_error)?
							.ok_or_else(|| not_found(()))?;

						let location =
							maybe_missing(&file_path.location, "file_path.location").map_err(internal_server_error)?;
						let path =
							maybe_missing(&location.path, "file_path.location.path").map_err(internal_server_error)?;

						let lru_entry = (
							Path::new(path).join(
								IsolatedFilePathData::try_from((location_id, &file_path)).map_err(not_found)?
							),
							maybe_missing(file_path.extension, "extension").map_err(not_found)?
						);

						state
							.file_metadata_cache
							.insert(lru_cache_key, lru_entry.clone());

						lru_entry
					};

					let metadata = file_path_full_path.metadata().map_err(internal_server_error)?;
					(!metadata.is_dir()).then_some(()).ok_or_else(|| not_found(()))?;

					let mut file = File::open(&file_path_full_path).await.map_err(|err| {
						InfallibleResponse::builder()
								.status(if err.kind() == io::ErrorKind::NotFound {
									StatusCode::NOT_FOUND
								} else {
									StatusCode::INTERNAL_SERVER_ERROR
								})
								.body(body::boxed(Full::from("")))
					})?;

					let resp = InfallibleResponse::builder().header("Content-Type", HeaderValue::from_str(&plz_for_the_love_of_all_that_is_good_replace_this_with_the_db_instead_of_adding_variants_to_it(&extension, &mut file, &metadata).await?).map_err(|err| {
						error!("Error converting mime-type into header value: {}", err);
						internal_server_error(())
					})?);

					serve_file(file, Ok(metadata), request.into_parts().0, resp).await
				},
			),
		)
		.route_layer(middleware::from_fn(cors_middleware))
		.with_state(LocalState {
			node,
			file_metadata_cache: Cache::new(100),
		})
}

#[track_caller]
fn bad_request(err: impl Debug) -> http::Response<BoxBody> {
	debug!("400: Bad Request at {}: {err:?}", Location::caller());

	InfallibleResponse::builder()
		.status(StatusCode::BAD_REQUEST)
		.body(body::boxed(Full::from("")))
}

#[track_caller]
fn not_found(err: impl Debug) -> http::Response<BoxBody> {
	debug!("404: Not Found at {}: {err:?}", Location::caller());

	InfallibleResponse::builder()
		.status(StatusCode::NOT_FOUND)
		.body(body::boxed(Full::from("")))
}

#[track_caller]
fn internal_server_error(err: impl Debug) -> http::Response<BoxBody> {
	debug!(
		"500 - Internal Server Error at {}: {err:?}",
		Location::caller()
	);

	InfallibleResponse::builder()
		.status(StatusCode::INTERNAL_SERVER_ERROR)
		.body(body::boxed(Full::from("")))
}

async fn cors_middleware<B>(req: Request<B>, next: Next<B>) -> Response<BoxBody> {
	if req.method() == Method::OPTIONS {
		return Response::builder()
			.header("Access-Control-Allow-Methods", "GET, HEAD, POST, OPTIONS")
			.header("Access-Control-Allow-Headers", "*")
			.header("Access-Control-Max-Age", "86400")
			.status(StatusCode::OK)
			.body(body::boxed(Full::from("")))
			.expect("Invalid static response!");
	}

	let mut response = next.run(req).await;

	response
		.headers_mut()
		.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));

	// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection
	response
		.headers_mut()
		.insert("Connection", HeaderValue::from_static("Keep-Alive"));

	response
		.headers_mut()
		.insert("Server", HeaderValue::from_static("Spacedrive"));

	response
}

/// Serve a Tokio file as a HTTP response.
///
/// This function takes care of:
///  - 304 Not Modified using ETag's
///  - Range requests for partial content
///
/// BE AWARE this function does not do any path traversal protection so that's up to the caller!
async fn serve_file(
	mut file: File,
	metadata: io::Result<Metadata>,
	req: request::Parts,
	mut resp: InfallibleResponse,
) -> Result<Response<BoxBody>, Response<BoxBody>> {
	// Handle `ETag` and `Content-Length` headers
	if let Ok(metadata) = metadata {
		// We only accept range queries if `files.metadata() == Ok(_)`
		// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Ranges
		resp = resp.header("Accept-Ranges", HeaderValue::from_static("bytes"));

		if let Ok(time) = metadata.modified() {
			let etag_header = format!(
				r#""{}""#,
				// The ETag's can be any value so we just use the modified time to make it easy.
				time.duration_since(UNIX_EPOCH)
					.expect("are you a time traveller? cause that's the only explanation for this error")
					.as_millis()
			);

			if let Some(etag) = req.headers.get("If-None-Match") {
				if etag.as_bytes() == etag_header.as_bytes() {
					return Ok(resp
						.status(StatusCode::NOT_MODIFIED)
						.body(body::boxed(Full::from(""))));
				}
			}

			// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag
			if let Ok(etag_header) = HeaderValue::from_str(&etag_header) {
				resp = resp.header("etag", etag_header);
			} else {
				error!("Failed to convert ETag into header value!");
			}
		}

		// Empty files
		if metadata.len() == 0 {
			return Ok(resp
				.status(StatusCode::OK)
				.header("Content-Length", HeaderValue::from_static("0"))
				.body(body::boxed(Full::from(""))));
		}

		// https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests
		if req.method == Method::GET {
			if let Some(range) = req.headers.get("range") {
				// TODO: Error handling
				let ranges = HttpRange::parse(range.to_str().map_err(bad_request)?, metadata.len())
					.map_err(bad_request)?;

				// TODO: Multipart requests are not support, yet
				if ranges.len() != 1 {
					todo!(); // TODO: Error handling
				}
				let range = ranges.first().expect("checked above");

				file.seek(SeekFrom::Start(range.start))
					.await
					.map_err(internal_server_error)?;

				// TODO: Serve using streaming body instead of loading the entire chunk. - Right now my impl is not working correctly
				let mut buf = Vec::with_capacity(range.length as usize);
				file.take(range.length)
					.read_to_end(&mut buf)
					.await
					.map_err(internal_server_error)?;

				return Ok(resp
					.status(StatusCode::PARTIAL_CONTENT)
					.header(
						"Content-Range",
						HeaderValue::from_str(&format!(
							"bytes {}-{}/{}",
							range.start,
							range.start + range.length - 1,
							metadata.len()
						))
						.map_err(internal_server_error)?,
					)
					.header(
						"Content-Length",
						HeaderValue::from_str(&range.length.to_string())
							.map_err(internal_server_error)?,
					)
					.body(body::boxed(Full::from(buf))));
				// TODO: Serve as stream instead of fixed set of bytes -> Show allow only loading part in the chunk into memory at a time. This will also be probs be required or P2P over custom URI.
				// .body(body::boxed(Limited::new(
				// 	StreamBody::new(ReaderStream::new(file)),
				// 	range.length.try_into().expect("integer overflow"),
				// )));
			}
		}
	}

	Ok(resp.body(body::boxed(StreamBody::new(ReaderStream::new(file)))))
}

// TODO: This should be determined from magic bytes when the file is indexed and stored it in the DB on the file path
async fn plz_for_the_love_of_all_that_is_good_replace_this_with_the_db_instead_of_adding_variants_to_it(
	ext: &str,
	file: &mut File,
	metadata: &Metadata,
) -> Result<String, Response<BoxBody>> {
	let mime_type = match ext {
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
		// HEIF/HEIC images
		"heif" | "heifs" => "image/heif,image/heif-sequence",
		"heic" | "heics" => "image/heic,image/heic-sequence",
		// AVIF images
		"avif" | "avci" | "avcs" => "image/avif",
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
		let mime_type = match ext {
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
					todo!();
					// "TODO: This filetype is not supported because of the missing mime type!",
				};
				mime_type
			}
		};

		format!("{mime_type}; charset={charset}")
	} else {
		mime_type.to_string()
	})
}
