# photon-ui

A blazing fast, minimal terminal UI framework for Rust.

Photon UI is a lightweight, high-performance TUI library built on [`crossterm`](https://github.com/crossterm-rs/crossterm). It features a custom differential renderer that only redraws what changed, ANSI-aware text wrapping, OSC 8 hyperlink support, terminal image rendering (Kitty and iTerm2 protocols), and a component-based architecture.

## Features

- **Differential renderer** — Only updates cells that changed between frames, minimizing terminal output and improving performance.
- **Component-based architecture** — Build UIs by composing simple, reusable `Component` trait implementations.
- **Built-in components** — Text, truncated text, boxes, spacers, input fields, multi-line editors, select lists, settings toggles, loaders, markdown, and image widgets.
- **Dual editing modes** — Both Emacs and vim keybindings for `Input` and `Editor` components, switchable at runtime.
- **Terminal image support** — Render images inline using the Kitty graphics protocol or iTerm2 inline images.
- **ANSI-aware text handling** — Properly measures and wraps text containing ANSI escape sequences, Unicode, and grapheme clusters.
- **Overlays** — Position floating content with anchor-based constraints (center, top-left, bottom-right, etc.).
- **Focus management** — The `TUI` runtime handles focus cycling and routes input events to the focused component.
- **Zero unnecessary dependencies** — Core functionality relies only on `crossterm`, `unicode-width`, `unicode-segmentation`, `pulldown-cmark`, `base64`, and `thiserror`.

## Quick start

Add `photon-ui` to your `Cargo.toml`:

```toml
[dependencies]
photon-ui = "0.1"
```

Build a simple app:

```rust
use photon_ui::{Component, TUI, Rendered, RenderError, TestTerminal};
use photon_ui::components::Text;

let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
tui.mount(Box::new(Text::new("Hello, world!", 0, 0)));
tui.render_frame().unwrap();
```

For a real application, use `ProcessTerminal` to drive the actual terminal:

```rust
use photon_ui::{TUI, Event};
use photon_ui::terminal::ProcessTerminal;
use photon_ui::components::{Text, Input};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut term = ProcessTerminal::new();
    term.start()?;

    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(Text::new("Name:", 0, 0)));
    tui.mount(Box::new(Input::new()));

    loop {
        tui.render_frame()?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            let event = match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => Event::Key(key),
                crossterm::event::Event::Resize(w, h) => Event::Resize(w, h),
                other => continue,
            };
            tui.handle_input(&event);
        }
    }
}
```

## Components

| Component | Description |
|-----------|-------------|
| `Text` | Static text with optional horizontal and vertical padding. |
| `TruncatedText` | Text that truncates with an ellipsis when wider than the container. |
| `Box` | A container with configurable vertical padding and optional background styling. |
| `Spacer` | Empty vertical space. |
| `Input` | Single-line text input with Emacs and vim modes, kill-ring, and undo. |
| `Editor` | Multi-line text editor with Emacs and vim modes, kill-ring, and undo. |
| `SelectList` | Scrollable list with keyboard navigation and selection. |
| `SettingsList` | Toggle list for boolean settings. |
| `Loader` | Animated spinner with optional colored message. |
| `CancellableLoader` | Spinner that can be cancelled via `Ctrl+C`. |
| `Markdown` | Renders CommonMark markdown (headings, bold, italic, inline code). |
| `ImageWidget` | Displays images inline via Kitty or iTerm2 graphics protocols. |

## Architecture

### Component trait

Every visible element implements the `Component` trait:

```rust
pub trait Component {
    fn render(&self, width: u16) -> Result<Rendered, RenderError>;
    fn handle_input(&mut self, event: &Event) -> InputResult;
    fn as_focusable(&self) -> Option<&dyn Focusable>;
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable>;
}
```

- `render` produces the lines of text (and optional cursor or image commands) for the current frame.
- `handle_input` receives events when the component has focus.
- `Focusable` components participate in the tab order managed by `TUI`.

### Renderer

The `Renderer` tracks the previous frame and computes a minimal set of cursor movements and writes needed to update the terminal. This avoids clearing the screen or redrawing unchanged content.

### TUI runtime

`TUI` owns the terminal backend, manages a stack of components, handles focus cycling (`Tab` / `Shift+Tab`), and supports overlays positioned with `Anchor` and `OverlayConstraints`.

## Input modes

Both `Input` and `Editor` support Emacs and vim editing styles:

| Emacs | Vim | Action |
|-------|-----|--------|
| `Ctrl+A` | `0` or `^` | Move to start of line |
| `Ctrl+E` | `$` | Move to end of line |
| `Ctrl+K` | `D` | Kill to end of line |
| `Ctrl+Y` | `p` | Yank (paste) |
| `Ctrl+W` | `daw` | Kill word |
| `Ctrl+F` / `Ctrl+B` | `l` / `h` | Move forward / backward |
| — | `i` / `Esc` | Enter / exit insert mode |

Enable vim mode at construction time or toggle it at runtime with `set_vim_mode_enabled`.

## Running the demo

The included demo showcases every component across four pages:

```bash
cargo run --example demo
```

Controls:

| Key | Action |
|-----|--------|
| `1`–`4` | Switch demo page |
| `Tab` / `Shift+Tab` | Cycle focus |
| `q` or `Ctrl+C` | Quit |
