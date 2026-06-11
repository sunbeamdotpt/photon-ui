use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    layout::{
        Border,
        Rect,
    },
    theme::{
        ColorMode,
        Palette,
        Style,
        Theme,
        stylize,
    },
};

/// A modal dialog that wraps content in a bordered box with an optional title.
///
/// The modal itself does not handle dismissal — that is the responsibility of
/// the caller (typically [`TUI`](crate::TUI) intercepting `Esc`).
///
/// # Example
///
/// ```
/// use photon_ui::components::{
///     Modal,
///     Text,
/// };
///
/// let modal = Modal::new(Box::new(Text::new("Are you sure?", 0, 0))).title("Confirm");
/// ```
pub struct Modal {
    content: Box<dyn Component>,
    title: Option<String>,
    border: Border,
    width: u16,
    focused: bool,
}

impl Modal {
    /// Create a new modal wrapping the given content.
    pub fn new(content: Box<dyn Component>) -> Self {
        Self {
            content,
            title: None,
            border: Border::ROUNDED,
            width: 40,
            focused: false,
        }
    }

    /// Set the title rendered in the top border.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the border style (default is rounded).
    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    /// Set the desired width of the modal content area.
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }
}

impl Focusable for Modal {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if let Some(f) = self.content.as_focusable_mut() {
            f.set_focused(focused);
        }
    }
}

impl Component for Modal {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let w = width.min(self.width);
        let rect = Rect::new(0, 0, w, 24); // height will be computed from content
        self.render_rect(rect)
    }

    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let mode = ColorMode::detect();
        let border_style = Style::new().fg(theme.border_default());
        let border_prefix = border_style.prefix(mode);
        let suffix = Style::suffix();

        let inner_w = rect.width.saturating_sub(2);
        let inner_h = rect.height.saturating_sub(2);

        // Render content inside the modal
        let content_rect = Rect::new(1, 1, inner_w, inner_h);
        let content_rendered = match self.content.render_rect(content_rect) {
            | Ok(r) => r,
            | Err(e) => return Err(e),
        };

        let content_h = content_rendered.lines.len().min(inner_h as usize) as u16;
        let _total_h = content_h + 2;

        let mut screen = Rendered::empty();

        let fill_w = inner_w as usize;

        // Top border
        {
            let mut top = String::new();
            // Left corner
            top.push_str(&border_prefix);
            top.push(self.border.top_left);
            top.push_str(suffix);

            if let Some(ref title) = self.title {
                let indicator = if self.focused { "▼ " } else { "▶ " };
                let max_title = fill_w.saturating_sub(2);
                let t = if title.len() > max_title {
                    &title[..max_title]
                } else {
                    title
                };
                let label = format!(" {}{} ", indicator, t);
                let label_styled = stylize(&label, &Style::new().fg(theme.text_primary()).bold());
                let t_visible = crate::utils::visible_width(&label_styled);
                let fill_count = fill_w.saturating_sub(t_visible);

                top.push_str(&label_styled);
                if fill_count > 0 {
                    top.push_str(&border_prefix);
                    top.push_str(&self.border.top.to_string().repeat(fill_count));
                    top.push_str(suffix);
                }
            } else {
                top.push_str(&border_prefix);
                top.push_str(&self.border.top.to_string().repeat(fill_w));
                top.push_str(suffix);
            }

            // Right corner
            top.push_str(&border_prefix);
            top.push(self.border.top_right);
            top.push_str(suffix);
            screen.lines.push(top);
        }

        // Content rows
        for i in 0..content_h {
            let mut line = String::new();
            line.push_str(&border_prefix);
            line.push(self.border.left);
            line.push_str(suffix);

            let content_line = content_rendered
                .lines
                .get(i as usize)
                .map(|s| s.as_str())
                .unwrap_or("");
            let pad = inner_w as usize - crate::utils::visible_width(content_line);
            line.push_str(content_line);
            if pad > 0 {
                line.push_str(&" ".repeat(pad));
            }

            line.push_str(&border_prefix);
            line.push(self.border.right);
            line.push_str(suffix);
            screen.lines.push(line);
        }

        // Bottom border
        {
            let mut bottom = String::new();
            bottom.push_str(&border_prefix);
            bottom.push(self.border.bottom_left);
            bottom.push_str(suffix);
            bottom.push_str(&border_prefix);
            bottom.push_str(&self.border.bottom.to_string().repeat(fill_w));
            bottom.push_str(suffix);
            bottom.push_str(&border_prefix);
            bottom.push(self.border.bottom_right);
            bottom.push_str(suffix);
            screen.lines.push(bottom);
        }

        // Propagate cursor
        if let Some((r, c)) = content_rendered.cursor &&
            r + 1 < screen.lines.len()
        {
            screen.cursor = Some((r + 1, c + 1));
        }

        Ok(screen)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        self.content.handle_input(event)
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
        theme::Theme,
    };

    #[test]
    fn modal_renders_with_border() {
        Theme::with(Theme::Light, || {
            let modal = Modal::new(Box::new(Text::new("hi", 0, 0)));
            let rendered = modal.render_rect(Rect::new(0, 0, 10, 5)).unwrap();
            assert!(rendered.lines[0].contains("╭"));
            assert!(rendered.lines[0].contains("╮"));
            assert!(rendered.lines[2].contains("╰"));
            assert!(rendered.lines[2].contains("╯"));
        });
    }

    #[test]
    fn modal_renders_title() {
        Theme::with(Theme::Light, || {
            let modal = Modal::new(Box::new(Text::new("hi", 0, 0))).title("Alert");
            let rendered = modal.render_rect(Rect::new(0, 0, 20, 5)).unwrap();
            assert!(rendered.lines[0].contains("Alert"));
        });
    }

    #[test]
    fn modal_forwards_focus() {
        Theme::with(Theme::Light, || {
            let mut modal = Modal::new(Box::new(Text::new("hi", 0, 0)));
            modal.set_focused(true);
            assert!(modal.focused());
        });
    }
}
