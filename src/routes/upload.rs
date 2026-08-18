
use axum::http::StatusCode;
use axum::extract::Multipart;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;
use crate::models::FileInfo;
use chrono::Utc;
use super::utils::validate_file_name::validate_filename;


pub async fn upload(
    mut multipart: Multipart,
) -> Result<Json<FileInfo>, (StatusCode, Json<Value>)> {


    // First we read the Header

    let field = multipart
        .next_field()
        .await
        .map_err(|e| {
            (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()})))
        })?
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"error": "No file field found"}))))?;

    let original_name = field
        .file_name()
        .unwrap_or("unnamed")
        .to_string();

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    println!("Receiving file: {} (type: {})", original_name, content_type);

    validate_filename(&original_name)?;

    // Second we read the Data

    let data = field.
        bytes().
        await.
        map_err(|e| {(StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()})))
    })?;

    println!("Received {} bytes", data.len());

    // Third we save the file to disk

    let unique_name = format!("{}-{}", Uuid::new_v4(), original_name);
    let save_path = format!("uploads/{}", unique_name);

    tokio::fs::write(&save_path, &data).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?;

    println!("Saved to: {}", save_path);
     

    // Fourth we create a unique Json

    Ok(Json(FileInfo {
        filename: unique_name,
        original_name,
        size: data.len() as u64,
        uploaded_at: Utc::now(),
        content_type: Some(content_type),
    }))
}
