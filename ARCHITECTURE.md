# Flint architecture

## System boundary

Flint is a Tauri desktop application. React owns presentation and user intent; Rust owns every operation that accesses launcher data, the network, Java, or operating-system processes. This keeps the security-sensitive and platform-sensitive launcher core out of the webview and makes it independently testable.

## React frontend

`src/App.tsx` loads profiles and Java status, selects or edits one profile, prevents repeated user interaction while launch work is active, and renders structured status events. Components do not know filesystem paths or Mojang URLs. `src/api.ts` is the typed IPC adapter, while `src/types.ts` describes wire payloads.

The UI still performs immediate validation for good feedback, but Rust repeats all validation because webview input is never trusted across IPC.

## Tauri IPC boundary

Five narrow commands exist: list, save, and delete profiles; detect Java; and launch Minecraft. Commands return serialized domain values or an `AppError` containing a stable code, a user-facing message, and optional technical detail. The backend emits `launcher-status` events rather than exposing raw downloader/process objects.

The current Tauri capability grants only core main-window behavior. Filesystem and shell plugins are not exposed to the frontend.

## Rust backend

`lib.rs` is the composition root, not the launcher implementation. It initializes app-data paths and rotating daily logs, owns the atomic launch guard, and connects commands to focused modules:

- `paths`: deterministic OS application-data layout.
- `profiles`: validation and durable JSON storage.
- `java`: candidate discovery plus real `java -version` execution.
- `minecraft::metadata`: Mojang models and rule evaluation.
- `minecraft::download`: integrity-checked cached files.
- `minecraft::install`: preparation orchestration and native extraction.
- `minecraft::arguments`: modern JVM/game argument expansion.
- `minecraft::process`: child lifecycle and exit reporting.

This separation allows a future CLI/test harness to use launcher modules without React.

## Minecraft metadata and downloads

Milestone 1 begins at Mojang's `version_manifest_v2.json`, locates exactly Minecraft 26.2, verifies its version JSON, and deserializes modern `arguments`, downloads, libraries, rules, assets, Java, natives, and logging metadata. It does not copy a hard-coded classpath from a tutorial.

Library and conditional argument rules are evaluated for Windows and no optional launcher features. Downloads are written only after size and SHA-1 checks pass. Existing files are reused only when those checks still succeed. Assets are prepared with a bounded concurrency of 12 to avoid thousands of serial transfers or unbounded resource use. Shared immutable data lives under `minecraft`; version-specific natives live beside the version.

The downloader currently buffers each individual response before committing it. That simplifies integrity and partial-file behavior for Milestone 1 but should evolve into streamed hashing and resumable transfers.

## Java and process launch

Minecraft 26.2 metadata requires Java 25. Flint considers `JAVA_HOME`, `PATH`, and common Windows vendor locations. A candidate counts only when executing `java -version` succeeds and reports exactly the required major version. The detector returns an actionable error rather than panicking.

Argument construction expands Mojang placeholders, joins the Windows classpath, adds the resolved logging configuration, and creates the conventional deterministic UUID v3 from `OfflinePlayer:<username>`. The access token remains a non-authenticated sentinel; Flint does not forge Microsoft credentials.

Minecraft runs in the profile's isolated game directory. Standard output and error append to the local Minecraft log. An atomic state flag prevents duplicate launches until the child exits, and exit status is returned to the interface through a structured event.

## Instances and shared cache

Each profile owns `instances/<id>/game`, so its worlds, options, servers, resource packs, screenshots, and future mods/config cannot leak into another instance. Large immutable Mojang assets and libraries are shared centrally because duplicating verified content provides no isolation benefit and wastes disk.

Profiles are non-secret metadata stored in `profiles/profiles.json`. Writes use a sibling temporary file before replacement. Deleting a profile record intentionally does not delete its instance directory in Milestone 1; destructive data lifecycle needs an explicit recovery design.

## Error propagation and logging

Expected failures cross IPC as readable structured errors. Technical details are sent in the error object and written to logs, while the status panel leads with an actionable message. Logs cover path setup, downloads, Java selection, process start, launch failure, and process exit. Tokens/passwords do not exist in this milestone and future sensitive values must be redacted before logging.

## Future architecture (not implemented)

### Loaders and mods

Fabric, Forge, and NeoForge should implement a loader/provider interface that resolves their metadata into an immutable launch plan. Mod management belongs above that provider layer and must model dependencies, conflicts, provenance, checksums, and per-instance enablement. It must not mutate the vanilla resolver into loader-specific branches.

### Microsoft authentication

Authentication should be a separate account service producing short-lived launch credentials. Refresh tokens must live in Windows Credential Manager or an equivalently protected OS facility, never profile JSON or frontend storage. Entitlement and Minecraft-profile checks are required before launching an authenticated identity.

### Server Vault

Vault records should reference server and account identifiers while secret material remains in OS-backed protected storage. The master/account credential must never become a default per-server password. Imports and generated credentials need explicit audit-safe lifecycle operations without secret logging.

### AutoAuth companion

AutoAuth requires an authenticated, versioned local protocol between Flint and a narrowly scoped client companion. It must only respond to configured offline-mode servers, distinguish login from registration, avoid chat/clipboard exposure, rate-limit attempts, and support Automatic, Login only, and Disabled policies. A threat model is required before implementation.

### Discord Rich Presence

Presence should be an optional adapter receiving a redacted activity model. Server addresses, usernames, and instance names must be suppressible, especially when future streamer mode is active.
