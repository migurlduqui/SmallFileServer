use axum::extract::Path;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use serde_json::{json, Value};
use std::path::Path as StdPath;
use super::utils::validate_file_name::safe_join;

pub async fn download(
    Path(filename): Path<String>,
) -> Result<(HeaderMap, Vec<u8>), (StatusCode, Json<Value>)> {


    let base_path = StdPath::new("uploads");
    let full_path = safe_join(base_path, &filename)?;

    // ─── STEP 2: Check if the file exists ───
    if !full_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("File '{}' not found", filename)})),
        ));
    }

    println!("Downloading file: {}", filename);

    // ─── STEP 3: Read the file from disk ───
    // fs::read() loads the ENTIRE file into memory as Vec<u8>.
    // For a learning project this is fine.
    // For very large files, you'd use streaming (tokio::fs::read).
    let data = tokio::fs::read(&full_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?;

    println!("Read {} bytes", data.len());

    let mime_type = mime_guess::from_path(&filename)
        .first_or_octet_stream()
        .to_string();

        // ─── STEP 5: Build HTTP headers ───
    let mut headers = HeaderMap::new();

    // Content-Type tells the browser what kind of file this is
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type).unwrap(),
    );

    // Content-Disposition tells the browser whether to show inline or download.
    // "attachment" forces a download dialog.
    // "inline" would try to display in the browser (PDF, images, etc.)
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .unwrap(),
    );

    // Content-Length tells the browser how many bytes to expect
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&data.len().to_string()).unwrap(),
    );

    println!("Sending file: {} ({} bytes, type: {})", filename, data.len(), mime_type);

    // ─── STEP 6: Return the response ───
    // Axum knows how to handle (HeaderMap, Vec<u8>) as an HTTP response.
    // It sends the headers first, then the raw bytes as the body.
    Ok((headers, data))

}