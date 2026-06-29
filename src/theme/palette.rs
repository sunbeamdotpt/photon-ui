//! Semantic color tokens for the Beam Design Language.
//!
//! Provides a [`Palette`] trait that maps semantic names (e.g. `bg.page`,
//! `text.primary`) to concrete [`Color`] values. The [`Theme`] enum
//! implements this trait for the built-in Light and Dark variants.
//!
//! A custom palette can be installed globally with [`Theme::set_palette`];
//! when present it takes precedence over the built-in Light/Dark colors.

use std::{
    cell::RefCell,
    sync::Arc,
};

use super::Color;

// ── Thread-local active theme ─────────────────────────────────────

thread_local! {
    static ACTIVE_THEME: RefCell<Theme> = const { RefCell::new(Theme::Dark) };
    static ACTIVE_PALETTE: RefCell<Option<PaletteHandle>> = const { RefCell::new(None) };
}

/// A handle to a [`Palette`], used to install custom themes.
pub type PaletteHandle = Arc<dyn Palette>;

/// The active theme variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Theme {
    /// Light variant of the Beam Design Language.
    Light,
    /// Dark variant of the Beam Design Language (default).
    #[default]
    Dark,
}

impl Theme {
    /// Set the globally active built-in theme.
    pub fn set(theme: Theme) {
        ACTIVE_THEME.with(|t| *t.borrow_mut() = theme);
    }

    /// Get the globally active built-in theme.
    pub fn current() -> Self {
        ACTIVE_THEME.with(|t| *t.borrow())
    }

    /// Install a custom palette globally.
    ///
    /// The custom palette takes precedence over the built-in Light/Dark theme
    /// until [`Theme::clear_palette`] is called.
    pub fn set_palette(palette: PaletteHandle) {
        ACTIVE_PALETTE.with(|p| *p.borrow_mut() = Some(palette));
    }

    /// Remove the globally active custom palette.
    pub fn clear_palette() {
        ACTIVE_PALETTE.with(|p| *p.borrow_mut() = None);
    }

    /// Returns true when a custom palette is currently installed.
    pub fn has_palette() -> bool {
        ACTIVE_PALETTE.with(|p| p.borrow().is_some())
    }

    /// Get the active palette.
    ///
    /// Returns the custom palette if one is installed, otherwise returns a
    /// handle to the current built-in theme.
    pub fn palette() -> PaletteHandle {
        ACTIVE_PALETTE.with(|p| match p.borrow().as_ref() {
            | Some(handle) => Arc::clone(handle),
            | None => Arc::new(Self::current()),
        })
    }

    /// Run a closure with a specific built-in theme, then restore the
    /// previous theme and any previously installed custom palette.
    ///
    /// The closure runs with the requested built-in theme and no custom
    /// palette; the previous state is restored before returning.
    pub fn with<T>(theme: Theme, f: impl FnOnce() -> T) -> T {
        let previous_theme = Self::current();
        let previous_palette = Self::palette_option();
        Self::set(theme);
        Self::clear_palette();
        let result = f();
        Self::set(previous_theme);
        match previous_palette {
            | Some(palette) => Self::set_palette(palette),
            | None => Self::clear_palette(),
        }
        result
    }

    fn palette_option() -> Option<PaletteHandle> {
        ACTIVE_PALETTE.with(|p| p.borrow().as_ref().map(Arc::clone))
    }
}

// ── Palette trait ─────────────────────────────────────────────────

/// A semantic color palette for terminal user interfaces.
///
/// Implement this trait to define a custom theme. Each method maps a
/// well-known UI role to a concrete [`Color`]. When a custom palette is
/// installed with [`Theme::set_palette`], components resolve colors through
/// these roles instead of the built-in Light/Dark colors.
pub trait Palette {
    /// Page / canvas background.
    fn background(&self) -> Color;
    /// Elevated surface background (cards, panels, navigation).
    fn surface(&self) -> Color;
    /// Input field background.
    fn field(&self) -> Color;

    /// Primary text color.
    fn text(&self) -> Color;
    /// Muted / secondary text color.
    fn text_muted(&self) -> Color;
    /// Text rendered on top of the accent color.
    fn text_on_accent(&self) -> Color;

    /// Accent / brand color.
    fn accent(&self) -> Color;
    /// Accent color when hovered.
    fn accent_hover(&self) -> Color;
    /// Rendered block cursor / caret colour in editable fields.
    ///
    /// Defaults to the accent colour so existing custom palettes do not need to
    /// be updated, but can be overridden to configure the cursor independently.
    fn cursor(&self) -> Color {
        self.accent()
    }

