//! A flexible container with optional borders, padding, title, and background.
//!
//! `Div` is a general-purpose layout box. It accepts a `Layout` and renders
//! its children into the layout's areas, then draws optional chrome around the
//! result. Think of it as the TUI equivalent of an HTML `<div>`.
//!
//! # Example
//!
//! ```
//! use photon_ui::{
//!     components::Div,
//!     layout::{
//!         Constraint,
//!         Layout,
//!     },
//! };
//!
//! let div = Div::new(Layout::vertical([
//!     Constraint::Length(1),
//!     Constraint::Min(3),
//!     Constraint::Length(1),
//! ]))
//! .child(Box::new(photon_ui::components::Text::new("Header", 0, 0)))
//! .child(Box::new(photon_ui::components::Text::new("Body", 0, 0)))
//! .child(Box::new(photon_ui::components::Text::new("Footer", 0, 0)))
//! .border(photon_ui::layout::Border::ROUNDED)
//! .padding(photon_ui::layout::Margin::new(1, 1))
//! .title("My Box");
//! ```

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    layout::{
        Border,
        Layout,
        Margin,
        Rect,
    },
    theme::{
        Style,
        Theme,
    },
};

/// A general-purpose container with optional chrome.
pub struct Div {
    layout: Layout,
    children: Vec<Box<dyn Component>>,
    border: Option<Border>,
    border_style: Style,
    padding: Margin,
    title: Option<String>,
    title_style: Style,
    background: Option<Style>,
    focused: bool,
    /// Which child receives keyboard input when this div is focused.
    focused_child: Option<usize>,
    /// Whether this div can be collapsed/expanded via keyboard.
    collapsible: bool,
    /// Whether this div is currently collapsed.
    collapsed: bool,
}

