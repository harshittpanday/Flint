# Flint architecture

## System boundary

Flint is a Tauri desktop application. React owns presentation and user intent; Rust owns every operation that accesses launcher data, the network, Java, or operating-system processes. This keeps the security-sensitive and platform-sensitive launcher core out of the webview and makes it independently testable.

## React frontend

`src/App.tsx` loads profiles, settings, the Mojang catalog, and Java inventory; it coordinates profile editing, play state, navigation, and window behavior. `ProfileForm` resolves Fabric/preset compatibility before save, while `ModManager` provides the intentionally small Modrinth surface. `ImportSetup` and `CosmeticsManager` use typed IPC and the scoped Tauri dialog plugin; filesystem validation and copying remain in Rust. `artwork.ts` maps versions to bundled artwork descriptors, while `HeroMedia` selects an optional local video or its required static fallback.

The UI still performs immediate validation for good feedback, but Rust repeats all validation because webview input is never trusted across IPC.

## Tauri IPC boundary

Narrow typed commands cover profiles, settings, catalog/loader discovery, Modrinth operations, Java inventory, and launch. Commands return serialized domain values or an `AppError` containing a stable code, a user-facing message, and optional technical detail. The backend emits `launcher-status` events rather than exposing raw downloader/process objects.

The current Tauri capability grants only core main-window behavior. Filesystem and shell plugins are not exposed to the frontend.

## Rust backend

`lib.rs` is the composition root, not the launcher implementation. It initializes app-data paths and rotating daily logs, owns the atomic launch guard, and connects commands to focused modules:

- `paths`: deterministic OS application-data layout.
- `profiles`: validation, durable JSON storage, and migration-safe optional-client state.
- `importer`: bounded, read-only inspection of an external Minecraft directory and selective copying into one instance.
- `cosmetics`: PNG validation plus isolated skin/cape files and preferences.
- `settings`: validated launcher preferences with safe defaults.
- `java`: deduplicated candidate discovery plus real `java -version` execution.
- `runtime_manager`: checksum-verified, atomic installation and reuse of versioned Temurin runtimes.
- `flint_client`: exact compatibility policy, embedded Fabric artifact, and versioned profile-local configuration.
- `autoauth`: per-server non-secret rules, Windows Credential Manager references, and the ephemeral loopback command bridge.
- `presence`: optional, failure-isolated Discord IPC activity adapter.
- `process_command`: common child-command construction and Windows release console policy.
- `minecraft::catalog`: validated cached Mojang version catalog with stale fallback.
- `minecraft::fabric`: compatible loader discovery and launch-plan overlay.
- `minecraft::modrinth`: compatibility-filtered per-profile mod lifecycle and presets.
- `minecraft::metadata`: Mojang models and rule evaluation.
- `minecraft::download`: streamed, integrity-checked, atomically promoted cached files.
- `minecraft::install`: preparation orchestration and native extraction.
- `minecraft::arguments`: modern JVM/game argument expansion.
- `minecraft::process`: child lifecycle and exit reporting.

This separation allows a future CLI/test harness to use launcher modules without React.

## Minecraft metadata and downloads

Flint begins at Mojang's `version_manifest_v2.json`, caches it for one hour, filters releases/snapshots for the UI, verifies the selected version JSON, and deserializes arguments, downloads, libraries, rules, assets, Java, natives, and logging metadata. Version 26.2 is the default rather than a hard restriction.

Library and conditional argument rules are evaluated for Windows and no optional launcher features. Downloads stream into unique sibling temporary files while computing SHA-1 and byte counts, flush to disk, and are promoted only after validation. Windows uses replace-existing/write-through move semantics; failed attempts clean temporary files without making partial data look final. Existing files are reused only when size and SHA-1 still match, and concurrent requests for one destination share a process-local lock. Assets retain bounded concurrency of 12 across different destinations. Shared immutable data lives under `minecraft`; version-specific natives live beside the version.

HTTP 408/425/429 and 5xx responses plus transient transport/locking failures receive at most three attempts with backoff; permanent HTTP failures stop immediately. Clients have connect and total-operation timeouts. The catalog validates cached JSON before fresh or stale reuse and atomically replaces refreshed metadata. Resumable range transfers are not implemented.

## Java and process launch

Flint reads the required Java major from the selected Mojang version metadata. A manual path has strict precedence; otherwise Flint considers managed runtimes, `JAVA_HOME`, `PATH`, and common Windows vendor locations, accepting only executable 64-bit candidates with the exact major. Candidate aliases are deduplicated by resolved `java.home`. If enabled and no match exists, the runtime manager queries Eclipse Adoptium, downloads a Temurin JRE (or JDK fallback), verifies provider SHA-256 and archive bounds, extracts without traversal/symlinks into staging, executes and validates the candidate, then atomically promotes it to `runtimes/java-<major>`. Majors coexist and are reused without modifying system Java.

Argument construction expands Mojang placeholders, joins the Windows classpath, adds the resolved logging configuration, and creates the conventional deterministic UUID v3 from `OfflinePlayer:<username>`. The access token remains a non-authenticated sentinel; Flint does not forge Microsoft credentials.

Minecraft runs in the profile's isolated game directory. Standard output and error append to the local Minecraft log. Every launcher-owned command—including `where.exe`, Java inspection, and the Minecraft/Fabric Java process—uses the shared process-command policy. Windows release builds apply `CREATE_NO_WINDOW`; debug builds retain their development console behavior. An atomic state flag prevents duplicate launches until the child exits, and exit status is returned to the interface through a structured event.

## Instances and shared cache

