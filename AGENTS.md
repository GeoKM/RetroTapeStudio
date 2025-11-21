# Repository Guidelines

## Project Structure & Module Organization
RetroTapeStudio is a single-crate Rust application built with `eframe`/`egui`. The entry point lives in `src/main.rs`, while reusable UI state or utilities should be moved into `src/lib.rs` and additional modules under `src/<feature>/`. `Cargo.toml` pins GUI, IO, and error-handling dependencies; update versions in lockstep. `build.rs` scans every file in `src/` and fails the build on non-ASCII input, so keep localized copy in separate resource files. Build output remains in `target/`; store sample media in an `assets/` folder outside source.

## Build, Test, and Development Commands
- `cargo run` – fast debug build for day-to-day changes.
- `cargo run --release` – optimized binary for profiling or demos.
- `cargo test` – executes the full Rust test harness.
- `cargo fmt && cargo clippy -- -D warnings` – enforces formatting and lint cleanliness; both must pass before reviews.
- `cargo check` – lightweight type-checking for quick iteration.

## Coding Style & Naming Conventions
Use the Rust 2021 edition defaults: `rustfmt` formatting (4-space indent, trailing commas, module-level imports) and `clippy` with warnings denied. Follow standard Rust casing (`snake_case` functions, `CamelCase` types, `SCREAMING_SNAKE_CASE` constants) and keep modules narrowly focused (e.g., `src/ui/timeline.rs` for timeline widgets, `src/core/session.rs` for data). Document non-obvious egui layouts or custom paint logic with concise `///` doc comments.

## Testing Guidelines
Favor colocated unit tests for stateful logic and create integration suites under `tests/` when UI workflows span multiple modules. Name tests after observable behavior (`test_recording_tracks_are_sorted`) and mock filesystem dependencies so they run quickly. Target serialization, tape buffer transforms, and undo/redo chains, and run `cargo test -- --include-ignored` before submitting diagnostics.

## Commit & Pull Request Guidelines
Model commits after the existing history: short, imperative subject lines that describe what changes, not how (`transport: add jog shuttle control`). Use the body for rationale or references. Pull requests must describe behavior changes, list validation commands (`cargo run --release`, `cargo test`), and include screenshots or clips for new egui panels. Link related issues, request reviews from module owners, and ensure fmt, clippy, and test checks are green before seeking approval.

## Security & Configuration Tips
ASCII-only enforcement means emoji or localized strings must live in assets loaded at runtime. Keep API tokens and platform-specific paths outside the repo, e.g., by reading from `.env`. When using `rfd` dialogs, sanitize user-selected paths and avoid auto-opening files. Before shipping binaries, scrub recordings or logs from the package.
