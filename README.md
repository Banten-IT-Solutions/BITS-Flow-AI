# BITS Flow AI

BITS Flow AI is a Linux desktop app for working with AI providers under isolated, per-account browser
profiles. Each account persona gets its own WebKitGTK data directory, so sessions for Google Flow,
Gemini, Suno, PixVerse, and custom providers never mix. The app combines a Tauri 2 shell, a Svelte 5
interface, and a Rust core that stores everything as local JSON files.

It is **not** an anti-detection tool, credential extractor, or hosted browser farm. Provider
credentials, cookies, and profiles stay on the device, inside the application data directory.

[![CI](https://github.com/Banten-IT-Solutions/BITS-Flow-AI/actions/workflows/ci.yml/badge.svg)](https://github.com/Banten-IT-Solutions/BITS-Flow-AI/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-56e5a3.svg)](LICENSE)

## What is inside

- **Home** — quick launch: pick a provider and an account persona, open the isolated browser window;
  persona cards show which persona is currently active.
- **Accounts** — CRUD for account personas (name, email, color). Each persona maps to
  `app_local_data_dir/profiles/<id>`, a dedicated WebKitGTK data directory.
- **Providers** — dynamic registry of provider URL shortcuts with category (Image / Video / Both).
  Seven defaults are seeded on first run; custom providers can be added, edited, and removed.
- **Generate** — Image and Video tabs that filter the provider registry by category.
- **Prompts** — shared prompt library with categories and pinning.
- **Settings** — language (system / Indonesian / English), theme (system / light / dark), close
  behavior, per-provider engine (system WebKitGTK or Chrome user agent), and the LRU cap of open
  persona windows.

Persona switching is LRU-managed: opening a persona beyond the cap closes the oldest hidden persona
windows (default cap: 3). Opening an existing persona navigates its window instead of creating a new
one, so provider sessions persist across switches within a session.

## Platform support

| Platform | Architecture | Distribution     | Status                    |
| -------- | ------------ | ---------------- | ------------------------- |
| Linux    | x86_64       | `.deb`, AppImage | Built and published by CI |
| Windows  | —            | —                | Not supported             |
| macOS    | —            | —                | Not supported             |

Linux desktop requires WebKitGTK 4.1 (`libwebkit2gtk-4.1-0` is pulled in by the `.deb`; AppImage
bundles its dependencies).

## Install

Download the latest artifacts from
[GitHub Releases](https://github.com/Banten-IT-Solutions/BITS-Flow-AI/releases).

```bash
# Debian / Ubuntu
sudo apt install ./BITS-Flow-AI_1.0.0_amd64.deb

# AppImage
chmod +x BITS-Flow-AI_1.0.0_amd64.AppImage
./BITS-Flow-AI_1.0.0_amd64.AppImage
```

## Security model

- All data stays local in the application-local data directory
  (`~/.local/share/id.co.bits.flowai/`): account, provider, prompt, and settings JSON files plus
  per-persona WebKitGTK profiles under `profiles/`.
- No telemetry. The app only loads the provider URLs you configure, over http/https only; URL
  schemes are validated at the IPC boundary in Rust and again in the UI.
- Provider sessions (cookies, local storage) never leave the device. The app does not read, export,
  or sync them.
- Inputs crossing the Tauri IPC boundary are validated with typed serde structs (IDs, emails,
  categories, engines, URL schemes, LRU bounds) before anything is written to disk.
- Writes are atomic-ish: temp file + rename, so a crash cannot truncate your data.

See [SECURITY.md](SECURITY.md) for reporting and operational guidance.

## Build from source

Prerequisites:

- Node.js 20.19+ (or 22.12+) and npm
- Rust stable (`rustup`)
- Tauri 2 Linux dependencies

```bash
npm ci
npm run lint
npm test
npm run check
npm run build
cargo fmt --check
cargo clippy -- -D warnings
cargo test
npm run tauri build
```

### Linux requirements

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

## Release engineering

- `.github/workflows/ci.yml` — frontend lint/test/type-check/build and Rust fmt/clippy/build/test.
- `.github/workflows/semantic-release.yml` — versions the app from Conventional Commits on `main`,
  tags `${version}`, publishes the GitHub Release with notes, and dispatches the artifact build.
- `.github/workflows/release.yml` — builds the `.deb` and `.AppImage` bundles and attaches them to
  the release.

## License

MIT © 2026 Banten IT Solutions (bitscoid). See [LICENSE](LICENSE).
