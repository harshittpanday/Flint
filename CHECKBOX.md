# Flint roadmap

`[x]` means the behavior passed the relevant verification available in this repository. Implemented but unverified runtime behavior remains unchecked.

## Project Foundation

- [x] Initialize Git repository
- [x] Create Tauri/React/TypeScript/Rust structure
- [x] Add frontend build, lint, and test commands
- [x] Add required project documentation
- [x] Ignore build, secret, log, instance, and Minecraft runtime data

## Launcher Core

- [x] Define typed frontend/backend IPC payloads
- [x] Prevent duplicate launches in launcher state
- [ ] Verify complete launcher pipeline with an observed game launch

## Minecraft Installation

- [x] Parse official manifest and modern version metadata in code
- [x] Implement client, library, asset, native, and logging downloads
- [x] Implement SHA-1 and size validation with cache reuse
- [x] Implement bounded asset download concurrency
- [ ] Verify clean first-run installation of Minecraft 26.2
- [ ] Add resumable downloads and retry policy

## Java Runtime

- [x] Detect and execute installed Java candidates
- [x] Enforce Java 25 with actionable errors
- [ ] Add Flint-managed Java runtimes

## Profiles

- [x] Create, edit, select, validate, and persist offline profiles
- [x] Generate stable profile IDs and offline player UUIDs
- [ ] Expose profile deletion in the UI

## Instances

- [x] Create isolated per-profile game directories
- [x] Share immutable libraries/assets centrally
- [ ] Add instance settings and memory controls
- [ ] Add export/import

## Fabric

- [ ] Fabric loader support

## Forge

- [ ] Forge loader support

## NeoForge

- [ ] NeoForge loader support

## Mods

- [ ] Mod search and installation
- [ ] Dependency resolution and updates

## Resource Packs / Shaders

- [ ] Resource-pack management
- [ ] Shader management

## Accounts

- [x] Local/offline identity model
- [ ] Account switcher and authenticated identities

## Microsoft Authentication

- [ ] Microsoft OAuth flow
- [ ] Xbox/Minecraft entitlement and profile flow
- [ ] Secure token storage and refresh

## Server Vault

- [ ] OS-backed encrypted credential storage design
- [ ] Server-specific credential management

## AutoAuth

- [ ] Companion protocol and threat model
- [ ] Optional automatic/login-only/disabled modes

## Streamer Mode

- [ ] Sensitive UI and log redaction mode

## Discord Rich Presence

- [ ] Optional local rich presence

## UI/UX

- [x] Usable profile, version, Play, runtime, and status interface
- [x] Busy/disabled Play states and understandable failures
- [ ] Accessibility and keyboard-flow audit
- [ ] Download speed, byte progress, and cancellation

## Testing

- [x] Frontend production build
- [x] Frontend lint
- [x] Frontend validation tests
- [x] Rust formatting check
- [x] Rust compile check
- [x] Rust unit tests
- [ ] Clean-cache installation test
- [ ] Observed Minecraft launch and normal exit test

## Packaging

- [x] Build debug Tauri desktop executable
- [ ] Tauri installer build
- [ ] Code signing
- [ ] Windows install/uninstall smoke test

## Release

- [ ] Milestone 1 release candidate
- [ ] Publish a release
