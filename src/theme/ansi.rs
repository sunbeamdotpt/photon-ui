//! ANSI escape sequence generation for RGB colors.
//!
//! Supports three rendering modes:
//! - **TrueColor** (24-bit RGB) — default, best quality
//! - **Color256** — xterm 256-color cube fallback
//! - **Basic16** — coarse 16-color fallback

use std::env;

use super::Color;

/// Terminal color support level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// 24-bit RGB (`\x1b[38;2;R;G;Bm`).
    #[default]
    TrueColor,
    /// xterm 256-color palette (`\x1b[38;5;Nm`).
    Color256,
    /// ANSI 16-color (`\x1b[30m`–`\x1b[37m`, `\x1b[90m`–`\x1b[97m`).
    Basic16,
}

impl ColorMode {
    /// Detect the best available color mode from environment variables.
    ///
    /// Checks `PHOTON_COLOR_MODE` first, then `COLORTERM`, then `TERM`.
    pub fn detect() -> Self {
        if let Ok(mode) = env::var("PHOTON_COLOR_MODE") {
            match mode.as_str() {
                | "truecolor" | "24bit" | "rgb" => return ColorMode::TrueColor,
                | "256" | "256color" => return ColorMode::Color256,
                | "16" | "basic" => return ColorMode::Basic16,
                | _ => {},
            }
        }
        if let Ok(ct) = env::var("COLORTERM") {
            if ct == "truecolor" || ct == "24bit" {
                return ColorMode::TrueColor;
            }
        }
        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") {
                return ColorMode::Color256;
            }
        }
        ColorMode::TrueColor
    }
}

/// Generate a foreground ANSI escape sequence for the given color.
pub fn fg(color: Color, mode: ColorMode) -> String {
    match mode {
        | ColorMode::TrueColor => format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2),
        | ColorMode::Color256 => format!("\x1b[38;5;{}m", rgb_to_256(color)),
        | ColorMode::Basic16 => format!("\x1b[{}m", rgb_to_16_fg(color)),
    }
}

/// Generate a background ANSI escape sequence for the given color.
pub fn bg(color: Color, mode: ColorMode) -> String {
    match mode {
        | ColorMode::TrueColor => format!("\x1b[48;2;{};{};{}m", color.0, color.1, color.2),
        | ColorMode::Color256 => format!("\x1b[48;5;{}m", rgb_to_256(color)),
        | ColorMode::Basic16 => format!("\x1b[{}m", rgb_to_16_bg(color)),
    }
}

/// Reset all ANSI attributes.
pub const RESET: &str = "\x1b[0m";

// ── 256-color conversion ──────────────────────────────────────────

fn rgb_to_256(color: Color) -> u8 {
    let Color(r, g, b) = color;

    // Check if grayscale
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + ((r - 8) / 10);
    }

    // 6x6x6 color cube
    let r = closest_cube_level(r);
    let g = closest_cube_level(g);
    let b = closest_cube_level(b);
    16 + 36 * r + 6 * g + b
}

fn closest_cube_level(v: u8) -> u8 {
    // Levels: 0, 95, 135, 175, 215, 255
    if v < 48 {
        0
    } else if v < 115 {
        1
    } else if v < 155 {
        2
    } else if v < 195 {
        3
    } else if v < 235 {
        4
    } else {
        5
    }
}

// ── 16-color conversion ───────────────────────────────────────────

fn rgb_to_16_fg(color: Color) -> u8 {
    30 + rgb_to_16_index(color)
}

fn rgb_to_16_bg(color: Color) -> u8 {
    40 + rgb_to_16_index(color)
}

fn rgb_to_16_index(color: Color) -> u8 {
    let Color(r, g, b) = color;
    let intensity = (r as u16 + g as u16 + b as u16) / 3;

    // Simple nearest-match to ANSI 8 colors + bright variants
    let is_bright = intensity > 128;
    let idx = if r > 128 && g < 128 && b < 128 {
        1 // red
    } else if r < 128 && g > 128 && b < 128 {
        2 // green
    } else if r > 128 && g > 128 && b < 128 {
        3 // yellow
    } else if r < 128 && g < 128 && b > 128 {
        4 // blue
    } else if r > 128 && g < 128 && b > 128 {
        5 // magenta
    } else if r < 128 && g > 128 && b > 128 {
        6 // cyan
    } else {
        0 // black/white
    };

    if is_bright && idx == 0 {
        7 // bright white
    } else if is_bright {
        idx + 60 // bright variant (90-97)
    } else {
        idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truecolor_fg() {
        let s = fg(Color::SUNBEAM_ORANGE, ColorMode::TrueColor);
        assert_eq!(s, "\x1b[38;2;250;82;15m");
    }

    #[test]
    fn truecolor_bg() {
        let s = bg(Color::WARM_IVORY, ColorMode::TrueColor);
        assert_eq!(s, "\x1b[48;2;255;250;237m");
    }

    #[test]
    fn color256_produces_valid_codes() {
        let s = fg(Color::SUNBEAM_ORANGE, ColorMode::Color256);
        assert!(s.starts_with("\x1b[38;5;"));
        assert!(s.ends_with('m'));
    }

    #[test]
    fn basic16_produces_valid_codes() {
        let s = fg(Color::WHITE, ColorMode::Basic16);
        assert!(s.starts_with("\x1b["));
        assert!(s.ends_with('m'));
    }
}
