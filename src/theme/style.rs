//! Composite text styles for terminal rendering.
//!
//! A [`Style`] bundles foreground color, background color, and text
//! attributes (bold, italic, underline, dim) into a single unit that
//! can be applied to strings via [`stylize`].

use super::{ansi, Color, ColorMode};

/// A terminal text style: colors + attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Generate the ANSI escape prefix for this style.
    pub fn prefix(&self, mode: ColorMode) -> String {
        let mut parts = Vec::new();
        if let Some(c) = self.fg {
            parts.push(ansi::fg(c, mode));
        }
        if let Some(c) = self.bg {
            parts.push(ansi::bg(c, mode));
        }
        if self.bold {
            parts.push("\x1b[1m".to_string());
        }
        if self.dim {
            parts.push("\x1b[2m".to_string());
        }
        if self.italic {
            parts.push("\x1b[3m".to_string());
        }
        if self.underline {
            parts.push("\x1b[4m".to_string());
        }
        parts.concat()
    }

    /// The ANSI reset suffix.
    pub const fn suffix() -> &'static str {
        ansi::RESET
    }
}

/// Wrap `text` with the ANSI codes for `style`, automatically resetting
/// at the end. Respects the active color mode.
pub fn stylize(text: &str, style: &Style) -> String {
    let mode = ColorMode::detect();
    format!("{}{}{}", style.prefix(mode), text, Style::suffix())
}

/// Like [`stylize`], but pads `text` with `pad` spaces on each side.
pub fn stylize_padded(text: &str, style: &Style, pad: usize) -> String {
    let padding = " ".repeat(pad);
    stylize(&format!("{}{}{}", padding, text, padding), style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_builder() {
        let s = Style::new().fg(Color::SUNBEAM_ORANGE).bold();
        assert_eq!(s.fg, Some(Color::SUNBEAM_ORANGE));
        assert!(s.bold);
        assert!(!s.italic);
    }

    #[test]
    fn stylize_produces_codes() {
        let s = Style::new().fg(Color::SUNBEAM_ORANGE);
        let out = stylize("hi", &s);
        assert!(out.contains("hi"));
        assert!(out.starts_with('\x1b'));
        assert!(out.ends_with("\x1b[0m"));
    }

    #[test]
    fn stylize_padded_applies_padding() {
        let s = Style::new().fg(Color::WHITE);
        let out = stylize_padded("ok", &s, 2);
        assert!(out.contains("  ok  "));
    }
}
