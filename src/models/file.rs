use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub filename: String,
    pub original_name: String,
    pub size: u64,           // bytes
    pub uploaded_at: DateTime<Utc>,
    pub content_type: Option<String>,
}