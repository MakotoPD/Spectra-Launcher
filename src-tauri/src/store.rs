//! Tiny helpers for reading/writing the launcher's JSON files atomically-ish.
//! Commands return `Result<_, String>` to Tauri, so everything maps errors to String.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;

/// Reads and parses a JSON file, returning `Ok(None)` if it doesn't exist.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("parse {}: {e}", path.display()))?;
    Ok(Some(value))
}

/// Serializes `value` (pretty) to `path`, creating parent dirs as needed.
///
/// The write is atomic: the JSON goes to a temp file in the same directory and is
/// then renamed over the target. A crash or two concurrent writers can therefore
/// never leave a half-written (corrupt) file — the reader always sees either the
/// old or the new complete content.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;

    let json = serde_json::to_vec_pretty(value).map_err(|e| format!("serialize: {e}"))?;

    // Unique temp name so concurrent writers to the same target don't clobber
    // each other's temp file before the rename.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = parent.join(format!(".{file_name}.{stamp}.{:x}.tmp", std::process::id()));

    std::fs::write(&tmp, &json).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    // rename is atomic on the same filesystem and replaces the destination on
    // both Windows (MoveFileEx) and Unix.
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("rename {} -> {}: {e}", tmp.display(), path.display()));
    }
    Ok(())
}
