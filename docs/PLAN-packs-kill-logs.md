# Plan: packs delete + kill/stop + logs (saved & live)

Status: ⬜ todo / ✅ done.

## A. Resource packs / shaders / datapacks — pełne wsparcie + usuwanie ⬜
- **Backend** ([content.rs](../src-tauri/src/commands/content.rs)): `PackInfo`/`ShaderInfo` + `filename` (surowa nazwa pliku/folderu). Komenda `delete_content(instance_id, kind, filename)` (kind → folder: resourcepacks/shaderpacks/datapacks; usuwa plik lub katalog).
- **Frontend** ([InstanceContent.vue](../app/components/InstanceContent.vue)): w wierszach packów/shaderów/datapacków pokaż ikonę (jest), nazwę (display), **opis** (z pack.mcmeta — jest dla packów), **nazwę pliku** (mono, drobno) i **przycisk usuń**.

## B. Zabij / zamknij instancję ⬜
- **Backend**: `AppState` + `pids: Mutex<HashMap<String,u32>>`; w [launch.rs](../src-tauri/src/commands/launch.rs) zapisz `child.id()` przy starcie, usuń przy wyjściu. Komendy `stop_instance` (łagodnie: taskkill /PID /T → WM_CLOSE / SIGTERM) i `kill_instance` (siłowo: /F / SIGKILL).
- **Frontend** ([id].vue](../app/pages/instance/[id].vue)): gdy instancja działa, przyciski **Zamknij** i **Zabij** zamiast/obok Graj.

## C. Zakładka „Logi" = tylko zapisane logi ⬜
- **Backend** ([content.rs](../src-tauri/src/commands/content.rs)): `list_log_files(instance_id)` → `{ name, kind: latest|archived|crash, rel, modified, size }` z `minecraft/logs/latest.log` + `minecraft/logs/*.log.gz` + `minecraft/crash-reports/*.txt`. `read_log_file(instance_id, rel)` → tekst (rozpakuj .gz).
- **Frontend** ([id].vue]): zakładka Logi = lista plików (najnowszy/archiwalne/crash) → wybór → podgląd treści (read-only). Bez logów na żywo.

## D. Logi na żywo po kliknięciu w Titlebar Activity ⬜
- **Titlebar** ([TitlebarActivity.vue](../app/components/TitlebarActivity.vue)): klikalny → otwiera modal z logami na żywo.
- **Modal** (globalny w [default.vue](../app/layouts/default.vue)): konsola na żywo z `useActivityCenter().logs` aktywnej/uruchomionej instancji, **kolorowana** wg poziomu (INFO/WARN/ERROR/DEBUG) + przyciemnione timestampy/wątki; autoscroll.

## Kolejność
A → B → C → D.
