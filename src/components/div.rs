//! A flexible container with optional borders, padding, title, and background.
//!
//! `Div` is a general-purpose layout box. It accepts a [`Layout`] and renders
//! its children into the layout's areas, then draws optional chrome around the
//! result. Think of it as the TUI equivalent of an HTML `<div>`.
//!
//! # Example
//!
//! ```
//! use photon_ui::components::Div;
//! use photon_ui::layout::{Constraint, layout::Layout};
//!
//! let div = Div::new(Layout::vertical([
//!         Constraint::Length(1),
//!         Constraint::Min(3),
//!         Constraint::Length(1),
//!     ]))
//!     .child(Box::new(photon_ui::components::Text::new("Header", 0, 0)))
//!     .child(Box::new(photon_ui::components::Text::new("Body", 0, 0)))
//!     .child(Box::new(photon_ui::components::Text::new("Footer", 0, 0)))
//!     .border(photon_ui::layout::Border::ROUNDED)
//!     .padding(photon_ui::layout::Margin::new(1, 1))
//!     .title("My Box");
//! ```

use crate::{
    Component, Event, Focusable, InputResult, RenderError, Rendered,
};
use crate::layout::{Border, Margin, Rect, layout::Layout};
use crate::theme::{Style, Theme};
use crate::theme::Palette;

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

    /// Compute the inner content rect after subtracting border and padding.
    fn inner_rect(&self, rect: Rect) -> Rect {
        let mut inner = rect;
        if self.border.is_some() {
            inner = inner.inner(Margin::new(1, 1));
        }
        inner = inner.inner(self.padding);
        inner
    }
}

impl Focusable for Div {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        // Propagate focus state to focusable children
        for child in &mut self.children {
            if let Some(f) = child.as_focusable_mut() {
                f.set_focused(focused);
            }
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
        let theme = Theme::current();
        let mut screen = Rendered::empty();

        // Fill background if requested
        if let Some(ref bg) = self.background {
            let prefix = bg.prefix(crate::theme::ColorMode::detect());
            let suffix = Style::suffix();
            for _ in 0..rect.height {
                let line = format!("{}{:width$}{}", prefix, "", suffix, width = rect.width as usize);
                screen.lines.push(line);
            }
        }

        // Compute inner rect for children
        let inner = self.inner_rect(rect);

        // Render children into the inner rect using the layout
        let areas = self.layout.split(inner);
        for (child, area) in self.children.iter().zip(areas.iter()) {
            if let Ok(rendered) = child.render_rect(*area) {
                rendered.blit_into_rect(&mut screen, *area);
            }
        }

        // Ensure screen has enough lines for the full rect
        while screen.lines.len() < rect.height as usize {
            screen.lines.push(String::new());
        }

        // Draw border if requested
        if let Some(ref border) = self.border {
            let border_style = if self.border_style == Style::new() {
                Style::new().fg(theme.border_default())
            } else {
                self.border_style.clone()
            };
            crate::layout::draw_border(&mut screen, rect, border, &border_style);

            // Draw title in the top border if set
            if let Some(ref title) = self.title {
                if !screen.lines.is_empty() {
                    let title_style = if self.title_style == Style::new() {
                        Style::new().fg(theme.text_primary()).bold()
                    } else {
                        self.title_style.clone()
                    };
                    let label = format!(" {} ", title);
                    let label_styled = crate::theme::stylize(&label, &title_style);
                    let top = &mut screen.lines[0];
                    let start = 2usize.min(top.len());
                    let end = (start + crate::utils::visible_width(&label_styled)).min(top.len());
                    if start < top.len() {
                        top.replace_range(start..end, &label_styled);
                    }
                }
            }
        }

        Ok(screen)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        for child in &mut self.children {
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
    use crate::components::Text;
    use crate::layout::Constraint;
    use crate::theme::Theme;

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
            let mut div = Div::new(Layout::vertical([Constraint::Length(1)]))
                .child(Box::new(crate::components::SelectList::new(
                    vec!["a".into()],
                    1,
                )));

            div.set_focused(true);
            assert!(div.focused());
        });
    }
}
