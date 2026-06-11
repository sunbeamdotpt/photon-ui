#![doc = include_str!("../README.md")]
#![deny(dead_code)]
#![deny(unused)]
#![deny(unused_mut)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
// for @siennathesane's sanity and to make it clear the scope of error handling. and because it's
// super fucking subtle and i'll miss it in code reviews sorry not sorry
#![deny(clippy::question_mark_used)]
// just keeps syntax consistent
#![deny(clippy::needless_borrow)]
// personal preference.
#![allow(bindings_with_variant_name)]

/// Fuzzy autocomplete engine.
pub mod autocomplete;
/// UI components (text, input, editor, table, etc.).
pub mod components;
/// Layer compositor.
pub mod compositor;
/// Event abstraction over crossterm.
pub mod events;
/// Fuzzy matching logic.
pub mod fuzzy;
/// Terminal image protocol encoding.
pub mod image;
/// Keybindings manager and default action maps.
pub mod keybindings;
/// Clipboard / kill-ring for editors.
pub mod kill_ring;
/// Full-terminal-size stacking surfaces.
pub mod layer;
/// Constraint-based layout engine.
pub mod layout;
/// Differential terminal renderer.
pub mod renderer;
/// Terminal trait and test double.
pub mod terminal;
/// Beam Design Language theme system.
pub mod theme;
/// TUI runtime and focus management.
pub mod tui;
/// Undo / redo stacks.
pub mod undo_stack;
/// ANSI-aware text wrapping and width measurement.
pub mod utils;
/// Word-boundary navigation helpers.
pub mod word_navigation;

pub use crossterm::event::KeyEvent;
pub use events::{
    Event,
    Key,
    Modifiers,
    matches_key,
};
pub use keybindings::{
    KeybindingsManager,
    default_bindings,
};
pub use layer::{
    Layer,
    Shadow,
};
pub use renderer::{
    InputResult,
    RenderError,
    RenderStrategy,
    Rendered,
    Renderer,
};
pub use terminal::{
    Terminal,
    TestTerminal,
};
pub use tui::{
    Anchor,
    Overlay,
    OverlayConstraints,
    OverlayPosition,
    TUI,
};

/// A UI element that can be rendered and respond to input.
///
/// All visible elements in a TUI application implement this trait. The
/// framework calls [`render`](Component::render) on every frame and
/// [`handle_input`](Component::handle_input) when the focused component should
/// process an event.
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

    /// Returns `true` if this component wants to receive
    /// `KeyEventKind::Release` events in addition to `Press` / `Repeat`.
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
/// Focus is managed by [`TUI`]; only the focused component receives input
/// events.
pub trait Focusable: Component {
    /// Returns `true` when this component currently has focus.
    fn focused(&self) -> bool;

    /// Set or clear the focused state.
    fn set_focused(&mut self, focused: bool);
}