Each profile owns `instances/<id>/game`, so its worlds, options, servers, resource packs, screenshots, and future mods/config cannot leak into another instance. Large immutable Mojang assets and libraries are shared centrally because duplicating verified content provides no isolation benefit and wastes disk.

Profiles are non-secret metadata stored in `profiles/profiles.json`; old Milestone 1 records deserialize with safe defaults. Writes use a sibling temporary file before replacement. Confirmed deletion removes the UUID-addressed profile record and isolated instance directory. Duplicate creates a new UUID and empty isolated game directory rather than copying worlds implicitly.

The importer accepts only a recognizable external Minecraft directory, rejects Flint-managed sources, skips symlinks, bounds individual files, and constructs destinations from known categories. It never reads launcher credential databases. Worlds are off by default. Imported Fabric JARs are opened only as ZIP data and never executed. Flint evaluates common Fabric semantic predicates, loader/environment constraints, and required local dependencies into Compatible, Incompatible, or Needs Review. SHA-1-identifiable Modrinth files can instead be reinstalled through the existing dependency resolver. Modular Fabric API JARs are consolidated to avoid duplicate package imports.

Cosmetics live under `instances/<id>/flint/cosmetics`. PNG headers, dimensions, and size are validated before copying. Preview bytes cross IPC by profile ID and become revocable `blob:` URLs; raw Windows paths never enter image sources. For an enabled compatible client, launcher preparation copies only the fixed managed skin/cape files into the game directory and emits protocol-v1 relative paths. The local-player mixin registers each PNG as a dynamic `TextureManager` texture and supplies its direct render identifier; it never routes the ID through Minecraft's resource-pack path constructor.

## Fabric and Modrinth

Fabric is an overlay on the prepared vanilla launch plan. Flint retrieves the official Fabric launcher profile, resolves Maven coordinates, obtains missing SHA-1 sidecars, appends verified libraries, merges arguments, and replaces the main class with KnotClient. Vanilla preparation remains unchanged.

Modrinth search and version queries are filtered by the profile's exact Minecraft version and Fabric. Managed JARs are SHA-1 verified under that profile's game/mods directory and tracked in an instance-local manifest. Required dependencies are resolved transitively; filenames are constrained to safe JAR basenames. Presets call this same resolver and must be previewed first.

## Error propagation and logging

Expected failures cross IPC as readable structured errors. The frontend preserves optional technical detail behind the status panel disclosure while leading with an actionable message. Download messages name the artifact; logs include the query-free URL, HTTP status, destination, integrity result, attempt, and underlying I/O/network error. Query strings and URL credentials are removed before logging, and account/AutoAuth secrets never enter downloader metadata. Logs also cover path setup, Java selection, process start, launch failure, and process exit. Discord activity uses the public Flint Application ID `1547183366091051019`, initializes when Flint starts, follows browsing/preparing/downloading/launching/playing states, and makes at most one failed connection attempt per enabled session. Tokens/passwords do not exist in this milestone and future sensitive values must be redacted before logging.

## Optional Flint Client

`flint-client/` is a Fabric Loom project and its remapped 0.3.0 artifact is embedded in the launcher. Enabling is allowed only for Fabric 1.21.11, installs only into that profile's `mods`, and writes launcher-owned `flint/client-v1.json`. Unsupported profile edits disable the managed integration without blocking ordinary Minecraft.

The client separately owns `flint/client-settings-v1.json` inside that instance. It uses a schema version, safe defaults, tolerant field migration, corruption fallback, and atomic replacement. The launcher never rewrites this file. A small registry supplies stable module IDs/categories; input, tick, and HUD mixins delegate to one runtime coordinator. Right Shift is the default menu key, press events are consumed once, ESC closes normally, and rebinding rejects conflicts with Minecraft's current bindings. This is an initial framework, not a claim that the complete future module catalog exists.

## AutoAuth

AutoAuth rules live outside profile JSON under the UUID instance and contain an opaque credential reference, exact server address, templates, and a Disabled/Login/Register mode. Legacy enabled rules deserialize as Login. Secrets use Windows Credential Manager. On launch a random-token loopback bridge exists only for the Minecraft child lifetime; endpoint/token travel through its environment, never disk. Each game-join event resets the client state, creates a random connection UUID, and after 60 ready client ticks sends the current server address and session ID to the bridge. The bridge validates the UUID, selects the configured mode, and returns one rendered command. The client verifies that the original network handler is still current, and both sides enforce one attempt per connection/session while allowing a later reconnect. No server/plugin prompt heuristics are claimed.

## Home artwork and motion

Every hero descriptor has a bundled static image and may add a bundled WebM/MP4 later. `HeroMedia` uses muted, looping, metadata-preloaded video only when configured, falls back on playback/load failure, renders the image for reduced-motion users, and pauses video while the document is hidden or the launcher is unfocused. No media URL is remote and no tracking or autoplay audio is present.

## Future architecture (not implemented)

### Additional loaders and richer mod solving

Forge and NeoForge should implement provider adapters rather than being folded into vanilla preparation. Future mod solving must add conflict, optional dependency, update, and enable/disable modeling while preserving the current provenance/checksum manifest.

### Microsoft authentication

Authentication should be a separate account service producing short-lived launch credentials. Refresh tokens must live in Windows Credential Manager or an equivalently protected OS facility, never profile JSON or frontend storage. Entitlement and Minecraft-profile checks are required before launching an authenticated identity.

### Server Vault

A broader vault, account sharing, generated credentials, automatic plugin detection, and export remain out of scope. Any future design must preserve the current OS-backed secret boundary and explicit audit-safe lifecycle.

### Streamer mode

Presence already receives only redacted launcher/game state and never server addresses, usernames, or profile names. A later streamer mode can extend that same redacted activity boundary to future integrations.
