# Flint development history

Entries are append-only and record what was true when work was performed.

## 2026-09-07 — Milestone 1 foundation (0.1.0)

### Implemented

- Created the Tauri 2, React 19, TypeScript, Vite, and Rust project from an empty repository.
- Added local/offline profile validation and JSON persistence with stable IDs and timestamps.
- Added isolated per-profile game directories plus shared version, library, and asset caches.
- Implemented Java discovery through `JAVA_HOME`, `PATH`, and common Windows vendor directories, with executable/version validation.
- Implemented Mojang manifest/version metadata parsing, rule-aware libraries and arguments, SHA-1/size verification, bounded asset downloads, Windows native extraction, logging configuration, and child Java process monitoring.
- Added typed IPC calls and structured launcher status events for Ready, Preparing, Downloading, Launching, Running, Failed, and Finished states.
- Added project, architecture, security, contribution, roadmap, and repository-map documentation.

### Technical decisions

- Limited Milestone 1 to vanilla Minecraft Java Edition 26.2 and Java 25 after verifying Mojang's live manifest and version metadata on 2026-09-07. This keeps the foundation current without pretending to support every historical argument format and platform combination.
- Kept mutable instance game data separate while sharing immutable Mojang artifacts to balance isolation and disk usage.
- Derived offline UUIDs with the established `OfflinePlayer:<username>` UUID v3 convention. No session is forged and no online authentication is bypassed.
- Kept networking and process control in Rust; the frontend only sends typed user intent and renders status.

### Bugs and fixes

- Vite/esbuild could not load the local configuration inside the restricted verification sandbox. The same production build passed in the normal Windows filesystem context.
- The first Vitest run reported no tests. Added focused profile-validation tests rather than allowing empty test suites.
- Rust was absent, so Rust 1.98.1 and rustfmt were installed. Cargo then identified the missing Microsoft C++ linker prerequisite; installing Visual Studio 2022 Build Tools with the C++ workload resolved it.
- Tauri's Windows resource build required an application icon. Added a source Flint SVG and generated the standard desktop/mobile icon assets.
- The current React Hooks lint rules flagged synchronous form-state resets in an effect. Removing the unnecessary effect fixed the issue.
- A Tauri build retry initially could not find newly installed Cargo from npm's process environment. Adding Cargo's bin directory to that build process resolved it.
- The first runtime-data ignore rule for `minecraft/` also matched the Rust `src/minecraft` module. Anchoring runtime directories to the repository root kept launcher source trackable while retaining runtime protection.
- A registry check found newer maintained frontend majors. Flint moved to Vite 8.2.2, ESLint 10.10.0, and Vitest 5.0.0; TypeScript remains on 5.9.3 because the maintained TypeScript ESLint release does not yet support TypeScript 7.

### Verification and limitations

- `npm run build` passed using Vite 8.2.2 with 24 modules transformed.
- `npm run lint` passed using ESLint 10.10.0 with no findings.
- Vitest 5.0.0 passed 3 frontend profile-validation tests.
- `cargo fmt --check` passed, and `cargo check` passed without warnings.
- `cargo test` passed 5 Rust unit tests covering profile persistence/validation, Java parsing, metadata rules, and placeholder replacement.
- `tauri build --debug --no-bundle` succeeded and produced `src-tauri/target/debug/flint.exe`.
- Mojang's live manifest reported 26.2 as the current release, with Java major 25, 131 libraries, asset index 32, modern arguments, and a logging configuration.
- The native application compiles, but an actual Minecraft window/process launch is not verified. This machine's installed Java 21 cannot run the selected Java 25 game.
- No claim is made that Minecraft has launched successfully.

## 2026-09-08 — Milestone 2 foundation (0.2.0)

### Verified Milestone 1 baseline

- The project owner reported manual verification that the existing 26.2 offline profile reached the main menu, reused its cache, and connected to multiplayer before Milestone 2 began.
- Existing logs corroborate a Java 25 Minecraft 26.2 process start on 2026-09-07, a running state, and a normal exit after 3 minutes 20 seconds. This historical evidence is not presented as a new Milestone 2 GUI test.

### Implemented

- Replaced the 26.2-only profile restriction with Mojang's dynamic release catalog, a one-hour manifest cache with stale-cache fallback, and optional snapshot visibility.
- Made Java selection metadata-driven, added 64-bit runtime validation, preserved automatic selection, and added a manual Java executable setting.
- Expanded isolated profiles with Vanilla/Fabric loader selection, Fabric Loader version, preset, per-profile RAM, last-played time, edit/delete/duplicate operations, and destructive deletion confirmation in the UI.
- Added Fabric Meta loader discovery and launcher-profile merging, including checksum-verified Fabric Maven libraries and the Knot client entry point.
- Added Modrinth Fabric search, compatibility-filtered installation, required dependency resolution, instance-local managed-mod manifests, listing, replacement, and removal.
- Added live-resolved Vanilla, Performance (Sodium, Lithium, Entity Culling), Visuals (Iris plus declared required dependencies), and Custom preset flows with a pre-install preview.
- Added launcher settings for RAM, resolution, snapshot visibility, Java selection, Discord Rich Presence, and keep/minimize/hide behavior.
- Added privacy-safe Discord activity states and elapsed time. RPC failures are warnings and never abort launch. A real Discord client ID and registered Flint asset remain a release-time configuration requirement.
- Added bounded three-attempt download retries and real library/asset task counts.
- Configured release builds as Windows GUI applications, aligned version metadata at 0.2.0, and produced x64 MSI and NSIS installers.

### Bugs and fixes

- The old launcher selected Java 25 before reading the chosen version. Metadata resolution now occurs first and the declared Java major drives selection.
- Fabric launcher libraries do not all include hashes in the profile JSON. Flint retrieves the corresponding Maven SHA-1 sidecar before accepting those artifacts.
- Mod installation paths now reject traversal and non-JAR filenames before writing or removing files.
- A sandboxed Java 25 execution reported access denied; the required outside-sandbox rerun and Flint detector test both passed, identifying the failure as sandbox policy rather than a runtime defect.

### Verification and limitations

- Frontend build and lint passed; Vitest passed 4 tests.
- Cargo formatting and check passed; Rust passed 15 tests. Two network-backed live tests passed against Fabric and Modrinth; the separately filtered Mojang fixture test was a conditional no-op because no fixture path was supplied.
- Live API validation found Fabric Loader 0.19.5 stable for 26.2 and compatible current Modrinth builds for all four preset projects.
- Java 21.0.12.1 and Java 25.0.4.1 both executed as 64-bit Temurin runtimes; Flint's focused detector test passed for each major.
- The optimized Tauri build passed and produced both installer formats.
- The 0.2.0 release process launched without a console window. Native computer control was unavailable in this session, so no new vanilla or Fabric main-menu launch is claimed.
