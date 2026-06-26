//! The Beam Design Language theme system for photon-ui.
//!
//! Provides color palettes, semantic tokens, ANSI rendering, and
//! composable text styles for terminal UIs.
//!
//! # Quick start
//!
//! ```
//! use photon_ui::theme::{
//!     Palette,
//!     Style,
//!     Theme,
//!     stylize,
//! };
//!
//! Theme::set(Theme::Light);
//! let accent = Theme::palette().accent();
//! let styled = stylize("Hello", &Style::new().fg(accent).bold());
//! ```
//!
//! # Switching themes on demand
//!
//! ```
//! use photon_ui::theme::{
//!     Palette,
//!     Theme,
//! };
//!
//! Theme::set(Theme::Dark);
//! assert_eq!(Theme::palette().background().to_hex(), "#1f1f1f");
//!
//! Theme::set(Theme::Light);
//! assert_eq!(Theme::palette().background().to_hex(), "#fffaed");
//! ```
//!
//! # Custom palettes
//!
//! Implement [`Palette`] and install it globally with [`Theme::set_palette`].
//! While installed, components resolve colors through the custom palette
//! instead of the built-in Light/Dark themes.
//!
//! ```
//! use std::sync::Arc;
//!
//! use photon_ui::theme::{
//!     Color,
//!     Palette,
//!     PaletteHandle,
//!     Theme,
//! };
//!
//! #[derive(Debug)]
//! struct NeonPalette;
//!
//! impl Palette for NeonPalette {
//!     fn background(&self) -> Color {
//!         Color::from_hex("#0a0a0a").unwrap()
//!     }
//!
//!     fn surface(&self) -> Color {
//!         Color::from_hex("#141414").unwrap()
//!     }
//!
//!     fn field(&self) -> Color {
//!         Color::from_hex("#1e1e1e").unwrap()
//!     }
//!
//!     fn text(&self) -> Color {
//!         Color::from_hex("#f0f0f0").unwrap()
//!     }
//!
//!     fn text_muted(&self) -> Color {
//!         Color::from_hex("#888888").unwrap()
//!     }
//!
//!     fn text_on_accent(&self) -> Color {
//!         Color::from_hex("#ffffff").unwrap()
//!     }
//!
//!     fn accent(&self) -> Color {
//!         Color::from_hex("#ff00ff").unwrap()
//!     }
//!
//!     fn accent_hover(&self) -> Color {
//!         Color::from_hex("#ff44ff").unwrap()
//!     }
//!
//!     fn border(&self) -> Color {
//!         Color::from_hex("#333333").unwrap()
//!     }
//!
//!     fn border_muted(&self) -> Color {
//!         Color::from_hex("#222222").unwrap()
//!     }
//!
//!     fn focus(&self) -> Color {
//!         Color::from_hex("#ff00ff").unwrap()
//!     }
//!
//!     fn success(&self) -> Color {
//!         Color::from_hex("#00ff00").unwrap()
//!     }
//!
//!     fn warning(&self) -> Color {
//!         Color::from_hex("#ffff00").unwrap()
//!     }
//!
//!     fn error(&self) -> Color {
//!         Color::from_hex("#ff0000").unwrap()
//!     }
//!
//!     fn info(&self) -> Color {
//!         Color::from_hex("#00ffff").unwrap()
//!     }
//! }
//!
//! Theme::set_palette(Arc::new(NeonPalette));
//! assert_eq!(Theme::palette().accent().to_hex(), "#ff00ff");
//! Theme::clear_palette();
//! ```

pub mod ansi;
pub mod color;
pub mod palette;
pub mod style;

pub use ansi::{
    ColorMode,
    RESET,
    bg,
    fg,
};
pub use color::Color;
pub use palette::{
    Palette,
    PaletteHandle,
    Theme,
};
pub use style::{
    Style,
    stylize,
    stylize_padded,
};
