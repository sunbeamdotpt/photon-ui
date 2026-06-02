use crate::{Component, Rendered, RenderError, Event, InputResult, Focusable};
use crossterm::event::KeyCode;

/// A scrollable list of selectable items with keyboard navigation.
///
/// Renders items with a `> ` prefix on the selected row. Supports vertical
/// scrolling when the item count exceeds `max_visible`.
pub struct SelectList {
    items: Vec<String>,
    selected: usize,
    max_visible: usize,
    scroll: usize,
    focused: bool,
}

impl SelectList {
    /// Create a new list with the given items and maximum visible rows.
    pub fn new(items: Vec<String>, max_visible: usize) -> Self {
        Self { items, selected: 0, max_visible, scroll: 0, focused: false }
    }

    /// Index of the currently selected item.
    pub fn selected(&self) -> usize { self.selected }

    /// Set the selected item index (clamped to valid range).
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.items.len().saturating_sub(1));
        self.scroll = self.selected.saturating_sub(self.max_visible.saturating_sub(1));
    }

    /// The currently selected item text, if any.
    pub fn selected_item(&self) -> Option<&str> {
        self.items.get(self.selected).map(|s| s.as_str())
    }
}

impl Focusable for SelectList {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Component for SelectList {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let visible_end = (self.scroll + self.max_visible).min(self.items.len());
        for i in self.scroll..visible_end {
            let prefix = if i == self.selected { "> " } else { "  " };
            let line = format!("{}{}", prefix, self.items[i]);
            lines.push(crate::utils::truncate_to_width(&line, width, "…"));
        }
        Ok(Rendered { lines, cursor: None, images: Vec::new() })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crossterm::event::KeyModifiers;
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down => {
                    if self.selected + 1 < self.items.len() {
                        self.selected += 1;
                        if self.selected >= self.scroll + self.max_visible {
                            self.scroll += 1;
                        }
                    }
                    InputResult::Handled
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                        if self.selected < self.scroll {
                            self.scroll = self.scroll.saturating_sub(1);
                        }
                    }
                    InputResult::Handled
                }
                KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.selected + 1 < self.items.len() {
                        self.selected += 1;
                        if self.selected >= self.scroll + self.max_visible {
                            self.scroll += 1;
                        }
                    }
                    InputResult::Handled
                }
                KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.selected > 0 {
                        self.selected -= 1;
                        if self.selected < self.scroll {
                            self.scroll = self.scroll.saturating_sub(1);
                        }
                    }
                    InputResult::Handled
                }
                KeyCode::Enter => InputResult::RequestRender,
                _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn select_list_renders() {
        let list = SelectList::new(vec!["a".into(), "b".into()], 10);
        let r = list.render(10).unwrap();
        assert_eq!(r.lines.len(), 2);
        assert!(r.lines[0].contains("> "));
    }

    #[test]
    fn select_list_navigation() {
        let mut list = SelectList::new(vec!["a".into(), "b".into(), "c".into()], 2);
        list.set_focused(true);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.selected(), 1);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.selected(), 2);
        assert_eq!(list.scroll, 1);
    }

    #[test]
    fn select_list_scroll_up() {
        let mut list = SelectList::new(vec!["a".into(), "b".into(), "c".into()], 2);
        list.set_focused(true);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.scroll, 1);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::empty())));
        // selected=1, scroll=1, item is still visible so scroll stays
        assert_eq!(list.scroll, 1);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.scroll, 0);
    }

    #[test]
    fn select_list_j_k_navigation() {
        let mut list = SelectList::new(vec!["a".into(), "b".into(), "c".into()], 2);
        list.set_focused(true);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.selected(), 1);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn select_list_j_scrolls() {
        let mut list = SelectList::new(vec!["a".into(), "b".into(), "c".into()], 2);
        list.set_focused(true);
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty())));
        list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty())));
        assert_eq!(list.selected(), 2);
        assert_eq!(list.scroll, 1);
    }
}
