//! Semantic color tokens for the Beam Design Language.
//!
//! Provides a [`Palette`] trait that maps semantic names (e.g. `bg.page`,
//! `text.primary`) to concrete [`Color`] values. The [`Theme`] enum
//! implements this trait for the built-in Light and Dark variants.

use std::cell::RefCell;

use super::Color;

// ── Thread-local active theme ─────────────────────────────────────

thread_local! {
    static ACTIVE_THEME: RefCell<Theme> = RefCell::new(Theme::Light);
}

/// The active theme variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    /// Set the globally active theme.
    pub fn set(theme: Theme) {
        ACTIVE_THEME.with(|t| *t.borrow_mut() = theme);
    }

    /// Get the globally active theme.
    pub fn current() -> Self {
        ACTIVE_THEME.with(|t| *t.borrow())
    }

    /// Run a closure with a specific theme, then restore the previous theme.
    pub fn with<T>(theme: Theme, f: impl FnOnce() -> T) -> T {
        let previous = Self::current();
        Self::set(theme);
        let result = f();
        Self::set(previous);
        result
    }
}

// ── Palette trait ─────────────────────────────────────────────────

/// A color palette that resolves semantic tokens to RGB values.
pub trait Palette {
    fn bg_page(&self) -> Color;
    fn bg_card(&self) -> Color;
    fn bg_nav(&self) -> Color;
    fn bg_input(&self) -> Color;

    fn text_primary(&self) -> Color;
    fn text_secondary(&self) -> Color;
    fn text_muted(&self) -> Color;
    fn text_on_accent(&self) -> Color;

    fn accent(&self) -> Color;
    fn accent_hover(&self) -> Color;
    fn section_label(&self) -> Color;

    fn border_default(&self) -> Color;
    fn border_subtle(&self) -> Color;
    fn border_focus(&self) -> Color;

    fn success(&self) -> Color;
    fn warning(&self) -> Color;
    fn error(&self) -> Color;
    fn info(&self) -> Color;
}

impl Palette for Theme {
    fn bg_page(&self) -> Color {
        match self {
            | Theme::Light => Color::WARM_IVORY,
            | Theme::Dark => Color::SUNBEAM_BLACK,
        }
    }

    fn bg_card(&self) -> Color {
        match self {
            | Theme::Light => Color::CREAM,
            | Theme::Dark => Color::CARD_DARK,
        }
    }

    fn bg_nav(&self) -> Color {
        match self {
            | Theme::Light => Color::CREAM,
            | Theme::Dark => Color::CARD_DARK,
        }
    }

    fn bg_input(&self) -> Color {
        match self {
            | Theme::Light => Color::WHITE,
            | Theme::Dark => Color::CARD_DARK,
        }
    }

    fn text_primary(&self) -> Color {
        match self {
            | Theme::Light => Color::SUNBEAM_BLACK,
            | Theme::Dark => Color::WHITE,
        }
    }

    fn text_secondary(&self) -> Color {
        match self {
            | Theme::Light => Color(0x66, 0x66, 0x66),
            | Theme::Dark => Color(0xbb, 0xbb, 0xbb),
        }
    }

    fn text_muted(&self) -> Color {
        match self {
            | Theme::Light => Color(0x7f, 0x63, 0x15),
            | Theme::Dark => Color(0x99, 0x99, 0x99),
        }
    }

    fn text_on_accent(&self) -> Color {
        Color::WHITE
    }

    fn accent(&self) -> Color {
        Color::SUNBEAM_ORANGE
    }

    fn accent_hover(&self) -> Color {
        Color::SUNBEAM_FLAME
    }

    fn section_label(&self) -> Color {
        match self {
            | Theme::Light => Color::SUNBEAM_ORANGE,
            | Theme::Dark => Color::SUNSHINE_700,
        }
    }

    fn border_default(&self) -> Color {
        match self {
            | Theme::Light => Color(0x7f, 0x63, 0x15),
            | Theme::Dark => Color(0x55, 0x55, 0x55),
        }
    }

    fn border_subtle(&self) -> Color {
        match self {
            | Theme::Light => Color(0xdd, 0xcc, 0xaa),
            | Theme::Dark => Color(0x44, 0x44, 0x44),
        }
    }

    fn border_focus(&self) -> Color {
        Color::BEAM_ORANGE
    }

    fn success(&self) -> Color {
        Color(0x22, 0x99, 0x55)
    }

    fn warning(&self) -> Color {
        Color::SUNSHINE_900
    }

    fn error(&self) -> Color {
        Color(0xdd, 0x33, 0x33)
    }

    fn info(&self) -> Color {
        Color(0x33, 0x77, 0xcc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_palette_values() {
        let t = Theme::Light;
        assert_eq!(t.bg_page(), Color::WARM_IVORY);
        assert_eq!(t.text_primary(), Color::SUNBEAM_BLACK);
        assert_eq!(t.accent(), Color::SUNBEAM_ORANGE);
    }

    #[test]
    fn dark_palette_values() {
        let t = Theme::Dark;
        assert_eq!(t.bg_page(), Color::SUNBEAM_BLACK);
        assert_eq!(t.text_primary(), Color::WHITE);
        assert_eq!(t.accent(), Color::SUNBEAM_ORANGE);
    }

    #[test]
    fn theme_switching() {
        Theme::set(Theme::Light);
        assert_eq!(Theme::current(), Theme::Light);

        Theme::set(Theme::Dark);
        assert_eq!(Theme::current(), Theme::Dark);

        Theme::set(Theme::Light); // restore
    }

    #[test]
    fn theme_with_restores_previous() {
        Theme::set(Theme::Light);
        let result = Theme::with(Theme::Dark, || {
            assert_eq!(Theme::current(), Theme::Dark);
            42
        });
        assert_eq!(result, 42);
        assert_eq!(Theme::current(), Theme::Light);
    }
}
