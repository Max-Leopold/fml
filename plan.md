# FML Rebuild Plan

## What is FML?

A terminal application for managing Factorio mods on headless Linux servers.
Without it, you either configure mods in the Factorio desktop client and copy
files to the server, or manually download zips and edit JSON. FML lets you do
everything from an SSH session.

## Reference

- **Factorio Mod Portal API:** See `factorio_mod_api.md` in the project root. This
  is the authoritative reference for endpoint URLs, query parameters, response JSON
  shapes, authentication requirements, and download URL construction. When
  implementing API calls, match the endpoint specs in that file exactly — do not
  guess at field names or URL formats.
- **Existing code:** The current `src/` directory contains the old implementation.
  It can be consulted for domain understanding but should not be preserved — we are
  rewriting from scratch.

## Core Operations

1. **Browse/search** the mod portal catalog
2. **Install** a mod with correct dependency resolution
3. **Remove** an installed mod (delete the zip from disk)
4. **Enable/disable** an installed mod
5. **Persist** enable/disable state to `mod-list.json` on exit

## Key Design Decisions

### Configuration
- `fml init` creates `fml.json` with `mods_dir_path` and `server_config_path`.
- Credentials (username + token) are read from the Factorio `server-settings.json`
  at the path stored in `server_config_path`. We do not duplicate credentials in
  our own config.
- The config struct has no `mods` field — the mods directory is the source of truth.

### Factorio Version Detection
- Auto-detect by reading `<mods_dir>/../data/base/info.json` (standard Factorio
  server layout). Parse the `version` field and extract major.minor (e.g., `"1.1"`).
- If auto-detection fails, error with a clear message explaining what file we looked
  for and why. Do not silently fall back or guess.
- The detected version is used in two places:
  1. **Mod list endpoint:** Pass as `version` query parameter to
     `GET /api/mods?version={version}` so the portal only returns mods that have
     at least one compatible release (see `factorio_mod_api.md`, List Mods →
     Query Parameters → `version`).
  2. **Release selection:** When resolving dependencies and picking specific
     releases, compare each `release.info_json.factorio_version` against the
     detected version (major.minor string match, e.g., `"1.1" == "1.1"`).

### Dependency Resolution
- Factorio dependency string format (see `factorio_mod_api.md`, InfoJson →
  dependencies array, and https://wiki.factorio.com/Tutorial:Mod_structure#dependencies):
  - `"mod-name"` or `"mod-name >= 1.0"` → required
  - `"! mod-name"` → incompatible
  - `"? mod-name"` → optional (skip for auto-install)
  - `"(?) mod-name"` → hidden optional (skip)
  - `"~ mod-name"` → required, no load-order constraint (treat as required)
- **Resolution algorithm:** The resolver collects the full dependency tree
  recursively **before** downloading anything. It walks the tree depth-first:
  for each required dependency, it fetches that mod's details from the API,
  picks the best release, parses that release's dependencies, and recurses.
  Only after the entire tree is resolved (or an error occurs) does downloading
  begin. This means a conflict or missing mod is caught before any files are
  written to disk.
- **Version matching:** Use the `semver` crate. Dependency version constraints
  (e.g., `>= 1.0.0`) are parsed as `semver::VersionReq`. Release versions are
  parsed as `semver::Version`. Matching uses `VersionReq::matches(&Version)`.
  When picking a release, iterate releases newest-first and pick the first one
  where both the version constraint is satisfied AND `release.factorio_version`
  matches the server's Factorio version (major.minor comparison, e.g., release
  `factorio_version: "1.1"` matches server version `"1.1"`).
- Skips `base` (the game itself, not a downloadable mod).
- Skips mods already installed in the mods directory **if** the installed version
  satisfies the version constraint. If the installed version does NOT satisfy
  the constraint, return an error: "Installed version X of mod Y does not satisfy
  required >= Z. Remove it first and retry." Do not auto-upgrade for MVP.
- Detects cycles and duplicates via a visited set keyed on mod name.
- **Blocks** install if any mod in the tree is marked incompatible (`!`) with an
  already-installed mod. Returns an error naming both mods.
- **Mod names are case-sensitive.** All comparisons are exact string match.