impl Div {
    /// Create a new `Div` with the given layout.
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            children: Vec::new(),
            border: None,
            border_style: Style::new(),
            padding: Margin::new(0, 0),
            title: None,
            title_style: Style::new(),
            background: None,
            focused: false,
            focused_child: None,
            collapsible: false,
            collapsed: false,
        }
    }

    /// Add a child component (builder style).
    pub fn child(mut self, child: Box<dyn Component>) -> Self {
        self.children.push(child);
        self
    }

    /// Add a child component (imperative style).
    pub fn push(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    /// Set the outer border.
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Style the outer border.
    pub fn border_styled(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set inner padding.
    pub fn padding(mut self, margin: Margin) -> Self {
        self.padding = margin;
        self
    }

    /// Set a title rendered in the top border.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Style the title.
    pub fn title_styled(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Fill the entire div area with a background style.
    pub fn background(mut self, style: Style) -> Self {
        self.background = Some(style);
        self
    }

    /// Make this div collapsible via Enter/Space when focused.
    pub fn collapsible(mut self, value: bool) -> Self {
        self.collapsible = value;
        self
    }

    /// Set the collapsed state (only meaningful when collapsible).
    pub fn collapsed(mut self, value: bool) -> Self {
        self.collapsed = value;
        self
    }

    /// Toggle the collapsed state.
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Compute the inner content rect after subtracting border and padding.
    fn inner_rect(&self, rect: Rect) -> Rect {
        let mut inner = rect;
        if self.border.is_some() {
            inner = inner.inner(Margin::new(1, 1));
        }
        inner = inner.inner(self.padding);
        inner
    }

    /// Cycle focus to the next/previous focusable child.
    ///
    /// Returns `Handled` if focus moved within this div, or `Ignored` if the
    /// cycle would move past the last/first child so the parent can handle it.
    fn cycle_child_focus(&mut self, delta: isize) -> InputResult {
        let focusable: Vec<usize> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.as_focusable().is_some())
            .map(|(i, _)| i)
            .collect();

        if focusable.is_empty() {
            return InputResult::Ignored;
        }

        let current = match self
            .focused_child
            .and_then(|idx| focusable.iter().position(|&i| i == idx))
        {
            | Some(pos) => pos,
            | None => {
                self.focused_child = Some(focusable[0]);
                if let Some(f) = self.children[focusable[0]].as_focusable_mut() {
                    f.set_focused(true);
                }
                return InputResult::Handled;
            },
        };

        // Try to cycle within the current child first (recursive descent).
        let current_idx = focusable[current];
        let tab_event = Event::Key(crossterm::event::KeyEvent::new(
            if delta > 0 {
                crossterm::event::KeyCode::Tab
            } else {
                crossterm::event::KeyCode::BackTab
            },
            crossterm::event::KeyModifiers::empty(),
        ));
        let child_result = self.children[current_idx].handle_input(&tab_event);
        if child_result != InputResult::Ignored {
            return InputResult::Handled;
        }

        // Current child couldn't cycle further, move to next/prev sibling.
        if delta > 0 && current + 1 >= focusable.len() {
            // Tab past last child — let parent handle it.
            return InputResult::Ignored;
        }
        if delta < 0 && current == 0 {
            // BackTab past first child — let parent handle it.
            return InputResult::Ignored;
        }

        let new_pos = if delta >= 0 {
            (current + delta as usize) % focusable.len()
        } else {
            let d = (-delta) as usize % focusable.len();
            (current + focusable.len() - d) % focusable.len()
        };
        let new_idx = focusable[new_pos];

        // Unfocus old child
        if let Some(f) = self.children[current_idx].as_focusable_mut() {
            f.set_focused(false);
        }
        // Focus new child
        self.focused_child = Some(new_idx);
        if let Some(f) = self.children[new_idx].as_focusable_mut() {
            f.set_focused(true);
        }
        InputResult::Handled
    }
}

impl Focusable for Div {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if focused && self.focused_child.is_none() {
            // Auto-focus the first focusable child when this div gains focus.
            self.focused_child = self
                .children
                .iter()
                .position(|c| c.as_focusable().is_some());
        }
        // Propagate focus state ONLY to the focused child.
        // Setting all children as focused breaks nested focus cycling
        // (multiple leaf components would think they're focused).
        if let Some(idx) = self.focused_child &&
            let Some(f) = self.children[idx].as_focusable_mut()
        {
            f.set_focused(focused);
        }
    }
}