    /// Default border color.
    fn border(&self) -> Color;
    /// Subtle border / divider color.
    fn border_muted(&self) -> Color;
    /// Focus ring / focused border color.
    fn focus(&self) -> Color;

    /// Success / positive state color.
    fn success(&self) -> Color;
    /// Warning / caution state color.
    fn warning(&self) -> Color;
    /// Error / negative state color.
    fn error(&self) -> Color;
    /// Informational state color.
    fn info(&self) -> Color;
}

impl Palette for Theme {
    fn background(&self) -> Color {
        match self {
            | Theme::Light => Color::WARM_IVORY,
            | Theme::Dark => Color::SUNBEAM_BLACK,
        }
    }

    fn surface(&self) -> Color {
        match self {
            | Theme::Light => Color::CREAM,
            | Theme::Dark => Color::CARD_DARK,
        }
    }

    fn field(&self) -> Color {
        match self {
            | Theme::Light => Color::WHITE,
            | Theme::Dark => Color::CARD_DARK,
        }
    }

    fn text(&self) -> Color {
        match self {
            | Theme::Light => Color::SUNBEAM_BLACK,
            | Theme::Dark => Color::WHITE,
        }
    }

    fn text_muted(&self) -> Color {
        match self {
            | Theme::Light => Color(0x66, 0x66, 0x66),
            | Theme::Dark => Color(0xbb, 0xbb, 0xbb),
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

    fn border(&self) -> Color {
        match self {
            | Theme::Light => Color(0x7f, 0x63, 0x15),
            | Theme::Dark => Color(0x55, 0x55, 0x55),
        }
    }

    fn border_muted(&self) -> Color {
        match self {
            | Theme::Light => Color(0xdd, 0xcc, 0xaa),
            | Theme::Dark => Color(0x44, 0x44, 0x44),
        }
    }

    fn focus(&self) -> Color {
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

    #[derive(Debug)]
    struct MagentaPalette;

    impl Palette for MagentaPalette {
        fn background(&self) -> Color {
            Color::from_hex("#111111").unwrap()
        }

        fn surface(&self) -> Color {
            Color::from_hex("#222222").unwrap()
        }

        fn field(&self) -> Color {
            Color::from_hex("#333333").unwrap()
        }

        fn text(&self) -> Color {
            Color::from_hex("#444444").unwrap()
        }

        fn text_muted(&self) -> Color {
            Color::from_hex("#555555").unwrap()
        }

        fn text_on_accent(&self) -> Color {
            Color::from_hex("#666666").unwrap()
        }

        fn accent(&self) -> Color {
            Color::from_hex("#ff00ff").unwrap()
        }

        fn accent_hover(&self) -> Color {
            Color::from_hex("#ff11ff").unwrap()
        }

        fn border(&self) -> Color {
            Color::from_hex("#777777").unwrap()
        }

        fn border_muted(&self) -> Color {
            Color::from_hex("#888888").unwrap()
        }

        fn focus(&self) -> Color {
            Color::from_hex("#999999").unwrap()
        }

        fn success(&self) -> Color {
            Color::from_hex("#00ff00").unwrap()
        }

        fn warning(&self) -> Color {
            Color::from_hex("#ffff00").unwrap()
        }

        fn error(&self) -> Color {
            Color::from_hex("#ff0000").unwrap()
        }

        fn info(&self) -> Color {
            Color::from_hex("#0000ff").unwrap()
        }
    }

    #[test]
    fn light_palette_values() {
        let t = Theme::Light;
        assert_eq!(t.background(), Color::WARM_IVORY);
        assert_eq!(t.text(), Color::SUNBEAM_BLACK);
        assert_eq!(t.accent(), Color::SUNBEAM_ORANGE);
    }

    #[test]
    fn dark_palette_values() {
        let t = Theme::Dark;
        assert_eq!(t.background(), Color::SUNBEAM_BLACK);
        assert_eq!(t.text(), Color::WHITE);
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

    #[test]
    fn custom_palette_overrides_built_in() {
        Theme::set(Theme::Light);
        Theme::set_palette(Arc::new(MagentaPalette));
        let palette = Theme::palette();
        assert_eq!(palette.accent(), Color::from_hex("#ff00ff").unwrap());
        assert_eq!(palette.background(), Color::from_hex("#111111").unwrap());
        Theme::clear_palette();
    }

    #[test]
    fn clear_palette_reverts_to_theme() {
        Theme::set(Theme::Light);
        Theme::set_palette(Arc::new(MagentaPalette));
        Theme::clear_palette();
        let palette = Theme::palette();
        assert_eq!(palette.background(), Color::WARM_IVORY);
        assert_eq!(palette.accent(), Color::SUNBEAM_ORANGE);
    }

    #[test]
    fn with_clears_and_restores_custom_palette() {
        Theme::set(Theme::Light);
        Theme::set_palette(Arc::new(MagentaPalette));
        Theme::with(Theme::Dark, || {
            assert!(!Theme::has_palette());
            assert_eq!(Theme::palette().accent(), Color::SUNBEAM_ORANGE);
        });
        assert!(Theme::has_palette());
        assert_eq!(
            Theme::palette().accent(),
            Color::from_hex("#ff00ff").unwrap()
        );
        Theme::clear_palette();
    }

    #[test]
    fn light_palette_exposes_all_roles() {
        let t = Theme::Light;
        assert_eq!(t.background(), Color::WARM_IVORY);
        assert_eq!(t.surface(), Color::CREAM);
        assert_eq!(t.field(), Color::WHITE);
        assert_eq!(t.text(), Color::SUNBEAM_BLACK);
        assert_eq!(t.text_muted(), Color(0x66, 0x66, 0x66));
        assert_eq!(t.text_on_accent(), Color::WHITE);
        assert_eq!(t.accent(), Color::SUNBEAM_ORANGE);
        assert_eq!(t.accent_hover(), Color::SUNBEAM_FLAME);
        assert_eq!(t.cursor(), Color::SUNBEAM_ORANGE);
        assert_eq!(t.border(), Color(0x7f, 0x63, 0x15));
        assert_eq!(t.border_muted(), Color(0xdd, 0xcc, 0xaa));
        assert_eq!(t.focus(), Color::BEAM_ORANGE);
        assert_eq!(t.success(), Color(0x22, 0x99, 0x55));
        assert_eq!(t.warning(), Color::SUNSHINE_900);
        assert_eq!(t.error(), Color(0xdd, 0x33, 0x33));
        assert_eq!(t.info(), Color(0x33, 0x77, 0xcc));
    }

    #[test]
    fn dark_palette_exposes_all_roles() {
        let t = Theme::Dark;
        assert_eq!(t.background(), Color::SUNBEAM_BLACK);
        assert_eq!(t.surface(), Color::CARD_DARK);
        assert_eq!(t.field(), Color::CARD_DARK);
        assert_eq!(t.text(), Color::WHITE);
        assert_eq!(t.text_muted(), Color(0xbb, 0xbb, 0xbb));
        assert_eq!(t.text_on_accent(), Color::WHITE);
        assert_eq!(t.accent(), Color::SUNBEAM_ORANGE);
        assert_eq!(t.accent_hover(), Color::SUNBEAM_FLAME);
        assert_eq!(t.cursor(), Color::SUNBEAM_ORANGE);
        assert_eq!(t.border(), Color(0x55, 0x55, 0x55));
        assert_eq!(t.border_muted(), Color(0x44, 0x44, 0x44));
        assert_eq!(t.focus(), Color::BEAM_ORANGE);
        assert_eq!(t.success(), Color(0x22, 0x99, 0x55));
        assert_eq!(t.warning(), Color::SUNSHINE_900);
        assert_eq!(t.error(), Color(0xdd, 0x33, 0x33));
        assert_eq!(t.info(), Color(0x33, 0x77, 0xcc));
    }

    #[test]
    fn custom_palette_exposes_all_roles() {
        Theme::set(Theme::Light);
        Theme::set_palette(Arc::new(MagentaPalette));
        let p = Theme::palette();
        assert_eq!(p.background(), Color::from_hex("#111111").unwrap());
        assert_eq!(p.surface(), Color::from_hex("#222222").unwrap());
        assert_eq!(p.field(), Color::from_hex("#333333").unwrap());
        assert_eq!(p.text(), Color::from_hex("#444444").unwrap());
        assert_eq!(p.text_muted(), Color::from_hex("#555555").unwrap());
        assert_eq!(p.text_on_accent(), Color::from_hex("#666666").unwrap());
        assert_eq!(p.accent(), Color::from_hex("#ff00ff").unwrap());
        assert_eq!(p.accent_hover(), Color::from_hex("#ff11ff").unwrap());
        assert_eq!(p.cursor(), Color::from_hex("#ff00ff").unwrap());
        assert_eq!(p.border(), Color::from_hex("#777777").unwrap());
        assert_eq!(p.border_muted(), Color::from_hex("#888888").unwrap());
        assert_eq!(p.focus(), Color::from_hex("#999999").unwrap());
        assert_eq!(p.success(), Color::from_hex("#00ff00").unwrap());
        assert_eq!(p.warning(), Color::from_hex("#ffff00").unwrap());
        assert_eq!(p.error(), Color::from_hex("#ff0000").unwrap());
        assert_eq!(p.info(), Color::from_hex("#0000ff").unwrap());
        Theme::clear_palette();
    }
}
