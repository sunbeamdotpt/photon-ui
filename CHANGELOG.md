# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.3] - 2026-06-26

### Fixed

- `AnsiCodeTracker` now preserves full 24-bit truecolor (`38;2;R;G;B` /
  `48;2;R;G;B`) and 256-color (`38;5;N` / `48;5;N`) SGR parameters. Previously
  it only stored the leading `38` / `48`, which made the compositor and text
  wrapper drop the actual color values and caused buttons and other styled
  components to render with the wrong colors after v0.2.0.

## [0.4.2] - 2026-06-26

### Fixed

- Removed an SGR reset (`\x1b[0m`) emitted before Kitty image placement that
  was corrupting text color rendering in the rest of the frame.

## [0.4.1] - 2026-06-26

### Fixed

- Kitty images were re-transmitted on every frame, causing the terminal to
  re-composite them and making the whole UI look washed out. The renderer now
  transmits each image id once and only re-emits the placement command on
  subsequent frames.

## [0.4.0] - 2026-06-26

### Added

- **Custom theme support** — implement the `Palette` trait and install a palette globally with `Theme::set_palette(Arc::new(...))`. Components resolve colors through the active palette instead of the built-in Light/Dark colors.
- `PaletteHandle` type alias exported from `photon_ui::theme`.
- Unit tests for `theme::palette`, `keybindings`, `SettingsList`, and `Spacer`.
- Integration tests for custom palette activation and `Theme::with` isolation.

### Changed

- Redesigned the `Palette` trait around real theme-design roles: `background`, `surface`, `field`, `text`, `text_muted`, `text_on_accent`, `accent`, `accent_hover`, `border`, `border_muted`, `focus`, `success`, `warning`, `error`, `info`.
- All components now resolve colors via `Theme::palette()`.
- `examples/demo.rs` displays `"Custom"` when a custom palette is active and clears it when toggling with `t`.

### Fixed

- Raised unit-test line coverage above 90%.

## [0.3.0] - 2026-06-25

### Added

- **Layer compositor** — new `Compositor`, `Layer`, and `Shadow` abstractions for stacking terminal surfaces with dim and drop-shadow effects.
- **TUI layer rendering** — `TUI` now composites frames through a stack of layers instead of a single surface.
- **Layer showcase demo** — new page 6 in `examples/demo.rs` demonstrates overlapping cards, dim backgrounds, and drop shadows.
- **Integration tests** for the compositor and TUI layer stack.
- **Image placement coordinates** — `ImageCommand` now carries explicit `row`/`col` screen coordinates, and the renderer positions the cursor before emitting each Kitty `a=p` command.
- **Kitty cell dimensions** — `encode_kitty` accepts `cols`/`rows` and emits `c=`/`r=` so the terminal scales the image to fit the reserved cell rectangle.
- **Size-aware `ImageWidget`** — parses image pixel dimensions to compute a default cell size, exposes `.with_size(cols, rows)`, renders placeholder lines that reserve matching screen space, and implements `render_rect` to clip inside Cassowary-layout rectangles.
- Demo image assets: `examples/reference.jpg` and `examples/reference.png` (copyright-free bee-with-cowboy-hat).

### Changed

- `examples/demo.rs` page 1 now uses a Cassowary-driven `Layout::vertical` that reserves explicit space for Markdown, the image, text samples, and the `blit_into_rect` demo.
- Updated `AGENTS.md` to reflect current project conventions.

### Fixed

- Drop shadows now render as a proper offset fringe.
- Layer demo card positioning and internal separator rendering.
- Resolved clippy warnings and `?`-operator usage to comply with project lints.

## [0.2.0] - 2026-06-05

### Added

- **Table sort & filter** — `Table` now supports live sorting and filtering with full keyboard interactivity.
  - `sort_by(column)` sorts by a `sortable` column; toggles ascending/descending on repeat calls.
  - `clear_sort()` removes sorting and restores original row order.
  - `set_filter(query)` / `clear_filter()` apply case-insensitive substring filtering across all columns.
  - Sort indicators (▲/▼) are rendered next to the active column header; customizable via `sort_indicator_asc()` and `sort_indicator_desc()`.
  - Interactive filter mode: press `'/'` (configurable via `filter_key()`) to type a filter live. `Enter` confirms, `Esc` exits, `Backspace` deletes.
- **Table hooks** — attach callbacks for user-driven state changes:
  - `on_select(|idx| { ... })` — fires when keyboard navigation changes the selected row.
  - `on_sort(|col, asc| { ... })` — fires when sort column or direction changes.
  - `on_filter(|query| { ... })` — fires when the filter query changes.
  - `on_filter_char(|c| -> Option<char> { ... })` — transform or reject individual characters during interactive filter input.
- **Table getters** — `selected_row()`, `selected_original_index()`, `displayed_row_count()`, `sort_column_index()`, `sort_ascending()`, `in_filter_mode()`, `filter_query()`.
- **Display order tracking** — `Table` now maintains an internal `display_indices` Vec so that selection, navigation, rendering, and callbacks all correctly respect the filtered + sorted view.
- New example: `examples/table_sort_filter_demo.rs` showcases sort, filter, hooks, and interactive filter mode.

### Changed

- `Table::selected()` now returns the **visible** index after sort/filter.
- `Table::set_selected()` clamps against the visible row count.
- `Table::handle_input()` navigation (Up/Down/j/k) now operates on the filtered/sorted display set.

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
