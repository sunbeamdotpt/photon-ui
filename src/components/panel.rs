//! A bordered panel component for wrapping content.
//!
//! Renders a rectangular frame with rounded corners (╭─╮││╰─╯) around
//! optional content lines. Supports titles, custom borders, and theming.

use crate::{
    Component,
    RenderError,
    Rendered,
    layout::Border,
    theme::{
        ColorMode,
        Palette,
        Style,
        Theme,
    },
};

/// A panel with a border around optional content.
pub struct Panel {
    title: Option<String>,
    lines: Vec<String>,
    border: Border,
    pad: u16,
}

impl Panel {
    /// Create an empty panel with the default rounded border.
    pub fn new() -> Self {
        Self {
            title: None,
            lines: Vec::new(),
            border: Border::ROUNDED,
            pad: 1,
        }
    }

    /// Set the panel title (rendered in the top border).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set content lines displayed inside the panel.
    pub fn lines(mut self, lines: Vec<String>) -> Self {
        self.lines = lines;
        self
    }

    /// Set the border style.
    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    /// Set inner padding (spaces between border and content).
    pub fn pad(mut self, pad: u16) -> Self {
        self.pad = pad;
        self
    }

    /// Compute the total height needed for this panel.
    ///
    /// Padding is horizontal only; vertical space is determined by the
    /// border (top/bottom rows) plus content lines.
    pub fn height(&self) -> u16 {
        let mut h = self.lines.len() as u16;
        if self.border.top != ' ' {
            h += 1;
        }
        if self.border.bottom != ' ' {
            h += 1;
        }
        h.max(1)
    }

