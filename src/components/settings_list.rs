use crate::{Component, Rendered, RenderError, Event, InputResult, Focusable};
use crossterm::event::KeyCode;

/// A scrollable list of toggleable settings rendered as checkboxes.
///
/// Each item is a `(name, value)` pair where `value` is `true` for checked
/// `[x]` and `false` for unchecked `[ ]`. Items can be toggled with Enter
/// or Space, and an optional `on_change` callback is fired on every toggle.
pub struct SettingsList {
    items: Vec<(String, bool)>,
    selected: usize,
    focused: bool,
    on_change: Option<fn(usize, bool)>,
}

impl SettingsList {
    /// Create a new settings list from `(name, value)` pairs.
    pub fn new(items: Vec<(String, bool)>) -> Self {
        Self { items: items, selected: 0, focused: false, on_change: None }
    }

    /// Attach a callback invoked when an item is toggled.
    ///
    /// The callback receives `(index, new_value)`.
    pub fn on_change(mut self, cb: fn(usize, bool)) -> Self {
        self.on_change = Some(cb);
        self
    }

    /// Return the current values of all items.
    pub fn values(&self) -> Vec<bool> {
        self.items.iter().map(|(_, v)| *v).collect()
    }

    /// Set the selected item index (clamped to valid range).
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.items.len().saturating_sub(1));
    }
}

impl Focusable for SettingsList {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Component for SettingsList {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        for (i, (name, value)) in self.items.iter().enumerate() {
            let prefix = if i == self.selected { "> " } else { "  " };
            let check = if *value { "[x]" } else { "[ ]" };
            let line = format!("{}{} {}", prefix, check, name);
            lines.push(crate::utils::truncate_to_width(&line, width, "…"));
        }
        Ok(Rendered { lines, cursor: None, images: Vec::new() })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down => {
                    if self.selected + 1 < self.items.len() { self.selected += 1; }
                    InputResult::Handled
                }
                KeyCode::Up => {
                    if self.selected > 0 { self.selected -= 1; }
                    InputResult::Handled
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some((_, value)) = self.items.get_mut(self.selected) {
                        *value = !*value;
                        if let Some(cb) = self.on_change {
                            cb(self.selected, *value);
                        }
                    }
                    InputResult::Handled
                }
                _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}
