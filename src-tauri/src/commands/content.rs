//! Reading an instance's on-disk content for the instance page tabs:
//! screenshots, worlds (saves), resource packs, datapacks and shaders.
//!
//! Formats (see the Minecraft wiki):
//!   - screenshots: plain image files in `screenshots/`
//!   - worlds:      `saves/<dir>/level.dat` (gzipped NBT) + optional `icon.png`
//!   - packs:       `pack.mcmeta` (JSON: pack.pack_format + pack.description) and
//!                  optional `pack.png`, either as a folder or zipped
//!   - shaders:     zip or folder, no standard manifest — just names
//!
//! Big images (screenshots, world icons) are returned as absolute paths for the
//! frontend to load via the asset protocol; small embedded pack icons are
//! returned inline as `data:` URLs.

use std::io::Read;
use std::path::Path;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::paths;

const IMAGE_EXTS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "gif"];

// ---------- screenshots ----------

#[derive(Serialize)]
pub struct ScreenshotInfo {
    name: String,
    path: String,
    modified: u64,
}

#[tauri::command]
pub fn list_screenshots(id: String) -> Result<Vec<ScreenshotInfo>, String> {
    let dir = paths::instance_game_dir(&id).join("screenshots");
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(out),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || !has_ext(&path, &IMAGE_EXTS) {
            continue;
        }
        out.push(ScreenshotInfo {
            name: file_name(&path),
            path: path.to_string_lossy().into_owned(),
            modified: modified_millis(&entry),
        });
    }
    // Newest first.
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

// ---------- worlds ----------

#[derive(Serialize)]
pub struct WorldInfo {
    folder: String,
    name: String,
    icon_path: Option<String>,
    last_played: Option<i64>,
    version: Option<String>,
    game_mode: Option<String>,
}

#[derive(Deserialize)]
struct LevelDat {
    #[serde(rename = "Data")]
    data: LevelData,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LevelData {
    #[serde(rename = "LevelName")]
    level_name: Option<String>,
    #[serde(rename = "LastPlayed")]
    last_played: Option<i64>,
    #[serde(rename = "Version")]
    version: Option<LevelVersion>,
    #[serde(rename = "GameType")]
    game_type: Option<i32>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LevelVersion {
    #[serde(rename = "Name")]
    name: Option<String>,
}

#[tauri::command]
pub fn list_worlds(id: String) -> Result<Vec<WorldInfo>, String> {
    let dir = paths::instance_game_dir(&id).join("saves");
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(out),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder = file_name(&path);
        let data = read_level_dat(&path.join("level.dat"));
        let icon = path.join("icon.png");
        out.push(WorldInfo {
            name: data
                .as_ref()
                .and_then(|d| d.level_name.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| folder.clone()),
            last_played: data.as_ref().and_then(|d| d.last_played),
            version: data.as_ref().and_then(|d| d.version.as_ref()).and_then(|v| v.name.clone()),
            game_mode: data.as_ref().and_then(|d| d.game_type).map(game_mode_name),
            icon_path: icon.is_file().then(|| icon.to_string_lossy().into_owned()),
            folder,
        });
    }
    out.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    Ok(out)
}

fn read_level_dat(path: &Path) -> Option<LevelData> {
    let bytes = std::fs::read(path).ok()?;
    // level.dat is gzip-compressed NBT.
    let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw).ok()?;
    fastnbt::from_bytes::<LevelDat>(&raw).ok().map(|l| l.data)
}

fn game_mode_name(t: i32) -> String {
    match t {
        0 => "survival",
        1 => "creative",
        2 => "adventure",
        3 => "spectator",
        _ => "unknown",
    }
    .to_string()
}

// ---------- resource packs / datapacks ----------

#[derive(Serialize)]
pub struct PackInfo {
    /// Display name (folder name, or file name without `.zip`).
    name: String,
    /// Raw file/folder name on disk (used for deletion).
    filename: String,
    description: Option<String>,
    pack_format: Option<i64>,
    /// Small icon as a `data:` URL (read from `pack.png`), if present.
    icon: Option<String>,
    is_zip: bool,
}

#[tauri::command]
pub fn list_resource_packs(id: String) -> Result<Vec<PackInfo>, String> {
    list_packs(&paths::instance_game_dir(&id).join("resourcepacks"))
}

