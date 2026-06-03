//! A visual divider line — horizontal or vertical.

use crate::{
    Component,
    RenderError,
    Rendered,
    layout::Direction,
    theme::{
        Palette,
        Style,
        Theme,
    },
};

/// A standalone divider line.
///
/// Draws a repeating character (`─` for horizontal, `│` for vertical) across
/// the full width or height of its assigned area. An optional label can be
/// centered on the line.
pub struct Divider {
    direction: Direction,
    style: Style,
    label: Option<String>,
}

impl Divider {
    /// Create a horizontal divider.
    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            style: Style::new(),
            label: None,
        }
    }

    /// Create a vertical divider.
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Vertical,
            style: Style::new(),
            label: None,
        }
    }

    /// Apply a style to the divider line.
    pub fn styled(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Center a label on the divider.
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Component for Divider {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let style = if self.style == Style::new() {
            Style::new().fg(theme.border_default())
        } else {
            self.style.clone()
        };

        let line = match self.direction {
            | Direction::Horizontal => {
                if let Some(ref label) = self.label {
                    let label = format!(" {} ", label);
                    let label_vw = crate::utils::visible_width(&label);
                    if label_vw + 4 > width as usize {
                        let line = "─".repeat(width as usize);
                        crate::theme::stylize(&line, &style)
                    } else {
                        let side = (width as usize - label_vw) / 2;
                        let left = "─".repeat(side);
                        let right = "─".repeat(width as usize - side - label_vw);
                        let text = crate::theme::stylize(&label, &style);
                        let left = crate::theme::stylize(&left, &style);
                        let right = crate::theme::stylize(&right, &style);
                        format!("{}{}{}", left, text, right)
                    }
                } else {
                    let line = "─".repeat(width as usize);
                    crate::theme::stylize(&line, &style)
                }
            },
            | Direction::Vertical => {
                let line = "│".repeat(width as usize);
                crate::theme::stylize(&line, &style)
            },
        };

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }

    fn render_rect(&self, rect: crate::layout::Rect) -> Result<Rendered, RenderError> {
        if self.direction == Direction::Vertical && rect.height > 1 {
            let theme = Theme::current();
            let style = if self.style == Style::new() {
                Style::new().fg(theme.border_default())
            } else {
                self.style.clone()
            };
            let ch = crate::theme::stylize("│", &style);
            let mut lines = Vec::new();
            for _ in 0..rect.height {
                lines.push(ch.clone());
            }
            Ok(Rendered {
                lines,
                cursor: None,
                images: Vec::new(),
            })
        } else {
            self.render(rect.width)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn divider_horizontal_renders_line() {
        Theme::with(Theme::Light, || {
            let div = Divider::horizontal();
            let rendered = div.render(10).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert!(rendered.lines[0].contains("─"));
        });
    }

    #[test]
    fn divider_vertical_renders_pipe() {
        Theme::with(Theme::Light, || {
            let div = Divider::vertical();
            let rendered = div.render(1).unwrap();
            assert!(rendered.lines[0].contains("│"));
        });
    }

    #[test]
    fn divider_labeled_renders_label() {
        Theme::with(Theme::Light, || {
            let div = Divider::horizontal().labeled("Section");
            let rendered = div.render(30).unwrap();
            assert!(rendered.lines[0].contains("Section"));
        });
    }

    #[test]
    fn divider_rect_vertical_multi_line() {
        Theme::with(Theme::Light, || {
            let div = Divider::vertical();
            let rendered = div
                .render_rect(crate::layout::Rect::new(0, 0, 1, 3))
                .unwrap();
            assert_eq!(rendered.lines.len(), 3);
            for line in &rendered.lines {
                assert!(line.contains("│"));
            }
        });
    }
}
