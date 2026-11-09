<div align="center">
  <h1>BITS Flow AI</h1>
  <p>
    <a href="https://bits.co.id">
      <img src="https://img.shields.io/badge/bits.co.id-Banten%20IT%20Solutions-00C853?style=for-the-badge&logo=statuspage&logoColor=white" alt="bits.co.id" />
    </a>
  </p>
  <p>Desktop profile manager for AI image &amp; video providers — one isolated browser session per account, switchable in one click</p>
  <br>
  <p>
    <a href="https://github.com/Banten-IT-Solutions/BITS-Flow-AI/actions/workflows/ci.yml">
      <img src="https://github.com/Banten-IT-Solutions/BITS-Flow-AI/actions/workflows/ci.yml/badge.svg" alt="CI" />
    </a>
    <img src="https://img.shields.io/badge/Tauri%202-24C8D8?style=flat&logo=tauri&logoColor=white" alt="Tauri 2" />
    <img src="https://img.shields.io/badge/Svelte%205-FF3E00?style=flat&logo=svelte&logoColor=white" alt="Svelte 5" />
    <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat&logo=typescript&logoColor=white" alt="TypeScript" />
    <img src="https://img.shields.io/badge/Vite-646CFF?style=flat&logo=vite&logoColor=white" alt="Vite" />
    <img src="https://img.shields.io/badge/Tailwind%20CSS-38B2AC?style=flat&logo=tailwindcss&logoColor=white" alt="Tailwind CSS" />
    <img src="https://img.shields.io/badge/Vitest-6E9F18?style=flat&logo=vitest&logoColor=white" alt="Vitest" />
    <img src="https://img.shields.io/badge/Linux-FCC624?style=flat&logo=linux&logoColor=black" alt="Linux" />
    <img src="https://img.shields.io/badge/license-MIT-56e5a3?style=flat" alt="MIT" />
  </p>
</div>

---

BITS Flow AI is a Tauri 2 desktop app for managing isolated browser profiles for AI
providers — Google Flow, Gemini, Suno, PixVerse, Leonardo, Krea, and more. Every account
("persona") gets its own WebKitGTK data directory, so cookies, localStorage, and login
sessions never mix. Switching personas keeps sessions alive on disk: no re-login, ever.
It replaces the classic "switch Chrome profile" routine with a purpose-built launcher
for daily free-tier AI image & video workflows.

## ✨ Features

| Feature                   | Description                                                                     |
| ------------------------- | ------------------------------------------------------------------------------- |
| **Persona Accounts**      | One isolated WebKitGTK data dir per account — cookies & sessions never cross    |
| **Hot Profile Switching** | Open persona windows stay cached (LRU) — switching is instant, no re-login      |
| **Dynamic Providers**     | Add any provider: name, URL, category (Image / Video / Both), engine            |
| **Generate Workspace**    | Image & Video tabs filter providers by category and launch with one prompt copy |
| **Prompt Library**        | Save, pin, categorize, and copy prompts; reuse across providers                 |
| **Bilingual UI**          | Bahasa Indonesia & English, configurable in Settings, follows system by default |
| **Dark / Light Theme**    | Theme tokens follow system or fixed, applied live without restart               |
| **Engine Fallback**       | Per-provider engine: WebKitGTK webview or system Chrome subprocess              |
| **Local-first Storage**   | Accounts, providers, prompts, and settings as JSON in the app data dir          |

## 🛠️ Tech Stack

| Layer             | Technology                                                                   |
| ----------------- | ---------------------------------------------------------------------------- |
| **Desktop Shell** | Tauri 2, WebKitGTK (per-window `data_directory` isolation), serde/serde_json |
| **Frontend**      | Svelte 5 (runes), TypeScript 6, Vite 8, Tailwind CSS 4, @lucide/svelte       |
| **i18n**          | Paraglide JS (inlang), compile-time typed messages, `id` + `en`              |
| **Validation**    | Zod at the IPC boundary, serde types on the Rust side                        |
| **Testing**       | Vitest (store logic), cargo test (LRU eviction plan), svelte-check           |
| **Tooling**       | Biome 2 (lint + format), tauri-cli                                           |
| **CI/CD**         | GitHub Actions: CI, semantic-release, Tauri bundle build                     |

## 📁 Project Structure

```text
BITS-Flow-AI/
├── src/
│   ├── lib/
│   │   ├── api.ts               # Zod schemas + typed invoke() wrappers
│   │   ├── stores.ts            # Active menu, accounts, providers, prompts state
│   │   ├── settings.ts          # Locale / theme resolution
│   │   ├── types.ts             # Shared domain types
│   │   ├── paraglide/           # Generated i18n output (git-ignored)
│   │   └── views/               # Home, Accounts, Providers, Generate, Prompts, Settings
│   ├── App.svelte               # Sidebar shell
│   └── main.ts
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs          # JSON CRUD commands (accounts, providers, prompts, settings)
│   │   ├── persona.rs           # Persona windows: data dirs, LRU eviction plan
│   │   ├── lib.rs               # Builder, window events
│   │   └── main.rs
│   ├── capabilities/            # Tauri permission capabilities
│   └── tauri.conf.json
├── messages/                    # id.json + en.json (i18n sources)
├── project.inlang/              # Paraglide project config
├── .github/
│   ├── workflows/               # ci.yml, semantic-release.yml, release.yml
│   └── dependabot.yml
├── scripts/                     # set-release-version.mjs (version sync across 3 manifests)
└── index.html
```

## 🚀 Quick Start

### Prerequisites