impl Component for Div {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let height = self.children.len() as u16 * 3;
        let rect = Rect::new(0, 0, width, height);
        self.render_rect(rect)
    }

    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError> {
        let theme = Theme::palette();
        let mut screen = Rendered::empty();

        // ── Collapsed state: render only a single-line header ──
        if self.collapsed {
            let indicator = if self.collapsible { "▶ " } else { "" };
            let title_text = self
                .title
                .as_ref()
                .map(|t| format!("{}{}", indicator, t))
                .unwrap_or_else(|| "▶".into());
            let header_style = if self.focused {
                Style::new().fg(theme.accent()).bold()
            } else {
                Style::new().fg(theme.text_muted())
            };
            let mut header = crate::theme::stylize(&title_text, &header_style);
            header = crate::utils::truncate_to_width(&header, rect.width, "…");
            let pad = rect.width as usize - crate::utils::visible_width(&header);
            if pad > 0 {
                header.push_str(&" ".repeat(pad));
            }
            screen.lines.push(header);
            // Pad to requested height so parent layout isn't disrupted
            while screen.lines.len() < rect.height as usize {
                screen.lines.push(String::new());
            }
            return Ok(screen);
        }

        // Fill background if requested
        if let Some(ref bg) = self.background {
            let prefix = bg.prefix(crate::theme::ColorMode::detect());
            let suffix = bg.suffix();
            for _ in 0..rect.height {
                let line = format!(
                    "{}{:width$}{}",
                    prefix,
                    "",
                    suffix,
                    width = rect.width as usize
                );
                screen.lines.push(line);
            }
        }

        // Compute inner rect for children
        let inner = self.inner_rect(rect);

        // Render children into the inner rect using the layout
        let areas = self.layout.split(inner);
        for (child, area) in self.children.iter().zip(areas.iter()) {
            if let Ok(rendered) = child.render_rect(*area) {
                // Blit into the local buffer using coordinates relative to this div's origin.
                // `layout.split()` returns areas in terminal coordinates (they include
                // rect.x/y), but `screen` is a fresh local buffer whose origin is (0, 0).
                let rel_area = Rect::new(
                    area.x.saturating_sub(rect.x),
                    area.y.saturating_sub(rect.y),
                    area.width,
                    area.height,
                );
                rendered.blit_into_rect(&mut screen, rel_area);
            }
        }

        // Ensure screen has enough lines for the full rect
        while screen.lines.len() < rect.height as usize {
            screen.lines.push(String::new());
        }

        // Draw border if requested
        if let Some(ref border) = self.border {
            let border_style = if self.border_style == Style::new() {
                Style::new().fg(theme.border())
            } else {
                self.border_style
            };
            // Draw border at the edges of the local buffer, not at absolute coords.
            crate::layout::draw_border(
                &mut screen,
                Rect::new(0, 0, rect.width, rect.height),
                border,
                &border_style,
            );

            // Draw title in the top border if set
            if let Some(ref title) = self.title &&
                !screen.lines.is_empty()
            {
                let title_style = if self.title_style == Style::new() {
                    Style::new().fg(theme.text()).bold()
                } else {
                    self.title_style
                };
                let indicator = if self.collapsible { "▼ " } else { "" };
                let label = format!(" {}{} ", indicator, title);
                let label_styled = crate::theme::stylize(&label, &title_style);
                let top = &mut screen.lines[0];
                let start_byte = crate::utils::byte_index_at_visual_pos(top, 2);
                let end_byte = crate::utils::byte_index_at_visual_pos(
                    top,
                    2 + crate::utils::visible_width(&label_styled),
                );
                if start_byte < top.len() {
                    top.replace_range(start_byte..end_byte.min(top.len()), &label_styled);
                }
            }
        }

        Ok(screen)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crossterm::event::KeyCode;

        // Toggle collapsed state on Enter or Space when collapsible.
        if self.collapsible &&
            let Event::Key(key) = event &&
            (key.code == KeyCode::Enter || key.code == KeyCode::Char(' '))
        {
            self.collapsed = !self.collapsed;
            return InputResult::Handled;
        }

        // When collapsed, don't route input to children.
        if self.collapsed {
            return InputResult::Ignored;
        }

        // Handle Tab / BackTab to cycle focus among children.
        if let Event::Key(key) = event {
            if key.code == KeyCode::Tab {
                return self.cycle_child_focus(1);
            }
            if key.code == KeyCode::BackTab {
                return self.cycle_child_focus(-1);
            }
        }

        // Route to the focused child first.
        if let Some(idx) = self.focused_child &&
            idx < self.children.len()
        {
            let result = self.children[idx].handle_input(event);
            if result != InputResult::Ignored {
                return result;
            }
        }

        // Fall through to other children.
        for (i, child) in self.children.iter_mut().enumerate() {
            if Some(i) == self.focused_child {
                continue;
            }
            let result = child.handle_input(event);
            if result != InputResult::Ignored {
                return result;
            }
        }
        InputResult::Ignored
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
    use crate::{
        components::Text,
        layout::Constraint,
        theme::Theme,
    };

    #[test]
    fn div_renders_children() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
            ]))
            .child(Box::new(Text::new("top", 0, 0)))
            .child(Box::new(Text::new("bottom", 0, 0)));

            let rendered = div.render_rect(Rect::new(0, 0, 10, 2)).unwrap();
            assert_eq!(rendered.lines.len(), 2);
            assert!(rendered.lines[0].contains("top"));
            assert!(rendered.lines[1].contains("bottom"));
        });
    }

    #[test]
    fn div_with_border() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(Text::new("hi", 0, 0)))
                .border(Border::ROUNDED);

            let rendered = div.render_rect(Rect::new(0, 0, 6, 3)).unwrap();
            assert!(rendered.lines[0].contains("╭"));
            assert!(rendered.lines[2].contains("╰"));
        });
    }

    #[test]
    fn div_with_title() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(Text::new("hi", 0, 0)))
                .border(Border::ROUNDED)
                .title("Box");

            let rendered = div.render_rect(Rect::new(0, 0, 10, 3)).unwrap();
            assert!(rendered.lines[0].contains("Box"));
        });
    }

    #[test]
    fn div_with_padding() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(Text::new("hi", 0, 0)))
                .padding(Margin::new(1, 1));

            let rendered = div.render_rect(Rect::new(0, 0, 6, 3)).unwrap();
            // Padding shifts content down by 1 row and in by 1 col
            assert!(rendered.lines[1].contains("hi"));
        });
    }

    #[test]
    fn div_focus_propagation() {
        Theme::with(Theme::Light, || {
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)])).child(Box::new(
                crate::components::SelectList::new(vec!["a".into()], 1),
            ));

            div.set_focused(true);
            assert!(div.focused());
        });
    }

    /// Regression test: nested divs with non-zero rect coordinates must not
    /// double-offset content.
    #[test]
    fn div_nonzero_rect_no_double_offset() {
        Theme::with(Theme::Light, || {
            let outer = Div::new(Layout::horizontal([
                Constraint::Length(10),
                Constraint::Length(10),
            ]))
            .child(Box::new(Text::new("left", 0, 0)))
            .child(Box::new(
                Div::new(Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]))
                .child(Box::new(Text::new("a", 0, 0)))
                .child(Box::new(Text::new("b", 0, 0))),
            ));

            // Outer rect starts at (0, 2). Inner div gets y = 2 from the layout.
            let rendered = outer.render_rect(Rect::new(0, 2, 20, 2)).unwrap();
            // The inner div's content should be at local rows 0 and 1,
            // NOT shifted down by 2 due to double-offsetting.
            assert_eq!(
                rendered.lines.len(),
                2,
                "expected 2 lines, got {}",
                rendered.lines.len()
            );
            assert!(rendered.lines[0].contains("left"));
            assert!(rendered.lines[0].contains("a"));
            assert!(rendered.lines[1].contains("b"));
        });
    }

    /// Border must be drawn at the edges of the local buffer even when the
    /// parent rect has non-zero coordinates.
    #[test]
    fn div_border_with_nonzero_rect() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(Text::new("hi", 0, 0)))
                .border(Border::ROUNDED);

            let rendered = div.render_rect(Rect::new(0, 5, 6, 3)).unwrap();
            assert!(
                rendered.lines[0].contains("╭"),
                "border should be at local row 0"
            );
            assert!(
                rendered.lines[2].contains("╰"),
                "border should be at local row 2"
            );
            // Note: draw_border() currently replaces the entire middle rows,
            // so child content inside bordered divs is overwritten. This is a
            // pre-existing issue unrelated to the nonzero-rect fix.
        });
    }

    /// Regression: Tab must descend into nested Divs to reach leaf focusables.
    /// When outer Div's focused_child is a nested Div, Tab should cycle within
    /// that nested Div instead of immediately returning Ignored.
    #[test]
    fn div_tab_descends_into_nested_focusables() {
        Theme::with(Theme::Light, || {
            let mut inner = Div::new(Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
            ]));
            let input1 = crate::components::Input::new();
            let input2 = crate::components::Input::new();
            inner.push(Box::new(input1));
            inner.push(Box::new(input2));

            let mut outer = Div::new(Layout::vertical([Constraint::Length(2)]));
            outer.push(Box::new(inner));

            outer.set_focused(true);
            assert_eq!(outer.focused_child, Some(0));

            // With the old code, this Tab would return Ignored because outer
            // has no next sibling. With the fix, it should descend into inner
            // and cycle to its first focusable child.
            let tab = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Tab,
                crossterm::event::KeyModifiers::empty(),
            ));
            let result = outer.handle_input(&tab);
            assert!(
                matches!(result, crate::InputResult::Handled),
                "Tab should descend into nested div and be handled"
            );
        });
    }

    /// Regression: Tab cycling must move between siblings inside nested Divs.
    #[test]
    fn div_tab_cycles_across_nested_siblings() {
        Theme::with(Theme::Light, || {
            let mut inner = Div::new(Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
            ]));
            let mut input1 = crate::components::Input::new();
            let mut input2 = crate::components::Input::new();
            input1.set_text("first");
            input2.set_text("second");
            inner.push(Box::new(input1));
            inner.push(Box::new(input2));

            let mut outer = Div::new(Layout::vertical([Constraint::Length(2)]));
            outer.push(Box::new(inner));
            outer.set_focused(true);

            let tab = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Tab,
                crossterm::event::KeyModifiers::empty(),
            ));

            // First Tab: descend into inner, cycle input1 → input2
            let r1 = outer.handle_input(&tab);
            assert!(matches!(r1, crate::InputResult::Handled));

            // Second Tab: inner exhausted (input2 has no next sibling).
            // Outer also has no next sibling → Ignored.
            let r2 = outer.handle_input(&tab);
            assert!(matches!(r2, crate::InputResult::Ignored));
        });
    }

    #[test]
    fn div_collapsible_renders_header_when_collapsed() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .border(Border::ROUNDED)
                .title("Panel")
                .collapsible(true)
                .collapsed(true)
                .child(Box::new(Text::new("hidden", 0, 0)));

            let rendered = div.render_rect(Rect::new(0, 0, 20, 5)).unwrap();
            assert_eq!(rendered.lines.len(), 5);
            assert!(rendered.lines[0].contains("▶"));
            assert!(rendered.lines[0].contains("Panel"));
            // Child content should not be visible
            assert!(!rendered.lines.iter().any(|l| l.contains("hidden")));
        });
    }

    #[test]
    fn div_collapsible_toggles_on_enter() {
        Theme::with(Theme::Light, || {
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .border(Border::ROUNDED)
                .title("Panel")
                .collapsible(true)
                .collapsed(true)
                .child(Box::new(Text::new("content", 0, 0)));

            let enter = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ));

            assert!(div.collapsed);
            let result = div.handle_input(&enter);
            assert!(matches!(result, crate::InputResult::Handled));
            assert!(!div.collapsed);

            // Second Enter should collapse again
            div.handle_input(&enter);
            assert!(div.collapsed);
        });
    }

    #[test]
    fn div_collapsible_ignores_child_input_when_collapsed() {
        Theme::with(Theme::Light, || {
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .collapsible(true)
                .collapsed(true)
                .child(Box::new(crate::components::Input::new()));

            let a_key = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('a'),
                crossterm::event::KeyModifiers::empty(),
            ));

            // Should return Ignored because collapsed div doesn't route to children
            let result = div.handle_input(&a_key);
            assert!(matches!(result, crate::InputResult::Ignored));
        });
    }

    #[test]
    fn div_push_and_builders() {
        let mut div = Div::new(Layout::vertical([Constraint::Length(1)]));
        div.push(Box::new(Text::new("hi", 0, 0)));
        let div = div
            .border(Border::ROUNDED)
            .border_styled(Style::new().bold())
            .padding(Margin::new(1, 1))
            .title("T")
            .title_styled(Style::new().italic())
            .background(Style::new().bg(crate::theme::Color::WHITE))
            .collapsible(true)
            .collapsed(true);
        assert!(div.border.is_some());
        assert!(div.background.is_some());
        assert!(div.collapsed);
    }

    #[test]
    fn div_toggle_collapsed() {
        let mut div = Div::new(Layout::vertical([Constraint::Length(1)])).collapsed(true);
        div.toggle_collapsed();
        assert!(!div.collapsed);
        div.toggle_collapsed();
        assert!(div.collapsed);
    }

    #[test]
    fn div_collapsed_non_collapsible_indicator() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .border(Border::ROUNDED)
                .title("Panel")
                .collapsed(true)
                .child(Box::new(Text::new("hidden", 0, 0)));

            let rendered = div.render_rect(Rect::new(0, 0, 20, 1)).unwrap();
            assert!(rendered.lines[0].contains("Panel"));
            assert!(!rendered.lines[0].contains("▶"));
        });
    }

    #[test]
    fn div_collapsed_unfocused_style() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .title("Panel")
                .collapsible(true)
                .collapsed(true)
                .child(Box::new(Text::new("hidden", 0, 0)));

            let rendered = div.render_rect(Rect::new(0, 0, 20, 1)).unwrap();
            assert!(rendered.lines[0].contains("Panel"));
        });
    }

    #[test]
    fn div_background_fill() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .background(Style::new().bg(crate::theme::Color::WHITE))
                .child(Box::new(Text::new("hi", 0, 0)));

            let rendered = div.render_rect(Rect::new(0, 0, 5, 2)).unwrap();
            assert!(rendered.lines.iter().all(|l| l.contains("\x1b[48")));
        });
    }

    #[test]
    fn div_border_with_custom_style_and_title() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .border(Border::ROUNDED)
                .border_styled(Style::new().fg(crate::theme::Color::SUNBEAM_ORANGE))
                .title("Box")
                .child(Box::new(Text::new("hi", 0, 0)));

            let rendered = div.render_rect(Rect::new(0, 0, 10, 3)).unwrap();
            assert!(rendered.lines[0].contains("Box"));
        });
    }

    #[test]
    fn div_handle_input_space_toggles() {
        Theme::with(Theme::Light, || {
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .collapsible(true)
                .collapsed(true)
                .child(Box::new(Text::new("hi", 0, 0)));

            let space = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(' '),
                crossterm::event::KeyModifiers::empty(),
            ));
            let result = div.handle_input(&space);
            assert!(matches!(result, crate::InputResult::Handled));
            assert!(!div.collapsed);
        });
    }

    #[test]
    fn div_handle_input_falls_through_children() {
        Theme::with(Theme::Light, || {
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(Text::new("hi", 0, 0)));

            let a_key = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('a'),
                crossterm::event::KeyModifiers::empty(),
            ));
            let result = div.handle_input(&a_key);
            assert!(matches!(result, crate::InputResult::Ignored));
        });
    }

    #[test]
    fn div_focusable_trait_objects() {
        let div = Div::new(Layout::vertical([Constraint::Length(1)]));
        assert!(div.as_focusable().is_some());
        let mut div = Div::new(Layout::vertical([Constraint::Length(1)]));
        assert!(div.as_focusable_mut().is_some());
    }

    #[test]
    fn div_cycle_child_focus_no_focusable_children() {
        Theme::with(Theme::Light, || {
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(Text::new("hi", 0, 0)));

            let tab = crate::events::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Tab,
                crossterm::event::KeyModifiers::empty(),
            ));
            let result = div.handle_input(&tab);
            assert!(matches!(result, crate::InputResult::Ignored));
        });
    }

    #[test]
    fn div_render_uses_computed_height() {
        Theme::with(Theme::Light, || {
            let div = Div::new(Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
            ]))
            .child(Box::new(Text::new("a", 0, 0)))
            .child(Box::new(Text::new("b", 0, 0)));

            let rendered = div.render(10).unwrap();
            assert_eq!(rendered.lines.len(), 6);
        });
    }
}
