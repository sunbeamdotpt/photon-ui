use crossterm::event::{
    KeyCode,
    KeyModifiers,
};

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    theme::{
        Style,
        Theme,
        stylize,
    },
};

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
        Self {
            items,
            selected: 0,
            focused: false,
            on_change: None,
        }
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
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for SettingsList {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::palette();
        let accent_style = Style::new().fg(theme.accent()).bold();
        let primary_style = Style::new().fg(theme.text());
        let dim_style = Style::new().fg(theme.text_muted());

        let mut lines = Vec::new();
        for (i, (name, value)) in self.items.iter().enumerate() {
            let is_selected = i == self.selected;
            let style = if is_selected && self.focused {
                &accent_style
            } else if is_selected {
                &primary_style
            } else {
                &dim_style
            };

            let prefix = if is_selected { "> " } else { "  " };
            let check = if *value { "[x]" } else { "[ ]" };
            let line = stylize(&format!("{}{} {}", prefix, check, name), style);
            lines.push(crate::utils::truncate_to_width(&line, width, "…"));
        }
        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            match key.code {
                | KeyCode::Down => {
                    if self.selected + 1 < self.items.len() {
                        self.selected += 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.selected + 1 < self.items.len() {
                        self.selected += 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some((_, value)) = self.items.get_mut(self.selected) {
                        *value = !*value;
                        if let Some(cb) = self.on_change {
                            cb(self.selected, *value);
                        }
                    }
                    InputResult::Handled
                },
                | _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> {
        Some(self)
    }

    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_items() {
        let list = SettingsList::new(vec![("A".into(), true), ("B".into(), false)]);
        let rendered = list.render(80).unwrap();
        assert_eq!(rendered.lines.len(), 2);
        assert!(rendered.lines[0].contains("[x]"));
        assert!(rendered.lines[1].contains("[ ]"));
    }

    #[test]
    fn navigation_down_and_up() {
        let mut list = SettingsList::new(vec![("A".into(), false), ("B".into(), false)]);
        list.set_focused(true);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        let rendered = list.render(80).unwrap();
        assert!(rendered.lines[1].contains("> "));

        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Up,
            crossterm::event::KeyModifiers::empty(),
        )));
        let rendered = list.render(80).unwrap();
        assert!(rendered.lines[0].contains("> "));
    }

    #[test]
    fn toggle_with_enter() {
        let mut list = SettingsList::new(vec![("A".into(), false)]);
        list.set_focused(true);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        )));
        let rendered = list.render(80).unwrap();
        assert!(rendered.lines[0].contains("[x]"));
        assert_eq!(list.values(), vec![true]);
    }

    #[test]
    fn ignored_input() {
        let mut list = SettingsList::new(vec![("A".into(), false)]);
        let result = list.handle_input(&Event::Resize(80, 24));
        assert!(matches!(result, InputResult::Ignored));
    }
}
