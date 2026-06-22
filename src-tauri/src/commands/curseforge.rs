//! CurseForge (Eternal/Core API) integration: search, version listing, project
//! info, mod install (with dependencies) and modpack (.zip) install as a new
//! instance. The API key is embedded at build time from `.env` (see build.rs).
//!
//! Results are normalized to the SAME shapes the Modrinth browser already
//! consumes (hits/versions/project), so the UI can switch providers with a flag.
//!
//! Reference: https://docs.curseforge.com/  (and PrismLauncher's Flame impl).

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::commands::instances;
use crate::commands::modrinth::{read_content_index, write_content_index, InstalledItem};
use crate::models::{Instance, Loader};
use crate::paths;

const API: &str = "https://api.curseforge.com/v1";
const GAME_ID: i64 = 432;
// Injected at build time from .env (build.rs). Empty → CurseForge disabled.
const CF_API_KEY: &str = env!("CURSEFORGE_API_KEY");

/// Whether a CurseForge API key was embedded at build time.
#[tauri::command]
pub fn cf_enabled() -> bool {
    !CF_API_KEY.trim().is_empty()
}

fn http() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(v) = reqwest::header::HeaderValue::from_str(CF_API_KEY) {
            headers.insert("x-api-key", v);
        }
        reqwest::Client::builder()
            .user_agent("MakotoPD/Mako-Launcher/0.1.0")
            .default_headers(headers)
            .build()
            .expect("build curseforge client")
    })
}

/// Sends a request, retrying once on HTTP 429 after the `Retry-After` delay.
async fn send(req: reqwest::RequestBuilder) -> Result<reqwest::Response, String> {
    if CF_API_KEY.trim().is_empty() {
        return Err("CurseForge is not configured (no API key)".into());
    }
    let mut attempt = 0u32;
    loop {
        let r = req.try_clone().ok_or("request is not retryable")?;
        let resp = r.send().await.map_err(|e| e.to_string())?;
        if resp.status().as_u16() != 429 || attempt >= 3 {
            return Ok(resp);
        }
        let wait = resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or((1u64 << attempt).min(8));
        tokio::time::sleep(std::time::Duration::from_secs(wait.clamp(1, 8))).await;
        attempt += 1;
    }
}

// ===== shared id mappings =====

fn class_id(kind: &str) -> i64 {
    match kind {
        "modpack" => 4471,
        "resourcepack" => 12,
        "shader" => 6552,
        "datapack" => 6945,
        _ => 6, // mod
    }
}

/// CurseForge modLoaderType numeric ids.
fn loader_id(loader: &str) -> Option<u8> {
    match loader {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}

/// Maps the Modrinth-style sort index to a CurseForge sortField.
fn sort_field(index: &str) -> u8 {
    match index {
        "downloads" => 6,  // TotalDownloads
        "newest" | "updated" => 3, // LastUpdated
        "follows" => 2,    // Popularity (no "follows" on CF)
        _ => 2,            // relevance → Popularity
    }
}

// ===== CF raw response types (only the fields we use) =====

#[derive(Deserialize)]
struct CfLogo {
    #[serde(default)]
    thumbnail_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Deserialize)]
