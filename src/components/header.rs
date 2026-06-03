//! Header component.
//!
//! Renders a left-aligned title with optional right-aligned action labels.

use crate::{
    Component,
    RenderError,
    Rendered,
    theme::{
        Palette,
        Style,
        Theme,
        stylize,
    },
    utils::{
        truncate_to_width,
        visible_width,
    },
};

/// A non-interactive header bar.
///
/// Renders as `Title          [Action1] [Action2]` where the title is
/// left-aligned, bold, and uses the primary text color; actions are
/// right-aligned and use the secondary text color.
pub struct Header {
    title: String,
    actions: Vec<String>,
}

impl Header {
    /// Create a new header with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            actions: Vec::new(),
        }
    }

    /// Add a right-aligned action label.
    ///
    /// Actions are rendered as `[label]`.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.actions.push(label.into());
        self
    }
}

impl Component for Header {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let primary_style = Style::new().fg(theme.text_primary()).bold();
        let secondary_style = Style::new().fg(theme.text_secondary());

        let action_labels: Vec<String> = self.actions.iter().map(|a| format!("[{}]", a)).collect();
        let actions_plain = action_labels.join(" ");
        let actions_vw = visible_width(&actions_plain);

        let title_vw = visible_width(&self.title);

        let line = if self.actions.is_empty() {
            let title_text = if title_vw > width as usize {
                truncate_to_width(&self.title, width, "…")
            } else {
                self.title.clone()
            };
            stylize(&title_text, &primary_style)
        } else {
            let padding_width = 1usize;
            let total_needed = title_vw + padding_width + actions_vw;

            let title_text = if total_needed > width as usize {
                let avail = (width as usize).saturating_sub(padding_width + actions_vw);
                truncate_to_width(&self.title, avail as u16, "…")
            } else {
                self.title.clone()
            };

            let title_styled = stylize(&title_text, &primary_style);
            let actions_styled = action_labels
                .iter()
                .map(|a| stylize(a, &secondary_style))
                .collect::<Vec<_>>()
                .join(" ");

            let title_styled_vw = visible_width(&title_styled);
            let actions_styled_vw = visible_width(&actions_styled);
            let pad_len = (width as usize).saturating_sub(title_styled_vw + actions_styled_vw);

            format!("{}{}{}", title_styled, " ".repeat(pad_len), actions_styled)
        };

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn renders_title_only() {
        Theme::with(Theme::Light, || {
            let header = Header::new("My App");
            let rendered = header.render(20).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert!(rendered.lines[0].contains("My App"));
        });
    }

    #[test]
    fn renders_actions_right_aligned() {
        Theme::with(Theme::Light, || {
            let header = Header::new("My App").action("Save").action("Delete");
            let rendered = header.render(40).unwrap();
            let line = &rendered.lines[0];

            assert!(line.contains("My App"));
            assert!(line.contains("[Save]"));
            assert!(line.contains("[Delete]"));

            let title_pos = line.find("My App").unwrap();
            let save_pos = line.find("[Save]").unwrap();
            assert!(save_pos > title_pos);
        });
    }

    #[test]
    fn title_uses_primary_color_and_bold() {
        Theme::with(Theme::Light, || {
            let header = Header::new("Title");
            let rendered = header.render(20).unwrap();
            let line = &rendered.lines[0];
            // Light theme text_primary is #1f1f1f = 31,31,31
            assert!(line.contains("\x1b[38;2;31;31;31m"));
            assert!(line.contains("\x1b[1m"));
        });
    }

    #[test]
    fn actions_use_secondary_color() {
        Theme::with(Theme::Light, || {
            let header = Header::new("Title").action("Help");
            let rendered = header.render(20).unwrap();
            let line = &rendered.lines[0];
            // Light theme text_secondary is #666666 = 102,102,102
            assert!(line.contains("\x1b[38;2;102;102;102m"));
        });
    }

    #[test]
    fn truncates_title_when_too_wide() {
        Theme::with(Theme::Light, || {
            let header = Header::new("Very Long Title Indeed").action("X");
            let rendered = header.render(15).unwrap();
            let line = &rendered.lines[0];
            assert!(visible_width(line) <= 15);
            assert!(line.contains("[X]"));
        });
    }

    #[test]
    fn empty_actions_renders_title_only() {
        Theme::with(Theme::Light, || {
            let header = Header::new("Only Title");
            let rendered = header.render(20).unwrap();
            assert!(rendered.lines[0].contains("Only Title"));
            // No action labels like [Save] should appear
            assert!(!rendered.lines[0].contains("[Only Title]"));
        });
    }
}
