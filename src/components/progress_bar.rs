//! Progress bar component.
//!
//! Renders as a bracketed bar with filled and empty segments, optionally
//! followed by a percentage label.

use crate::{
    Component,
    RenderError,
    Rendered,
};
use crate::theme::{
    Palette,
    Style,
    Theme,
    stylize,
};

/// A non-interactive progress bar.
///
/// Renders as `[██████░░░░]  60%` (or without the percentage if hidden).
/// The filled segment uses the theme's accent color; the empty segment uses
/// the theme's default border color.
pub struct ProgressBar {
    label: String,
    value: f32,
    width: u16,
    show_percent: bool,
}

impl ProgressBar {
    /// Create a new progress bar with the given label and value.
    ///
    /// `value` is clamped to the range `0.0..=1.0`.
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        Self {
            label: label.into(),
            value: value.clamp(0.0, 1.0),
            width: 20,
            show_percent: true,
        }
    }

    /// Set the total bar width in columns (including brackets).
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Hide the percentage label.
    pub fn hide_percent(mut self) -> Self {
        self.show_percent = false;
        self
    }
}

impl Component for ProgressBar {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let accent_style = Style::new().fg(theme.accent());
        let empty_style = Style::new().fg(theme.border_default());

        let inner_width = self.width.saturating_sub(2) as usize;
        let filled = (self.value * inner_width as f32).round() as usize;
        let filled = filled.min(inner_width);
        let empty = inner_width.saturating_sub(filled);

        let filled_str = "█".repeat(filled);
        let empty_str = "░".repeat(empty);

        let filled_styled = stylize(&filled_str, &accent_style);
        let empty_styled = stylize(&empty_str, &empty_style);

        let bar = format!("[{}{}]", filled_styled, empty_styled);

        let mut line = if self.label.is_empty() {
            bar
        } else {
            format!("{} {}", self.label, bar)
        };

        if self.show_percent {
            let percent = (self.value * 100.0).round() as u8;
            line.push_str(&format!("  {}%", percent));
        }

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
    fn renders_with_percent() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("", 0.6).width(10);
            let rendered = pb.render(80).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            let line = &rendered.lines[0];
            assert!(line.contains('['));
            assert!(line.contains(']'));
            assert!(line.contains("60%"));
        });
    }

    #[test]
    fn hides_percent() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("", 0.6).width(10).hide_percent();
            let rendered = pb.render(80).unwrap();
            assert!(!rendered.lines[0].contains('%'));
        });
    }

    #[test]
    fn label_is_prepended() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("Loading", 0.5).width(10);
            let rendered = pb.render(80).unwrap();
            assert!(rendered.lines[0].starts_with("Loading "));
        });
    }

    #[test]
    fn value_is_clamped() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("", 1.5).width(10);
            let rendered = pb.render(80).unwrap();
            assert!(rendered.lines[0].contains("100%"));
        });
    }

    #[test]
    fn zero_value_renders_empty() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("", 0.0).width(10);
            let rendered = pb.render(80).unwrap();
            let line = &rendered.lines[0];
            let start = line.find('[').unwrap();
            let end = line.find(']').unwrap();
            let inner = &line[start + 1..end];
            // No filled blocks inside the brackets
            assert!(!inner.contains('█'));
        });
    }

    #[test]
    fn full_value_renders_full() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("", 1.0).width(10);
            let rendered = pb.render(80).unwrap();
            let line = &rendered.lines[0];
            let start = line.find('[').unwrap();
            let end = line.find(']').unwrap();
            let inner = &line[start + 1..end];
            // No empty blocks inside the brackets
            assert!(!inner.contains('░'));
        });
    }

    #[test]
    fn uses_accent_color() {
        Theme::with(Theme::Light, || {
            let pb = ProgressBar::new("", 0.5).width(10);
            let rendered = pb.render(80).unwrap();
            // Light theme accent is SUNBEAM_ORANGE (#fa520f = 250,82,15)
            assert!(rendered.lines[0].contains("\x1b[38;2;250;82;15m"));
        });
    }
}
