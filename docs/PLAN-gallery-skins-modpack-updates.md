# Plan: galeria Modrinth + model skórki + aktualizacje modpacka

Trzy funkcje. Status: ⬜ todo / 🟦 w toku / ✅ done.

## 1. Modrinth browser — zakładki „Opis" i „Galeria" ✅
Prawa kolumna modala ([ModrinthBrowser.vue](../app/components/ModrinthBrowser.vue)) ma mieć przełącznik dwóch zakładek: **Opis** (obecny markdown `body`) i **Galeria** (zdjęcia projektu).

- **Backend** ([modrinth.rs](../src-tauri/src/commands/modrinth.rs)): `ProjectFull` + pole `gallery: Vec<GalleryItem { url, title, description, featured }>` (Modrinth zwraca `gallery` w `GET /project/{id}`).
- **TS** ([modrinth.ts](../app/types/modrinth.ts)): `ModrinthProjectFull.gallery`.
- **UI**: mały tab-switch nad treścią; Galeria = siatka miniatur → klik powiększa (lightbox z tytułem/opisem + strzałki + klawiatura, jak w screenshotach instancji).
- **i18n**: `modrinth.tabDescription`, `modrinth.tabGallery`, `modrinth.noGallery`.

## 2. Strona „Skórki" — wybór model: klasyczny (4px ramię) / slim (3px) ✅
Możliwość ustawienia modelu zapisanej skórki; ma wpływać na render 3D, miniaturę-popiersie i upload (variant).

- **Backend** ([skins.rs](../src-tauri/src/commands/skins.rs)): komenda `set_skin_model(id, model)` aktualizująca `model` w `skins.json`.
- **Frontend** ([skins.vue](../app/pages/skins.vue)): przy wybranej zapisanej skórce przełącznik **Klasyczny / Slim** → `set_skin_model` + ponowny render popiersia (`savedBust`) + przeładowanie viewera z nowym modelem. `apply_skin` już używa `skin.model`.
- **i18n**: `skins.model`, `skins.classic`, `skins.slim`.

## 3. Strona instancji — powiadomienie o nowej wersji modpacka + changelog ✅
Jeśli instancja powstała z modpacka Modrinth: pokaż baner „dostępna nowa wersja" z rozwijanym changelogiem; przycisk aktualizacji.

- **Model** ([models.rs](../src-tauri/src/models.rs)): `Instance` + `modpack_project_id`, `modpack_version_id` (Option).
- **Install** ([modrinth.rs](../src-tauri/src/commands/modrinth.rs)): `modrinth_install_modpack` przyjmuje `project_id` + `version_id` i zapisuje na instancji. Rdzeń instalacji wydzielić do `apply_modpack(instance_id, bytes)` (re-użycie przy aktualizacji).
- **Komendy**:
  - `check_modpack_update(instance_id) -> Option<{ version_id, version_number, changelog, date_published }>` (porównanie najnowszej wersji projektu modpacka z zainstalowaną).
  - `update_modpack(instance_id)` — pobiera najnowszy `.mrpack`, aplikuje pliki + overrides do istniejącego game-dir, aktualizuje `modpack_version_id` i content index.
- **Frontend** ([id].vue](../app/pages/instance/[id].vue)): po wejściu (gdy modpack) sprawdź aktualizację → baner w hero: „Nowa wersja: X" + rozwijany changelog (markdown przez `marked`) + przycisk „Aktualizuj".
- **Composable** ([useModrinth.ts](../app/composables/useModrinth.ts)): `checkModpackUpdate`, `updateModpack`.
- **i18n**: `instance.updateAvailable`, `instance.showChangelog`, `instance.updateNow`, `instance.updated`.

## Kolejność realizacji
1. Galeria (najprostsza, izolowana).
2. Model skórki.
3. Aktualizacje modpacka (największa: model + install refactor + komendy + UI).
