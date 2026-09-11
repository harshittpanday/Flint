# Flint security

## Reporting

This project does not yet publish a security email or hosted disclosure channel. Do not open a public issue containing an exploit, credential, token, private server address, or other sensitive data. Contact the repository owner privately through the hosting platform's private security-reporting feature when one is available. If no private channel is configured, report only that private coordination is needed and withhold sensitive details until a channel is agreed.

## Current scope

Flint stores non-secret offline profile/settings/cosmetics/AutoAuth-rule metadata and downloads public artifacts from Mojang, Fabric, Modrinth, and Eclipse Adoptium HTTPS endpoints. TLS verification remains enabled. Publisher checksums and size bounds are verified before artifacts can execute. Each profile has an isolated mutable game directory and managed-mod manifest.

Local/offline identities are not authenticated sessions. Flint does not create Microsoft tokens, forge sessions, or bypass online-mode server authentication. AutoAuth is an opt-in convenience for a user's own credentials on an exact configured offline-mode server.

## Credentials and logging

- Never commit credentials, tokens, passwords, private keys, `.env` files, runtime profiles, or instance data.
- Never place secrets in URLs, frontend events, crash text, analytics, or process arguments when a safer channel exists.
- Logs may contain paths, public download URLs, process status, and Minecraft output. They must not contain future access/refresh tokens or vault passwords.
- Raw authentication payloads must not be logged when Microsoft support is added.

The `.gitignore` excludes common secrets, logs, and all known Flint/Minecraft runtime directory names. Contributors must still inspect staged files because ignore rules are defense in depth, not secret management.

## Local data assumptions

Anyone with access to the Windows user account can currently read offline profile names, usernames, worlds, settings, and logs in that user's Flint application-data directory. Flint does not claim to encrypt non-secret game data. Filesystem permissions inherit the user's Windows profile protections.

## AutoAuth credentials and local bridge

Passwords are stored by the `keyring` Windows-native backend in Windows Credential Manager. Instance configuration contains only an opaque credential reference, exact server address, enable flag, and command templates. At launch Flint binds an ephemeral loopback-only port and passes its random token to the child process through environment variables. Flint Client can request a command only for an exact enabled server after the player explicitly types `/flintauth login` or `/flintauth register`; the launcher and client each reject repeated attempts. The password is zeroized after command construction and is never sent through Tauri IPC, profile JSON, logs, Discord, diagnostics, imports, or exports.

This boundary protects against accidental disclosure and remote requests; it does not defend against another malicious process already running as the same Windows user and able to inspect the Minecraft process. AutoAuth must remain disabled on untrusted servers. Removing an entry deletes its Credential Manager item, with rollback if configuration persistence fails.

## Dependencies and network

Use maintained dependencies, retain lockfiles, review advisories, and do not disable TLS validation. Minecraft metadata remains untrusted input even when delivered by an official endpoint: paths from archives are constrained during extraction and all metadata errors must fail closed.

Fabric Maven coordinates and Modrinth filenames are untrusted. Flint constrains generated paths, rejects traversal/non-JAR mod names, and only removes files recorded in the selected instance's managed manifest. Discord presence receives only launcher state and game version; server addresses, usernames, and profile names are not sent. The numeric Discord Application ID is public configuration, but bot tokens and client secrets must never be added. RPC connection failure must never block Minecraft.

Existing installations are untrusted input. The importer rejects Flint-managed/non-Minecraft sources, ignores symlinks, copies only known categories, applies a 512 MB per-file bound, and never scans or copies launcher accounts, authentication databases, tokens, logs, or caches. It does not mutate the source and never executes JARs. Uncertain mod metadata is displayed as a warning and skipped; Modrinth identification uses a local SHA-1 digest rather than fuzzy names or file uploads. Cosmetic imports accept PNG only, are capped at 2 MB, validate dimensions, and are copied into the selected UUID instance. Preview IPC is profile-addressed and does not expose filesystem paths to the WebView image source.