struct CfAuthor {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct CfCategoryRef {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct CfFileIndex {
    #[serde(default)]
    game_version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMod {
    id: i64,
    name: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    download_count: f64,
    #[serde(default)]
    logo: Option<CfLogo>,
    #[serde(default)]
    authors: Vec<CfAuthor>,
    #[serde(default)]
    categories: Vec<CfCategoryRef>,
    #[serde(default)]
    latest_files_indexes: Vec<CfFileIndex>,
    #[serde(default)]
    screenshots: Vec<CfScreenshot>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfScreenshot {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    thumbnail_url: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct CfHashEntry {
    value: String,
    algo: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfDependency {
    mod_id: i64,
    relation_type: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFile {
    id: i64,
    #[serde(default)]
    mod_id: i64,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    file_date: String,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    file_length: u64,
    #[serde(default)]
    file_fingerprint: u64,
    #[serde(default)]
    download_count: f64,
    #[serde(default = "default_release_type")]
    release_type: i64,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    hashes: Vec<CfHashEntry>,
    #[serde(default)]
    dependencies: Vec<CfDependency>,
}

fn default_release_type() -> i64 {
    1
}

#[derive(Deserialize)]
struct CfPagination {
    #[serde(default)]
    total_count: u64,
}

#[derive(Deserialize)]
struct CfListResponse<T> {
    data: Vec<T>,
    #[serde(default)]
    pagination: Option<CfPagination>,
}

#[derive(Deserialize)]
struct CfDataResponse<T> {
    data: T,
}

// ===== normalized output (mirrors the Modrinth frontend types) =====

#[derive(Serialize)]
pub struct Hit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    author: String,
    project_type: String,
    downloads: u64,
    follows: u64,
    icon_url: Option<String>,
    categories: Vec<String>,
    versions: Vec<String>,
    client_side: Option<String>,
    server_side: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    hits: Vec<Hit>,
    total_hits: u64,
    offset: u64,
    limit: u64,
}

#[derive(Serialize)]
pub struct VersionFileHashes {
    sha1: Option<String>,
    sha512: Option<String>,
}

#[derive(Serialize)]
pub struct VersionFile {
    url: String,
    filename: String,
    primary: bool,
    size: u64,
    hashes: VersionFileHashes,
}

#[derive(Serialize)]
pub struct Dependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

#[derive(Serialize)]
pub struct Version {
    id: String,
    project_id: String,
    name: String,
    version_number: String,
    version_type: String,
    loaders: Vec<String>,
    game_versions: Vec<String>,
    downloads: u64,
    date_published: String,
    files: Vec<VersionFile>,
    dependencies: Vec<Dependency>,
}

#[derive(Serialize)]
pub struct GalleryItem {
    url: String,
    raw_url: Option<String>,
    title: Option<String>,
    description: Option<String>,
    featured: bool,
}

#[derive(Serialize)]
pub struct ProjectFull {
    id: String,
    title: String,
    description: String,
    body: String,
    icon_url: Option<String>,
    gallery: Vec<GalleryItem>,
}

// ===== conversions =====

fn logo_url(logo: &Option<CfLogo>) -> Option<String> {
    logo.as_ref().and_then(|l| l.thumbnail_url.clone().or_else(|| l.url.clone()))
}

fn mod_to_hit(m: CfMod, kind: &str) -> Hit {
    let versions: Vec<String> = {
        let mut v: Vec<String> = m
            .latest_files_indexes
            .iter()
            .map(|i| i.game_version.clone())
            .filter(|s| s.contains('.'))
            .collect();
        v.sort();
        v.dedup();
        v
    };
    Hit {
        project_id: m.id.to_string(),
        slug: m.slug,
        title: m.name,
        description: m.summary,
        author: m.authors.first().map(|a| a.name.clone()).unwrap_or_default(),
        project_type: kind.to_string(),
        downloads: m.download_count as u64,
        follows: 0,
        icon_url: logo_url(&m.logo),
        categories: m.categories.into_iter().map(|c| c.name).collect(),
        versions,
        client_side: None,
        server_side: None,
    }
}

/// Splits a CF file's `gameVersions` (which mixes MC versions and loader names).
fn split_game_versions(raw: &[String]) -> (Vec<String>, Vec<String>) {
    let mut mc = Vec::new();
    let mut loaders = Vec::new();
    for v in raw {
        let lower = v.to_lowercase();
        match lower.as_str() {
            "forge" | "fabric" | "quilt" | "neoforge" => loaders.push(lower),
            _ if v.contains('.') => mc.push(v.clone()),
            _ => {}
        }
    }
    (mc, loaders)
}

fn file_to_version(f: CfFile) -> Version {
    let (game_versions, loaders) = split_game_versions(&f.game_versions);
    let version_type = match f.release_type {
        2 => "beta",
        3 => "alpha",
        _ => "release",
    }
    .to_string();
    let sha1 = f
        .hashes
        .iter()
        .find(|h| h.algo == 1)
        .map(|h| h.value.clone());
    let deps = f
        .dependencies
        .iter()
        .filter(|d| d.relation_type == 3) // RequiredDependency
        .map(|d| Dependency {
            project_id: Some(d.mod_id.to_string()),
            version_id: None,
            dependency_type: "required".to_string(),
        })
        .collect();
    let version_number = if f.display_name.is_empty() { f.file_name.clone() } else { f.display_name.clone() };
    Version {
        id: f.id.to_string(),
        project_id: f.mod_id.to_string(),
        name: f.display_name,
        version_number,
        version_type,
        loaders,
        game_versions,
        downloads: f.download_count as u64,
        date_published: f.file_date,
        files: vec![VersionFile {
            url: f.download_url.unwrap_or_default(),
            filename: f.file_name,
            primary: true,
            size: f.file_length,
            hashes: VersionFileHashes { sha1, sha512: None },
        }],
        dependencies: deps,
    }
}

// ===== commands: browse =====

#[derive(Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub project_type: String,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub index: String,
    #[serde(default)]
    pub offset: u64,
    #[serde(default)]
    pub limit: u64,
}

#[tauri::command]
pub async fn curseforge_search(params: SearchParams) -> Result<SearchResponse, String> {
    let kind = params.project_type.clone();
    let page_size = if params.limit == 0 { 20 } else { params.limit.min(50) };

    let mut query: Vec<(String, String)> = vec![
        ("gameId".into(), GAME_ID.to_string()),
        ("classId".into(), class_id(&kind).to_string()),
        ("index".into(), params.offset.to_string()),
        ("pageSize".into(), page_size.to_string()),
        ("sortField".into(), sort_field(&params.index).to_string()),
        ("sortOrder".into(), "desc".into()),
    ];
    if !params.query.trim().is_empty() {
        query.push(("searchFilter".into(), params.query.clone()));
    }
    if let Some(v) = params.game_versions.first() {
        query.push(("gameVersion".into(), v.clone()));
    }
    // Single loader filter (CF accepts modLoaderType as a single value).
    if let Some(id) = params.loaders.iter().find_map(|l| loader_id(l)) {
        query.push(("modLoaderType".into(), id.to_string()));
    }
    // Map selected category names → CurseForge numeric ids.
    if !params.categories.is_empty() {
        let cats = categories_for(class_id(&kind)).await;
        let ids: Vec<String> = params
            .categories
            .iter()
            .filter_map(|c| cats.iter().find(|(n, _)| n.eq_ignore_ascii_case(c)).map(|(_, id)| id.to_string()))
            .collect();
        if !ids.is_empty() {
            query.push(("categoryIds".into(), format!("[{}]", ids.join(","))));
        }
    }

    let resp = send(http().get(format!("{API}/mods/search")).query(&query)).await?;
    if !resp.status().is_success() {
        return Err(format!("CurseForge search failed: {}", resp.status()));
    }
    let parsed: CfListResponse<CfMod> = resp.json().await.map_err(|e| e.to_string())?;
    let total_hits = parsed.pagination.map(|p| p.total_count).unwrap_or(0);
    let hits = parsed.data.into_iter().map(|m| mod_to_hit(m, &kind)).collect();
    Ok(SearchResponse { hits, total_hits, offset: params.offset, limit: page_size })
}

#[tauri::command]
pub async fn curseforge_versions(
    project_id: String,
    loaders: Option<Vec<String>>,
    game_versions: Option<Vec<String>>,
) -> Result<Vec<Version>, String> {
    let mut query: Vec<(String, String)> = vec![("pageSize".into(), "50".into()), ("index".into(), "0".into())];
    if let Some(v) = game_versions.and_then(|g| g.into_iter().next()) {
        query.push(("gameVersion".into(), v));
    }
    if let Some(id) = loaders.and_then(|l| l.iter().find_map(|x| loader_id(x))) {
        query.push(("modLoaderType".into(), id.to_string()));
    }

    let resp = send(http().get(format!("{API}/mods/{project_id}/files")).query(&query)).await?;
    if !resp.status().is_success() {
        return Err(format!("CurseForge versions failed: {}", resp.status()));
    }
    let parsed: CfListResponse<CfFile> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(parsed.data.into_iter().map(file_to_version).collect())
}

#[tauri::command]
pub async fn curseforge_project(id: String) -> Result<ProjectFull, String> {
    // Base project + the long HTML description (separate endpoint).
    let resp = send(http().get(format!("{API}/mods/{id}"))).await?;
    if !resp.status().is_success() {
        return Err(format!("CurseForge project failed: {}", resp.status()));
    }
    let m: CfDataResponse<CfMod> = resp.json().await.map_err(|e| e.to_string())?;
    let m = m.data;

    let body = match send(http().get(format!("{API}/mods/{id}/description"))).await {
        Ok(r) if r.status().is_success() => r
            .json::<CfDataResponse<String>>()
            .await
            .map(|d| d.data)
            .unwrap_or_default(),
        _ => String::new(),
    };

    let gallery = m
        .screenshots
        .into_iter()
        .filter_map(|s| {
            let raw = s.url.clone();
            let thumb = s.thumbnail_url.or_else(|| s.url.clone());
            thumb.map(|url| GalleryItem {
                url,
                raw_url: raw,
                title: s.title,
                description: s.description,
                featured: false,
            })
        })
        .collect();

    Ok(ProjectFull {
        id: m.id.to_string(),
        title: m.name,
        description: m.summary,
        body,
        icon_url: logo_url(&m.logo),
        gallery,
    })
}

#[derive(Serialize, Clone)]
pub struct Category {
    name: String,
    /// CurseForge's numeric category id (as string), used as the filter value.
    header: String,
}

#[derive(Deserialize)]
struct CfCategory {
    id: i64,
    name: String,
}

/// name → id pairs per classId, fetched once and cached for the session.
fn category_cache() -> &'static std::sync::Mutex<HashMap<i64, Vec<(String, i64)>>> {
    static C: std::sync::OnceLock<std::sync::Mutex<HashMap<i64, Vec<(String, i64)>>>> =
        std::sync::OnceLock::new();
    C.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

async fn categories_for(class_id: i64) -> Vec<(String, i64)> {
    if let Ok(cache) = category_cache().lock() {
        if let Some(v) = cache.get(&class_id) {
            return v.clone();
        }
    }
    let fetched = match send(
        http()
            .get(format!("{API}/categories"))
            .query(&[("gameId", GAME_ID.to_string()), ("classId", class_id.to_string())]),
    )
    .await
    {
        Ok(r) if r.status().is_success() => r
            .json::<CfListResponse<CfCategory>>()
            .await
            .map(|p| p.data.into_iter().map(|c| (c.name, c.id)).collect::<Vec<_>>())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    if let Ok(mut cache) = category_cache().lock() {
        cache.insert(class_id, fetched.clone());
    }
    fetched
}

#[tauri::command]
pub async fn curseforge_categories(project_type: String) -> Result<Vec<Category>, String> {
    let cats = categories_for(class_id(&project_type)).await;
    Ok(cats.into_iter().map(|(name, id)| Category { name, header: id.to_string() }).collect())
}

// ===== install: mods (+ required deps) into an instance =====

/// A mod that couldn't be downloaded because its author disabled third-party
/// distribution (CurseForge returns no download URL). The user must grab it
/// manually from the project page.
#[derive(Serialize, Clone)]
pub struct BlockedMod {
    name: String,
    file_id: String,
    url: String,
}

#[derive(Serialize)]
pub struct CfInstallResult {
    added: Vec<InstalledItem>,
    blocked: Vec<BlockedMod>,
}

#[tauri::command]
pub async fn curseforge_install_with_deps(
    instance_id: String,
    project_id: String,
    file_id: String,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<CfInstallResult, String> {
    let mut index = read_content_index(&instance_id);
    let mut visited: std::collections::HashSet<String> =
        index.items.iter().map(|i| i.project_id.clone()).collect();
    let mut added = Vec::new();
    let mut blocked = Vec::new();

    install_rec(
        &instance_id,
        &project_id,
        &file_id,
        false,
        &game_version,
        &loader,
        &mut visited,
        &mut index,
        &mut added,
        &mut blocked,
    )
    .await?;

    write_content_index(&instance_id, &index)?;
    Ok(CfInstallResult { added, blocked })
}

#[allow(clippy::too_many_arguments)]
fn install_rec<'a>(
    instance_id: &'a str,
    project_id: &'a str,
    file_id: &'a str,
    is_dependency: bool,
    game_version: &'a Option<String>,
    loader: &'a Option<String>,
    visited: &'a mut std::collections::HashSet<String>,
    index: &'a mut crate::commands::modrinth::ContentIndex,
    added: &'a mut Vec<InstalledItem>,
    blocked: &'a mut Vec<BlockedMod>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        if is_dependency && visited.contains(project_id) {
            return Ok(());
        }
        let file = fetch_file(project_id, file_id).await?;
        let m = fetch_mod(project_id).await?;
        let (kind, folder) = kind_and_folder(&m);

        let dir = paths::instance_game_dir(instance_id).join(folder);
        std::fs::create_dir_all(&dir).map_err(|e| format!("create {folder}: {e}"))?;

        match &file.download_url {
            Some(url) if !url.is_empty() => {
                let bytes = download(url).await?;
                std::fs::write(dir.join(safe_name(&file.file_name)), &bytes)
                    .map_err(|e| format!("write file: {e}"))?;
            }
            _ => {
                // Author opted out of third-party distribution — report it.
                blocked.push(BlockedMod {
                    name: file.file_name.clone(),
                    file_id: file_id.to_string(),
                    url: format!("https://www.curseforge.com/minecraft/mc-mods/{}/download/{}", m.slug, file_id),
                });
                visited.insert(project_id.to_string());
                return Ok(());
            }
        }

        visited.insert(project_id.to_string());
        let (game_versions, loaders) = split_game_versions(&file.game_versions);
        let item = InstalledItem {
            project_id: project_id.to_string(),
            version_id: file_id.to_string(),
            kind: kind.to_string(),
            name: m.name.clone(),
            filename: file.file_name.clone(),
            version_number: if file.display_name.is_empty() { file.file_name.clone() } else { file.display_name.clone() },
            icon_url: logo_url(&m.logo),
            game_versions,
            loaders,
            dependency: is_dependency,
            installed_at: chrono::Utc::now().to_rfc3339(),
            provider: "curseforge".to_string(),
        };
        index.items.retain(|i| i.project_id != item.project_id);
        index.items.push(item.clone());
        added.push(item);

        // Required dependencies.
        for dep in file.dependencies.iter().filter(|d| d.relation_type == 3) {
            let dep_pid = dep.mod_id.to_string();
            if visited.contains(&dep_pid) {
                continue;
            }
            if let Some(dep_file) = resolve_latest_file(&dep_pid, loader, game_version).await? {
                install_rec(
                    instance_id, &dep_pid, &dep_file, true, game_version, loader, visited, index, added, blocked,
                )
                .await?;
            }
        }
        Ok(())
    })
}

async fn fetch_file(project_id: &str, file_id: &str) -> Result<CfFile, String> {
    let resp = send(http().get(format!("{API}/mods/{project_id}/files/{file_id}"))).await?;
    if !resp.status().is_success() {
        return Err(format!("CurseForge file failed: {}", resp.status()));
    }
    let d: CfDataResponse<CfFile> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(d.data)
}

async fn fetch_mod(project_id: &str) -> Result<CfMod, String> {
    let resp = send(http().get(format!("{API}/mods/{project_id}"))).await?;
    if !resp.status().is_success() {
        return Err(format!("CurseForge mod failed: {}", resp.status()));
    }
    let d: CfDataResponse<CfMod> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(d.data)
}

/// Newest file id for a project matching the loader/game version (best effort).
async fn resolve_latest_file(
    project_id: &str,
    loader: &Option<String>,
    game_version: &Option<String>,
) -> Result<Option<String>, String> {
    let mut query: Vec<(String, String)> = vec![("pageSize".into(), "50".into())];
    if let Some(v) = game_version {
        query.push(("gameVersion".into(), v.clone()));
    }
    if let Some(id) = loader.as_ref().and_then(|l| loader_id(l)) {
        query.push(("modLoaderType".into(), id.to_string()));
    }
    let resp = send(http().get(format!("{API}/mods/{project_id}/files")).query(&query)).await?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let parsed: CfListResponse<CfFile> = resp.json().await.map_err(|e| e.to_string())?;
    // Files come newest-first; take the first.
    Ok(parsed.data.into_iter().next().map(|f| f.id.to_string()))
}

/// Latest file `(id, display_name)` for a project matching loader/game version,
/// used for update checks. CF returns files newest-first. None if nothing found.
pub async fn latest_file(
    project_id: &str,
    loaders: &Option<Vec<String>>,
    game_versions: &Option<Vec<String>>,
) -> Option<(String, String)> {
    let mut query: Vec<(String, String)> = vec![("pageSize".into(), "50".into())];
    if let Some(g) = game_versions.as_ref().and_then(|v| v.first()) {
        query.push(("gameVersion".into(), g.clone()));
    }
    if let Some(id) = loaders.as_ref().and_then(|l| l.iter().find_map(|x| loader_id(x))) {
        query.push(("modLoaderType".into(), id.to_string()));
    }
    let resp = send(http().get(format!("{API}/mods/{project_id}/files")).query(&query)).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let parsed: CfListResponse<CfFile> = resp.json().await.ok()?;
    let f = parsed.data.into_iter().next()?;
    let num = if f.display_name.is_empty() { f.file_name } else { f.display_name };
    Some((f.id.to_string(), num))
}

fn kind_and_folder(m: &CfMod) -> (&'static str, &'static str) {
    // CF doesn't return classId on the mod object reliably across endpoints, so
    // infer from category names; default to mod.
    let cats: Vec<String> = m.categories.iter().map(|c| c.name.to_lowercase()).collect();
    if cats.iter().any(|c| c.contains("resource pack")) {
        ("resourcepack", "resourcepacks")
    } else if cats.iter().any(|c| c.contains("shader")) {
        ("shader", "shaderpacks")
    } else if cats.iter().any(|c| c.contains("data pack")) {
        ("datapack", "datapacks")
    } else {
        ("mod", "mods")
    }
}

// ===== match local mods by CurseForge fingerprint (Murmur2) =====

/// CurseForge's file fingerprint: MurmurHash2 (seed 1) over the file bytes with
/// whitespace bytes (\t \n \r space) stripped. Matches PrismLauncher / the CF app.
fn cf_fingerprint(data: &[u8]) -> u32 {
    const M: u32 = 0x5bd1_e995;
    const R: u32 = 24;
    let filtered: Vec<u8> = data
        .iter()
        .copied()
        .filter(|&b| b != 9 && b != 10 && b != 13 && b != 32)
        .collect();
    let len = filtered.len() as u32;
    let mut h: u32 = 1 ^ len;
    let mut chunks = filtered.chunks_exact(4);
    for c in chunks.by_ref() {
        let mut k = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);
        h = h.wrapping_mul(M);
        h ^= k;
    }
    let rem = chunks.remainder();
    match rem.len() {
        3 => {
            h ^= (rem[2] as u32) << 16;
            h ^= (rem[1] as u32) << 8;
            h ^= rem[0] as u32;
            h = h.wrapping_mul(M);
        }
        2 => {
            h ^= (rem[1] as u32) << 8;
            h ^= rem[0] as u32;
            h = h.wrapping_mul(M);
        }
        1 => {
            h ^= rem[0] as u32;
            h = h.wrapping_mul(M);
        }
        _ => {}
    }
    h ^= h >> 13;
    h = h.wrapping_mul(M);
    h ^= h >> 15;
    h
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFingerprintMatch {
    file: CfFile,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFingerprintData {
    #[serde(default)]
    exact_matches: Vec<CfFingerprintMatch>,
}

#[derive(Deserialize)]
struct CfFingerprintResponse {
    data: CfFingerprintData,
}

/// Matches local jars in `mods/` against CurseForge by file fingerprint and
/// records any matches in the content index (so manually-added CF mods gain
/// update/version features). Returns how many were newly matched.
#[tauri::command]
pub async fn curseforge_match_local(instance_id: String) -> Result<usize, String> {
    let dir = paths::instance_game_dir(&instance_id).join("mods");
    let known: std::collections::HashSet<String> =
        read_content_index(&instance_id).items.iter().map(|i| i.filename.clone()).collect();

    // fingerprint -> base jar filename (only jars not already linked).
    let mut by_fp: HashMap<u64, String> = HashMap::new();
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(0) };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let raw = entry.file_name().to_string_lossy().into_owned();
        let base = raw.strip_suffix(".disabled").unwrap_or(&raw).to_string();
        if !base.ends_with(".jar") || known.contains(&base) {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&path) {
            by_fp.insert(cf_fingerprint(&bytes) as u64, base);
        }
    }
    if by_fp.is_empty() {
        return Ok(0);
    }

