use crossterm::event::KeyCode;

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    theme::{
        Palette,
        Style,
        Theme,
        stylize,
    },
};

/// A horizontal tab bar with keyboard navigation.
///
/// Renders as a horizontal tab bar. The active tab is highlighted with the
/// theme's accent color and bold; inactive tabs use the secondary text color.
/// When focused, a `│` prefix is shown.
pub struct Tabs {
    items: Vec<String>,
    active: usize,
    focused: bool,
}

impl Tabs {
    /// Create a new tab bar with the given items.
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            active: 0,
            focused: false,
        }
    }

    /// Index of the currently active tab.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Set the active tab index (clamped to valid range).
    pub fn set_active(&mut self, index: usize) {
        self.active = index.min(self.items.len().saturating_sub(1));
    }
}

impl Focusable for Tabs {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for Tabs {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let accent_style = Style::new().fg(theme.accent()).bold();
        let inactive_style = Style::new().fg(theme.text_secondary());

        let mut line = String::new();
        if self.focused {
            line.push('│');
            line.push(' ');
        }

        for (i, item) in self.items.iter().enumerate() {
            if i == self.active {
                let text = format!(" [{}] ", item);
                line.push_str(&stylize(&text, &accent_style));
            } else {
                let text = format!("  {}  ", item);
                line.push_str(&stylize(&text, &inactive_style));
            }
        }

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crossterm::event::KeyModifiers;
        if let Event::Key(key) = event {
            match key.code {
                | KeyCode::Right => {
                    if self.active + 1 < self.items.len() {
                        self.active += 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Left => {
                    if self.active > 0 {
                        self.active -= 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Char('l') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.active + 1 < self.items.len() {
                        self.active += 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Char('h') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.active > 0 {
                        self.active -= 1;
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
    use crossterm::event::KeyCode;

    use super::*;

    #[test]
    fn tabs_new() {
        let tabs = Tabs::new(vec!["a", "b", "c"]);
        assert_eq!(tabs.active(), 0);
        assert_eq!(tabs.items.len(), 3);
    }

    #[test]
    fn tabs_set_active() {
        let mut tabs = Tabs::new(vec!["a", "b", "c"]);
        tabs.set_active(1);
        assert_eq!(tabs.active(), 1);
        tabs.set_active(10);
        assert_eq!(tabs.active(), 2);
    }

    #[test]
    fn tabs_focusable() {
        let mut tabs = Tabs::new(vec!["a", "b"]);
        assert!(!tabs.focused());
        tabs.set_focused(true);
        assert!(tabs.focused());
    }

    #[test]
    fn tabs_render_unfocused() {
        Theme::with(Theme::Light, || {
            let tabs = Tabs::new(vec!["Tab 1", "Tab 2"]);
            let rendered = tabs.render(80).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert!(!rendered.lines[0].starts_with('│'));
        });
    }

    #[test]
    fn tabs_render_focused() {
        Theme::with(Theme::Light, || {
            let mut tabs = Tabs::new(vec!["Tab 1", "Tab 2"]);
            tabs.set_focused(true);
            let rendered = tabs.render(80).unwrap();
            assert!(rendered.lines[0].starts_with('│'));
        });
    }

    #[test]
    fn tabs_handle_input_right() {
        let mut tabs = Tabs::new(vec!["a", "b", "c"]);
        tabs.set_focused(true);
        let result = tabs.handle_input(&Event::Key(KeyCode::Right.into()));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(tabs.active(), 1);
    }

    #[test]
    fn tabs_handle_input_left() {
        let mut tabs = Tabs::new(vec!["a", "b", "c"]);
        tabs.set_focused(true);
        tabs.set_active(2);
        let result = tabs.handle_input(&Event::Key(KeyCode::Left.into()));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(tabs.active(), 1);
    }

    #[test]
    fn tabs_handle_input_h_l() {
        let mut tabs = Tabs::new(vec!["a", "b", "c"]);
        tabs.set_focused(true);
        tabs.set_active(1);
        let result = tabs.handle_input(&Event::Key(KeyCode::Char('h').into()));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(tabs.active(), 0);

        let result = tabs.handle_input(&Event::Key(KeyCode::Char('l').into()));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(tabs.active(), 1);
    }

    #[test]
    fn tabs_handle_input_clamps() {
        let mut tabs = Tabs::new(vec!["a", "b"]);
        tabs.set_focused(true);
        tabs.set_active(1);
        let result = tabs.handle_input(&Event::Key(KeyCode::Right.into()));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(tabs.active(), 1); // clamped

        tabs.set_active(0);
        let result = tabs.handle_input(&Event::Key(KeyCode::Left.into()));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(tabs.active(), 0); // clamped
    }
}
