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
- [x] Verify Milestone 1 launcher pipeline with an observed Minecraft 26.2 game launch (project-owner verification)
- [ ] Re-verify the complete launcher pipeline after Milestone 2 changes

## Minecraft Installation

- [x] Parse official manifest and modern version metadata in code
- [x] Implement client, library, asset, native, and logging downloads
- [x] Implement SHA-1 and size validation with cache reuse
- [x] Implement bounded asset download concurrency
- [x] Verify clean first-run installation and second-run cache reuse for Minecraft 26.2 (project-owner verification)
- [x] Add bounded download retry policy
- [ ] Add resumable downloads

## Java Runtime

- [x] Detect and execute installed Java candidates
- [x] Deduplicate equivalent Java runtime aliases by resolved installation
- [x] Select an installed 64-bit Java major from Mojang version metadata with actionable errors
- [x] Support a validated manual Java executable override
- [ ] Add Flint-managed Java runtimes

## Profiles

- [x] Create, edit, duplicate, delete, select, validate, and persist offline profiles
- [x] Generate stable profile IDs and offline player UUIDs
- [x] Require confirmation before profile/instance deletion

## Instances

- [x] Create isolated per-profile game directories
- [x] Share immutable libraries/assets centrally
- [x] Add per-instance loader, preset, memory, and last-played metadata
- [ ] Add export/import

## Fabric

- [x] Fabric metadata, compatible version selection, verified libraries, and launch-plan integration
- [ ] Observed Fabric Minecraft launch

## Forge

- [ ] Forge loader support

## NeoForge

- [ ] NeoForge loader support

## Mods

- [x] Compatibility-filtered Modrinth search, install, list, replacement, and removal foundation
- [x] Required dependency resolution and per-profile managed-mod manifests
- [ ] Mod update and conflict UI

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

- [x] Failure-isolated, privacy-safe presence adapter and disable setting
- [x] Configure the reviewed public Flint Discord Application ID
- [ ] Live Discord activity verification with a configured Flint application ID/assets

## UI/UX

- [x] Usable profile, version, Play, runtime, and status interface
- [x] Busy/disabled Play states and understandable failures
- [x] Home/Profile/Mods/Settings navigation and obvious Play action
- [x] Consumer launcher layout verified at standard and 780×620 browser viewports
- [ ] Accessibility and keyboard-flow audit
- [ ] Download speed, byte progress, and cancellation
- [x] Real current-library and completed/total task progress

## Testing

- [x] Frontend production build
- [x] Frontend lint
- [x] Frontend validation tests
- [x] Rust formatting check
- [x] Rust compile check
- [x] Rust unit tests
- [x] Windows release child-process console policy tests
- [x] Java runtime identity/deduplication tests
- [x] Discord RPC failure-isolation and activity-label tests
- [x] Live Fabric and Modrinth 26.2 metadata compatibility tests
- [x] Historical clean-cache and normal-exit evidence for the Milestone 1 baseline
- [ ] Post-Milestone 2 observed vanilla main-menu and normal-exit test
- [ ] Observed Fabric main-menu and normal-exit test
- [ ] Manually observe a terminal-free Play and Minecraft session in the Day 3 release build

## Packaging

- [x] Build debug Tauri desktop executable
- [x] Optimized Windows GUI executable with no release console subsystem
- [x] Tauri x64 MSI and NSIS installer builds
- [x] Direct rebuilt 0.2.0 executable smoke test (responsive Flint window and exit code 0)
- [ ] Code signing
- [ ] Windows install/uninstall smoke test

## Release

- [x] Milestone 1 launcher baseline manually verified by the project owner
- [ ] Publish a release
