//! Border drawing primitives for terminal UIs.
//!
//! Provides [`Border`] — a set of characters for drawing rectangular
//! outlines — and [`draw_border`] for rendering them into a [`Rendered`]
//! buffer with ANSI styling.

use crate::layout::Rect;
use crate::renderer::Rendered;
use crate::theme::{ColorMode, Style};

/// Characters used to draw a rectangular border.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Border {
    pub left: char,
    pub right: char,
    pub top: char,
    pub bottom: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
}

impl Border {
    /// Single-line box-drawing border (┌─┐││└─┘).
    pub const THIN: Self = Self {
        left: '│',
        right: '│',
        top: '─',
        bottom: '─',
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
    };

    /// Rounded box-drawing border (╭─╮││╰─╯).
    pub const ROUNDED: Self = Self {
        left: '│',
        right: '│',
        top: '─',
        bottom: '─',
        top_left: '╭',
        top_right: '╮',
        bottom_left: '╰',
        bottom_right: '╯',
    };

    /// Thick/heavy box-drawing border (┏━┓┃┃┗━┛).
    pub const THICK: Self = Self {
        left: '┃',
        right: '┃',
        top: '━',
        bottom: '━',
        top_left: '┏',
        top_right: '┓',
        bottom_left: '┗',
        bottom_right: '┛',
    };

    /// Double-line box-drawing border (╔═╗║║╚═╝).
    pub const DOUBLE: Self = Self {
        left: '║',
        right: '║',
        top: '═',
        bottom: '═',
        top_left: '╔',
        top_right: '╗',
        bottom_left: '╚',
        bottom_right: '╝',
    };

    /// No border — useful as a default.
    pub const NONE: Self = Self {
        left: ' ',
        right: ' ',
        top: ' ',
        bottom: ' ',
        top_left: ' ',
        top_right: ' ',
        bottom_left: ' ',
        bottom_right: ' ',
    };

    /// A left-only border (▐) — useful for callouts.
    pub const LEFT: Self = Self {
        left: '▐',
        right: ' ',
        top: ' ',
        bottom: ' ',
        top_left: '▐',
        top_right: ' ',
        bottom_left: '▐',
        bottom_right: ' ',
    };

    /// How many columns the border consumes on each axis.
    ///
    /// For a full border this is (2, 2); for a left-only border it's (1, 0).
    pub fn size(&self) -> (u16, u16) {
        let h = if self.left != ' ' || self.right != ' ' { 1 } else { 0 };
        let v = if self.top != ' ' || self.bottom != ' ' { 1 } else { 0 };
        (h, v)
    }

    /// The inner rect after removing border padding.
    pub fn inner(&self, rect: Rect) -> Rect {
        let (h, v) = self.size();
        Rect::new(
            rect.x + h,
            rect.y + v,
            rect.width.saturating_sub(h * 2),
            rect.height.saturating_sub(v * 2),
        )
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::THIN
    }
}

