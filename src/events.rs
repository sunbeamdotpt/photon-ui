use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};

/// Unified input event type used by the framework.
///
/// Wraps crossterm events into a simpler enum that components consume via
/// [`Component::handle_input`](crate::Component::handle_input).
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A keyboard event.
    Key(KeyEvent),
    /// A mouse event (click, scroll, drag).
    Mouse(crossterm::event::MouseEvent),
    /// Terminal resized to the given dimensions `(cols, rows)`.
    Resize(u16, u16),
    /// Bracketed paste payload.
    Paste(String),
    /// Terminal gained focus.
    FocusGained,
    /// Terminal lost focus.
    FocusLost,
}

/// A portable representation of a key press with modifier state.
///
/// Unlike raw crossterm [`KeyEvent`], this type
/// is `Hash` and `Eq`, making it suitable for use in maps such as
/// [`KeybindingsManager`](crate::keybindings::KeybindingsManager).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// A printable character with modifiers.
    Char(char, Modifiers),
    /// A special key code with modifiers.
    Code(KeyCode, Modifiers),
}

/// Modifier keys held during an event.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Shift was held.
    pub shift: bool,
    /// Control was held.
    pub ctrl: bool,
    /// Alt / Option was held.
    pub alt: bool,
}

impl Modifiers {
    /// No modifiers.
    pub fn none() -> Self { Self::default() }
    /// Control only.
    pub fn ctrl() -> Self { Self { ctrl: true, ..Default::default() } }
    /// Shift only.
    pub fn shift() -> Self { Self { shift: true, ..Default::default() } }
    /// Alt only.
    pub fn alt() -> Self { Self { alt: true, ..Default::default() } }
    /// Control + Shift.
    pub fn ctrl_shift() -> Self { Self { ctrl: true, shift: true, ..Default::default() } }
}

impl Key {
    /// Enter / Return with no modifiers.
    pub fn enter() -> Self { Key::Code(KeyCode::Enter, Modifiers::none()) }
    /// Left arrow with no modifiers.
    pub fn left() -> Self { Key::Code(KeyCode::Left, Modifiers::none()) }
    /// Right arrow with no modifiers.
    pub fn right() -> Self { Key::Code(KeyCode::Right, Modifiers::none()) }
    /// Up arrow with no modifiers.
    pub fn up() -> Self { Key::Code(KeyCode::Up, Modifiers::none()) }
    /// Down arrow with no modifiers.
    pub fn down() -> Self { Key::Code(KeyCode::Down, Modifiers::none()) }
    /// Backspace with no modifiers.
    pub fn backspace() -> Self { Key::Code(KeyCode::Backspace, Modifiers::none()) }
    /// Delete with no modifiers.
    pub fn delete() -> Self { Key::Code(KeyCode::Delete, Modifiers::none()) }
    /// Home with no modifiers.
    pub fn home() -> Self { Key::Code(KeyCode::Home, Modifiers::none()) }
    /// End with no modifiers.
    pub fn end() -> Self { Key::Code(KeyCode::End, Modifiers::none()) }
    /// Tab with no modifiers.
    pub fn tab() -> Self { Key::Code(KeyCode::Tab, Modifiers::none()) }
    /// Escape with no modifiers.
    pub fn esc() -> Self { Key::Code(KeyCode::Esc, Modifiers::none()) }
    /// Character with Control held.
    pub fn ctrl(c: char) -> Self { Key::Char(c, Modifiers::ctrl()) }
    /// Character with Control + Shift held.
    pub fn ctrl_shift(c: char) -> Self { Key::Char(c, Modifiers::ctrl_shift()) }
    /// Character with Alt held.
    pub fn alt(c: char) -> Self { Key::Char(c, Modifiers::alt()) }
    /// Character with no modifiers.
    pub fn char(c: char) -> Self { Key::Char(c, Modifiers::none()) }
}

/// Check whether an [`Event`] matches a specific [`Key`].
///
/// Returns `false` for non-key events and for `Release` key events
/// (only `Press` and `Repeat` are matched).
///
/// # Example
///
/// ```
/// use photon_ui::{Event, Key, matches_key};
/// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
///
/// let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
/// assert!(matches_key(&event, &Key::ctrl('c')));
/// ```
pub fn matches_key(event: &Event, key: &Key) -> bool {
    let Event::Key(key_event) = event else { return false };
    if key_event.kind == KeyEventKind::Release {
        return false;
    }
    let km = key_event.modifiers;
    let mods = Modifiers {
        shift: km.contains(KeyModifiers::SHIFT),
        ctrl: km.contains(KeyModifiers::CONTROL),
        alt: km.contains(KeyModifiers::ALT),
    };
    match key {
        Key::Char(expected, expected_mods) => {
            if let KeyCode::Char(c) = key_event.code {
                c == *expected && mods == *expected_mods
            } else {
                false
            }
        }
        Key::Code(expected, expected_mods) => {
            key_event.code == *expected && mods == *expected_mods
        }
    }
}
