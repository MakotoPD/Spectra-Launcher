mod commands;
mod discord;
mod models;
mod paths;
mod store;

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Shared runtime state managed by Tauri.
#[derive(Default)]
pub struct AppState {
    /// IDs of instances currently running, to prevent double launches.
    pub running: Mutex<HashSet<String>>,
    /// Running instance id -> game process PID (for stop/kill).
    pub pids: Mutex<HashMap<String, u32>>,
    /// Instances adopted from a previous launcher session (live lock files). They
    /// have no owning wait-task, so stop/kill must clean their state up directly.
    pub adopted: Mutex<HashSet<String>>,
    /// Lazily-connected Discord Rich Presence client.
    pub discord: Mutex<Option<discord_rich_presence::DiscordIpcClient>>,
    /// Running instance id -> (name, mc_version) for computing Discord presence
    /// across multiple simultaneous instances.
    pub discord_playing: Mutex<HashMap<String, (String, String)>>,
    /// Serializes the install/verify phase so two instances can't provision the
    /// same shared Java runtime concurrently (which would corrupt it).
    pub install_lock: tokio::sync::Mutex<()>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            // Create the data layout up front so every command can assume it exists.
            if let Err(e) = paths::ensure_base_dirs() {
                log::error!("failed to create data directories: {e}");
            }

            // Adopt instances still running from a previous launcher session and
            // clear stale run locks left by games that have since exited.
            commands::launch::reconcile_running(app.handle());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            paths::get_launcher_paths,
            // Settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_system_memory_mb,
            // Instances
            commands::instances::list_instances,
            commands::instances::get_instance,
            commands::instances::create_instance,
            commands::instances::update_instance,
            commands::instances::delete_instance,
            commands::instances::read_image_data_url,
            commands::instances::get_instance_icon_path,
            commands::instances::get_instance_path,
            commands::instances::open_instance_folder,
            commands::instances::open_instance_game_folder,
            commands::instances::reveal_in_explorer,
            commands::instances::copy_file,
            commands::instances::duplicate_instance,
            // Instance content (tabs)
            commands::content::list_screenshots,
            commands::content::list_worlds,
            commands::content::list_resource_packs,
            commands::content::list_data_packs,
            commands::content::list_shaders,
            commands::content::list_servers,
            commands::content::add_server,
            commands::content::delete_server,
            commands::content::delete_content,
            commands::content::list_log_files,
            commands::content::read_log_file,
            commands::content::upload_log_to_mclogs,
            // Auth
            commands::auth::auth_login,
            commands::auth::auth_login_offline,
            commands::auth::auth_get_login_url,
            commands::auth::auth_login_with_code,
            commands::auth::auth_refresh_active,
            commands::auth::list_accounts,
            commands::auth::set_active_account,
            commands::auth::remove_account,
            // Launch
            commands::launch::launch_instance,
            commands::launch::repair_instance,
            commands::launch::is_instance_running,
            commands::launch::stop_instance,
            // Server ping
            commands::ping::ping_server,
            // Version metadata (pickers)
            commands::meta::get_minecraft_versions,
            commands::meta::get_loader_versions,
            // Java detection
            commands::java::detect_java_installations,
            commands::java::validate_java_path,
            // Modrinth (browse + download content/modpacks)
            commands::modrinth::modrinth_search,
            commands::modrinth::modrinth_versions,
            commands::modrinth::check_mod_updates,
            commands::modrinth::match_local_mods,
            commands::modrinth::modrinth_match_file,
            commands::modrinth::modrinth_project,
            commands::modrinth::modrinth_categories,
            commands::modrinth::modrinth_install_with_deps,
            commands::modrinth::get_installed_content,
            commands::modrinth::modrinth_install_modpack,
            commands::modrinth::import_file,
            commands::modrinth::export_mrpack,
            commands::modrinth::check_modpack_update,
            commands::modrinth::update_modpack,
            // CurseForge
            commands::curseforge::cf_enabled,
            commands::curseforge::curseforge_search,
            commands::curseforge::curseforge_versions,
            commands::curseforge::curseforge_project,
            commands::curseforge::curseforge_categories,
            commands::curseforge::curseforge_install_with_deps,
            commands::curseforge::curseforge_match_local,
            commands::curseforge::curseforge_match_file,
            commands::curseforge::curseforge_install_modpack,
            commands::curseforge::curseforge_import_modpack_file,
            commands::curseforge::export_curseforge,
            commands::curseforge::get_blocked_mods,
            commands::curseforge::resolve_blocked_mods,
            commands::curseforge::default_downloads_dir,
            // Import / export
            commands::import::detect_external_instances,
            commands::import::import_external_instance,
            commands::import::list_dir,
            commands::import::export_instance,
            commands::import::write_text_file,
            // Mod management
            commands::mods::list_mods,
            commands::mods::set_mod_enabled,
            commands::mods::delete_mod,
            // Skins
            commands::skins::list_skins,
            commands::skins::save_skin,
            commands::skins::set_skin_model,
            commands::skins::delete_skin,
            commands::skins::get_skin_path,
            commands::skins::get_skin_data_url,
            commands::skins::fetch_skin_data_url,
            commands::skins::get_player_skin,
            commands::skins::import_player_skin,
            commands::skins::apply_skin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
