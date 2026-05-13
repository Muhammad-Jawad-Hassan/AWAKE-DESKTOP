# Contributing to Awake

Thanks for considering a contribution. This project aims to stay small, reliable, and easy to reason about, so please keep that in mind when proposing changes.

## Getting set up

See [README.md § Development setup](README.md#development-setup). In short: Rust (stable), Node 20+, pnpm 9+, and `pnpm tauri dev`.

## Project philosophy

One rule decides most design arguments here:

> The session is the central concept of the application. All other functionality should operate as an independent capability attached to that session.

Concretely, that means:

- The core (`src-tauri/src/core/`) never imports anything platform-specific, and never needs to know _why_ a session was started.
- Platform-specific code lives only under `src-tauri/src/platform/<os>/`, behind the trait interfaces defined in `core::ports`.
- Prefer simplicity over configurability. If a setting doesn't have a clear use case, leave it out.
- New activity types, platform capabilities, or profile fields should degrade gracefully (report "unsupported", don't crash) on platforms that can't provide them.

## Code style

- **Rust**: run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` before committing. Comments are rare and short: one line, only when the code genuinely can't explain itself.
- **TypeScript/React**: run `pnpm format`, `pnpm lint`, and `pnpm typecheck`. Components stay small and typed; avoid `any`.
- Match the JSON field naming already in use: Rust structs exposed to the frontend use `#[serde(rename_all = "camelCase")]`.

## Tests

- Core domain logic (session, timer, inactivity, activity scheduling, profiles, config) has full unit test coverage in `src-tauri/src/core/*.rs`, and new logic there should too.
- Run `cargo test` and `pnpm test` before opening a PR.
- Platform-specific code (`src-tauri/src/platform/<os>/`) is hard to unit test in CI; if you change it, describe how you manually verified it on the affected OS in your PR description.

## Submitting a change

1. Fork the repo and create a branch off `main`.
2. Make your change, with tests where it's reasonable to add them.
3. Run the full local check: `pnpm typecheck && pnpm lint && pnpm test && (cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test)`.
4. Open a PR describing what changed and why. Link any related issue.
5. CI runs the same checks across macOS, Windows, and Linux, so expect it to catch platform-specific issues you couldn't test locally.

## Reporting bugs / requesting features

Open a GitHub issue. For bugs, include your OS/version, the steps to reproduce, and what you expected instead. For security issues, see [SECURITY.md](SECURITY.md) instead, and please don't open a public issue.

## Code of Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before participating.
