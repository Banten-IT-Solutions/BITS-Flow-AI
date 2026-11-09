# Security Policy

## Supported versions

BITS Flow AI 1.x receives security fixes on the `main` branch and in tagged releases. Older
pre-release builds are not supported.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting:

1. Open the repository **Security** tab.
2. Choose **Report a vulnerability**.
3. Include the affected version, platform, reproduction steps, and impact.

Do not open a public issue for a vulnerability. Do not include real provider passwords, session
cookies, or profile data in a report; use synthetic values and redact logs.

If private reporting is unavailable, open a minimal public issue asking the maintainer to establish
a private channel; do not disclose the vulnerability details in that issue.

## Threat model

BITS Flow AI is a local single-user desktop application. The security boundary is the operating
system user account. It does not defend against:

- malware running as the same OS user;
- a compromised operating system, kernel, or WebKitGTK;
- malicious provider pages you choose to open;
- users deliberately exporting their own profile data;
- network attackers who can already intercept TLS or modify the local machine.

The design mitigates:

- accidental data loss via temp-file + rename writes;
- non-HTTP URL schemes (`file://`, `javascript://`, custom schemes) via validation at the IPC
  boundary in Rust and in the UI;
- malformed persisted data via serde types on the Rust side and Zod schemas on the frontend;
- persona label injection via strict persona-ID validation (alphanumeric, `-`, `_`, max length).

## Data handling

- All application data lives in the OS application-local data directory
  (`~/.local/share/id.co.bits.flowai/` on Linux): four JSON files (`accounts.json`,
  `providers.json`, `prompts.json`, `settings.json`) and per-persona WebKitGTK profiles under
  `profiles/`.
- There is no telemetry, analytics, or crash reporting. The app performs no network requests other
  than loading the provider URLs you open.
- Provider session cookies and local storage stay inside each persona's profile directory and are
  never read, exported, or transmitted by the app.
- The Chrome user agent option changes only the `User-Agent` header persona windows send; it does
  not change any other fingerprinting surface.

## Updates and signing

- Release artifacts (`.deb`, `.AppImage`) are built by CI from tags via tauri-action and attached to
  GitHub Releases. There is no in-app updater; install updates manually.
- Verify release asset checksums before installing. There is currently no code signing for release
  artifacts; treat OS warnings about unsigned local packages as expected.
