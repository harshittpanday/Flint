# Repository map

```text
Flint/
├── src/                         React/TypeScript interface
│   ├── components/
│   │   ├── ModManager.tsx       Modrinth search/install/remove foundation
│   │   ├── ProfileForm.tsx      Offline profile editor and validation feedback
│   │   └── StatusLog.tsx        Compact launcher progress/error history
│   ├── api.ts                   Typed Tauri IPC/event client
│   ├── App.tsx                  Milestone 2 navigation, settings, profiles, launch state
│   ├── types.ts                 Shared frontend payload shapes
│   ├── validation.ts            Testable frontend input rules
│   └── styles.css               Flint's Windows desktop layout and visual system
├── src-tauri/                   Native Tauri application
│   ├── capabilities/default.json Minimal main-window capability
│   ├── icons/                    Source SVG and generated platform app icons
│   ├── src/
│   │   ├── minecraft/
│   │   │   ├── metadata.rs      Mojang JSON models and OS/feature rule evaluation
│   │   │   ├── catalog.rs       Cached dynamic Mojang version catalog
│   │   │   ├── download.rs      Verified, atomic-ish cached downloads
│   │   │   ├── fabric.rs        Fabric Meta discovery and launch-plan overlay
│   │   │   ├── modrinth.rs      Compatible per-profile mods and presets
│   │   │   ├── install.rs       Vanilla preparation and native extraction pipeline
│   │   │   ├── arguments.rs     Placeholder expansion and offline launch arguments
│   │   │   └── process.rs       Java child process and lifecycle reporting
│   │   ├── error.rs             Structured IPC-safe application errors
│   │   ├── java.rs              Compatible Windows Java discovery/validation
│   │   ├── paths.rs             OS app-data and cache/instance layout
│   │   ├── profiles.rs          Validated JSON profile persistence
│   │   ├── settings.rs          Validated launcher preferences
│   │   ├── presence.rs          Optional privacy-safe Discord RPC adapter
│   │   ├── lib.rs               Tauri state, commands, logging, composition root
│   │   └── main.rs              Desktop binary entry point
│   ├── Cargo.toml               Rust crate dependencies and targets
│   └── tauri.conf.json          Window, build, bundle, and CSP configuration
├── ARCHITECTURE.md              Current and future system design with rationale
├── CHECKBOX.md                  Evidence-based product roadmap
├── CONTRIBUTING.md              Development and quality rules
├── HISTORY.md                   Append-only development journal
├── SECURITY.md                  Reporting and security boundaries
├── README.md                    Factual setup, usage, and current status
└── package.json                 Frontend/Tauri commands and JS dependency manifest
```

Generated dependency trees, build outputs, and all local Minecraft runtime data are intentionally omitted and ignored by Git.