| Requirement | Notes                                                                          |
| ----------- | ------------------------------------------------------------------------------ |
| Node.js     | 24+ (LTS)                                                                      |
| Rust        | Stable via [rustup](https://rustup.rs)                                         |
| Linux deps  | `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev` |

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

### Build locally

```bash
npm install           # installs deps + compiles Paraglide messages
npm run tauri dev     # run the app in development mode
npm run tauri build   # produce .deb + .AppImage in src-tauri/target/release/bundle
```

### First run

1. Open **Accounts** and add your first persona (name + email).
2. Open **Providers** — seeded defaults are ready (Flow, Gemini, Suno, PixVerse,
   Leonardo, Krea, ImagineArt). Add your own if needed.
3. On **Home**, pick a provider + persona, press **Open**. Log in inside the window —
   the session persists in that persona's data dir.

## 💻 Development

### Commands

| Command                      | Description                                       |
| ---------------------------- | ------------------------------------------------- |
| `npm run dev`                | Vite dev server (frontend only)                   |
| `npm run tauri dev`          | Full app in development mode                      |
| `npm run build`              | Build frontend to `dist/`                         |
| `npm run check`              | Type-check with svelte-check                      |
| `npm test`                   | Run frontend tests (Vitest)                       |
| `npm run lint`               | Biome check (lint + format)                       |
| `npm run format`             | Biome fix (lint + format)                         |
| `npm run compile:i18n`       | Compile Paraglide messages to `src/lib/paraglide` |
| `npm run tauri build`        | Bundle `.deb` + `.AppImage`                       |
| `cd src-tauri && cargo test` | Rust unit tests (LRU eviction plan)               |

### Code Style & Conventions

- **Formatter + Linter:** Biome 2 (`biome check .`) — config in `biome.json`
- **Type checking:** svelte-check 4 with strict TypeScript
- **EditorConfig:** `.editorconfig` enforces `lf`, `utf-8`, 2 spaces
- **Commits:** [Conventional Commits](https://www.conventionalcommits.org) — enforced by semantic-release
  - `feat:` → new minor version, `fix:` → patch, `feat!:` / `BREAKING CHANGE:` → major
- **i18n:** zero hardcoded UI strings — all copy goes through Paraglide `m.*` messages
  (`messages/id.json` is the base locale); dates and numbers use the native `Intl` API

### Dependabot

Automated dependency updates via `.github/dependabot.yml`:

| Ecosystem        | Directory    | Schedule            |
| ---------------- | ------------ | ------------------- |
| `npm`            | `/`          | Weekly Monday 04:00 |
| `cargo`          | `/src-tauri` | Weekly Monday 04:00 |
| `github-actions` | `/`          | Weekly Monday 04:00 |

- Groups `minor`+`patch` updates per ecosystem; majors come as separate grouped PRs
- Ignores major bumps for `typescript` (svelte-check peer range)
- Commit prefixes: `chore(deps)` / `chore(ci)`

## ⚙️ Configuration

All app data lives in the XDG app-local data dir — nothing leaves the device:

| Path                            | Contents                                      |
| ------------------------------- | --------------------------------------------- |
| `<data>/profiles.json`          | Persona accounts                              |
| `<data>/providers.json`         | Provider registry (seeded on first run)       |
| `<data>/prompts.json`           | Prompt library                                |
| `<data>/settings.json`          | Language, theme, engine, close behavior, LRU  |
| `<data>/profiles/<persona-id>/` | Per-account WebKitGTK data (cookies, storage) |

`<data>` resolves to `$XDG_DATA_HOME/id.co.bits.flowai` (typically
`~/.local/share/id.co.bits.flowai`). Deleting a persona's folder resets that account
to a logged-out state.

In-app **Settings** cover: language (system / Indonesia / English), theme
(system / light / dark), per-provider engine (webview / chrome), close behavior,
and persona window cache size (LRU).

## 🔄 CI/CD Workflows

| Workflow                                    | Trigger                 | Output                                                         |
| ------------------------------------------- | ----------------------- | -------------------------------------------------------------- |
| `CI` (`ci.yml`)                             | `push` / `pull_request` | Biome, Vitest, svelte-check, vite build, cargo fmt/clippy/test |
| `Semantic Release` (`semantic-release.yml`) | `push` to `main`        | Version bump, git tag, CHANGELOG.md, GitHub release notes      |
| `Release` (`release.yml`)                   | tag `1.2.3` / `v1.2.3`  | `.deb` + `.AppImage` bundles attached to the release           |

Flow: conventional commits on `main` → semantic-release cuts a tag → dispatches the
Release workflow → `tauri-apps/tauri-action` builds and uploads bundles. No secrets
required beyond the default `GITHUB_TOKEN`.

Latest artifacts: [Releases](https://github.com/Banten-IT-Solutions/BITS-Flow-AI/releases).

## 📚 Documentation

- [CONTRIBUTING.md](CONTRIBUTING.md) — dev setup, commit conventions, release flow
- [SECURITY.md](SECURITY.md) — security model & reporting
- [CHANGELOG.md](CHANGELOG.md) — release history (generated by semantic-release)

## 🔐 Security

- Provider sessions live only in per-persona WebKitGTK data dirs on disk — never
  uploaded, never exported by the app
- All writes are atomic (tmp + rename); IPC inputs are validated (URL scheme whitelist,
  persona ID charset, JSON schema)
- No telemetry, no analytics, no network calls beyond the provider pages you open
- Destructive actions (delete persona, clear data) require explicit confirmation
- See [SECURITY.md](SECURITY.md) for reporting guidance

## 📄 License

[MIT](LICENSE) © Banten IT Solutions.

---

<div align="center">
  <strong>BITS Flow AI</strong> — Developed with ❤️ by <a href="https://bits.co.id"><strong>Banten IT Solutions</strong></a>
</div>