### Architecture — Message Passing
- The main loop owns all app state directly. No `Arc<Mutex<>>` on app state.
- A single `tokio::sync::mpsc` channel carries all events: terminal input, ticks,
  and async task results.
- When the user triggers an async action (search, install, delete), the handler
  spawns a `tokio::spawn` task with owned data (no references to app state).
- The task sends its result back through the channel.
- Rendering is a pure function of app state. No side effects in draw functions.

### TUI Framework
- Use `ratatui` (the maintained fork of the dead `tui` crate) with `crossterm`.

### Error Handling
- `anyhow` for all application errors.
- Errors from async tasks are sent back through the event channel.
- Displayed to the user in the TUI status bar.
- No `.unwrap()` on fallible operations except true invariants (compiled regexes,
  channel sends where receiver is known alive).

### Network and I/O Error Handling
- **API calls (fetch mod list, fetch details, download):** On failure, propagate the
  error back through the event channel. The status bar shows the error. The app does
  not crash. The user can retry by pressing Enter again.
- **File I/O (reading installed mods, writing mod-list.json, deleting files):** Same
  pattern — errors become status bar messages, never panics.
- **Partial downloads:** If a download is interrupted, delete the partially-written
  zip file before returning the error. Do not leave corrupt files in the mods dir.
- **Auth failures:** If the mod portal returns a non-200 response during download
  (likely auth error), surface a clear message: "Download failed: check username and
  token in server-settings.json".

---

## Implementation Checkpoints

Work through these in order. Each checkpoint produces compilable, testable code.
Do not move to the next checkpoint until the current one compiles and is correct.