    /// Build the border style from the current theme.
    fn border_style(&self) -> Style {
        Style::new().fg(Theme::current().border_default())
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Panel {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let _theme = Theme::current();
        let border_style = self.border_style();
        let (border_w, _border_h) = self.border.size();
        let pad = self.pad as usize;
        let inner_width = width.saturating_sub(border_w * 2 + self.pad * 2) as usize;

        let mode = ColorMode::detect();
        let prefix = border_style.prefix(mode);
        let suffix = Style::suffix();

        let mut rendered = Rendered::empty();
        let total_width = inner_width + pad * 2; // width between the two borders

        // ── Top border (with optional title) ──
        {
            let mut top = String::new();
            top.push_str(&prefix);
            top.push(self.border.top_left);

            let available = total_width;
            let title_text = self.title.as_ref().map(|t| {
                let max_title = available.saturating_sub(2);
                let t = if t.len() > max_title {
                    &t[..max_title]
                } else {
                    t
                };
                format!(" {} ", t)
            });

            let fill = if let Some(ref t) = title_text {
                let t_visible = crate::utils::visible_width(t);
                let fill_count = available.saturating_sub(t_visible);
                let mut s = t.clone();
                s.push_str(&self.border.top.to_string().repeat(fill_count));
                s
            } else {
                self.border.top.to_string().repeat(available)
            };

            top.push_str(&fill);
            top.push(self.border.top_right);
            top.push_str(suffix);
            rendered.lines.push(top);
        }

        // ── Content rows ──
        let content_height = self.lines.len().max(1);
        for i in 0..content_height {
            let mut line = String::new();
            if self.border.left != ' ' {
                line.push_str(&prefix);
                line.push(self.border.left);
                line.push_str(suffix);
            }
            for _ in 0..pad {
                line.push(' ');
            }

            let content = if i < self.lines.len() {
                crate::utils::truncate_to_width(&self.lines[i], inner_width as u16, "")
            } else {
                String::new()
            };
            line.push_str(&content);

            let content_visible = crate::utils::visible_width(&content);
            for _ in content_visible..inner_width {
                line.push(' ');
            }

            for _ in 0..pad {
                line.push(' ');
            }

            if self.border.right != ' ' && width > 1 {
                line.push_str(&prefix);
                line.push(self.border.right);
                line.push_str(suffix);
            }
            rendered.lines.push(line);
        }

        // ── Bottom border ──
        {
            let mut bottom = String::new();
            bottom.push_str(&prefix);
            bottom.push(self.border.bottom_left);
            bottom.push_str(&self.border.bottom.to_string().repeat(total_width));
            bottom.push(self.border.bottom_right);
            bottom.push_str(suffix);
            rendered.lines.push(bottom);
        }

        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn panel_renders_rounded_border() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().lines(vec!["Hello".into()]);
            let rendered = panel.render(9).unwrap();
            assert_eq!(rendered.lines.len(), 3);
            assert!(rendered.lines[0].contains('╭'));
            assert!(rendered.lines[0].contains('╮'));
            assert!(rendered.lines[1].contains('│'));
            assert!(rendered.lines[2].contains('╰'));
            assert!(rendered.lines[2].contains('╯'));
        });
    }

    #[test]
    fn panel_renders_thin_border() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().border(Border::THIN).lines(vec!["Hi".into()]);
            let rendered = panel.render(6).unwrap();
            assert!(rendered.lines[0].contains('┌'));
            assert!(rendered.lines[0].contains('┐'));
            assert!(rendered.lines[2].contains('└'));
            assert!(rendered.lines[2].contains('┘'));
        });
    }

    #[test]
    fn panel_renders_thick_border() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().border(Border::THICK).lines(vec!["X".into()]);
            let rendered = panel.render(5).unwrap();
            assert!(rendered.lines[0].contains('┏'));
            assert!(rendered.lines[0].contains('┓'));
            assert!(rendered.lines[2].contains('┗'));
            assert!(rendered.lines[2].contains('┛'));
        });
    }

    #[test]
    fn panel_renders_double_border() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().border(Border::DOUBLE).lines(vec!["X".into()]);
            let rendered = panel.render(5).unwrap();
            assert!(rendered.lines[0].contains('╔'));
            assert!(rendered.lines[0].contains('╗'));
            assert!(rendered.lines[2].contains('╚'));
            assert!(rendered.lines[2].contains('╝'));
        });
    }

    #[test]
    fn panel_with_title() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().title("Test").lines(vec!["Content".into()]);
            let rendered = panel.render(15).unwrap();
            // Title should appear in top line
            assert!(rendered.lines[0].contains("Test"));
            // Content should appear inside
            assert!(rendered.lines[1].contains("Content"));
        });
    }

    #[test]
    fn panel_content_is_inside_border() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().lines(vec!["A".into()]);
            let rendered = panel.render(5).unwrap();
            // 5 cols: ╭ A ╮  → row 1 should have A between borders
            let row1 = &rendered.lines[1];
            assert!(row1.contains('A'));
            assert!(row1.contains('│'));
        });
    }

    #[test]
    fn panel_with_padding() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().pad(2).lines(vec!["X".into()]);
            let rendered = panel.render(7).unwrap();
            // Padding is horizontal only; height = top border + content + bottom border = 3
            assert_eq!(rendered.lines.len(), 3);
            // Content should be indented by border + pad = 1 + 2 = 3 cols
            let row1 = &rendered.lines[1];
            assert!(row1.contains('X'));
        });
    }

    #[test]
    fn panel_empty_lines_still_has_border() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new();
            let rendered = panel.render(5).unwrap();
            assert!(rendered.lines[0].contains('╭'));
            assert!(rendered.lines[0].contains('╮'));
        });
    }

    #[test]
    fn panel_trims_long_content() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().lines(vec!["This is way too long".into()]);
            let rendered = panel.render(10).unwrap();
            // Inner width = 10 - 2*1 (border) - 2*1 (pad) = 6
            // Content should be truncated
            let row1 = &rendered.lines[1];
            assert!(!row1.contains("way too long"));
        });
    }

    #[test]
    fn panel_uses_theme_color() {
        Theme::with(Theme::Light, || {
            let panel = Panel::new().lines(vec!["Hi".into()]);
            let rendered = panel.render(6).unwrap();
            // Border should have ANSI codes
            assert!(rendered.lines[0].starts_with('\x1b'));
        });
    }

    #[test]
    fn panel_height_calculation() {
        let panel = Panel::new().lines(vec!["a".into(), "b".into()]);
        // top border + 2 content lines + bottom border = 4
        assert_eq!(panel.height(), 4);
    }

    #[test]
    fn panel_default_is_rounded() {
        let panel = Panel::default();
        assert_eq!(panel.border, Border::ROUNDED);
    }
}
