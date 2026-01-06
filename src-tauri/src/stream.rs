//! Custom streaming protocol for audio/video files with range request support.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::UriSchemeContext;
use tracing::{info, warn};

/// Get MIME type from file extension
fn get_mime_type(path: &str) -> &'static str {
    let ext = path.split('.').next_back().unwrap_or("").to_lowercase();
    match ext.as_str() {
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        // Documents
        "pdf" => "application/pdf",
        // Default
        _ => "application/octet-stream",
    }
}

/// Build an HTTP response with graceful error handling.
/// Falls back to a plain 500 response if the builder fails.
fn build_response(status: StatusCode, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .body(body)
        .unwrap_or_else(|e| {
            warn!("Failed to build HTTP response: {}", e);
            // Last resort fallback - return empty response
            Response::new(b"Internal server error".to_vec())
        })
}

/// Build an HTTP response with headers and graceful error handling.
fn build_response_with_headers(
    status: StatusCode,
    body: Vec<u8>,
    headers: Vec<(&str, &str)>,
) -> Response<Vec<u8>> {
    let mut builder = Response::builder().status(status);
    
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    
    builder.body(body.clone()).unwrap_or_else(|e| {
        warn!("Failed to build HTTP response with headers: {}", e);
        // Fallback without headers - use the cloned body
        Response::new(body)
    })
}

/// Parse Range header value like "bytes=0-1023" or "bytes=1024-"
fn parse_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    let range_str = range_header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let start: u64 = if parts[0].is_empty() {
        // Suffix range like "-500" means last 500 bytes
        let suffix_len: u64 = parts[1].parse().ok()?;
        file_size.saturating_sub(suffix_len)
    } else {
        parts[0].parse().ok()?
    };

    let end: u64 = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse().ok()?
    };

    // Clamp end to file size
    let end = end.min(file_size - 1);

    if start <= end && start < file_size {
        Some((start, end))
    } else {
        None
    }
}

/// Handle the custom stream:// protocol for media files
pub fn handle_stream_protocol<R: tauri::Runtime>(
    _ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri = request.uri();
    let path_str = uri.path();

    // URL decode the path (handles %20 for spaces, etc.)
    // Path comes as /Users/... so we keep the leading slash
    let decoded_path = urlencoding::decode(path_str).unwrap_or_else(|_| path_str.into());
    let file_path = PathBuf::from(decoded_path.as_ref());

    info!("Stream request for: {:?}", file_path);

    // Check if file exists
    if !file_path.exists() {
        warn!("Stream file not found: {:?}", file_path);
        return build_response(StatusCode::NOT_FOUND, b"File not found".to_vec());
    }

    // Open the file
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to open stream file: {}", e);
            return build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open file: {}", e).into_bytes(),
            );
        }
    };

    // Get file size
    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            warn!("Failed to get file metadata: {}", e);
            return build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get file size: {}", e).into_bytes(),
            );
        }
    };

    let mime_type = get_mime_type(&decoded_path);

    // Check for Range header
    let range_header = request
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());

    if let Some(range) = range_header {
        // Handle range request
        if let Some((start, end)) = parse_range(range, file_size) {
            let length = end - start + 1;

            // Seek to start position
            if let Err(e) = file.seek(SeekFrom::Start(start)) {
                warn!("Failed to seek in file: {}", e);
                return build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to seek: {}", e).into_bytes(),
                );
            }

            // Read the requested range
            let mut buffer = vec![0u8; length as usize];
            if let Err(e) = file.read_exact(&mut buffer) {
                warn!("Failed to read file range: {}", e);
                return build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read: {}", e).into_bytes(),
                );
            }

            info!(
                "Stream range response: bytes {}-{}/{} ({} bytes)",
                start, end, file_size, length
            );

            return build_response_with_headers(
                StatusCode::PARTIAL_CONTENT,
                buffer,
                vec![
                    (header::CONTENT_TYPE.as_str(), mime_type),
                    (header::CONTENT_LENGTH.as_str(), &length.to_string()),
                    (
                        header::CONTENT_RANGE.as_str(),
                        &format!("bytes {}-{}/{}", start, end, file_size),
                    ),
                    (header::ACCEPT_RANGES.as_str(), "bytes"),
                    (header::ACCESS_CONTROL_ALLOW_ORIGIN.as_str(), "*"),
                ],
            );
        }
    }

    // Full file response (no range or invalid range)
    let mut buffer = Vec::with_capacity(file_size as usize);
    if let Err(e) = file.read_to_end(&mut buffer) {
        warn!("Failed to read file: {}", e);
        return build_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e).into_bytes(),
        );
    }

    info!("Stream full response: {} bytes", buffer.len());

    build_response_with_headers(
        StatusCode::OK,
        buffer,
        vec![
            (header::CONTENT_TYPE.as_str(), mime_type),
            (header::CONTENT_LENGTH.as_str(), &file_size.to_string()),
            (header::ACCEPT_RANGES.as_str(), "bytes"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN.as_str(), "*"),
        ],
    )
}