#[tauri::command]
pub fn list_data_packs(id: String) -> Result<Vec<PackInfo>, String> {
    list_packs(&paths::instance_game_dir(&id).join("datapacks"))
}

fn list_packs(dir: &Path) -> Result<Vec<PackInfo>, String> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(out),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(pack) = read_pack(&path) {
            out.push(pack);
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

fn read_pack(path: &Path) -> Option<PackInfo> {
    if path.is_dir() {
        let mcmeta = std::fs::read_to_string(path.join("pack.mcmeta")).ok();
        let icon = std::fs::read(path.join("pack.png")).ok().map(png_data_url);
        let (description, pack_format) = parse_mcmeta(mcmeta.as_deref());
        Some(PackInfo {
            name: file_name(path),
            filename: file_name(path),
            description,
            pack_format,
            icon,
            is_zip: false,
        })
    } else if path.is_file() && has_ext(path, &["zip"]) {
        let (mcmeta, icon) = read_zip_pack_assets(path);
        let (description, pack_format) = parse_mcmeta(mcmeta.as_deref());
        Some(PackInfo {
            name: strip_zip(&file_name(path)),
            filename: file_name(path),
            description,
            pack_format,
            icon: icon.map(png_data_url),
            is_zip: true,
        })
    } else {
        None
    }
}

/// Reads `pack.mcmeta` (as text) and `pack.png` (as bytes) from a zip pack.
fn read_zip_pack_assets(path: &Path) -> (Option<String>, Option<Vec<u8>>) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, None),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return (None, None),
    };

    let mcmeta = archive.by_name("pack.mcmeta").ok().and_then(|mut f| {
        let mut s = String::new();
        f.read_to_string(&mut s).ok().map(|_| s)
    });
    let icon = archive.by_name("pack.png").ok().and_then(|mut f| {
        let mut b = Vec::new();
        f.read_to_end(&mut b).ok().map(|_| b)
    });
    (mcmeta, icon)
}

/// Pulls `pack.pack_format` and a flattened `pack.description` out of a mcmeta.
fn parse_mcmeta(text: Option<&str>) -> (Option<String>, Option<i64>) {
    let Some(text) = text else {
        return (None, None);
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(text) else {
        return (None, None);
    };
    let pack = &v["pack"];
    let pack_format = pack["pack_format"].as_i64();
    let description = match &pack["description"] {
        serde_json::Value::Null => None,
        d => {
            let s = flatten_text(d);
            (!s.trim().is_empty()).then_some(s)
        }
    };
    (description, pack_format)
}

/// Flattens a Minecraft text component (string | object | array) to plain text.
fn flatten_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr.iter().map(flatten_text).collect(),
        serde_json::Value::Object(obj) => {
            let mut s = obj.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
            if let Some(extra) = obj.get("extra") {
                s.push_str(&flatten_text(extra));
            }
            s
        }
        _ => String::new(),
    }
}

// ---------- servers (servers.dat) ----------

