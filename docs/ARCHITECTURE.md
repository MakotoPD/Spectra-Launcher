# Mako Launcher — Architecture

A modular, public Minecraft launcher. Simple to use, simple to extend.

## Stack

| Layer        | Choice                                                                 |
|--------------|-----------------------------------------------------------------------|
| Desktop      | Tauri 2                                                                |
| Frontend     | Nuxt 4 (`ssr: false`, static via `nuxt generate`), Pinia, Tailwind, @nuxt/ui |
| MC engine    | [`lyceris`](https://github.com/BatuhanAksoyy/lyceris) — install/launch, Microsoft auth, Fabric/Quilt/Forge/NeoForge, **auto-manages Java** |
| Skin preview | `skinview3d`                                                           |
| Modpacks     | Modrinth API (planned)                                                 |

Because Lyceris handles version metadata, libraries, assets **and** Java
provisioning, the backend does not hand-roll any of that. We own the *launcher*
concerns: instances, accounts, settings, skins, and translating them into a
Lyceris `Config` at launch time.

## Data layout

Everything lives under one relocatable **data root**
(`%APPDATA%/MakoLauncher` on Windows, override with `MAKO_DATA_DIR`):

```text
<data root>/
├── launcher.json          global settings        (models::Settings)
├── accounts.json          Microsoft accounts      (models::AccountsFile)
├── instances/
│   └── <instance-id>/
│       ├── instance.json  instance metadata       (models::Instance)
│       └── minecraft/     Lyceris game_dir: versions, libraries, assets,
│                          mods, saves, config, resourcepacks, options.txt …
├── runtimes/              shared managed Java (Lyceris runtime_dir)
├── skins/                 <id>.png files + skins.json index
├── cache/                 Modrinth manifests, icons, temp downloads
└── logs/                  the launcher's own logs
```

**Why this shape:** each instance is fully self-contained in its own folder, so
it can be copied, zipped, backed up or shared as a unit — the core of "modular".
Only Java runtimes are shared (`runtimes/`) to avoid re-downloading JDKs per
instance. Defined in [`src-tauri/src/paths.rs`](../src-tauri/src/paths.rs).

## Backend modules (`src-tauri/src/`)

| File                    | Responsibility                                              |
|-------------------------|------------------------------------------------------------|
| `paths.rs`              | Data layout + `get_launcher_paths`                          |
| `models.rs`             | Serde types shared with the frontend                       |
| `store.rs`              | JSON read/write helpers                                     |
| `commands/instances.rs` | List/create/get/update/delete instances                    |
| `commands/settings.rs`  | Load/save `launcher.json`                                   |
| `commands/auth.rs`      | Microsoft OAuth (embedded webview), account storage        |
| `commands/launch.rs`    | Build Lyceris `Config`, install + launch, stream progress  |
| `commands/skins.rs`     | Local skin library (remote apply: TODO)                    |
| `lib.rs`                | Plugin + command registration, `AppState`                  |

## Auth flow (full Microsoft, no offline)

`auth_login` opens an embedded webview at the MSA OAuth URL, intercepts
navigation to `https://login.live.com/oauth20_desktop.srf?code=…`, exchanges the
code via `lyceris::auth::microsoft::authenticate`, and persists the account.
Tokens are refreshed (`auth_refresh_active`) right before each launch.

## Launch flow

`launch_instance(id)`:
1. Load the instance + settings, refresh the active account's token.
2. Build a Lyceris `ConfigBuilder` (game_dir = instance `minecraft/`, shared
   `runtime_dir`, memory, custom JVM args, loader).
3. `install()` (verifies/repairs files) then `launch()`.
4. Stream events to the UI and resolve once the game starts:
   - `mc://multi-progress` `{ current, total }`
   - `mc://file-progress` `{ path, current, total }`
   - `mc://console` `{ line }`
   - `mc://exited` `{ instance_id, code }`

## Frontend (`app/`)

- `types/launcher.ts` — TS mirror of `models.rs`.
- `stores/useInstancesStore.ts`, `stores/useAccountStore.ts` — Pinia.
- `composables/useMinecraftLaunch.ts` — launch + live progress/console state.

## Roadmap / not yet built

- Version + loader pickers (Mojang manifest + loader APIs).
- Modrinth modpack browsing/install (`cache/`).
- Skin upload to Mojang (`apply_skin`) + skinview3d 3D preview UI.
- Settings store wiring + first-run onboarding.
- macOS/Linux validation of the embedded-webview auth window.
