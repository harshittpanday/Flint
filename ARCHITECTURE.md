# Flint architecture

## System boundary

Flint is a Tauri desktop application. React owns presentation and user intent; Rust owns every operation that accesses launcher data, the network, Java, or operating-system processes. This keeps the security-sensitive and platform-sensitive launcher core out of the webview and makes it independently testable.

## React frontend

`src/App.tsx` loads profiles, settings, the Mojang catalog, and Java inventory; it coordinates profile editing, play state, navigation, and window behavior. `ProfileForm` resolves Fabric/preset compatibility before save, while `ModManager` provides the intentionally small Modrinth surface. Components do not know runtime filesystem paths. `src/api.ts` is the typed IPC adapter and `src/types.ts` describes wire payloads.

The UI still performs immediate validation for good feedback, but Rust repeats all validation because webview input is never trusted across IPC.

## Tauri IPC boundary

Narrow typed commands cover profiles, settings, catalog/loader discovery, Modrinth operations, Java inventory, and launch. Commands return serialized domain values or an `AppError` containing a stable code, a user-facing message, and optional technical detail. The backend emits `launcher-status` events rather than exposing raw downloader/process objects.

The current Tauri capability grants only core main-window behavior. Filesystem and shell plugins are not exposed to the frontend.

## Rust backend

`lib.rs` is the composition root, not the launcher implementation. It initializes app-data paths and rotating daily logs, owns the atomic launch guard, and connects commands to focused modules:

- `paths`: deterministic OS application-data layout.
- `profiles`: validation and durable JSON storage.
- `settings`: validated launcher preferences with safe defaults.
- `java`: candidate discovery plus real `java -version` execution.
- `presence`: optional, failure-isolated Discord IPC activity adapter.
- `minecraft::catalog`: cached Mojang version catalog with stale fallback.
- `minecraft::fabric`: compatible loader discovery and launch-plan overlay.
- `minecraft::modrinth`: compatibility-filtered per-profile mod lifecycle and presets.
- `minecraft::metadata`: Mojang models and rule evaluation.
- `minecraft::download`: integrity-checked cached files.
- `minecraft::install`: preparation orchestration and native extraction.
- `minecraft::arguments`: modern JVM/game argument expansion.
- `minecraft::process`: child lifecycle and exit reporting.

This separation allows a future CLI/test harness to use launcher modules without React.

## Minecraft metadata and downloads

Flint begins at Mojang's `version_manifest_v2.json`, caches it for one hour, filters releases/snapshots for the UI, verifies the selected version JSON, and deserializes arguments, downloads, libraries, rules, assets, Java, natives, and logging metadata. Version 26.2 is the default rather than a hard restriction.

Library and conditional argument rules are evaluated for Windows and no optional launcher features. Downloads are written only after size and SHA-1 checks pass. Existing files are reused only when those checks still succeed. Assets are prepared with a bounded concurrency of 12 to avoid thousands of serial transfers or unbounded resource use. Shared immutable data lives under `minecraft`; version-specific natives live beside the version.

The downloader currently buffers each individual response before committing it. That simplifies integrity and partial-file behavior for Milestone 1 but should evolve into streamed hashing and resumable transfers.

## Java and process launch

Flint reads the required Java major from the selected Mojang version metadata. It considers `JAVA_HOME`, `PATH`, and common Windows vendor locations, and accepts only executable 64-bit candidates with the exact required major. A manual path is validated by the same code.

Argument construction expands Mojang placeholders, joins the Windows classpath, adds the resolved logging configuration, and creates the conventional deterministic UUID v3 from `OfflinePlayer:<username>`. The access token remains a non-authenticated sentinel; Flint does not forge Microsoft credentials.

Minecraft runs in the profile's isolated game directory. Standard output and error append to the local Minecraft log. An atomic state flag prevents duplicate launches until the child exits, and exit status is returned to the interface through a structured event.

## Instances and shared cache

Each profile owns `instances/<id>/game`, so its worlds, options, servers, resource packs, screenshots, and future mods/config cannot leak into another instance. Large immutable Mojang assets and libraries are shared centrally because duplicating verified content provides no isolation benefit and wastes disk.

Profiles are non-secret metadata stored in `profiles/profiles.json`; old Milestone 1 records deserialize with safe defaults. Writes use a sibling temporary file before replacement. Confirmed deletion removes the UUID-addressed profile record and isolated instance directory. Duplicate creates a new UUID and empty isolated game directory rather than copying worlds implicitly.

## Fabric and Modrinth

Fabric is an overlay on the prepared vanilla launch plan. Flint retrieves the official Fabric launcher profile, resolves Maven coordinates, obtains missing SHA-1 sidecars, appends verified libraries, merges arguments, and replaces the main class with KnotClient. Vanilla preparation remains unchanged.

Modrinth search and version queries are filtered by the profile's exact Minecraft version and Fabric. Managed JARs are SHA-1 verified under that profile's game/mods directory and tracked in an instance-local manifest. Required dependencies are resolved transitively; filenames are constrained to safe JAR basenames. Presets call this same resolver and must be previewed first.

## Error propagation and logging

Expected failures cross IPC as readable structured errors. Technical details are sent in the error object and written to logs, while the status panel leads with an actionable message. Logs cover path setup, downloads, Java selection, process start, launch failure, and process exit. Tokens/passwords do not exist in this milestone and future sensitive values must be redacted before logging.

## Future architecture (not implemented)

### Additional loaders and richer mod solving

Forge and NeoForge should implement provider adapters rather than being folded into vanilla preparation. Future mod solving must add conflict, optional dependency, update, and enable/disable modeling while preserving the current provenance/checksum manifest.

### Microsoft authentication

Authentication should be a separate account service producing short-lived launch credentials. Refresh tokens must live in Windows Credential Manager or an equivalently protected OS facility, never profile JSON or frontend storage. Entitlement and Minecraft-profile checks are required before launching an authenticated identity.

### Server Vault

Vault records should reference server and account identifiers while secret material remains in OS-backed protected storage. The master/account credential must never become a default per-server password. Imports and generated credentials need explicit audit-safe lifecycle operations without secret logging.

### AutoAuth companion

AutoAuth requires an authenticated, versioned local protocol between Flint and a narrowly scoped client companion. It must only respond to configured offline-mode servers, distinguish login from registration, avoid chat/clipboard exposure, rate-limit attempts, and support Automatic, Login only, and Disabled policies. A threat model is required before implementation.

### Streamer mode

Presence already receives only redacted launcher/game state and never server addresses, usernames, or profile names. A later streamer mode can extend that same redacted activity boundary to future integrations.