/// Draw a border into `target` within the given `rect`.
///
/// The border characters are styled with `style`. Existing content in
/// `target` is preserved; border characters overwrite the edges of the
/// rect.
///
/// # Panics
///
/// Panics if `rect` extends beyond `target`'s current dimensions.
pub fn draw_border(target: &mut Rendered, rect: Rect, border: &Border, style: &Style) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let mode = ColorMode::detect();
    let prefix = style.prefix(mode);
    let suffix = Style::suffix();

    let needed_rows = rect.y as usize + rect.height as usize;
    while target.lines.len() < needed_rows {
        target.lines.push(String::new());
    }

    let top = rect.y as usize;
    let bottom = (rect.y + rect.height - 1) as usize;
    let inner_width = rect.width.saturating_sub(2) as usize;
    let interior = " ".repeat(inner_width);

    // Helper: build a horizontal edge (top or bottom)
    let build_edge = |l: char, edge: char, r: char| -> String {
        let mut line = String::new();
        if l != ' ' {
            line.push_str(&prefix);
            line.push(l);
            line.push_str(suffix);
        }
        if edge != ' ' && inner_width > 0 {
            line.push_str(&prefix);
            for _ in 0..inner_width {
                line.push(edge);
            }
            line.push_str(suffix);
        } else if inner_width > 0 {
            line.push_str(&interior);
        }
        if r != ' ' && rect.width > 1 {
            line.push_str(&prefix);
            line.push(r);
            line.push_str(suffix);
        }
        line
    };

    // Helper: build a middle row with left/right borders only
    let build_side = |l: char, r: char| -> String {
        let mut line = String::new();
        if l != ' ' {
            line.push_str(&prefix);
            line.push(l);
            line.push_str(suffix);
        }
        if inner_width > 0 {
            line.push_str(&interior);
        }
        if r != ' ' && rect.width > 1 {
            line.push_str(&prefix);
            line.push(r);
            line.push_str(suffix);
        }
        line
    };

    // Top edge
    if rect.height >= 1 {
        target.lines[top] = build_edge(border.top_left, border.top, border.top_right);
    }

    // Middle rows
    for y in (top + 1)..bottom {
        target.lines[y] = build_side(border.left, border.right);
    }

    // Bottom edge
    if rect.height >= 2 {
        target.lines[bottom] = build_edge(border.bottom_left, border.bottom, border.bottom_right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, Theme};

    fn empty_rendered(width: u16, height: u16) -> Rendered {
        Rendered {
            lines: vec![String::new(); height as usize],
            cursor: None,
            images: Vec::new(),
        }
    }

    #[test]
    fn thin_border_3x3() {
        Theme::with(Theme::Light, || {
            let mut target = empty_rendered(3, 3);
            let rect = Rect::new(0, 0, 3, 3);
            let style = Style::new().fg(Color::SUNBEAM_ORANGE);
            draw_border(&mut target, rect, &Border::THIN, &style);

            assert!(target.lines[0].contains('┌'));
            assert!(target.lines[0].contains('┐'));
            assert!(target.lines[1].contains('│'));
            assert!(target.lines[2].contains('└'));
            assert!(target.lines[2].contains('┘'));
        });
    }

    #[test]
    fn thin_border_5x3_with_title_space() {
        Theme::with(Theme::Light, || {
            let mut target = empty_rendered(5, 3);
            let rect = Rect::new(0, 0, 5, 3);
            let style = Style::new().fg(Color::WHITE);
            draw_border(&mut target, rect, &Border::THIN, &style);

            // Top: ┌───┐
            assert!(target.lines[0].contains('┌'));
            assert!(target.lines[0].contains('┐'));
            // Middle: │   │
            assert!(target.lines[1].contains('│'));
            // Bottom: └───┘
            assert!(target.lines[2].contains('└'));
            assert!(target.lines[2].contains('┘'));
        });
    }

    #[test]
    fn border_includes_ansi_codes() {
        Theme::with(Theme::Light, || {
            let mut target = empty_rendered(3, 3);
            let rect = Rect::new(0, 0, 3, 3);
            let style = Style::new().fg(Color::SUNBEAM_ORANGE);
            draw_border(&mut target, rect, &Border::THIN, &style);

            // Border chars should be wrapped in ANSI codes
            assert!(target.lines[0].starts_with('\x1b'));
            assert!(target.lines[0].contains("\x1b[0m"));
        });
    }

    #[test]
    fn left_border_only() {
        Theme::with(Theme::Light, || {
            let mut target = empty_rendered(3, 3);
            let rect = Rect::new(0, 0, 3, 3);
            let style = Style::new().fg(Color::SUNBEAM_ORANGE);
            draw_border(&mut target, rect, &Border::LEFT, &style);

            assert!(target.lines[0].contains('▐'));
            assert!(target.lines[1].contains('▐'));
            assert!(target.lines[2].contains('▐'));
            // No right border
            assert!(!target.lines[1].contains('│'));
        });
    }

    #[test]
    fn border_inner_rect() {
        let border = Border::THIN;
        let outer = Rect::new(0, 0, 10, 5);
        let inner = border.inner(outer);
        assert_eq!(inner.x, 1);
        assert_eq!(inner.y, 1);
        assert_eq!(inner.width, 8);
        assert_eq!(inner.height, 3);
    }

    #[test]
    fn empty_border_inner_is_full() {
        let border = Border::NONE;
        let outer = Rect::new(0, 0, 10, 5);
        let inner = border.inner(outer);
        assert_eq!(inner, outer);
    }
}
