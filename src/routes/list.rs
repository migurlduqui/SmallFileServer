use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::models::FileInfo;

pub async fn list() -> Result<Json<Vec<FileInfo>>, (StatusCode, Json<Value>)> {
    let mut files: Vec<FileInfo> = Vec::new();

    // Start recursive traversal from the uploads directory
    collect_files(Path::new("uploads"), &mut files)?;

    Ok(Json(files))
}

/// Recursively collects all files from a directory and its subdirectories.
///
/// This function:
/// 1. Reads all entries in the given directory
/// 2. For each entry that is a file, extracts metadata and pushes to `files`
/// 3. For each entry that is a subdirectory, recurses into it
///
/// The `filename` field stores the relative path from `uploads/`,
/// so subfolder files look like: "subfolder/photo.jpg"
fn collect_files(
    dir: &Path,
    files: &mut Vec<FileInfo>,
) -> Result<(), (StatusCode, Json<Value>)> {
    // Read the directory entries
    let entries = fs::read_dir(dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"501": format!("Failed to read directory: {}", e)})),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"501": format!("Failed to read entry: {}", e)})),
            )
        })?;

        let path = entry.path();

        if path.is_dir() {
            // RECURSE: go into subdirectories
            collect_files(&path, files)?;
        } else if path.is_file() {
            // Get file metadata
            let metadata = fs::metadata(&path).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"501": format!("Failed to read metadata: {}", e)})),
                )
            })?;

            // Get the relative path from uploads/
            // e.g., "uploads/subfolder/photo.jpg" → "subfolder/photo.jpg"
            let relative_path = path
                .strip_prefix("uploads/")
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // Get the filename for display
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Detect MIME type from extension
            let content_type = mime_guess::from_path(&filename)
                .first()
                .map(|m| m.to_string());

            // Get the modification time as a readable format
            let modified = metadata
                .modified()
                .ok()
                .map(|time| {
                    let duration = time
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    // Convert to chrono DateTime for consistent formatting
                    let naive = chrono::DateTime::from_timestamp(
                        duration.as_secs() as i64,
                        duration.subsec_nanos(),
                    )
                    .unwrap_or_default();
                    naive
                })
                .unwrap_or_default();

            files.push(FileInfo {
                filename: relative_path,     // "subfolder/photo.jpg"
                original_name: filename,     // "photo.jpg"
                size: metadata.len(),
                uploaded_at: modified,
                content_type,
            });
        }
        // Skip symlinks and other non-file, non-directory entries
    }

    Ok(())
}