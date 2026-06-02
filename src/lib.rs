//! photon-ui — Blazing fast minimal TUI
//!
//! This crate provides a lightweight, high-performance terminal UI framework
//! built on top of [`crossterm`]. It features a custom differential renderer,
//! ANSI-aware text wrapping, OSC 8 hyperlink support, and a component-based
//! architecture.
//!
//! # Quick start
//!
//! ```no_run
//! use photon_ui::{TUI, Component, Rendered, RenderError, TestTerminal};
//! use photon_ui::components::Text;
//!
//! let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
//! tui.mount(Box::new(Text::new("Hello, world!", 0, 0)));
//! tui.render_frame().unwrap();
//! ```

pub mod autocomplete;
pub mod components;
pub mod events;
pub mod layout;
pub mod fuzzy;
pub mod image;
pub mod keybindings;
pub mod kill_ring;
pub mod renderer;
pub mod terminal;
pub mod theme;
pub mod tui;
pub mod undo_stack;
pub mod utils;
pub mod word_navigation;

pub use crossterm::event::KeyEvent;
pub use events::{Event, Key, Modifiers, matches_key};
pub use keybindings::{KeybindingsManager, default_bindings};
pub use renderer::{InputResult, RenderError, RenderStrategy, Rendered, Renderer};
pub use terminal::{Terminal, TestTerminal};
pub use tui::{Anchor, Overlay, OverlayConstraints, OverlayPosition, TUI};

/// A UI element that can be rendered and respond to input.
///
/// All visible elements in a TUI application implement this trait. The framework
/// calls [`render`](Component::render) on every frame and [`handle_input`](Component::handle_input)
/// when the focused component should process an event.
///
/// Components that can receive focus should also implement [`Focusable`].
pub trait Component {
    /// Render this component into lines of text at the given width.
    ///
    /// The returned [`Rendered`] must satisfy the invariant that every line's
    /// visible width is ≤ `width`.
    fn render(&self, width: u16) -> Result<Rendered, RenderError>;

    /// Render this component into a specific rectangular area.
    ///
    /// The default implementation delegates to [`render`](Component::render)
    /// with the rect's width, ignoring height bounds. Components that want
    /// to be layout-aware (e.g. clip to height, scroll, center vertically)
    /// should override this.
    fn render_rect(&self, rect: crate::layout::Rect) -> Result<Rendered, RenderError> {
        self.render(rect.width)
    }

    /// Handle an input event (key press, resize, mouse, etc.).
    ///
    /// The default implementation ignores all events. Override this to add
    /// interactivity.
    fn handle_input(&mut self, _event: &events::Event) -> InputResult {
        InputResult::Ignored
    }

    /// Returns `true` if this component wants to receive `KeyEventKind::Release`
    /// events in addition to `Press` / `Repeat`.
    ///
    /// Most components should leave this as `false`.
    fn wants_key_release(&self) -> bool {
        false
    }

    /// Cast this component to a [`Focusable`] reference, if supported.
    fn as_focusable(&self) -> Option<&dyn Focusable> {
        None
    }

    /// Cast this component to a mutable [`Focusable`] reference, if supported.
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        None
    }
}

/// Extension of [`Component`] for elements that can receive keyboard focus.
///
/// Focus is managed by [`TUI`]; only the focused component receives input events.
pub trait Focusable: Component {
    /// Returns `true` when this component currently has focus.
    fn focused(&self) -> bool;

    /// Set or clear the focused state.
    fn set_focused(&mut self, focused: bool);
}
