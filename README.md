# photon-ui

A blazing fast, minimal terminal UI framework for Rust.

Photon UI is a lightweight, high-performance TUI library built on [`crossterm`](https://github.com/crossterm-rs/crossterm). It features a custom differential renderer that only redraws what changed, ANSI-aware text wrapping, OSC 8 hyperlink support, terminal image rendering (Kitty and iTerm2 protocols), a Cassowary constraint-based layout engine, and a component-based architecture.

## Features

- **Differential renderer** — Only updates cells that changed between frames, minimizing terminal output and improving performance.
- **Component-based architecture** — Build UIs by composing simple, reusable [`Component`] trait implementations.
- **Built-in components** — 27 components including text, input, editor, buttons, tables, trees, sidebars, tabs, modals, panels, progress bars, markdown, and image widgets.
- **Constraint-based layout** — Split terminal space using the Cassowary solver via [`Layout`], [`Constraint`], and [`Rect`].
- **Dual editing modes** — Both Emacs and vim keybindings for [`Input`] and [`Editor`] components, switchable at runtime.
- **Terminal image support** — Render images inline using the Kitty graphics protocol or iTerm2 inline images.
- **ANSI-aware text handling** — Properly measures and wraps text containing ANSI escape sequences, Unicode, and grapheme clusters.
- **Overlays and modals** — Position floating content with [`Anchor`]-based constraints and modal dialogs that capture input.
- **Focus management** — The [`TUI`] runtime handles focus cycling and routes input events to the focused component.
- **Zero unnecessary dependencies** — Core functionality relies only on `crossterm`, `unicode-width`, `unicode-segmentation`, `pulldown-cmark`, `base64`, `thiserror`, and `kasuari`.

## Quick start

Add `photon-ui` to your `Cargo.toml`:

```toml
[dependencies]
photon-ui = "0.1"
```

Build a simple app:

```rust,no_run
use photon_ui::{Component, TUI, TestTerminal};
use photon_ui::components::Text;

let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
tui.mount(Box::new(Text::new("Hello, world!", 0, 0)));
tui.render_frame().unwrap();
```

For a real application, use [`ProcessTerminal`] to drive the actual terminal:

```rust,no_run
use photon_ui::{TUI, Event};
use photon_ui::terminal::{ProcessTerminal, Terminal};
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
| [`Text`] | Static text with optional horizontal and vertical padding. |
| [`TruncatedText`] | Text that truncates with an ellipsis when wider than the container. |
| [`Box`] | A container with configurable vertical padding and optional background styling. |
| [`Spacer`] | Empty vertical space. |
| [`Input`] | Single-line text input with Emacs and vim modes, kill-ring, and undo. |
| [`Editor`] | Multi-line text editor with Emacs and vim modes, kill-ring, and undo. |
| [`SelectList`] | Scrollable list with keyboard navigation and selection. |
| [`SettingsList`] | Toggle list for boolean settings. |
| [`Loader`] | Animated spinner with optional colored message. |
| [`CancellableLoader`] | Spinner that can be cancelled via `Ctrl+C`. |
| [`Markdown`] | Renders CommonMark markdown (headings, bold, italic, inline code, lists). |
| [`ImageWidget`] | Displays images inline via Kitty or iTerm2 graphics protocols. |
| [`Button`] | Clickable button with primary, ghost, cream, text, and dark variants. |
| [`Breadcrumbs`] | Hierarchical breadcrumb navigation trail. |
| [`Container`] | Generic container for composing child components. |
| [`Div`] | Layout container that splits space among children with borders and padding. |
| [`Divider`] | Horizontal or vertical divider line, optionally with a label. |
| [`Header`] | Page header with title and right-aligned actions. |
| [`Modal`] | Modal dialog overlay with title, border, and focus capture. |
| [`Panel`] | Panel container with optional rounded, thin, thick, or double borders. |
| [`ProgressBar`] | Progress bar with optional label and percentage display. |
| [`Sidebar`] | Sidebar navigation with icons and keyboard selection. |
| [`StatusBar`] | Status bar with left, center, and right segments. |
| [`Table`] | Table with sortable columns, row selection, and keyboard navigation. |
| [`Tabs`] | Tab bar with keyboard navigation between tabs. |
| [`TreeView`] | Collapsible tree view with expand/collapse and keyboard navigation. |

## Architecture

### Component trait

Every visible element implements the [`Component`] trait:

```text
pub trait Component {
    fn render(&self, width: u16) -> Result<Rendered, RenderError>;
    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError>;
    fn handle_input(&mut self, event: &Event) -> InputResult;
    fn wants_key_release(&self) -> bool;
    fn as_focusable(&self) -> Option<&dyn Focusable>;
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable>;
}
```

- [`render`](Component::render) produces the lines of text (and optional cursor or image commands) for the current frame.
- [`render_rect`](Component::render_rect) renders into a specific rectangular area, used by layout-aware components.
- [`handle_input`](Component::handle_input) receives events when the component has focus.
- [`Focusable`] components participate in the tab order managed by [`TUI`].

### Renderer

The [`Renderer`] tracks the previous frame and computes a minimal set of cursor movements and writes needed to update the terminal. This avoids clearing the screen or redrawing unchanged content. It automatically selects one of three strategies:

- **FirstRender** — outputs all lines (assumes a clean alternate screen).
- **FullRedraw** — clears the screen and redraws everything (used on resize).
- **Diff** — computes the first and last changed line and only rewrites that region.

### Layout engine

The layout system uses the `kasuari` Cassowary constraint solver. A [`Layout`] splits a [`Rect`] into sub-rects based on [`Constraint`] values ([`Length`], [`Min`], [`Max`], [`Percentage`], [`Ratio`], [`Fill`]) with configurable [`Direction`], [`Flex`], [`Margin`], and [`Spacing`].

```rust,no_run
use photon_ui::layout::{Constraint, Direction, Layout, Rect};

let layout = Layout::vertical([
    Constraint::Length(3),   // header
    Constraint::Min(10),     // main content
    Constraint::Length(1),   // status bar
]);
let areas = layout.split(Rect::new(0, 0, 80, 24));
```

### TUI runtime

[`TUI`] owns the terminal backend, manages a stack of components, handles focus cycling (`Tab` / `Shift+Tab`), and supports overlays positioned with [`Anchor`] and [`OverlayConstraints`]. Modal dialogs capture all input until dismissed with `Esc`.

## Input modes

Both [`Input`] and [`Editor`] support Emacs and vim editing styles:

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

The included demo showcases every component across multiple pages:

```bash
cargo run --example demo
```

Controls:

| Key | Action |
|-----|--------|
| `1`–`5` | Switch demo page |
| `Tab` / `Shift+Tab` | Cycle focus |
| `q` or `Ctrl+C` | Quit |

[`Component`]: https://docs.rs/photon-ui/latest/photon_ui/trait.Component.html
[`Focusable`]: https://docs.rs/photon-ui/latest/photon_ui/trait.Focusable.html
[`TUI`]: https://docs.rs/photon-ui/latest/photon_ui/struct.TUI.html
[`Renderer`]: https://docs.rs/photon-ui/latest/photon_ui/struct.Renderer.html
[`Layout`]: https://docs.rs/photon-ui/latest/photon_ui/layout/layout/struct.Layout.html
[`Rect`]: https://docs.rs/photon-ui/latest/photon_ui/layout/rect/struct.Rect.html
[`Constraint`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html
[`Direction`]: https://docs.rs/photon-ui/latest/photon_ui/layout/direction/enum.Direction.html
[`Flex`]: https://docs.rs/photon-ui/latest/photon_ui/layout/flex/enum.Flex.html
[`Margin`]: https://docs.rs/photon-ui/latest/photon_ui/layout/margin/struct.Margin.html
[`Spacing`]: https://docs.rs/photon-ui/latest/photon_ui/layout/spacing/enum.Spacing.html
[`Anchor`]: https://docs.rs/photon-ui/latest/photon_ui/enum.Anchor.html
[`OverlayConstraints`]: https://docs.rs/photon-ui/latest/photon_ui/struct.OverlayConstraints.html
[`ProcessTerminal`]: https://docs.rs/photon-ui/latest/photon_ui/terminal/struct.ProcessTerminal.html
[`Text`]: https://docs.rs/photon-ui/latest/photon_ui/components/text/struct.Text.html
[`TruncatedText`]: https://docs.rs/photon-ui/latest/photon_ui/components/truncated_text/struct.TruncatedText.html
[`Box`]: https://docs.rs/photon-ui/latest/photon_ui/components/box_component/struct.Box.html
[`Spacer`]: https://docs.rs/photon-ui/latest/photon_ui/components/spacer/struct.Spacer.html
[`Input`]: https://docs.rs/photon-ui/latest/photon_ui/components/input/struct.Input.html
[`Editor`]: https://docs.rs/photon-ui/latest/photon_ui/components/editor/struct.Editor.html
[`SelectList`]: https://docs.rs/photon-ui/latest/photon_ui/components/select_list/struct.SelectList.html
[`SettingsList`]: https://docs.rs/photon-ui/latest/photon_ui/components/settings_list/struct.SettingsList.html
[`Loader`]: https://docs.rs/photon-ui/latest/photon_ui/components/loader/struct.Loader.html
[`CancellableLoader`]: https://docs.rs/photon-ui/latest/photon_ui/components/cancellable_loader/struct.CancellableLoader.html
[`Markdown`]: https://docs.rs/photon-ui/latest/photon_ui/components/markdown/struct.Markdown.html
[`ImageWidget`]: https://docs.rs/photon-ui/latest/photon_ui/components/image_widget/struct.ImageWidget.html
[`Button`]: https://docs.rs/photon-ui/latest/photon_ui/components/button/struct.Button.html
[`Breadcrumbs`]: https://docs.rs/photon-ui/latest/photon_ui/components/breadcrumbs/struct.Breadcrumbs.html
[`Container`]: https://docs.rs/photon-ui/latest/photon_ui/components/container/struct.Container.html
[`Div`]: https://docs.rs/photon-ui/latest/photon_ui/components/div/struct.Div.html
[`Divider`]: https://docs.rs/photon-ui/latest/photon_ui/components/divider/struct.Divider.html
[`Header`]: https://docs.rs/photon-ui/latest/photon_ui/components/header/struct.Header.html
[`Modal`]: https://docs.rs/photon-ui/latest/photon_ui/components/modal/struct.Modal.html
[`Panel`]: https://docs.rs/photon-ui/latest/photon_ui/components/panel/struct.Panel.html
[`ProgressBar`]: https://docs.rs/photon-ui/latest/photon_ui/components/progress_bar/struct.ProgressBar.html
[`Sidebar`]: https://docs.rs/photon-ui/latest/photon_ui/components/sidebar/struct.Sidebar.html
[`StatusBar`]: https://docs.rs/photon-ui/latest/photon_ui/components/status_bar/struct.StatusBar.html
[`Table`]: https://docs.rs/photon-ui/latest/photon_ui/components/table/struct.Table.html
[`Tabs`]: https://docs.rs/photon-ui/latest/photon_ui/components/tabs/struct.Tabs.html
[`TreeView`]: https://docs.rs/photon-ui/latest/photon_ui/components/tree_view/struct.TreeView.html
[`Length`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html#variant.Length
[`Min`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html#variant.Min
[`Max`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html#variant.Max
[`Percentage`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html#variant.Percentage
[`Ratio`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html#variant.Ratio
[`Fill`]: https://docs.rs/photon-ui/latest/photon_ui/layout/constraint/enum.Constraint.html#variant.Fill
