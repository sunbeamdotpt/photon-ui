//! The Beam Design Language theme system for photon-ui.
//!
//! Provides color palettes, semantic tokens, ANSI rendering, and
//! composable text styles for terminal UIs.
//!
//! # Quick start
//!
//! ```
//! use photon_ui::theme::{Theme, Palette, stylize, Style};
//!
//! Theme::set(Theme::Light);
//! let accent = Theme::current().accent();
//! let styled = stylize("Hello", &Style::new().fg(accent).bold());
//! ```
//!
//! # Switching themes on demand
//!
//! ```
//! use photon_ui::theme::{Theme, Palette};
//!
//! Theme::set(Theme::Dark);
//! assert_eq!(Theme::current().bg_page().to_hex(), "#1f1f1f");
//!
//! Theme::set(Theme::Light);
//! assert_eq!(Theme::current().bg_page().to_hex(), "#fffaed");
//! ```

pub mod ansi;
pub mod color;
pub mod palette;
pub mod style;

pub use ansi::{fg, bg, ColorMode, RESET};
pub use color::Color;
pub use palette::{Palette, Theme};
pub use style::{stylize, stylize_padded, Style};
