use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};

type ValidationError = (StatusCode, Json<Value>);

fn invalid() -> ValidationError {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "Invalid file name"})),
    )
}

/// True if `path`'s components escape upward (`..`) or don't start with a
/// plain relative segment (e.g. an absolute path).
fn has_unsafe_components(path: &Path) -> bool {
    let components: Vec<_> = path.components().collect();
    let starts_normal = components
        .first()
        .map_or(false, |c| matches!(c, Component::Normal(_)));
    !starts_normal || components.iter().any(|c| matches!(c, Component::ParentDir))
}

/// Validates that a client-supplied name is a single plain path segment —
/// no `/`, no `..`, not absolute. For names that get embedded into a new
/// path rather than joined directly (e.g. upload's UUID-prefixed filename).
pub fn validate_filename(name: &str) -> Result<(), ValidationError> {
    let path = Path::new(name);
    if path.components().count() != 1 || has_unsafe_components(path) {
        return Err(invalid());
    }
    Ok(())
}

/// Joins `filename` onto `base_dir` and rejects if the result would escape it.
pub fn safe_join(base_dir: &Path, filename: &str) -> Result<PathBuf, ValidationError> {
    let full_path = base_dir.join(filename);
    if has_unsafe_components(&full_path) {
        return Err(invalid());
    }
    Ok(full_path)
}
