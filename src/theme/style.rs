//! Composite text styles for terminal rendering.
//!
//! A [`Style`] bundles foreground color, background color, and text
//! attributes (bold, italic, underline, dim) into a single unit that
//! can be applied to strings via [`stylize`].

use super::{
    Color,
    ColorMode,
    ansi,
};

/// A terminal text style: colors + attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Style {
    /// Foreground color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>,
    /// Bold text attribute.
    pub bold: bool,
    /// Italic text attribute.
    pub italic: bool,
    /// Underline text attribute.
    pub underline: bool,
    /// Dim / faint text attribute.
    pub dim: bool,
}

impl Style {
    /// Create a new default style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the foreground color.
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set the background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Enable bold text.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Enable italic text.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Enable underlined text.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Enable dim text.
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
    ///
    /// When the style includes bold or faint, the suffix emits `\x1b[22m`
    /// before `\x1b[0m`. Some macOS terminals do not reliably clear bold on a
    /// full reset alone, so the explicit intensity reset prevents bold from
    /// leaking into subsequent text.
    pub fn suffix(&self) -> &'static str {
        if self.bold || self.dim {
            "\x1b[22m\x1b[0m"
        } else {
            ansi::RESET
        }
    }
}

/// Wrap `text` with the ANSI codes for `style`, automatically resetting
/// at the end. Respects the active color mode.
pub fn stylize(text: &str, style: &Style) -> String {
    let mode = ColorMode::detect();
    format!("{}{}{}", style.prefix(mode), text, style.suffix())
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

    /// Regression: bold styles must emit an explicit intensity reset before the
    /// full reset to prevent bold from leaking on macOS terminals.
    #[test]
    fn stylize_bold_emits_bold_off() {
        let s = Style::new().bold();
        let out = stylize("hi", &s);
        assert!(out.ends_with("\x1b[22m\x1b[0m"));
    }

    /// Non-bold styles should keep the plain reset suffix.
    #[test]
    fn stylize_non_bold_uses_plain_reset() {
        let s = Style::new().fg(Color::WHITE);
        let out = stylize("hi", &s);
        assert!(out.ends_with("\x1b[0m"));
        assert!(!out.contains("\x1b[22m"));
    }

    #[test]
    fn style_italic_prefix_and_suffix() {
        let s = Style::new().italic();
        let out = stylize("hi", &s);
        assert!(out.contains("\x1b[3m"));
        assert!(out.ends_with("\x1b[0m"));
    }

    #[test]
    fn style_dim_prefix_and_suffix() {
        let s = Style::new().dim();
        let out = stylize("hi", &s);
        assert!(out.contains("\x1b[2m"));
        assert!(out.ends_with("\x1b[22m\x1b[0m"));
    }

    #[test]
    fn style_full_prefix() {
        let s = Style::new()
            .fg(Color::WHITE)
            .bg(Color::BLACK)
            .bold()
            .italic()
            .underline()
            .dim();
        let prefix = s.prefix(ColorMode::TrueColor);
        assert!(prefix.contains("\x1b[38;2;255;255;255m"));
        assert!(prefix.contains("\x1b[48;2;0;0;0m"));
        assert!(prefix.contains("\x1b[1m"));
        assert!(prefix.contains("\x1b[2m"));
        assert!(prefix.contains("\x1b[3m"));
        assert!(prefix.contains("\x1b[4m"));
    }
}