**After completing each checkpoint**, commit the results with a descriptive
message explaining what was done and why. Do not reference checkpoint numbers
in commit messages — just describe the change naturally (e.g., "Add dependency
resolver with cycle detection and version constraint matching").

---

### Phase 1: Foundation

#### Checkpoint 1.1 — Project skeleton and dependencies
- [ ] Delete all existing source files in `src/`.
- [ ] Replace `Cargo.toml` dependencies with:
  - `ratatui` + `crossterm` (TUI)
  - `tokio` with `full` feature (async runtime)
  - `reqwest` with `json` feature (HTTP)
  - `serde` + `serde_json` (serialization)
  - `anyhow` (errors)
  - `semver` (version parsing)
  - `zip` (reading mod archives)
  - `clap` with `derive` feature (CLI args)
  - `regex` (dependency parsing)
- [ ] Create the directory structure:
  ```
  src/
  ├── main.rs
  ├── app.rs
  ├── config.rs
  ├── event.rs
  ├── handler.rs
  ├── ui.rs
  └── factorio/
      ├── mod.rs
      ├── api.rs
      ├── types.rs
      ├── installed.rs
      ├── mod_list.rs
      └── resolver.rs
  ```
- [ ] Create minimal `main.rs` that parses CLI args (just `init` subcommand for now)
  and prints "Hello, FML!".
- [ ] Verify: `cargo build` succeeds, `cargo run` prints the message.

#### Checkpoint 1.2 — Configuration
- [ ] Implement `config.rs`:
  - `FmlConfig` struct with `mods_dir_path: String` and `server_config_path: String`.
  - `FmlConfig::init()` — interactive wizard that asks for both paths, validates
    they exist, canonicalizes them, writes `fml.json` to current directory.
  - `FmlConfig::load()` — reads `fml.json` from current directory, returns error
    if not found with message to run `fml init`.
- [ ] Wire `init` subcommand in `main.rs` to `FmlConfig::init()`.
- [ ] Verify: `cargo run -- init` creates a valid `fml.json`.

#### Checkpoint 1.3 — Factorio types and server settings
- [ ] Implement `factorio/types.rs`:
  - `Mod` struct: `name`, `title`, `summary`, `downloads_count`, `releases: Vec<Release>`.
  - `Release` struct: `download_url`, `file_name`, `version: semver::Version`,
    `factorio_version: String`, `sha1: String`, `dependencies: Vec<Dependency>`.
  - `Dependency` struct: `name`, `version_req: semver::VersionReq`, `dep_type: DependencyType`.
  - `DependencyType` enum: `Required`, `Optional`, `Incompatible`.
  - `impl FromStr for Dependency` — parse the dependency string format using regex
    (see format in "Dependency Resolution" above).
  - `ServerSettings` struct with `username` and `token` fields. Deserialize only
    the fields we need — ignore unknown fields (Factorio's `server-settings.json`
    has many other fields we don't care about).
  - `fn read_server_settings(path: &str) -> Result<ServerSettings>` — reads and
    parses the file. If `username` or `token` is empty/missing, return an error
    explaining that valid Factorio credentials are required for downloading mods.
- [ ] Implement Factorio version detection:
  - `fn detect_factorio_version(mods_dir: &str) -> Result<String>` — reads
    `<mods_dir>/../data/base/info.json`, parses the `version` field, returns
    major.minor string (e.g., `"1.1"`).
- [ ] Write unit tests for `Dependency::from_str` covering all prefix types and
  version constraints.
- [ ] Verify: `cargo test` passes.

---

### Phase 2: Factorio Domain Layer

#### Checkpoint 2.1 — API client
- [ ] Implement `factorio/api.rs`. Refer to `factorio_mod_api.md` for all endpoint
  specs and response shapes.
  - `async fn fetch_mod_list(factorio_version: &str) -> Result<Vec<ModListEntry>>`
    — calls `GET /api/mods?page_size=max&hide_deprecated=true&version={version}`.
    `page_size=max` returns all results in a single page (see `factorio_mod_api.md`),
    so no pagination handling is needed.
    `ModListEntry` has `name`, `title`, `downloads_count`, `summary`.
    Sort results by `downloads_count` descending.
  - `async fn fetch_mod_details(name: &str) -> Result<Mod>` — calls
    `GET /api/mods/{name}/full`. Maps the JSON response into the `Mod` type from
    `types.rs`. The full endpoint returns `info_json.dependencies` on each release
    (see `factorio_mod_api.md`, Release → InfoJson).
  - `async fn download_mod(release: &Release, username: &str, token: &str, dir: &str) -> Result<()>`
    — downloads the zip to `dir/release.file_name`. Uses the authenticated download
    URL format from `factorio_mod_api.md`: `https://mods.factorio.com{download_url}?username={}&token={}`.
    After download completes, verify the file is a valid zip archive (open it with
    the `zip` crate). If verification fails, delete the file and return an error.
    If the HTTP response is non-200, return an error with a clear message (likely
    auth failure — suggest checking credentials).
- [ ] All functions return `anyhow::Result`. No `Box<dyn Error>`.
- [ ] Verify: write a small test or example that fetches the mod list and prints the
  first 5 entries (can be `#[ignore]` test since it hits the network).

#### Checkpoint 2.2 — Installed mods
- [ ] Implement `factorio/installed.rs`:
  - `InstalledMod` struct: `name`, `version`, `title`, `factorio_version`.
  - `fn read_installed_mods(mods_dir: &str) -> Result<Vec<InstalledMod>>` — scans
    directory for `.zip` files, reads `info.json` from each zip archive, parses
    into `InstalledMod`.
  - `fn delete_mod(mod_name: &str, mods_dir: &str) -> Result<()>` — finds and
    deletes the zip file. Match precisely: the filename format is
    `{mod_name}_{version}.zip` so match on `{mod_name}_` prefix (with underscore),
    not bare `mod_name`.
- [ ] Verify: unit test that `delete_mod("bob", ...)` would NOT match
  `boblogistics_1.0.0.zip`.

#### Checkpoint 2.3 — mod-list.json
- [ ] Implement `factorio/mod_list.rs`:
  - `ModList` struct wrapping `HashMap<String, ModEntry>`.
  - `ModEntry`: `name`, `enabled: bool`.
  - `fn load_or_create(mods_dir: &str) -> Result<ModList>`.
  - `fn save(&self, mods_dir: &str) -> Result<()>`.
  - Always includes `base` as enabled.
  - The on-disk format is Factorio's native `mod-list.json`:
    ```json
    {
      "mods": [
        { "name": "base", "enabled": true },
        { "name": "some-mod", "enabled": true },
        { "name": "other-mod", "enabled": false }
      ]
    }
    ```
    Internally we use a HashMap keyed by name for O(1) lookups, but serialize
    to/from the array format Factorio expects.
- [ ] Verify: round-trip test (create → save → load → compare).

#### Checkpoint 2.4 — Dependency resolver
- [ ] Implement `factorio/resolver.rs`:
  - The resolver must be testable without hitting the network. Accept an
    `async fn`/trait/closure for fetching mod details so tests can provide
    fake responses.
  - `struct ResolveResult { to_download: Vec<(String, Release)> }`.
  - The resolve function takes: mod name, factorio version, list of installed
    mods, and the fetch function. It returns `Result<ResolveResult>`:
    1. Fetch full mod details via the provided fetch function.
    2. Find the latest release where `release.factorio_version` matches the
       server's Factorio version (compare major.minor).
    3. If no compatible release exists, return error.
    4. Parse dependencies from that release's `info_json.dependencies`
       (see `factorio_mod_api.md`, Release → InfoJson → dependencies).
    5. For each dependency:
       - **Optional / hidden optional (`?`, `(?)`):** skip entirely.
       - **Incompatible (`!`):** check if that mod is in `installed`. If yes,
         return error describing the conflict. If no, skip (nothing to do).
       - **Required (no prefix, or `~`):**
         - Skip `base`.
         - Skip if already in `installed` (and installed version satisfies
           the version constraint).
         - Skip if already in the `to_download` list (cycle/duplicate detection
           via a visited set keyed on mod name).
         - Recurse: fetch that dependency's details and resolve its deps.
         - Pick the latest release that satisfies the version constraint AND
           matches `factorio_version`.
    6. Return flat list of all mods to download, in dependency-first order.
  - **Error handling during resolution:** If fetching a dependency's details
    fails (network error, mod not found on portal), return an error that names
    the failing dependency and the root mod that needed it.
- [ ] Write unit tests using fake/mock fetch functions:
  - Test that `base` is skipped.
  - Test cycle detection (A depends on B, B depends on A).
  - Test that incompatible deps produce a blocking error.
  - Test that already-installed mods are skipped.
  - Test that version constraints are respected.
  - Test that optional dependencies are not downloaded.
- [ ] Verify: `cargo test` passes.

---

### Phase 3: TUI Application

#### Checkpoint 3.1 — Event system
- [ ] Implement `event.rs`:
  - `enum AppEvent`:
    - `Key(KeyEvent)` — terminal key press
    - `Tick` — periodic UI refresh
    - `ModListLoaded(Result<Vec<ModListEntry>>)` — async result
    - `ModInstalled(Result<String>)` — mod name on success, error on failure
    - `ModDeleted(Result<String>)` — mod name on success
    - `InstalledModsLoaded(Result<(Vec<InstalledMod>, ModList)>)` — startup load
    - `Error(String)` — general error display
  - `fn spawn_event_loop(tx: mpsc::Sender<AppEvent>)` — spawns a tokio task that
    reads crossterm events and sends `Key`/`Tick` variants. Tick rate ~250ms.
- [ ] Verify: compiles.

#### Checkpoint 3.2 — App state
- [ ] Implement `app.rs`:
  - `App` struct — owns all state directly, no Arc/Mutex:
    - `tab: Tab` (Manage / Install)
    - `active_block: ActiveBlock` (which panel has focus)
    - `install_mods: Vec<ModListEntry>` (catalog from portal)
    - `install_filter: String` (search text)
    - `install_selected: Option<usize>` (cursor in filtered list)
    - `manage_mods: Vec<ManageMod>` (installed mods with enable/disable state)
    - `manage_selected: Option<usize>`
    - `status_message: Option<(String, Instant)>` (transient status bar message)
    - `factorio_version: String`
    - `server_settings: ServerSettings`
    - `mods_dir: String`
    - `should_quit: bool`
    - `show_quit_popup: bool`
    - `loading: bool` (true until initial mod list fetch completes)
    - `installing: bool` (true while an install task is in progress — prevents
      launching a second concurrent install)
  - `ManageMod` struct: `installed_mod: InstalledMod`, `enabled: bool`.
  - Helper methods:
    - `fn filtered_install_mods(&self) -> Vec<&ModListEntry>` — filter by
      `install_filter`, case-insensitive match on name and title.
    - `fn set_status(&mut self, msg: String)` — sets status with current time.
    - `fn clear_expired_status(&mut self)` — clears status if older than 5 seconds.
    - Navigation: `move_up()`, `move_down()`, `select_tab()`, `set_active_block()`.
- [ ] Verify: compiles.

#### Checkpoint 3.3 — Rendering
- [ ] Implement `ui.rs` — all rendering in one file for MVP:
  - `pub fn draw(app: &App, frame: &mut Frame)` — top-level draw function.
  - Layout: tab bar at top, main content area, status bar at bottom.
  - Tab bar: highlights active tab.
  - **Manage tab:** single list of installed mods. Each item shows title with a
    `✔` prefix if enabled or blank if disabled. Selected item highlighted.
    If no mods installed, show centered "No mods installed" message.
  - **Install tab:** search bar at top, mod list below. Each item shows title with
    `✔` if already installed. Selected item highlighted.
    While loading initial mod list, show "Loading..." in the list area.
  - **Status bar:** shows keybinding hints by default. Shows transient messages
    (success/error) when `status_message` is set.
  - **Quit popup:** centered overlay asking "Save changes? (y/n)" when
    `show_quit_popup` is true.
  - All draw functions are pure: `fn(state, frame)`. No side effects. No task
    spawning. No mutex locking.
- [ ] Verify: compiles (rendering won't be visually testable until checkpoint 3.5).

#### Checkpoint 3.4 — Input handling
- [ ] Implement `handler.rs`:
  - `pub fn handle_event(event: AppEvent, app: &mut App, tx: mpsc::Sender<AppEvent>)`:
    - `Key` events → match on active block, dispatch to key handler.
    - `Tick` → call `app.clear_expired_status()`.
    - `ModListLoaded(Ok(mods))` → store in `app.install_mods`, set `loading = false`.
    - `ModListLoaded(Err(e))` → set status to error message.
    - `ModInstalled(Ok(name))` → refresh installed mods list, set success status.
    - `ModInstalled(Err(e))` → set status to error message.
    - `ModDeleted(Ok(name))` → remove from manage list, mark as uninstalled in
      install list, set success status.
    - `InstalledModsLoaded(Ok((mods, mod_list)))` → populate `manage_mods` with
      enable/disable state from mod_list.
    - etc.
  - Key handlers by active block:
    - **Global:** `Ctrl+C` → show quit popup. `Tab` → switch tabs.
    - **ManageModList:** `Up/Down` → move cursor. `Enter` → toggle enable/disable.
      `d` → delete mod (spawn async task that calls `delete_mod`, sends
      `ModDeleted` result back).
    - **InstallModList:** `Up/Down` → move cursor. `Enter` → install mod (spawn
      async task that runs resolver then downloads all mods, sends `ModInstalled`
      result back). `/` or any char → switch to search block.
    - **InstallSearch:** chars → append to filter, reset cursor. `Backspace` →
      pop from filter. `Enter` or `Down` or `Esc` → switch to install list.
    - **QuitPopup:** `y` → save mod-list.json + quit. `n` → quit without saving.
      `Esc` → close popup.
  - When spawning async tasks, clone only the data the task needs (mod name,
    credentials, paths). Never pass app state to a task.
- [ ] Verify: compiles.

#### Checkpoint 3.5 — Main loop integration
- [ ] Implement `main.rs`:
  - Parse CLI args. If `init` subcommand, run `FmlConfig::init()` and exit.
  - Load config via `FmlConfig::load()`.
  - Read server settings from config's `server_config_path`.
  - Detect Factorio version from mods directory.
  - Create `App` with initial state.
  - Create event channel (`tokio::sync::mpsc`).
  - Spawn terminal event loop.
  - Spawn initial async tasks:
    - Fetch mod list from portal (sends `ModListLoaded` when done).
    - Read installed mods from disk (sends `InstalledModsLoaded` when done).
  - Enter main loop:
    ```
    loop {
        terminal.draw(|frame| draw(&app, frame))?;
        if let Some(event) = rx.recv().await {
            handle_event(event, &mut app, tx.clone());
        }
        if app.should_quit { break; }
    }
    ```
  - On exit: restore terminal (disable raw mode, leave alternate screen, show cursor).
  - Panic hook that restores terminal before printing panic.
- [ ] Verify: `cargo run` launches the TUI, shows loading state, then displays
  the mod list. Tab switches between manage and install. `Ctrl+C` shows quit
  popup. `n` or `y` exits cleanly.

---

### Phase 4: Core Features Working End-to-End

#### Checkpoint 4.1 — Search and browse
- [ ] Typing in the install tab filters the mod list in real time.
- [ ] Filtering is case-insensitive and matches on both `name` and `title`.
- [ ] Cursor resets to top when filter changes.
- [ ] `/` key activates the search bar from the mod list.
- [ ] `Esc` or `Down` from search bar returns focus to the mod list.
- [ ] Verify: launch app, type a mod name, see filtered results.

#### Checkpoint 4.2 — Install with dependency resolution
- [ ] Pressing `Enter` on a mod in the install list triggers installation.
  If `app.installing` is true, ignore the keypress (prevent concurrent installs).
- [ ] The install flow is a single async task spawned from the handler:
  1. Run the resolver to get the list of mods to download. The resolver is called
     with `api::fetch_mod_details` as the fetch function (this is where the
     test-injectable interface from 2.4 gets wired to the real API).
  2. Download each mod in the list sequentially to the mods directory.
  3. If any download fails: delete the partially-written file for the failed
     download (don't leave corrupt zips in the mods dir). Mods that were already
     successfully downloaded earlier in the list are **kept** — no rollback of
     successful downloads. Return an error that names which mod failed and how
     many were successfully downloaded before it.
  4. Re-read installed mods from disk to get the updated state.
  5. Send a result message back through the event channel with the updated
     installed mods list (or error).
- [ ] The handler processes the result: updates `manage_mods`, marks mods as
  installed in `install_mods`, sets status message.
- [ ] Already-installed dependencies are skipped by the resolver.
- [ ] `base` is always skipped.
- [ ] Incompatible mod conflicts block the install with an error shown in the
  status bar.
- [ ] On success: the mod appears in the manage tab as enabled, and shows `✔` in
  the install tab.
- [ ] Status bar shows result: "Installed X + N dependencies" or error message.
- [ ] Verify: install a mod with known dependencies (e.g., a Bob's or Angel's mod).
  Check that all dependency zips appear in the mods directory.

#### Checkpoint 4.3 — Enable/disable and manage
- [ ] Manage tab lists all installed mods with `✔` for enabled, blank for disabled.
- [ ] `Enter` toggles the enabled state.
- [ ] `d` deletes the selected mod's zip file from disk and removes it from the list.
- [ ] Deletion uses precise filename matching: `{mod_name}_{version}.zip`.
- [ ] Verify: toggle a mod, delete a mod, confirm file is gone from disk.

#### Checkpoint 4.4 — Save on exit
- [ ] `Ctrl+C` opens the quit popup.
- [ ] `y` writes `mod-list.json` to the mods directory with current enable/disable
  state, then exits.
- [ ] `n` exits without writing.
- [ ] `Esc` closes the popup without quitting.
- [ ] The saved `mod-list.json` includes `base` as always-enabled plus all
  installed mods with their current state.
- [ ] Verify: enable/disable some mods, quit with `y`, inspect `mod-list.json`.

---

### Phase 5: Polish (Post-MVP)

These are not part of the MVP but are listed here for future reference. Do not
implement these unless explicitly asked.

- [ ] Download progress bar in the tab bar area
- [ ] Mod detail panel (description, dependency list, download count)
- [ ] Markdown rendering for mod descriptions
- [ ] Mod update checking (compare installed version vs. latest portal version)
- [ ] Import from existing `mod-list.json` (download all listed mods)
- [ ] Sorting/filtering options in the manage tab
- [ ] Confirmation prompt before deleting a mod
- [ ] Retry logic for failed API calls
- [ ] Logging to file for debugging
