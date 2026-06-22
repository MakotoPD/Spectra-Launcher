# Mako Launcher — przewodnik

Praktyczny przewodnik dla Ciebie: jak działa logowanie, konta, motyw, język,
instancje i uruchamianie gry. Architektura techniczna jest opisana osobno w
[ARCHITECTURE.md](ARCHITECTURE.md).

---

## 1. Konta i logowanie

Launcher obsługuje **dwa typy kont**:

| Typ | Do czego | Wymaga internetu |
|-----|----------|------------------|
| **Microsoft** (online) | Serwery online, oryginalne konto Minecraft, skórki | Tak (przy logowaniu) |
| **Offline** | Singleplayer, serwery w trybie offline | Nie |

Konta zapisują się w `accounts.json` (patrz [sekcja 6](#6-gdzie-zapisują-się-dane)).
Jedno konto jest zawsze **aktywne** — to nim uruchamiana jest gra.

### 1a. Logowanie przez Microsoft

W *Ustawieniach → Konta* kliknij **„Zaloguj przez Microsoft"**.

Co się dzieje pod spodem:
1. Otwiera się wbudowane okno logowania Microsoft (prawdziwa strona logowania
   Microsoft, nie nasza).
2. Po zalogowaniu launcher przechwytuje kod autoryzacji z przekierowania
   (`login.live.com/oauth20_desktop.srf?code=…`), wymienia go na token i pobiera
   profil gracza.
3. Konto zostaje zapisane i ustawione jako aktywne.

Tokeny **odświeżają się automatycznie** tuż przed każdym uruchomieniem gry
(`auth_refresh_active`), więc nie trzeba logować się ponownie codziennie. Gdy
token wygaśnie i nie da się go odświeżyć — zaloguj się jeszcze raz.

> Uwaga techniczna: okno logowania tworzone jest po stronie Rusta
> (`WebviewWindowBuilder` + `on_navigation`). Działa pewnie na Windows; na
> macOS/Linux może wymagać dopracowania (tworzenie okna na głównym wątku).

### 1b. Logowanie offline

W *Ustawieniach → Konta* wpisz nazwę gracza i kliknij **„Dodaj konto offline"**.

Zasady:
- Nazwa: **3–16 znaków**, tylko litery, cyfry i podkreślnik (`_`).
- Konto offline nie ma żadnych tokenów — działa **tylko** w singleplayer i na
  serwerach w trybie offline (`online-mode=false`).
- UUID jest generowany raz i zapisywany, więc konto zachowuje tożsamość (i swoje
  światy) między uruchomieniami. Dodanie offline'a o tej samej nazwie nie tworzy
  duplikatu — używa istniejącego.

### 1c. Zarządzanie kontami

- **Ustaw jako aktywne** — wybiera konto, którym uruchomi się gra.
- **Usuń** (kosz) — kasuje konto; jeśli usuniesz aktywne, aktywne staje się
  pierwsze z listy.

### Komendy (Tauri `invoke`)

| Komenda | Opis |
|---------|------|
| `auth_login` | Pełny flow Microsoft (otwiera okno, zwraca konto) |
| `auth_login_offline` `{ username }` | Tworzy/aktywuje konto offline |
| `auth_refresh_active` | Odświeża token aktywnego konta (offline zwraca bez zmian) |
| `list_accounts` | `{ accounts, active_uuid }` |
| `set_active_account` `{ uuid }` | Zmienia aktywne konto |
| `remove_account` `{ uuid }` | Usuwa konto |

W kodzie używaj store'a zamiast surowych komend:

```ts
const accounts = useAccountStore()
await accounts.load()
await accounts.login()                 // Microsoft
await accounts.loginOffline('Steve')   // offline
await accounts.setActive(uuid)
await accounts.remove(uuid)
accounts.activeAccount                  // getter
```

---

## 2. Motyw (wygląd)

W *Ustawieniach → Wygląd*. Zmiany są natychmiastowe i **zapamiętywane**
(localStorage, klucz `mako-theme`).

- **Tryb**: `Ciemny` lub `OLED` (czysta czerń — tła i panele schodzą do `#000`,
  idealne na ekrany OLED).
- **Kolor akcentu**: kolor wiodący całego UI (przyciski, aktywne elementy).
  To kolor `primary` z @nuxt/ui, zmieniany w locie.

W kodzie:

```ts
const theme = useThemeStore()
theme.setMode('oled')      // 'dark' | 'oled'
theme.setAccent('violet')  // patrz ACCENT_COLORS
theme.bgClass              // klasa tła powłoki aplikacji
```

Motyw stosowany jest globalnie w `plugins/theme.client.ts` przy starcie, więc
działa **wszędzie**. OLED dodaje klasę `oled` na `<html>` (patrz nadpisania w
`app/assets/css/main.css`).

---

## 2b. Stylowanie — kolory, tła, akcent i gradienty

@nuxt/ui v4 + Tailwind v4. Wszystkie kolory to **zmienne CSS**, więc reagują na
motyw (Ciemny/OLED) i na wybrany akcent **automatycznie**.

### Złota zasada: używaj aliasu `primary`

Kolor akcentu z ustawień to kolor **`primary`** w @nuxt/ui. Jest podmieniany w
locie (`appConfig.ui.colors.primary`), więc **każda klasa `primary` aktualizuje
się sama**, gdy użytkownik zmieni akcent. Nigdy nie wpisuj na sztywno `sky`,
`blue` itd. — użyj `primary`.

```vue
<UButton color="primary" />            <!-- komponenty @nuxt/ui -->
<div class="bg-primary text-inverted"> <!-- własny markup -->
```

### Akcent — klasy

| Chcesz | Klasa |
|--------|-------|
| Tło / tekst / ramka akcentem | `bg-primary` · `text-primary` · `border-primary` · `ring-primary` |
| Konkretny odcień (50–950) | `bg-primary-500` · `text-primary-400` · `hover:bg-primary-600` |
| Półprzezroczyście | `bg-primary-500/10` · `text-primary/80` · `border-primary-500/30` |
| Surowa zmienna CSS | `var(--ui-primary)` · `var(--ui-color-primary-500)` |

Pełna skala odcieni `--ui-color-primary-50 … 950` jest dostępna jako klasy
`*-primary-50 … *-primary-950`.

### Tła, tekst, obramowania (semantyczne — same dopasują się do Ciemny/OLED)

Używaj tych zamiast sztywnych `bg-neutral-900`. W trybie OLED nadpisujemy
`--ui-bg` itd. (`app/assets/css/main.css`), więc elementy z `bg-default`
**automatycznie** staną się czarne; `bg-neutral-900` już nie.

| Tło | Tekst | Ramka |
|-----|-------|-------|
| `bg-default` (`--ui-bg`) | `text-default` (`--ui-text`) | `border-default` |
| `bg-muted` | `text-muted` | `border-muted` |
| `bg-elevated` | `text-dimmed` | `border-accented` |
| `bg-accented` | `text-toned` | `border-inverted` |
| `bg-inverted` | `text-highlighted` / `text-inverted` | |

Inne tokeny: `--ui-radius` (`rounded-[var(--ui-radius)]`), `--ui-container`,
`--ui-header-height`.

### Gradienty z akcentu

> Tailwind v4: kierunek to **`bg-linear-to-*`** (nie `bg-gradient-to-*`!), a stopy
> to `from-` / `via-` / `to-`. Wszystkie podążają za akcentem, jeśli użyjesz `primary`.

```html
<!-- prosty -->
<div class="bg-linear-to-br from-primary-500 to-primary-700"></div>

<!-- delikatny „glow" u góry -->
<div class="bg-linear-to-b from-primary-500/20 to-transparent"></div>

<!-- radialny (przez surową zmienną) -->
<div class="bg-[radial-gradient(circle_at_top,var(--ui-primary),transparent_70%)]"></div>

<!-- gradientowy tekst -->
<h1 class="bg-linear-to-r from-primary-400 to-primary-600 bg-clip-text text-transparent">
  Mako
</h1>

<!-- poświata/cień w kolorze akcentu -->
<div class="shadow-[0_0_30px_-5px_var(--ui-primary)]"></div>

<!-- obramowanie-gradient (tło pod spodem + maska) -->
<div class="rounded-xl bg-linear-to-br from-primary-500/40 to-transparent p-px">
  <div class="rounded-[11px] bg-default p-4">treść</div>
</div>
```

### Zmiana akcentu z kodu

```ts
const theme = useThemeStore()
theme.setAccent('violet')   // jeden z ACCENT_COLORS
theme.accent                // bieżący akcent
```

---

## 3. Język (i18n)

W *Ustawieniach → Język*. Język jest:
- **wykrywany automatycznie** przy pierwszym uruchomieniu (z systemu/przeglądarki),
- **zapamiętywany** w ciasteczku `mako_locale`,
- przełączany bez zmiany adresu (strategia `no_prefix` — desktopowo, bez `/en/…`).

### Dodanie nowego języka — 2 kroki

1. Skopiuj `i18n/locales/en.json` na np. `i18n/locales/de.json` i przetłumacz.
2. Dodaj wpis w `nuxt.config.ts → i18n.locales`:
   ```ts
   { code: 'de', name: 'Deutsch', file: 'de.json' }
   ```

To wszystko. W komponentach używaj `{{ $t('nav.home') }}` lub:

```ts
const { t, locale, setLocale } = useI18n()
t('settings.title')
setLocale('pl')
```

---

## 4. Instancje

Każda instancja to osobny, samowystarczalny profil gry (wersja MC + loader +
mody + światy) w `instances/<id>/`.

```ts
const instances = useInstancesStore()
await instances.load()
await instances.create({
  name: 'Fabric 1.21',
  mcVersion: '1.21.4',
  loader: { type: 'fabric', version: '0.16.9' }, // lub { type: 'vanilla' }
  memoryMb: 4096,
})
await instances.remove(id)
```

Obsługiwane loadery: `vanilla`, `fabric`, `quilt`, `forge`, `neoforge`
(wszystko poza vanilla wymaga wersji loadera).

---

## 5. Uruchamianie gry

```ts
const mc = useMinecraftLaunch()
await mc.launch(instanceId)
```

Przebieg:
1. Odświeżenie tokenu aktywnego konta (offline pomijane).
2. Zbudowanie konfiguracji Lyceris (katalog gry instancji, współdzielona Java,
   pamięć, argumenty, loader).
3. `install()` — sprawdza i pobiera brakujące/uszkodzone pliki (w tym **Javę** —
   Lyceris dobiera ją automatycznie do wersji MC).
4. `launch()` — start gry. Funkcja kończy się, gdy proces wystartuje.

Zdarzenia na żywo (nasłuchuje ich `useMinecraftLaunch`):

| Zdarzenie | Dane |
|-----------|------|
| `mc://multi-progress` | `{ current, total }` — pobieranie bibliotek/zasobów/Javy |
| `mc://file-progress` | `{ path, current, total }` |
| `mc://console` | `{ line }` — logi gry |
| `mc://exited` | `{ instance_id, code }` |

Stan ze store'a: `mc.stage` (`idle`/`installing`/`running`), `mc.progress`,
`mc.log`, `mc.runningId`, `mc.error`.

---

## 6. Gdzie zapisują się dane

Korzeń danych: `%APPDATA%/MakoLauncher` (Windows). Można nadpisać zmienną
środowiskową `MAKO_DATA_DIR` (np. instalacja przenośna).

```
MakoLauncher/
├── launcher.json     ustawienia globalne
├── accounts.json     konta (Microsoft + offline)
├── instances/<id>/   instancje (instance.json + minecraft/)
├── runtimes/         współdzielona Java
├── skins/            zapisane skórki
├── cache/ · logs/
```

Komenda `get_launcher_paths` zwraca te ścieżki (przydatne do przycisków
„Otwórz folder").
