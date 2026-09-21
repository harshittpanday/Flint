# Flint

<img src="branding/flint-logo-source.png" alt="Flint pixel-art logo" width="128" />

Flint is a Windows-first desktop launcher for Minecraft: Java Edition. The project emphasizes a small, clear interface, isolated instances, privacy, and a launcher core that can later support mod loaders without coupling Minecraft installation logic to the UI.

## Current status

Flint v0.3.0 is a beta consumer Minecraft launcher. Current post-v0.3 work adds conservative cross-version importing, automatic Flint-managed Temurin runtimes, and an optional Fabric 1.21.11 Flint Client with local-only cosmetics, automatic per-server AutoAuth, and an initial in-game menu/module foundation.

The project owner manually verified the earlier Minecraft 26.2 main-menu, cache-reuse, Fabric, Modrinth, Discord, and multiplayer baseline. v0.3 automated and packaging results are recorded in `HISTORY.md`; they do **not** substitute for a new in-game cosmetics, AutoAuth, vanilla, or Fabric main-menu observation.

Release profiles can select versions dynamically from Mojang's official catalog; snapshots are opt-in. Version 26.2 remains the default and regression baseline. A version must provide a Mojang Java requirement and metadata compatible with Flint's modern/legacy argument parser.

## Working features

- Create, edit, duplicate, and delete local/offline profiles with validated Minecraft usernames and deletion confirmation.
- Persist profile identifiers, names, usernames, version selection, and timestamps.
- Give every profile its own `instances/<profile-id>/game` directory.
- Detect installed 64-bit Java runtimes or securely download, checksum, install, and reuse an isolated Temurin runtime matching Mojang's declared major.
- Deduplicate Java aliases that resolve to the same runtime installation.
- Read Mojang's official version manifest and selected version metadata.
- Download the client, libraries, Windows natives, asset index, assets, and logging configuration through streamed, bounded transfers.
- Verify cached/downloaded files using Mojang-provided SHA-1 and size data, replace invalid cache entries through unique temporary files, and atomically promote validated results.
- Build modern rule-aware JVM/game arguments and start Minecraft as a child process.
- Suppress visible console windows for launcher-owned helper and Java processes in Windows release builds while retaining captured logs.
- Prevent concurrent preparation/running launches and concurrent writes to one artifact; report actionable artifact errors with safe technical detail to the UI and logs.
- Write Flint and Minecraft output logs outside the repository.
- Select Vanilla or a compatible Fabric Loader release per profile.
- Preview and apply compatible Performance or Visuals presets from live Modrinth metadata.
- Search, install, list, replace, and remove compatible Fabric mods per isolated profile.
- Configure RAM, resolution, snapshots, Java selection, Discord presence, and window behavior.
- Preview and selectively import settings, servers, resources, shaders, configs, explicitly compatible Fabric mods, and opt-in worlds from an existing installation without modifying the source.
- Validate and store local PNG skins/capes per profile with Classic/Slim and cape-enable preferences.
- Optionally install the bundled Flint Client into an isolated Fabric 1.21.11 profile for local-only skin/cape rendering.
- Open Flint Client with Right Shift, rebind that key with vanilla-conflict warnings, and persist per-instance settings atomically. The current working module set is FPS, coordinates, ping, speed, memory, keystrokes, reach, armor, and client-only Fullbright.
- Store explicitly configured per-server AutoAuth passwords in Windows Credential Manager and deliver one configured login/register command after client readiness through a token-authenticated per-launch loopback bridge.
- Navigate a polished Home, Profiles, Mods, and Settings interface with a data-driven image/optional-video hero, keyboard focus, loading, disabled, error, and destructive-confirmation states.
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
- Network access to Eclipse Adoptium when an exact 64-bit Java major is unavailable locally
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
├── runtimes/java-<major>/         # verified Flint-managed Temurin runtimes
└── logs/                          # Flint and Minecraft logs
```

## Current limitations

- Windows is the only launch target currently evaluated by the rule resolver and release pipeline.
- Very old releases that omit a Java requirement produce an actionable unsupported-metadata error rather than a guessed runtime choice.
- Microsoft authentication is not implemented.
- Forge/NeoForge, Microsoft authentication, resource-pack management, updater, and telemetry are not implemented.
- Downloads use bounded concurrency but do not yet offer pause/resume or retry controls.
- Required Modrinth dependencies are installed, but optional dependency recommendations, conflicts, and mod updates are not yet modeled in the UI.
- Discord Rich Presence uses Flint's public Application ID `1547183366091051019`. The Discord Developer Portal must retain the registered `flint` image asset; RPC remains optional and failure-isolated when Discord is closed or unavailable.
- Flint Client 0.3.0 is intentionally limited to Fabric on Minecraft 1.21.11. Its skins and capes are visible only to the local player and are not Mojang/Microsoft entitlements or server-visible cosmetics. Combo/CPS/effects HUD, HUD editing, zoom/freelook, time/weather, custom hand, trails/particles, audio visualization, toggle sprint, and Auto GG are not implemented yet.
- AutoAuth is only for offline-mode servers the user owns or is authorized to use. It requires Flint Client and an exact per-profile server rule set to Disabled, Login, or Register. On each game-join event the client waits briefly for readiness, requests one command, and then stops; Windows Credential Manager and owned-server behavior still need manual interactive verification.
- Importer compatibility is conservative but range-aware: common Fabric predicates, loader requirements, environment, and required local dependencies are evaluated. Exact SHA-1 matches can be reinstalled through Modrinth's compatible-version resolver; unknown metadata remains review-only and incompatible mods remain skipped.
- MSI and NSIS bundles are unsigned beta artifacts. A post-change vanilla and Fabric main-menu launch still requires manual confirmation.

See [ARCHITECTURE.md](ARCHITECTURE.md), [TREE.md](TREE.md), and [CHECKBOX.md](CHECKBOX.md) for implementation details and roadmap status.
