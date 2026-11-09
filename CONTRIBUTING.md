# Contributing to BITS Flow AI

## Prerequisites

- Node.js 20.19+ (or 22.12+) and npm
- Rust stable with `rustfmt` and `clippy` (via [rustup](https://rustup.rs))
- Tauri 2 Linux dependencies:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

## Setup

```bash
npm ci
npm run compile:i18n
npm run lint
npm test
npm run check
npm run tauri dev
```

Do not commit `node_modules/`, `dist/`, `src-tauri/target/`, `src/lib/paraglide/` (generated),
application data files, provider profiles, or cookies.

## Required checks

Before opening a pull request:

```bash
npm run lint        # Biome (format + lint)
npm test            # Vitest
npm run check       # svelte-check
npm run build       # vite build

cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Code standards

- Rust: `cargo fmt`, no clippy warnings, all IPC commands return `Result<_, String>`, no `unwrap`
  in command paths (tests may `unwrap`).
- Validate every input at the IPC boundary: IDs, names, emails, categories, engines, URL schemes
  (http/https only), and numeric ranges.
- TypeScript: strict mode; parse Tauri command results with the Zod schemas in `src/lib/api.ts`.
- UI strings come from Paraglide (`messages/id.json`, `messages/en.json`) — zero hardcoded UI
  strings. Indonesian is the base locale; English translations must stay complete.
- Data model changes must keep `src/lib/types.ts` and `src-tauri/src/commands.rs` in sync, including
  serde `camelCase` field names.
- Keep the light/dark theme tokens in `src/app.css` consistent; icons come from `@lucide/svelte`
  only.

## Tests

- Add Vitest tests for pure frontend logic (locale resolution, provider filtering, and similar).
- Add Rust unit tests for pure Rust logic (LRU eviction planning lives in `src-tauri/src/persona.rs`).
- Manual smoke checklist for UI changes: switch language, switch theme, open two personas, exceed the
  LRU cap, and confirm the oldest hidden persona window closes.

## Commits and pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `perf:`,
  `docs:`, `refactor:`, `test:`, `ci:`, and `feat!:` / `BREAKING CHANGE:` for incompatible changes.
  Releases are automated from these messages by semantic-release; the version commit and tag are
  produced by CI, not by hand.
- Keep commits focused and explain user-visible behavior.
- Link related issues and include screenshots for UI changes when possible.
- Do not include credentials, real account data, or provider cookies in commits, logs, issues, or
  screenshots.

## Security reports

Follow [SECURITY.md](SECURITY.md). Do not open a public issue for a vulnerability.

## License

By contributing, you agree that your contribution is licensed under the repository's MIT License.
