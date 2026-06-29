//! Raw RGB color definitions for the Beam Design Language palette.

/// An RGB color with 8-bit channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    /// Beam gold — `#ffe295`.
    pub const BEAM_GOLD: Self = Self(0xff, 0xe2, 0x95);
    /// Beam orange — `#ff8105`.
    pub const BEAM_ORANGE: Self = Self(0xff, 0x81, 0x05);
    /// Pure black — `#000000`.
    pub const BLACK: Self = Self(0x00, 0x00, 0x00);
    /// Bright yellow — `#ffd900`.
    pub const BRIGHT_YELLOW: Self = Self(0xff, 0xd9, 0x00);
    /// Dark card surface — `#2a2a2a`.
    pub const CARD_DARK: Self = Self(0x2a, 0x2a, 0x2a);
    /// Warm cream — `#fff0c2`.
    pub const CREAM: Self = Self(0xff, 0xf0, 0xc2);
    /// Sunbeam black — `#1f1f1f`.
    pub const SUNBEAM_BLACK: Self = Self(0x1f, 0x1f, 0x1f);
    /// Sunbeam flame — `#fb6424`.
    pub const SUNBEAM_FLAME: Self = Self(0xfb, 0x64, 0x24);
    // ── Primary colors ──────────────────────────────────────────────
    /// Sunbeam orange — `#fa520f`.
    pub const SUNBEAM_ORANGE: Self = Self(0xfa, 0x52, 0x0f);
    /// Sunshine 300 — `#ffd06a`.
    pub const SUNSHINE_300: Self = Self(0xff, 0xd0, 0x6a);
    /// Sunshine 500 — `#ffb83e`.
    pub const SUNSHINE_500: Self = Self(0xff, 0xb8, 0x3e);
    /// Sunshine 700 — `#ffa110`.
    pub const SUNSHINE_700: Self = Self(0xff, 0xa1, 0x10);
    // ── Sunshine scale ──────────────────────────────────────────────
    /// Sunshine 900 — `#ff8a00`.
    pub const SUNSHINE_900: Self = Self(0xff, 0x8a, 0x00);
    // ── Surfaces ────────────────────────────────────────────────────
    /// Warm ivory — `#fffaed`.
    pub const WARM_IVORY: Self = Self(0xff, 0xfa, 0xed);
    // ── Neutrals ────────────────────────────────────────────────────
    /// Pure white — `#ffffff`.
    pub const WHITE: Self = Self(0xff, 0xff, 0xff);

    /// Parse a hex color string (e.g. `"#fa520f"` or `"fa520f"`).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.strip_prefix('#').unwrap_or(hex);
        if s.len() != 6 {
            return None;
        }
        let r = match u8::from_str_radix(&s[0..2], 0x10) {
            | Ok(v) => v,
            | Err(_) => return None,
        };
        let g = match u8::from_str_radix(&s[2..4], 0x10) {
            | Ok(v) => v,
            | Err(_) => return None,
        };
        let b = match u8::from_str_radix(&s[4..6], 0x10) {
            | Ok(v) => v,
            | Err(_) => return None,
        };
        Some(Self(r, g, b))
    }

    /// Return the color as a 6-digit hex string.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_hex() {
        assert_eq!(Color::from_hex("#fa520f"), Some(Color::SUNBEAM_ORANGE));
        assert_eq!(Color::from_hex("fa520f"), Some(Color::SUNBEAM_ORANGE));
        assert_eq!(Color::from_hex("fff"), None);
        assert_eq!(Color::from_hex("#gg0000"), None);
        assert_eq!(Color::from_hex("#00zz00"), None);
        assert_eq!(Color::from_hex("#0000aa1"), None);
        assert_eq!(Color::from_hex("#0000zz"), None);
    }

    #[test]
    fn color_to_hex() {
        assert_eq!(Color::SUNBEAM_ORANGE.to_hex(), "#fa520f");
        assert_eq!(Color::WHITE.to_hex(), "#ffffff");
        assert_eq!(Color::BLACK.to_hex(), "#000000");
    }
}