    let fingerprints: Vec<u64> = by_fp.keys().copied().collect();
    let resp = send(
        http()
            .post(format!("{API}/fingerprints"))
            .json(&serde_json::json!({ "fingerprints": fingerprints })),
    )
    .await?;
    if !resp.status().is_success() {
        return Err(format!("fingerprints failed: {}", resp.status()));
    }
    let parsed: CfFingerprintResponse = resp.json().await.map_err(|e| e.to_string())?;
    if parsed.data.exact_matches.is_empty() {
        return Ok(0);
    }

    // Fetch mod metadata (name/icon/categories) for the matched projects.
    let mod_ids: Vec<i64> = parsed
        .data
        .exact_matches
        .iter()
        .map(|m| m.file.mod_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let mods = bulk_mods(&mod_ids).await.unwrap_or_default();

    let mut index = read_content_index(&instance_id);
    let mut count = 0usize;
    for m in &parsed.data.exact_matches {
        let file = &m.file;
        let Some(filename) = by_fp.get(&file.file_fingerprint) else { continue };
        let cf_mod = mods.get(&file.mod_id);
        let (kind, _folder) = cf_mod.map(kind_and_folder).unwrap_or(("mod", "mods"));
        let (game_versions, loaders) = split_game_versions(&file.game_versions);
        let item = InstalledItem {
            project_id: file.mod_id.to_string(),
            version_id: file.id.to_string(),
            kind: kind.to_string(),
            name: cf_mod.map(|m| m.name.clone()).unwrap_or_else(|| filename.clone()),
            filename: filename.clone(),
            version_number: if file.display_name.is_empty() { file.file_name.clone() } else { file.display_name.clone() },
            icon_url: cf_mod.and_then(|m| logo_url(&m.logo)),
            game_versions,
            loaders,
            dependency: false,
            installed_at: chrono::Utc::now().to_rfc3339(),
            provider: "curseforge".to_string(),
        };
        index.items.retain(|i| i.project_id != item.project_id && i.filename != item.filename);
        index.items.push(item);
        count += 1;
    }
    write_content_index(&instance_id, &index)?;
    Ok(count)
}

/// Matches a single local jar against CurseForge by fingerprint and records it
/// if found. Returns whether a match was recorded.
#[tauri::command]
pub async fn curseforge_match_file(instance_id: String, filename: String) -> Result<bool, String> {
    let dir = paths::instance_game_dir(&instance_id).join("mods");
    let path = if dir.join(&filename).is_file() {
        dir.join(&filename)
    } else {
        dir.join(format!("{filename}.disabled"))
    };
    let bytes = std::fs::read(&path).map_err(|e| format!("read {filename}: {e}"))?;
    let fp = cf_fingerprint(&bytes) as u64;

    let resp = send(
        http()
            .post(format!("{API}/fingerprints"))
            .json(&serde_json::json!({ "fingerprints": [fp] })),
    )
    .await?;
    if !resp.status().is_success() {
        return Ok(false);
    }
    let parsed: CfFingerprintResponse = resp.json().await.map_err(|e| e.to_string())?;
    let Some(m) = parsed.data.exact_matches.into_iter().next() else { return Ok(false) };
    let file = m.file;

    let mods = bulk_mods(&[file.mod_id]).await.unwrap_or_default();
    let cf_mod = mods.get(&file.mod_id);
    let (kind, _folder) = cf_mod.map(kind_and_folder).unwrap_or(("mod", "mods"));
    let (game_versions, loaders) = split_game_versions(&file.game_versions);

    let mut index = read_content_index(&instance_id);
    let item = InstalledItem {
        project_id: file.mod_id.to_string(),
        version_id: file.id.to_string(),
        kind: kind.to_string(),
        name: cf_mod.map(|m| m.name.clone()).unwrap_or_else(|| filename.clone()),
        filename: filename.clone(),
        version_number: if file.display_name.is_empty() { file.file_name.clone() } else { file.display_name.clone() },
        icon_url: cf_mod.and_then(|m| logo_url(&m.logo)),
        game_versions,
        loaders,
        dependency: false,
        installed_at: chrono::Utc::now().to_rfc3339(),
        provider: "curseforge".to_string(),
    };
    index.items.retain(|i| i.project_id != item.project_id && i.filename != item.filename);
    index.items.push(item);
    write_content_index(&instance_id, &index)?;
    Ok(true)
}

async fn bulk_mods(mod_ids: &[i64]) -> Result<HashMap<i64, CfMod>, String> {
    if mod_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let resp = send(
        http()
            .post(format!("{API}/mods"))
            .json(&serde_json::json!({ "modIds": mod_ids })),
    )
    .await?;
    if !resp.status().is_success() {
        return Err(format!("bulk mods failed: {}", resp.status()));
    }
    let parsed: CfListResponse<CfMod> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(parsed.data.into_iter().map(|m| (m.id, m)).collect())
}

// ===== install: modpack (.zip with manifest.json) as a new instance =====

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifest {
    #[serde(default)]
    name: String,
    minecraft: CfManifestMc,
    #[serde(default)]
    files: Vec<CfManifestFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifestMc {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<CfManifestLoader>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifestLoader {
    id: String,
    #[serde(default)]
    primary: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifestFile {
    #[serde(rename = "fileID")]
    file_id: i64,
    #[serde(default)]
    required: bool,
}

#[derive(Clone, Serialize)]
struct ModpackProgress {
    current: u64,
    total: u64,
    name: String,
}

/// Parses a CurseForge manifest loader id like "forge-47.2.0" / "fabric-0.15.0".
fn loader_from_manifest(loaders: &[CfManifestLoader]) -> Loader {
    let primary = loaders.iter().find(|l| l.primary).or_else(|| loaders.first());
    let Some(l) = primary else { return Loader::Vanilla };
    let (kind, ver) = l.id.split_once('-').unwrap_or((l.id.as_str(), ""));
    let v = ver.to_string();
    match kind.to_lowercase().as_str() {
        "forge" => Loader::Forge(v),
        "fabric" => Loader::Fabric(v),
        "quilt" => Loader::Quilt(v),
        "neoforge" => Loader::NeoForge(v),
        _ => Loader::Vanilla,
    }
}

/// Imports a CurseForge modpack from a local `.zip` (manifest.json + overrides)
/// as a new instance. Resolves each manifest file's download URL via the API;
/// files whose authors block distribution are reported as `blocked`.
#[tauri::command]
pub async fn curseforge_import_modpack_file(
    app: AppHandle,
    path: String,
    name_override: Option<String>,
) -> Result<Instance, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("read file: {e}"))?;
    install_modpack_bytes(&app, bytes, name_override, None).await
}

/// Installs a CurseForge modpack straight from the browser: resolves the chosen
/// file's download URL, fetches the `.zip` and processes it like a local import.
#[tauri::command]
pub async fn curseforge_install_modpack(
    app: AppHandle,
    project_id: String,
    file_id: String,
    name_override: Option<String>,
) -> Result<Instance, String> {
    let file = fetch_file(&project_id, &file_id).await?;
    let url = file
        .download_url
        .filter(|u| !u.is_empty())
        .ok_or("this modpack file is not available for third-party download")?;
    let bytes = download(&url).await?;
    let cf_mod = fetch_mod(&project_id).await.ok();
    let icon_url = cf_mod.and_then(|m| logo_url(&m.logo));
    install_modpack_bytes(&app, bytes, name_override, icon_url).await
}

pub(crate) async fn install_modpack_bytes(
    app: &AppHandle,
    bytes: Vec<u8>,
    name_override: Option<String>,
    icon_url: Option<String>,
) -> Result<Instance, String> {
    let (manifest, raw_manifest) = parse_cf_manifest(&bytes)?;

    let mc_version = manifest.minecraft.version.clone();
    let loader = loader_from_manifest(&manifest.minecraft.mod_loaders);
    let name = name_override
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| manifest.name.clone());

    let mut instance = instances::create_instance(name, mc_version, loader, None, None)?;

    // Download + apply the modpack's icon, if any.
    if let Some(url) = icon_url.filter(|u| !u.trim().is_empty()) {
        if let Ok(data) = download(&url).await {
            if std::fs::write(paths::instance_icon_file(&instance.id), &data).is_ok() {
                instance.icon = Some("icon.png".to_string());
                let _ = crate::store::write_json(&paths::instance_config_file(&instance.id), &instance);
            }
        }
    }

    // Resolve all file download URLs in one bulk request.
    let file_ids: Vec<i64> = manifest.files.iter().map(|f| f.file_id).collect();
    let resolved = bulk_files(&file_ids).await.unwrap_or_default();

    // Fetch mod metadata to populate the content index (so we know names, icons, kinds).
    let mod_ids: Vec<i64> = resolved.values().map(|f| f.mod_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let mods = bulk_mods(&mod_ids).await.unwrap_or_default();

    let game_dir = paths::instance_game_dir(&instance.id);
    let total = manifest.files.len() as u64;
    let mut blocked: Vec<String> = Vec::new();
    let mut index = read_content_index(&instance.id);

    for (i, f) in manifest.files.iter().enumerate() {
        let _ = app.emit(
            "modrinth://modpack-progress",
            ModpackProgress { current: i as u64 + 1, total, name: instance.name.clone() },
        );
        let Some(file) = resolved.get(&f.file_id) else { continue };
        let cf_mod = mods.get(&file.mod_id);
        let (kind, _folder) = cf_mod.map(kind_and_folder).unwrap_or(("mod", "mods"));

        let folder = "mods"; // CF modpack files are mods; overrides carry the rest.
        let url = match &file.download_url {
            Some(u) if !u.is_empty() => u.clone(),
            _ => {
                blocked.push(file.file_name.clone());
                continue;
            }
        };
        let dir = game_dir.join(folder);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut dest = dir.join(safe_name(&file.file_name));
        if !f.required {
            dest.set_extension(format!("{}.disabled", dest.extension().and_then(|e| e.to_str()).unwrap_or("jar")));
        }
        if let Ok(data) = download(&url).await {
            let _ = std::fs::write(&dest, &data);
            let (game_versions, loaders) = split_game_versions(&file.game_versions);
            index.items.push(InstalledItem {
                project_id: file.mod_id.to_string(),
                version_id: file.id.to_string(),
                kind: kind.to_string(),
                name: cf_mod.map(|m| m.name.clone()).unwrap_or_else(|| safe_name(&file.file_name)),
                filename: safe_name(&file.file_name),
                version_number: if file.display_name.is_empty() { file.file_name.clone() } else { file.display_name.clone() },
                icon_url: cf_mod.and_then(|m| logo_url(&m.logo)),
                game_versions,
                loaders,
                dependency: false,
                installed_at: chrono::Utc::now().to_rfc3339(),
                provider: "curseforge".to_string(),
            });
        }
    }

    let _ = write_content_index(&instance.id, &index);

    // Extract overrides/ into the game dir.
    extract_overrides(&bytes, &game_dir)?;
    // Keep the manifest for reference.
    let _ = std::fs::write(paths::instance_dir(&instance.id).join("manifest.json"), raw_manifest);

    if !blocked.is_empty() {
        log::warn!("CurseForge modpack: {} blocked file(s) not downloaded", blocked.len());
    }
    Ok(instance)
}

async fn bulk_files(file_ids: &[i64]) -> Result<HashMap<i64, CfFile>, String> {
    if file_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let resp = send(
        http()
            .post(format!("{API}/mods/files"))
            .json(&serde_json::json!({ "fileIds": file_ids })),
    )
    .await?;
    if !resp.status().is_success() {
        return Err(format!("CurseForge bulk files failed: {}", resp.status()));
    }
    let parsed: CfListResponse<CfFile> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(parsed.data.into_iter().map(|f| (f.id, f)).collect())
}

fn parse_cf_manifest(bytes: &[u8]) -> Result<(CfManifest, String), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("open zip: {e}"))?;
    let mut f = archive
        .by_name("manifest.json")
        .map_err(|_| "not a CurseForge modpack (no manifest.json)".to_string())?;
    let mut raw = String::new();
    f.read_to_string(&mut raw).map_err(|e| e.to_string())?;
    let manifest: CfManifest = serde_json::from_str(&raw).map_err(|e| format!("parse manifest: {e}"))?;
    Ok((manifest, raw))
}

fn extract_overrides(bytes: &[u8], game_dir: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let Some(rel) = name.strip_prefix("overrides/").or_else(|| name.strip_prefix("client-overrides/")) else {
            continue;
        };
        let dest = join_safe(game_dir, rel)?;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        std::fs::write(&dest, &buf).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ===== helpers =====

async fn download(url: &str) -> Result<Vec<u8>, String> {
    // CDN downloads don't need the API key, but reusing the client is fine.
    let resp = http().get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("download failed ({}): {url}", resp.status()));
    }
    Ok(resp.bytes().await.map_err(|e| e.to_string())?.to_vec())
}

fn safe_name(name: &str) -> String {
    Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string())
}

fn join_safe(base: &Path, rel: &str) -> Result<PathBuf, String> {
    let mut out = base.to_path_buf();
    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            _ => return Err(format!("unsafe path in modpack: {rel}")),
        }
    }
    Ok(out)
}
