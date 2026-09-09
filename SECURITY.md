# Flint security

## Reporting

This project does not yet publish a security email or hosted disclosure channel. Do not open a public issue containing an exploit, credential, token, private server address, or other sensitive data. Contact the repository owner privately through the hosting platform's private security-reporting feature when one is available. If no private channel is configured, report only that private coordination is needed and withhold sensitive details until a channel is agreed.

## Current scope

Milestone 2 stores non-secret offline profile/settings metadata and downloads public artifacts from Mojang, Fabric, and Modrinth HTTPS endpoints. TLS verification remains enabled. Downloads are checked against publisher SHA-1 values and sizes where available before use. Each profile has an isolated mutable game directory and managed-mod manifest.

Local/offline identities are not authenticated sessions. Flint does not create Microsoft tokens, forge sessions, or bypass online-mode server authentication. Credential-bearing features are not implemented.

## Credentials and logging

- Never commit credentials, tokens, passwords, private keys, `.env` files, runtime profiles, or instance data.
- Never place secrets in URLs, frontend events, crash text, analytics, or process arguments when a safer channel exists.
- Logs may contain paths, public download URLs, process status, and Minecraft output. They must not contain future access/refresh tokens or vault passwords.
- Raw authentication payloads must not be logged when Microsoft support is added.

The `.gitignore` excludes common secrets, logs, and all known Flint/Minecraft runtime directory names. Contributors must still inspect staged files because ignore rules are defense in depth, not secret management.

## Local data assumptions

Anyone with access to the Windows user account can currently read offline profile names, usernames, worlds, settings, and logs in that user's Flint application-data directory. Flint does not claim to encrypt non-secret game data. Filesystem permissions inherit the user's Windows profile protections.

## Future Server Vault

Server Vault secrets must use Windows Credential Manager, DPAPI-backed storage, or another reviewed OS secure-storage mechanism. Profile JSON must contain references only. Credentials must be server-specific by default, export must be explicit and protected, memory lifetimes should be minimized, and destructive/reset workflows must be recoverable or clearly confirmed.

## Future AutoAuth

AutoAuth requires a threat model before code. The companion must authenticate local messages, bind credentials to an explicitly configured offline-mode server, resist spoofed chat prompts and replay, never print passwords into visible chat or logs, and expose per-server Automatic, Login only, and Disabled controls. Streamer/privacy mode must redact related activity.

## Dependencies and network

Use maintained dependencies, retain lockfiles, review advisories, and do not disable TLS validation. Minecraft metadata remains untrusted input even when delivered by an official endpoint: paths from archives are constrained during extraction and all metadata errors must fail closed.

Fabric Maven coordinates and Modrinth filenames are untrusted. Flint constrains generated paths, rejects traversal/non-JAR mod names, and only removes files recorded in the selected instance's managed manifest. Discord presence receives only launcher state and game version; server addresses, usernames, and profile names are not sent. The numeric Discord Application ID is public configuration, but bot tokens and client secrets must never be added. RPC connection failure must never block Minecraft.
