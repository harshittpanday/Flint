# Flint

<img src="branding/flint-logo-source.png" alt="Flint pixel-art logo" width="128" />

Flint is a Windows-first desktop launcher for Minecraft: Java Edition. The project emphasizes a small, clear interface, isolated instances, privacy, and a launcher core that can later support mod loaders without coupling Minecraft installation logic to the UI.

## Current status

Flint is a **post-v0.2.0 beta** consumer Minecraft launcher. The current milestone adds Flint-owned branding, a safer setup migration assistant, and profile-local cosmetics foundations while preserving dynamic Mojang versions, Fabric, isolated Modrinth mods and presets, Discord presence, and release-safe Windows child processes.

The frontend/native suites, live Fabric/Modrinth metadata checks, optimized Tauri executable, and NSIS packaging pass on Windows. The project owner manually verified the earlier Minecraft 26.2 main-menu, cache-reuse, Fabric, Modrinth, and multiplayer baseline. Native UI automation was unavailable for this post-beta run, so this document does **not** claim a new main-menu or visual workflow observation. The current MSI regeneration is blocked by the remote session's unavailable Windows Installer service; the prior MSI artifact remains but is not a current build result.

Release profiles can select versions dynamically from Mojang's official catalog; snapshots are opt-in. Version 26.2 remains the default and regression baseline. A version must provide a Mojang Java requirement and metadata compatible with Flint's modern/legacy argument parser.

## Working features

- Create, edit, duplicate, and delete local/offline profiles with validated Minecraft usernames and deletion confirmation.
- Persist profile identifiers, names, usernames, version selection, and timestamps.
- Give every profile its own `instances/<profile-id>/game` directory.
- Detect installed 64-bit Java runtimes and select the major declared by the chosen version, or validate a manual executable.
- Deduplicate Java aliases that resolve to the same runtime installation.
- Read Mojang's official version manifest and selected version metadata.
- Download the client, libraries, Windows natives, asset index, assets, and logging configuration.
- Verify cached/downloaded files using the Mojang-provided SHA-1 and size where supplied.
- Build modern rule-aware JVM/game arguments and start Minecraft as a child process.
- Suppress visible console windows for launcher-owned helper and Java processes in Windows release builds while retaining captured logs.
- Prevent concurrent preparation/running launches and report progress/errors to the UI.
- Write Flint and Minecraft output logs outside the repository.
- Select Vanilla or a compatible Fabric Loader release per profile.
- Preview and apply compatible Performance or Visuals presets from live Modrinth metadata.
- Search, install, list, replace, and remove compatible Fabric mods per isolated profile.
- Configure RAM, resolution, snapshots, Java selection, Discord presence, and window behavior.
- Preview and selectively import settings, servers, resources, shaders, configs, explicitly compatible Fabric mods, and opt-in worlds from an existing installation without modifying the source.
- Validate and store local PNG skins/capes per profile with Classic/Slim and cape-enable preferences.
- Track a migration-safe optional Flint Client status without making Minecraft launch depend on it.
- Navigate a polished Home, Profiles, Mods, and Settings interface with keyboard focus, loading, disabled, error, and destructive-confirmation states.
- Build normal x64 MSI and NSIS Windows installers with Start Menu/uninstall integration supplied by Tauri.

Offline profiles are not authenticated accounts. They cannot join online-mode servers and do not bypass server authentication.

## Technology

- Tauri 2 and Rust for native filesystem, networking, installation, Java detection, logging, and process work
- React 19 and TypeScript for the UI and typed IPC client
- Vite for frontend development/building
- `reqwest` with rustls, Tokio, Serde, and focused Rust crates in the launcher core

## Prerequisites

- Windows 10 or 11 with WebView2
- Node.js 20.19 or newer and npm
- Rust stable (MSVC target)
- Visual Studio 2022 Build Tools with **Desktop development with C++** and a Windows SDK
- Compatible 64-bit Java runtimes for the Minecraft versions you use (Temurin or Microsoft OpenJDK are suitable)
- Network access to Mojang/Minecraft metadata and asset hosts on first launch

## Development

```powershell
npm install
npm run dev
npm run build
npm run lint
npm test
npm run tauri dev
```

Rust-only checks:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Run `npm run tauri dev`, create a local profile, and select **PLAY**. First preparation downloads the vanilla runtime files and can take several minutes. Subsequent runs validate and reuse the shared cache.

Create Windows release bundles with `npm run tauri build`. Successful MSI and NSIS artifacts are written below `src-tauri/target/release/bundle/`; do not distribute the debug target directory.

## Runtime data

Runtime state is resolved with the operating system's application-data convention and never stored in this repository:

```text
Flint app data/
├── instances/<profile-id>/game/   # isolated mutable Minecraft data
├── minecraft/
│   ├── versions/                  # version metadata, client, natives
│   ├── libraries/                 # shared libraries
│   └── assets/                    # shared indexes, objects, log configs
│   └── metadata/                  # cached Mojang catalog
├── profiles/profiles.json         # non-secret local profile metadata
├── settings/settings.json         # launcher preferences
└── logs/                          # Flint and Minecraft logs
```

## Current limitations

- Windows is the only launch target currently evaluated by the rule resolver and release pipeline.
- Very old releases that omit a Java requirement produce an actionable unsupported-metadata error rather than a guessed runtime choice.
- Microsoft authentication is not implemented.
- No managed Java download, Forge/NeoForge, resource-pack management, server credentials, updater, or telemetry exists.
- Downloads use bounded concurrency but do not yet offer pause/resume or retry controls.
- Required Modrinth dependencies are installed, but optional dependency recommendations, conflicts, and mod updates are not yet modeled in the UI.
- Discord Rich Presence uses Flint's public Application ID `1547183366091051019`. The Discord Developer Portal must retain the registered `flint` image asset; RPC remains optional and failure-isolated when Discord is closed or unavailable.
- Local cosmetics are not Mojang/Microsoft account cosmetics and currently have no in-game effect. The optional Flint Client Fabric mod, its installer/updater, badge, cosmetics rendering, and module registry remain planned architecture—not shipped functionality.
- Importer mod compatibility is intentionally conservative: only Fabric JAR metadata that explicitly names the target Minecraft version is copied. Unknown ranges/dependencies are shown as warnings and skipped.
- MSI and NSIS bundles are unsigned beta artifacts. A post-change vanilla and Fabric main-menu launch still requires manual confirmation.

See [ARCHITECTURE.md](ARCHITECTURE.md), [TREE.md](TREE.md), and [CHECKBOX.md](CHECKBOX.md) for implementation details and roadmap status.
