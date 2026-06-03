//! Beam Design Language button component.
//!
//! Supports five visual variants: Primary, Dark, Cream, Ghost, and Text.
//! Each variant maps to semantic palette colors and renders as a single
//! line of styled text with appropriate foreground/background colors.

use crate::events::Event;
use crate::layout::Rect;
use crate::theme::{stylize_padded, Color, Palette, Style, Theme};
use crate::{Component, InputResult, RenderError, Rendered};

/// Visual variant of a button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Dark background, white text. Default.
    #[default]
    Dark,
    /// Cream background, dark text.
    Cream,
    /// Transparent with a border.
    Ghost,
    /// Plain text, accent color, underlined.
    Text,
    /// Accent (orange) background, white text.
    Primary,
}

/// A styled button component.
///
/// Renders as a single line of text with padding and ANSI colors.
/// The width is determined by the label length plus padding.
pub struct Button {
    label: String,
    variant: ButtonVariant,
    pad: usize,
}

impl Button {
    /// Create a new button with the given label and variant.
    pub fn new(label: impl Into<String>, variant: ButtonVariant) -> Self {
        Self {
            label: label.into(),
            variant,
            pad: 1,
        }
    }

    /// Create a primary button.
    pub fn primary(label: impl Into<String>) -> Self {
        Self::new(label, ButtonVariant::Primary)
    }

    /// Create a dark button.
    pub fn dark(label: impl Into<String>) -> Self {
        Self::new(label, ButtonVariant::Dark)
    }

    /// Create a cream button.
    pub fn cream(label: impl Into<String>) -> Self {
        Self::new(label, ButtonVariant::Cream)
    }

    /// Create a ghost button.
    pub fn ghost(label: impl Into<String>) -> Self {
        Self::new(label, ButtonVariant::Ghost)
    }

    /// Create a text button.
    pub fn text(label: impl Into<String>) -> Self {
        Self::new(label, ButtonVariant::Text)
    }

    /// Set horizontal padding (spaces on each side).
    pub fn pad(mut self, pad: usize) -> Self {
        self.pad = pad;
        self
    }

    /// Build the ANSI style for this button given the active theme.
    fn build_style(&self) -> Style {
        let theme = Theme::current();
        match self.variant {
            ButtonVariant::Primary => Style::new()
                .fg(Color::WHITE)
                .bg(theme.accent())
                .bold(),
            // Dark is a fixed visual style: near-black bg, white text.
            // In dark mode use CARD_DARK so it's visible against the black page.
            ButtonVariant::Dark => match theme {
                Theme::Light => Style::new().fg(Color::WHITE).bg(Color::SUNBEAM_BLACK).bold(),
                Theme::Dark => Style::new().fg(Color::WHITE).bg(Color::CARD_DARK).bold(),
            },
            // Cream is always cream bg + dark text, regardless of theme.
            ButtonVariant::Cream => Style::new()
                .fg(Color::SUNBEAM_BLACK)
                .bg(Color::CREAM)
                .bold(),
            ButtonVariant::Ghost => Style::new()
                .fg(theme.accent())
                .bold(),
            ButtonVariant::Text => Style::new()
                .fg(theme.accent())
                .underline(),
        }
    }
}

impl Component for Button {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let style = self.build_style();