#[derive(Serialize)]
pub struct ServerInfo {
    name: String,
    ip: String,
    /// Favicon as a `data:` URL, if the server set one.
    icon: Option<String>,
    hidden: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ServersFile {
    servers: Vec<ServerEntry>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ServerEntry {
    name: Option<String>,
    ip: Option<String>,
    /// base64-encoded PNG (no `data:` prefix).
    icon: Option<String>,
    hidden: Option<i8>,
}

/// Reads the saved multiplayer server list (`servers.dat`, uncompressed NBT).
#[tauri::command]
pub fn list_servers(id: String) -> Result<Vec<ServerInfo>, String> {
    let path = paths::instance_game_dir(&id).join("servers.dat");
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return Ok(Vec::new()),
    };
    let parsed: ServersFile = fastnbt::from_bytes(&bytes).map_err(|e| format!("parse servers.dat: {e}"))?;
    Ok(parsed
        .servers
        .into_iter()
        .map(|s| ServerInfo {
            name: s.name.unwrap_or_default(),
            ip: s.ip.unwrap_or_default(),
            icon: s.icon.map(|b64| format!("data:image/png;base64,{b64}")),
            hidden: s.hidden.unwrap_or(0) != 0,
        })
        .collect())
}

/// Appends a server to `servers.dat`, preserving existing entries/fields.
#[tauri::command]
pub fn add_server(id: String, name: String, ip: String) -> Result<(), String> {
    use fastnbt::Value;
    use std::collections::HashMap;

    let game_dir = paths::instance_game_dir(&id);
    let path = game_dir.join("servers.dat");

    let mut root: Value = match std::fs::read(&path) {
        Ok(bytes) => fastnbt::from_bytes(&bytes).map_err(|e| format!("parse servers.dat: {e}"))?,
        Err(_) => Value::Compound(HashMap::new()),
    };

    let Value::Compound(map) = &mut root else { return Err("invalid servers.dat".into()) };
    let servers = map.entry("servers".to_string()).or_insert_with(|| Value::List(Vec::new()));
    let Value::List(list) = servers else { return Err("invalid servers list".into()) };

    let mut entry = HashMap::new();
    entry.insert("name".to_string(), Value::String(name));
    entry.insert("ip".to_string(), Value::String(ip));
    list.push(Value::Compound(entry));

    std::fs::create_dir_all(&game_dir).map_err(|e| e.to_string())?;
    let bytes = fastnbt::to_bytes(&root).map_err(|e| format!("encode servers.dat: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("write servers.dat: {e}"))
}

/// Removes the server at `index` (matching `list_servers` order) from `servers.dat`.
#[tauri::command]
pub fn delete_server(id: String, index: usize) -> Result<(), String> {
    use fastnbt::Value;

    let path = paths::instance_game_dir(&id).join("servers.dat");
    let bytes = std::fs::read(&path).map_err(|e| format!("read servers.dat: {e}"))?;
    let mut root: Value = fastnbt::from_bytes(&bytes).map_err(|e| format!("parse servers.dat: {e}"))?;

    let Value::Compound(map) = &mut root else { return Err("invalid servers.dat".into()) };
    let Some(Value::List(list)) = map.get_mut("servers") else { return Ok(()) };
    if index < list.len() {
        list.remove(index);
    }

    let out = fastnbt::to_bytes(&root).map_err(|e| format!("encode servers.dat: {e}"))?;
    std::fs::write(&path, out).map_err(|e| format!("write servers.dat: {e}"))
}

// ---------- shaders ----------

#[derive(Serialize)]
pub struct ShaderInfo {
    name: String,
    filename: String,
    is_zip: bool,
}

#[tauri::command]
pub fn list_shaders(id: String) -> Result<Vec<ShaderInfo>, String> {
    let dir = paths::instance_game_dir(&id).join("shaderpacks");
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(out),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.push(ShaderInfo { name: file_name(&path), filename: file_name(&path), is_zip: false });
        } else if path.is_file() && has_ext(&path, &["zip"]) {
            out.push(ShaderInfo { name: strip_zip(&file_name(&path)), filename: file_name(&path), is_zip: true });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

// ---------- delete content (packs/shaders/datapacks) ----------

/// Deletes a resource pack / shader / datapack (file or folder) by raw filename.
#[tauri::command]
pub fn delete_content(id: String, kind: String, filename: String) -> Result<(), String> {
    let folder = match kind.as_str() {
        "resourcepack" => "resourcepacks",
        "shader" => "shaderpacks",
        "datapack" => "datapacks",
        other => return Err(format!("unknown content kind: {other}")),
    };
    // Defend against path traversal — only a bare file name is allowed.
    let safe = std::path::Path::new(&filename)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or("invalid filename")?;
    let target = paths::instance_game_dir(&id).join(folder).join(&safe);
    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| format!("delete: {e}"))
    } else if target.exists() {
        std::fs::remove_file(&target).map_err(|e| format!("delete: {e}"))
    } else {
        Ok(())
    }
}

// ---------- saved logs (logs/ + crash-reports/) ----------

#[derive(Serialize)]
pub struct LogFile {
    name: String,
    /// "latest" | "log" | "archived" | "crash"
    kind: String,
    /// Path relative to the game dir (e.g. "logs/latest.log").
    rel: String,
    modified: u64,
    size: u64,
}

/// Lists saved game logs: `logs/latest.log`, rotated `logs/*.log(.gz)` and
/// `crash-reports/*.txt`. Newest first.
#[tauri::command]
pub fn list_log_files(id: String) -> Result<Vec<LogFile>, String> {
    let game = paths::instance_game_dir(&id);
    let mut out = Vec::new();

    if let Ok(entries) = std::fs::read_dir(game.join("logs")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = file_name(&path);
            let lower = name.to_lowercase();
            let kind = if lower == "latest.log" {
                "latest"
            } else if lower.ends_with(".gz") {
                "archived"
            } else if lower.ends_with(".log") {
                "log"
            } else {
                continue;
            };
            out.push(LogFile {
                rel: format!("logs/{name}"),
                name,
                kind: kind.into(),
                modified: modified_millis(&entry),
                size: entry.metadata().map(|m| m.len()).unwrap_or(0),
            });
        }
    }

    if let Ok(entries) = std::fs::read_dir(game.join("crash-reports")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && has_ext(&path, &["txt", "log"]) {
                let name = file_name(&path);
                out.push(LogFile {
                    rel: format!("crash-reports/{name}"),
                    name,
                    kind: "crash".into(),
                    modified: modified_millis(&entry),
                    size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                });
            }
        }
    }

    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

/// Reads a saved log file's text (decompressing `.gz`). Capped to the last ~1 MB.
#[tauri::command]
pub fn read_log_file(id: String, rel: String) -> Result<String, String> {
    let text = read_log_text(&id, &rel)?;
    const MAX: usize = 1_000_000;
    if text.len() > MAX {
        Ok(text[text.len() - MAX..].to_string())
    } else {
        Ok(text)
    }
}

/// Reads and decodes a saved log file's full text (gzip-aware), with the same
/// path validation as [`read_log_file`].
fn read_log_text(id: &str, rel: &str) -> Result<String, String> {
    if (!rel.starts_with("logs/") && !rel.starts_with("crash-reports/")) || rel.contains("..") {
        return Err("invalid log path".into());
    }
    let path = paths::instance_game_dir(id).join(rel);
    let bytes = std::fs::read(&path).map_err(|e| format!("read log: {e}"))?;

    if rel.ends_with(".gz") {
        let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
        let mut s = String::new();
        decoder.read_to_string(&mut s).map_err(|e| format!("gunzip: {e}"))?;
        Ok(s)
    } else {
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// Result of sharing a log to mclo.gs (https://api.mclo.gs/).
#[derive(Serialize)]
pub struct MclogsPaste {
    pub id: String,
    pub url: String,
    pub raw: String,
}

#[derive(Deserialize)]
struct MclogsResponse {
    success: bool,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    raw: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// Uploads a saved log file to mclo.gs and returns the shareable links.
/// mclo.gs caps logs at 10 MiB / 25,000 lines, so the content is trimmed to fit.
#[tauri::command]
pub async fn upload_log_to_mclogs(id: String, rel: String) -> Result<MclogsPaste, String> {
    let mut content = read_log_text(&id, &rel)?;

    // Respect mclo.gs limits: keep the most recent lines/bytes.
    const MAX_LINES: usize = 25_000;
    const MAX_BYTES: usize = 10 * 1024 * 1024;
    let line_count = content.lines().count();
    if line_count > MAX_LINES {
        let skip = line_count - MAX_LINES;
        content = content.lines().skip(skip).collect::<Vec<_>>().join("\n");
    }
    if content.len() > MAX_BYTES {
        content = content[content.len() - MAX_BYTES..].to_string();
    }

    let resp = reqwest::Client::builder()
        .user_agent("MakoLauncher")
        .build()
        .map_err(|e| e.to_string())?
        .post("https://api.mclo.gs/1/log")
        .form(&[("content", content.as_str()), ("source", "MakoLauncher")])
        .send()
        .await
        .map_err(|e| format!("upload failed: {e}"))?;

    let body: MclogsResponse = resp.json().await.map_err(|e| format!("bad response: {e}"))?;
    if !body.success {
        return Err(body.error.unwrap_or_else(|| "mclo.gs rejected the log".into()));
    }
    Ok(MclogsPaste {
        id: body.id.unwrap_or_default(),
        url: body.url.unwrap_or_default(),
        raw: body.raw.unwrap_or_default(),
    })
}

// ---------- helpers ----------

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

fn strip_zip(name: &str) -> String {
    name.strip_suffix(".zip").unwrap_or(name).to_string()
}

fn has_ext(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| exts.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn modified_millis(entry: &std::fs::DirEntry) -> u64 {
    entry
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn png_data_url(bytes: Vec<u8>) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    format!("data:image/png;base64,{b64}")
}
