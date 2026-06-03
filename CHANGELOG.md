# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-06-03

### Added

- Added `homepage`, `repository`, `keywords`, and `categories` metadata to `Cargo.toml` for crates.io discoverability.
- Added comprehensive documentation for all public APIs (~200 new doc comments across 22 source files).
  - Module-level docs for every `pub mod` in `lib.rs`, `components/mod.rs`, and `layout/mod.rs`.
  - Docs for all public struct fields, enum variants, methods, traits, and constructors.
- README is now embedded as the crate root documentation via `#![doc = include_str!("../README.md")]`.

### Changed

- Expanded README with full component inventory (27 components), a layout engine section with a code example, and accurate architecture descriptions.
- README code examples updated to compile cleanly as doctests.

### Fixed

- Fixed broken intra-doc links in `src/tui.rs`, `src/components/container.rs`, and `src/components/div.rs`.
- Fixed compiler warnings: unused imports and variables in test files, non-snake-case test function names.
- Replaced `unwrap()` in `src/image.rs` with explicit `match` to comply with the project's `deny(clippy::unwrap_used)` lint.
- Replaced `expect()` in `src/layout/layout.rs` with a safe fallback to `Rect::ZERO` to comply with the project's `deny(clippy::expect_used)` lint.

## [0.1.0] - 2025-01-15

### Added

- Initial release of photon-ui.
- Component-based TUI architecture with `Component` and `Focusable` traits.
- Differential renderer with `FirstRender`, `FullRedraw`, and `Diff` strategies.
- Cassowary constraint-based layout engine (`kasuari`).
- 27 built-in components: `Box`, `Breadcrumbs`, `Button`, `CancellableLoader`, `Container`, `Div`, `Divider`, `Editor`, `Header`, `ImageWidget`, `Input`, `Loader`, `Markdown`, `Modal`, `Panel`, `ProgressBar`, `SelectList`, `SettingsList`, `Sidebar`, `Spacer`, `StatusBar`, `Table`, `Tabs`, `Text`, `TreeView`, `TruncatedText`.
- Dual editing modes (Emacs / vim) for `Input` and `Editor`.
- Terminal image support via Kitty and iTerm2 protocols.
- ANSI-aware text wrapping, truncation, and width measurement.
- OSC 8 hyperlink support.
- Focus management, overlays, modals, and tab cycling.
- Interactive demo (`cargo run --example demo`).
