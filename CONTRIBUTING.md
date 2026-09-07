# Contributing to Flint

## Prerequisites and setup

Use Windows 10/11, Node.js 20.19+, npm, stable Rust with the MSVC target, Visual Studio 2022 C++ Build Tools plus a Windows SDK, WebView2, and Java 25.

```powershell
npm install
npm run tauri dev
```

## Required checks

Before submitting behavior changes, run:

```powershell
npm run build
npm run lint
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Use `cargo clippy --all-targets --all-features -- -D warnings` when the native toolchain is available. Launcher changes should also be exercised from a clean throwaway app-data/cache state without deleting a contributor's real instance data.

## Conventions

- Keep React responsible for presentation, forms, status, and typed calls—not direct launcher filesystem or process logic.
- Keep Rust modules focused. Metadata, downloads, runtime selection, arguments, persistence, and process control should remain separable.
- Type IPC inputs and outputs on both sides. Avoid TypeScript `any` and unstructured string protocols.
- Return structured, actionable errors. Log enough technical context to diagnose problems without dumping large payloads.
- Format Rust with rustfmt and follow idiomatic ownership/error handling; do not panic on expected runtime failures.
- Keep TypeScript strict, components small, and validation independently testable.
- Never log or commit credentials, tokens, secrets, runtime instances, or downloaded Minecraft content.
- Preserve instance isolation. Shared cache data must be immutable or safely replaceable.

Update README, TREE, ARCHITECTURE, SECURITY, and the append-only HISTORY when behavior or boundaries change. Update CHECKBOX only after the relevant verification passes; code existing is not sufficient evidence that the behavior works.