        let line = match self.variant {
            ButtonVariant::Ghost => {
                // Ghost: [ label ] with brackets in muted color
                let theme = Theme::current();
                let bracket_style = Style::new().fg(theme.border_default());
                let bracket_open = crate::theme::stylize("[", &bracket_style);
                let bracket_close = crate::theme::stylize("]", &bracket_style);
                let inner = stylize_padded(&self.label, &style, self.pad);
                format!("{}{}{}", bracket_open, inner, bracket_close)
            }
            _ => stylize_padded(&self.label, &style, self.pad),
        };

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }

    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError> {
        // Center the button vertically within the rect
        let mut rendered = self.render(rect.width)?;
        let height = rendered.lines.len();
        let pad_top = (rect.height as usize).saturating_sub(height) / 2;

        let mut lines = Vec::new();
        for _ in 0..pad_top {
            lines.push(String::new());
        }
        lines.extend(rendered.lines);
        while lines.len() < rect.height as usize {
            lines.push(String::new());
        }
        rendered.lines = lines;
        Ok(rendered)
    }

    fn handle_input(&mut self, _event: &Event) -> InputResult {
        InputResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, Theme};

    #[test]
    fn primary_button_renders() {
        Theme::with(Theme::Light, || {
            let btn = Button::primary("Click me");
            let rendered = btn.render(80).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert!(rendered.lines[0].contains("Click me"));
            // Should have ANSI codes
            assert!(rendered.lines[0].starts_with('\x1b'));
        });
    }

    #[test]
    fn dark_button_renders() {
        Theme::with(Theme::Light, || {
            let btn = Button::dark("Submit");
            let rendered = btn.render(80).unwrap();
            assert!(rendered.lines[0].contains("Submit"));
        });
    }

    #[test]
    fn ghost_button_has_brackets() {
        Theme::with(Theme::Light, || {
            let btn = Button::ghost("Cancel");
            let rendered = btn.render(80).unwrap();
            let line = &rendered.lines[0];
            assert!(line.contains('['));
            assert!(line.contains(']'));
            assert!(line.contains("Cancel"));
        });
    }

    #[test]
    fn text_button_is_underlined() {
        Theme::with(Theme::Light, || {
            let btn = Button::text("Link");
            let rendered = btn.render(80).unwrap();
            // Underline ANSI code is \x1b[4m
            assert!(rendered.lines[0].contains("\x1b[4m"));
        });
    }

    #[test]
    fn button_padding() {
        Theme::with(Theme::Light, || {
            let btn = Button::primary("OK").pad(2);
            let rendered = btn.render(80).unwrap();
            // Should have 2 spaces on each side
            assert!(rendered.lines[0].contains("  OK  "));
        });
    }

    #[test]
    fn button_respects_theme() {
        // Light theme: primary bg is orange
        let light_line = Theme::with(Theme::Light, || {
            Button::primary("Test").render(80).unwrap().lines[0].clone()
        });

        // Dark theme: primary bg is still orange, but text colors differ
        let dark_line = Theme::with(Theme::Dark, || {
            Button::primary("Test").render(80).unwrap().lines[0].clone()
        });

        // Both should contain the label
        assert!(light_line.contains("Test"));
        assert!(dark_line.contains("Test"));
    }

    // ── Regression: dark-mode visibility ─────────────────────────────

    /// Regression: Dark button must not use white background in dark mode.
    /// Previously `bg(text_primary())` produced white-on-white.
    #[test]
    fn dark_button_not_white_on_white_in_dark_mode() {
        let line = Theme::with(Theme::Dark, || {
            Button::dark("Dark").render(80).unwrap().lines[0].clone()
        });
        // White bg ANSI: \x1b[48;2;255;255;255m
        assert!(
            !line.contains("\x1b[48;2;255;255;255m"),
            "Dark button must not have white bg in dark mode"
        );
        // Should have CARD_DARK bg (#2a2a2a)
        assert!(
            line.contains("\x1b[48;2;42;42;42m"),
            "Dark button should use CARD_DARK (#2a2a2a) bg in dark mode"
        );
        // Text should be visible (white fg)
        assert!(
            line.contains("\x1b[38;2;255;255;255m"),
            "Dark button should have white text"
        );
    }

    /// Regression: Dark button must use black background in light mode.
    #[test]
    fn dark_button_uses_black_bg_in_light_mode() {
        let line = Theme::with(Theme::Light, || {
            Button::dark("Dark").render(80).unwrap().lines[0].clone()
        });
        assert!(
            line.contains("\x1b[48;2;31;31;31m"),
            "Dark button should use SUNBEAM_BLACK (#1f1f1f) bg in light mode"
        );
    }

    /// Regression: Cream button must always use cream bg + dark text.
    /// Previously in dark mode it used bg_card() which was nearly invisible.
    #[test]
    fn cream_button_always_cream_colored() {
        let light_line = Theme::with(Theme::Light, || {
            Button::cream("Cream").render(80).unwrap().lines[0].clone()
        });
        let dark_line = Theme::with(Theme::Dark, || {
            Button::cream("Cream").render(80).unwrap().lines[0].clone()
        });

        // Cream bg: #fff0c2 = (255, 240, 194)
        let cream_bg = "\x1b[48;2;255;240;194m";
        assert!(light_line.contains(cream_bg), "Cream button bg in light mode");
        assert!(dark_line.contains(cream_bg), "Cream button bg in dark mode");

        // Dark text: SUNBEAM_BLACK #1f1f1f = (31, 31, 31)
        let dark_fg = "\x1b[38;2;31;31;31m";
        assert!(light_line.contains(dark_fg), "Cream button fg in light mode");
        assert!(dark_line.contains(dark_fg), "Cream button fg in dark mode");
    }
}
