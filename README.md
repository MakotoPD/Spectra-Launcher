# Mako Launcher

Mako Launcher is a modern, modular, and feature-rich Minecraft launcher built with Tauri v2 (Rust) and Nuxt 4 (Vue 3). It focuses on performance, sleek design, and seamless integration with modern Minecraft ecosystems like Modrinth.

## 🚀 Features

### Core Capabilities
* **Instance Management:** Create, edit, duplicate, and delete Minecraft instances. Fully self-contained instances with their own `.minecraft` directories.
* **Mod Loaders:** Full support for Vanilla, Fabric, Quilt, Forge, and NeoForge.
* **Authentication:** Microsoft Account (MSA) login and Offline accounts support.
* **Java Management:** Automatic detection and provisioning of required Java versions (managed by the Lyceris engine). Custom Java paths and JVM arguments are supported.
* **Cross-Restart Resilience:** Instances remain tracked even if the launcher is restarted while the game is running, thanks to a robust lock-file system.

### Content & Modding (Modrinth Integration)
* **Modrinth Browser:** Search and install mods, resource packs, and shaders directly from Modrinth.
* **Modpack Support:** Install Modrinth modpacks (`.mrpack`) with automatic dependency resolution. Update and manage installed modpacks.
* **Local Mod Management:** Enable, disable, and delete mods locally. The launcher matches local `.jar` files with Modrinth using SHA-1 hashing to check for updates.
* **Import/Export:** 
  * Import instances from Prism Launcher, CurseForge, and Modrinth apps.
  * Import from ZIP archives or `.mrpack` files.
  * Export your instances to standard Modrinth `.mrpack` or full `.zip` backups.

### Instance Customization & Utilities
* **Quick Play (Minecraft 1.20+):** Jump directly into a specific singleplayer world or connect to a multiplayer server right from the launcher UI.
* **Server Ping:** Built-in Minecraft Server List Ping (SLP) protocol to check server latency (ms), MOTD, player counts, and favicons directly in the Servers tab.
* **Crash Detection & Reporting:** Automatically detects when the game crashes (non-zero exit code), finds the latest crash report, and presents a dedicated UI with syntax highlighting and a 1-click export to `mclo.gs`.
* **Live Logs:** Real-time console output viewer with auto-scrolling, plus access to past logs and crash reports.
* **JVM Presets:** Quick selection of optimized Java arguments (e.g., Aikar's Flags, ZGC, Shenandoah) per instance.
* **Skins Management:** Browse, save, and apply custom skins (classic and slim models) to your account directly within the launcher.
* **Content Tabs:** View and manage worlds (with NBT parsing for metadata), screenshots, resource packs, data packs, shaders, and servers.

### Extras
* **Discord Rich Presence:** Show your current game status on Discord.
* **Playtime Tracking:** Accumulate and view total playtime per instance.
* **Modern UI:** Built with Nuxt UI and TailwindCSS, featuring a customizable theme and beautiful animations.

## 🛠️ Technology Stack

* **Backend:** Rust 🦀 via [Tauri v2](https://v2.tauri.app/)
  * Engine: `lyceris` (Minecraft installation, launching, auth)
  * Async Runtime: `tokio`
  * HTTP Client: `reqwest`
  * Utils: `serde`, `zip`, `flate2`, `fastnbt` (for NBT parsing), `sysinfo` (RAM allocation)
* **Frontend:** TypeScript 📘 via [Nuxt 4](https://nuxt.com/) (Vue 3)
  * UI Framework: `@nuxt/ui`, TailwindCSS, Headless UI
  * State Management: `Pinia`
  * Drag & Drop: `vue-draggable-plus`
  * Text Editor: `tiptap`
  * Localization: `@nuxtjs/i18n` (English, Polish)
  * 3D Skin Viewer: `skinview3d`

## 📂 Architecture & Data Layout

The launcher uses a strictly modular data architecture. The UI communicates with the Rust backend entirely via Tauri commands and asynchronous events (e.g., `mc://console`, `mc://crashed`, `mc://multi-progress`).

### Data Directory Structure
By default, data is stored in the OS-specific data directory under `MakoLauncher` (e.g., `%APPDATA%\MakoLauncher` on Windows), but can be overridden with the `MAKO_DATA_DIR` environment variable for portable installations.

```text
<data root>/
├── launcher.json                 # Global settings
├── accounts.json                 # Saved accounts & tokens
├── instances/
│   └── <instance-id>/
│       ├── instance.json         # Metadata (name, version, loader, overrides)
│       ├── instance.lock         # Running state lock (prevents double-launching)
│       ├── content.json          # Modrinth installed content index
│       └── minecraft/            # The actual game directory
├── runtimes/                     # Shared Java runtimes (managed by Lyceris)
├── skins/                        # Downloaded and cached skins
├── cache/                        # Temporary files, Modrinth manifests, icons
└── logs/                         # Launcher application logs
```

## 🔒 Privacy & Security
* The launcher communicates directly with Microsoft/Xbox APIs for authentication.
* Modrinth is the sole external provider for mods and modpacks.
* All settings, instances, and authentication tokens are stored locally on your machine.
* Opt-in features include Discord Rich Presence and Playtime Tracking. Crash reporting to `mclo.gs` is strictly user-initiated.

## 💻 System Requirements
* **OS:** Windows 10/11 (64-bit)
* **WebView:** Microsoft Edge WebView2 (Windows)
* **Disk Space:** Varies by installed instances and shared Java runtimes.

## ⚙️ Development

### Prerequisites
* [Rust](https://www.rust-lang.org/) (v1.77.2+)
* [Node.js](https://nodejs.org/) (v20+)
* [pnpm](https://pnpm.io/)

### Setup
1. Clone the repository
2. Install frontend dependencies:
   ```bash
   pnpm install
   ```
3. Run the development server (starts Nuxt + Tauri):
   ```bash
   pnpm run dev
   ```
4. Build for production:
   ```bash
   pnpm run build
   ```