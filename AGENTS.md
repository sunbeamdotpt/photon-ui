# photon-ui — Agent Reference

## Project overview

`photon-ui` is a blazing fast, minimal terminal UI framework for Rust, built on top of [`crossterm`](https://github.com/crossterm-rs/crossterm). It provides a component-based architecture with a custom differential renderer, ANSI-aware text wrapping, OSC 8 hyperlink support, terminal image rendering (Kitty and iTerm2 protocols), and a constraint-based layout engine.

- **Language:** Rust (Edition 2024)
- **Version:** 0.1.0
- **License:** MIT
- **Author:** Sienna Meridian Satterwhite <sienna@sunbeam.pt>

---

## Semantic Memory Search (Optional)

If a `sunbeam-memory` MCP server is available in your environment, use it for codebase search instead of `grep` or `rg`.

1. **Initialize the repository first.** Before searching, ensure this codebase is indexed:
   - Call `add_watch_target` with the absolute path to this repository.
   - Wait for indexing to complete, then search.
2. **Prefer semantic search.** Use `search_facts` with natural-language queries about behavior, design decisions, known issues, and prior changes.
3. **Store useful findings.** If you discover something future agents should remember (a gotcha, invariant, or decision), call `store_fact` with a concise note and a source URN when possible.

`sunbeam-memory` is **optional**. If the server is not available, skip these steps and use `grep` / `rg` / `Read` as usual. Do not fail, stall, or ask the user to install it.

---

## Technology stack

### Core dependencies
| Crate | Purpose |
|-------|---------|
| `crossterm` | Cross-platform terminal I/O (raw mode, events, cursor control) |
| `unicode-width` | Correct display-width calculation for Unicode characters |
| `unicode-segmentation` | Grapheme cluster boundary detection |
| `pulldown-cmark` | CommonMark markdown parsing |
| `base64` | Image encoding for terminal graphics protocols |
| `thiserror` | Derive macro for error types |
| `kasuari` | Cassowary constraint solver for the layout engine |

### Dev dependencies
| Crate | Purpose |
|-------|---------|
| `insta` | Snapshot testing |
| `proptest` | Property-based testing |
| `criterion` | Benchmark harness (with HTML reports) |

## Build and test commands

```bash
# Build the library
cargo build

# Run all tests (unit + integration)
cargo test

# Run the interactive demo
cargo run --example demo

# Run benchmarks
cargo bench

# Format code (uses rustfmt.toml configuration)
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Benchmarks
Four Criterion benchmarks live in `benches/`:
- `wrap_text` — ANSI-aware text wrapping performance
- `render_components` — Component render throughput
- `frame_loop` — Full frame loop performance
- `event_handling` — Event dispatch overhead

## Code organization

```
src/
├── lib.rs              # Crate root: Component & Focusable traits, public exports
├── components/         # All UI components (26 modules)
│   ├── mod.rs          # Re-exports every component
│   ├── text.rs
│   ├── input.rs        # Single-line input (Emacs + vim modes)
│   ├── editor.rs       # Multi-line editor (Emacs + vim modes)
│   ├── select_list.rs
│   ├── settings_list.rs
│   ├── loader.rs
│   ├── markdown.rs
│   ├── image_widget.rs
│   ├── div.rs          # Layout container
│   ├── table.rs
│   ├── tree_view.rs
│   └── ... (see components/mod.rs for full list)
├── layout/             # Constraint-based layout engine
│   ├── layout.rs       # Cassowary solver integration (kasuari)
│   ├── constraint.rs   # Length, Min, Max, Percentage, Ratio, Fill
│   ├── rect.rs         # Terminal rectangle primitives
│   └── ...
├── theme/              # Beam Design Language theme system
│   ├── palette.rs      # Light / Dark themes
│   ├── color.rs
│   ├── style.rs
│   └── ansi.rs
├── terminal.rs         # Terminal trait + TestTerminal + ProcessTerminal
├── renderer.rs         # Differential renderer (FirstRender / FullRedraw / Diff)
├── tui.rs              # TUI runtime: focus management, overlays, modals, layout
├── events.rs           # Event abstraction over crossterm
├── keybindings.rs      # KeybindingsManager with default action maps
├── utils.rs            # ANSI-aware text wrapping, truncation, width measurement
├── image.rs            # Kitty / iTerm2 image protocol encoding + dimension parsing
├── autocomplete.rs     # Fuzzy autocomplete engine
├── fuzzy.rs            # Fuzzy matching logic
├── kill_ring.rs        # Kill-ring (clipboard) for editors
├── undo_stack.rs       # Undo / redo stacks
└── word_navigation.rs  # Word-boundary navigation helpers

tests/                  # Integration and component-specific tests (42+ files)
benches/                # Criterion benchmarks
examples/               # Interactive demos
├── demo.rs             # Full 5-page component showcase
```

## Architecture at a glance

### Component trait
Every visible element implements `Component`:

```rust
pub trait Component {
    fn render(&self, width: u16) -> Result<Rendered, RenderError>;
    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError>;
    fn handle_input(&mut self, event: &Event) -> InputResult;
    fn wants_key_release(&self) -> bool;
    fn as_focusable(&self) -> Option<&dyn Focusable>;
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable>;
}
```

- `render` produces lines of text, an optional cursor position, and image commands.
- `handle_input` receives events when the component has focus (or during fallthrough).
- Focusable components also implement `Focusable` to participate in tab-order managed by `TUI`.

### Renderer
`Renderer` tracks the previous frame and automatically selects one of three strategies:
- **FirstRender** — outputs all lines (assumes clean alternate screen).
- **FullRedraw** — clears screen + scrollback, then redraws everything (used on resize).
- **Diff** — computes first/last changed line and only rewrites the changed region.

The diff strategy emits `\x1b[?2026h`/`\x1b[?2026l` (synchronized updates) and `\x1b[2K` (clear line) to minimize terminal output.

### TUI runtime
`TUI` owns the terminal backend, a stack of components, overlays, and an optional modal. It handles:
- Focus cycling via `Tab` / `Shift+Tab`
- Layout splitting when a `Layout` is set
- Overlay positioning with `Anchor` and `OverlayConstraints`
- Modal capture (Esc dismisses, focus restores on close)
- Stale image cleanup for Kitty graphics protocol

### Layout engine
The layout system uses the `kasuari` Cassowary constraint solver. A `Layout` splits a `Rect` into sub-rects based on `Constraint` values (`Length`, `Min`, `Max`, `Percentage`, `Ratio`, `Fill`) with configurable `Direction`, `Flex`, `Margin`, and `Spacing`.

## Code style guidelines

The project enforces an **extremely strict** linting policy in `src/lib.rs`:

```rust
#![deny(dead_code)]
#![deny(unused)]
#![deny(unused_mut)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(clippy::question_mark_used)]
#![deny(clippy::needless_borrow)]
#![allow(bindings_with_variant_name)]
```

### Key rules for contributors
- **No `unwrap` or `expect` outside tests.** Every fallible operation must be propagated or handled explicitly.
- **No `?` operator.** The project explicitly denies `clippy::question_mark_used`; use explicit `match` or `if let` instead.
- **No dead code or unused imports.** The compiler will reject them.
- **No undocumented `unsafe` blocks.** (The codebase currently contains no `unsafe`.)
- **Match arm leading pipes are required.** `match_arm_leading_pipes = "Always"` in `rustfmt.toml`.
- **Import grouping:** `group_imports = "StdExternalCrate"` in `rustfmt.toml`.

### rustfmt configuration
The repository includes a `rustfmt.toml` with specific preferences (e.g., `brace_style = "PreferSameLine"`, `imports_layout = "Vertical"`, `wrap_comments = true`). Run `cargo fmt` before committing.

## Testing instructions

### Where tests live
- **Unit tests:** Embedded in source files under `#[cfg(test)]` modules.
- **Integration / component tests:** In the `tests/` directory, one file per module (e.g., `input_tests.rs`, `renderer_tests.rs`).
- **Property tests:** `tests/layout_engine_proptests.rs` and `tests/layout_type_integration_tests.rs` use `proptest`.
- **Snapshot tests:** Some test files use `insta` for snapshot assertions.

### Testing with `TestTerminal`
`TestTerminal` is the primary test double. It records all writes, cursor moves, and cursor visibility changes so tests can assert on exact ANSI sequences:

```rust
use photon_ui::terminal::TestTerminal;

let mut term = TestTerminal::new(80, 24);
term.write("hello").unwrap();
assert!(term.written().contains(&"hello".to_string()));
assert_eq!(term.cursor_moves(), &[(5, 10)]);
```

`ProcessTerminal::new_test()` (available only under `#[cfg(test)]`) provides another in-memory terminal that reports a fixed 80×24 size.

### Running specific tests
```bash
cargo test input_tests       # Run tests matching "input_tests"
cargo test --test lib_tests  # Run a specific integration test file
cargo test -- --nocapture    # Show println! / eprintln! output
```

### Regression tests
The test suite contains many regression tests named `blit_into_rect_preserves_ansi_reset`, `diff_resets_ansi_before_clear`, etc. When fixing ANSI-related bugs, add a regression test that asserts on the exact escape sequences emitted.

## Development conventions

### Component implementation checklist
When adding a new component:
1. Create `src/components/your_component.rs`.
2. Add `pub mod your_component;` and a `pub use` in `src/components/mod.rs`.
3. Implement `Component`. If the component can receive focus, also implement `Focusable`.
4. Add a corresponding test file in `tests/` (e.g., `tests/your_component_tests.rs`).
5. If appropriate, add the component to `examples/demo.rs`.

### ANSI handling
The framework takes ANSI escape sequences very seriously:
- `utils::visible_width` skips CSI (`\x1b[…`) and OSC (`\x1b]…`) sequences when measuring width.
- `utils::truncate_to_width` preserves ANSI prefixes/infixes and appends `\x1b[0m` when truncation occurs inside styled text.
- `Rendered::blit_into_rect` preserves `\x1b[0m` at boundaries to prevent background color bleeding.
- The renderer always emits `\x1b[0m` before `\x1b[2K` or `\x1b[2J`.

### Error handling
Because `unwrap`, `expect`, and `?` are all denied in non-test code, the typical pattern is:
```rust
match some_fallible_operation() {
    Ok(value) => value,
    Err(e) => return Err(e.into()),
}
```
or propagate `io::Result` / `Result<T, RenderError>` manually.

### Image support
Images are supported via:
- **Kitty graphics protocol** — `image::encode_kitty`
- **iTerm2 inline images** — `image::encode_iterm2`

`ImageWidget` accepts raw image bytes and a MIME type. The TUI runtime automatically deletes stale images by tracking image IDs across frames.

### Vim vs Emacs editing modes
Both `Input` and `Editor` support dual editing modes. Vim mode is toggled at runtime via `set_vim_mode_enabled(true)`. When implementing new editor features, consider how they map in both modes.

## Security considerations

- **No `unsafe` code.** The crate denies `clippy::undocumented_unsafe_blocks` and currently contains zero `unsafe` blocks.
- **Terminal escape injection.** All user-facing text rendering goes through `utils::visible_width` and `utils::wrap_text_with_ansi`, which are designed to handle arbitrary ANSI sequences safely. However, when adding new components that accept external strings, ensure ANSI sequences are either escaped or measured correctly to prevent width-overflow panics.
- **Image data.** `ImageWidget` accepts raw image bytes; it does not validate image content beyond parsing dimensions from headers. Do not assume image data is well-formed.

## Useful environment variables

| Variable | Effect |
|----------|--------|
| `PHOTON_UI_HARDWARE_CURSOR` | If set, `TUI` shows the hardware cursor instead of hiding it. |
