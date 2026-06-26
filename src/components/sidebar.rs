use crossterm::event::KeyCode;

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    theme::{
        ColorMode,
        Style,
        Theme,
        stylize,
    },
};

/// A single item in a sidebar.
pub struct SidebarItem {
    label: String,
    icon: Option<String>,
}

impl SidebarItem {
    /// Create a new sidebar item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    /// Set an optional icon for this item.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A vertical navigation sidebar with selectable items.
///
/// Renders items vertically with a `> ` prefix on the selected row when
/// focused. Supports an optional left border and keyboard navigation.
pub struct Sidebar {
    items: Vec<SidebarItem>,
    selected: usize,
    focused: bool,
    show_border: bool,
}

impl Sidebar {
    /// Create a new sidebar with the given items.
    pub fn new(items: Vec<SidebarItem>) -> Self {
        Self {
            items,
            selected: 0,
            focused: false,
            show_border: true,
        }
    }

    /// Index of the currently selected item.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected item index (clamped to valid range).
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.items.len().saturating_sub(1));
    }

    /// Hide the left border.
    pub fn hide_border(mut self) -> Self {
        self.show_border = false;
        self
    }
}

impl Focusable for Sidebar {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for Sidebar {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::palette();
        let mut lines = Vec::new();

        let content_width = if self.show_border {
            width.saturating_sub(1)
        } else {
            width
        };

        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == self.selected && self.focused;
            let style = if is_selected {
                Style::new().fg(theme.accent()).bold()
            } else {
                Style::new().fg(theme.text_muted())
            };

            let prefix = if is_selected { "> " } else { "  " };
            let icon = item
                .icon
                .as_ref()
                .map(|s| format!("{} ", s))
                .unwrap_or_default();
            let line = format!("{}{}{}", prefix, icon, item.label);
            let line = crate::utils::truncate_to_width(&line, content_width, "…");
            lines.push(stylize(&line, &style));
        }

        if self.show_border && width > 0 {
            let border_style = Style::new().fg(theme.border());
            let mode = ColorMode::detect();
            let border_prefix = border_style.prefix(mode);
            let border_suffix = Style::suffix();
            let border = format!("{}{}{}", border_prefix, '▐', border_suffix);

            for line in &mut lines {
                let mut new_line = border.clone();
                new_line.push_str(line);
                *line = new_line;
            }
        }

        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crossterm::event::KeyModifiers;
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
                // Allow Tab / BackTab to propagate so TUI can cycle focus.
                | KeyCode::Tab | KeyCode::BackTab => InputResult::Ignored,
                // When focused, consume all other keys to prevent fallthrough
                // to sibling components (e.g. Tabs reacting to Left/Right).
                | _ => {
                    if self.focused {
                        InputResult::Handled
                    } else {
                        InputResult::Ignored
                    }
                },
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
    fn sidebar_new() {
        let sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        assert_eq!(sidebar.selected(), 0);
        assert!(!sidebar.focused());
    }

    #[test]
    fn sidebar_set_selected() {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        sidebar.set_selected(1);
        assert_eq!(sidebar.selected(), 1);
        sidebar.set_selected(10);
        assert_eq!(sidebar.selected(), 1);
    }

    #[test]
    fn sidebar_focusable() {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A")]);
        assert!(!sidebar.focused());
        sidebar.set_focused(true);
        assert!(sidebar.focused());
    }

    #[test]
    fn sidebar_hide_border() {
        let sidebar = Sidebar::new(vec![SidebarItem::new("A")]).hide_border();
        assert!(!sidebar.show_border);
    }

    #[test]
    fn sidebar_renders_items() {
        Theme::with(Theme::Light, || {
            let sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
            let rendered = sidebar.render(10).unwrap();
            assert_eq!(rendered.lines.len(), 2);
        });
    }

    #[test]
    fn sidebar_selected_shows_prefix() {
        Theme::with(Theme::Light, || {
            let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
            sidebar.set_focused(true);
            let rendered = sidebar.render(10).unwrap();
            assert!(rendered.lines[0].contains("> "));
            assert!(rendered.lines[1].contains("  "));
        });
    }

    #[test]
    fn sidebar_icon_renders() {
        Theme::with(Theme::Light, || {
            let sidebar = Sidebar::new(vec![SidebarItem::new("Home").icon("🏠")]);
            let rendered = sidebar.render(20).unwrap();
            assert!(rendered.lines[0].contains("🏠"));
            assert!(rendered.lines[0].contains("Home"));
        });
    }

    #[test]
    fn sidebar_keyboard_navigation() {
        let mut sidebar = Sidebar::new(vec![
            SidebarItem::new("A"),
            SidebarItem::new("B"),
            SidebarItem::new("C"),
        ]);
        sidebar.set_focused(true);

        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 1);

        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 2);

        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 2); // clamped
    }

    #[test]
    fn sidebar_j_k_navigation() {
        let mut sidebar = Sidebar::new(vec![
            SidebarItem::new("A"),
            SidebarItem::new("B"),
            SidebarItem::new("C"),
        ]);
        sidebar.set_focused(true);

        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 1);

        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('k'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 0);
    }

    #[test]
    fn sidebar_clamps_up() {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        sidebar.set_focused(true);

        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Up,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 0);
    }

    #[test]
    fn sidebar_border_present_by_default() {
        Theme::with(Theme::Light, || {
            let sidebar = Sidebar::new(vec![SidebarItem::new("A")]);
            let rendered = sidebar.render(10).unwrap();
            assert!(rendered.lines[0].contains('▐'));
        });
    }

    #[test]
    fn sidebar_hide_border_no_border() {
        Theme::with(Theme::Light, || {
            let sidebar = Sidebar::new(vec![SidebarItem::new("A")]).hide_border();
            let rendered = sidebar.render(10).unwrap();
            assert!(!rendered.lines[0].contains('▐'));
        });
    }

    /// Regression: when focused, unhandled keys must be consumed (Handled) so
    /// they don't fall through to sibling components.
    #[test]
    fn sidebar_focused_consumes_unhandled_keys() {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        sidebar.set_focused(true);

        let left = Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Left,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(sidebar.handle_input(&left), InputResult::Handled);

        let right = Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Right,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(sidebar.handle_input(&right), InputResult::Handled);
    }

    /// Tab must propagate (Ignored) so TUI can cycle focus.
    #[test]
    fn sidebar_tab_propagates_for_focus_cycle() {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A")]);
        sidebar.set_focused(true);

        let tab = Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(sidebar.handle_input(&tab), InputResult::Ignored);
    }
}
