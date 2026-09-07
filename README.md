# Flint

Flint is a Windows-first desktop launcher for Minecraft: Java Edition. The project emphasizes a small, clear interface, isolated instances, privacy, and a launcher core that can later support mod loaders without coupling Minecraft installation logic to the UI.

## Current status

Flint is at **Milestone 1 (launcher foundation), version 0.1.0**. The React interface, offline profile persistence, isolated game directories, Java 25 detection, Mojang metadata/download pipeline, cache integrity checks, native extraction, launch argument construction, and Java child-process management are implemented.

The frontend production build, lint, frontend tests, Rust compile check, Rust unit tests, and a debug Tauri executable build pass on Windows. An actual Minecraft launch still requires Java 25 and manual verification. Flint does **not** claim a manually verified Minecraft launch yet.

The only supported game version is **vanilla Minecraft Java Edition 26.2**, verified against Mojang's live release manifest on 2026-09-07. Other versions are deliberately rejected in Milestone 1.

## Working features

- Create and edit local/offline profiles with validated Minecraft usernames.
- Persist profile identifiers, names, usernames, version selection, and timestamps.
- Give every profile its own `instances/<profile-id>/game` directory.
- Detect and execute a compatible installed Java 25 runtime.
- Read Mojang's official version manifest and selected version metadata.
- Download the client, libraries, Windows natives, asset index, assets, and logging configuration.
- Verify cached/downloaded files using the Mojang-provided SHA-1 and size where supplied.
- Build modern rule-aware JVM/game arguments and start Minecraft as a child process.
- Prevent concurrent preparation/running launches and report progress/errors to the UI.
- Write Flint and Minecraft output logs outside the repository.

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
- A 64-bit Java 25 runtime (Temurin or Microsoft OpenJDK are suitable)
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

## Runtime data

Runtime state is resolved with the operating system's application-data convention and never stored in this repository:

```text
Flint app data/
├── instances/<profile-id>/game/   # isolated mutable Minecraft data
├── minecraft/
│   ├── versions/                  # version metadata, client, natives
│   ├── libraries/                 # shared libraries
│   └── assets/                    # shared indexes, objects, log configs
├── profiles/profiles.json         # non-secret local profile metadata
└── logs/                          # Flint and Minecraft logs
```

## Current limitations

- Windows is the only launch target evaluated by the Milestone 1 rule resolver.
- Only Minecraft 26.2 and Java 25 are accepted. This development machine currently has Java 21, so a local launch remains blocked until Java 25 is installed.
- Microsoft authentication is not implemented.
- No managed Java download, loader, mods, resource-pack management, server credentials, Discord integration, updater, or telemetry exists.
- Downloads use bounded concurrency but do not yet offer pause/resume or retry controls.
- Profiles can be deleted through the backend IPC, but the current minimal UI exposes create/edit/select only.
- A debug desktop executable builds successfully. Release installer packaging and a real-game launch remain to be manually verified before release.

See [ARCHITECTURE.md](ARCHITECTURE.md), [TREE.md](TREE.md), and [CHECKBOX.md](CHECKBOX.md) for implementation details and roadmap status.
