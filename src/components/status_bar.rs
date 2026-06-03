//! Status bar component.
//!
//! Renders as a single line with left-aligned, center-aligned, and
//! right-aligned segments. Each zone may contain multiple segments joined by
//! two spaces.

use crate::{
    Component,
    RenderError,
    Rendered,
    utils::{
        truncate_to_width,
        visible_width,
    },
};
use crate::theme::{
    Palette,
    Style,
    Theme,
    stylize,
};

/// A single piece of text in a status bar zone.
#[derive(Clone)]
pub struct Segment {
    text: String,
    style: Style,
}

impl Segment {
    /// Create a new segment with the given text.
    ///
    /// The default style is the theme's secondary text color; override it with
    /// [`styled`](Segment::styled).
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }

    /// Set a custom style for this segment.
    pub fn styled(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// A non-interactive status bar with three alignment zones.
///
/// Segments in the `left` zone are rendered at the left edge, `center`
/// segments are centered, and `right` segments are at the right edge.
/// Multiple segments within the same zone are joined with two spaces.
pub struct StatusBar {
    left: Vec<Segment>,
    center: Vec<Segment>,
    right: Vec<Segment>,
}

impl StatusBar {
    /// Create an empty status bar.
    pub fn new() -> Self {
        Self {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }

    /// Add a segment to the left zone.
    pub fn left(mut self, segment: Segment) -> Self {
        self.left.push(segment);
        self
    }

    /// Add a segment to the center zone.
    pub fn center(mut self, segment: Segment) -> Self {
        self.center.push(segment);
        self
    }

    /// Add a segment to the right zone.
    pub fn right(mut self, segment: Segment) -> Self {
        self.right.push(segment);
        self
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

fn join_zone(segments: &[Segment], default_style: Style) -> String {
    segments
        .iter()
        .map(|s| {
            let style = if s.style == Style::default() {
                default_style
            } else {
                s.style
            };
            stylize(&s.text, &style)
        })
        .collect::<Vec<_>>()
        .join("  ")
}

impl Component for StatusBar {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let default_style = Style::new().fg(theme.text_secondary());

        let left_str = join_zone(&self.left, default_style);
        let center_str = join_zone(&self.center, default_style);
        let right_str = join_zone(&self.right, default_style);

        let width_usize = width as usize;
        let mut left_w = visible_width(&left_str);
        let mut right_w = visible_width(&right_str);
        let mut center_w = visible_width(&center_str);

        let mut left = left_str;
        let mut right = right_str;
        let mut center = center_str;

        // Reserve minimum space for right and center so they remain visible.
        let min_right = if self.right.is_empty() { 0 } else { right_w.min(5) };
        let min_center = if self.center.is_empty() { 0 } else { center_w.min(5) };
        let left_max = width_usize.saturating_sub(min_right + min_center);

        // Cap left to remaining space after reservations.
        if left_w > left_max {
            left = truncate_to_width(&left, left_max as u16, "…");
            left_w = visible_width(&left);
        }

        // Cap right to remaining space after left.
        let avail_right = width_usize.saturating_sub(left_w);
        if right_w > avail_right {
            if avail_right > 0 {
                right = truncate_to_width(&right, avail_right as u16, "…");
                right_w = visible_width(&right);
            } else {
                right = String::new();
                right_w = 0;
            }
        }

        // Center gets whatever is between left and right.
        let middle_start = left_w;
        let middle_end = width_usize.saturating_sub(right_w);
        let avail_center = middle_end.saturating_sub(middle_start);
        if center_w > avail_center {
            if avail_center > 0 {
                center = truncate_to_width(&center, avail_center as u16, "…");
                center_w = visible_width(&center);
            } else {
                center = String::new();
                center_w = 0;
            }
        }

        let mut line = left;

        if center_w > 0 {
            let center_pos = middle_start + (avail_center.saturating_sub(center_w)) / 2;
            let current_w = visible_width(&line);
            if center_pos > current_w {
                line.push_str(&" ".repeat(center_pos - current_w));
            }
            line.push_str(&center);
        }

        if right_w > 0 {
            let right_pos = width_usize - right_w;
            let current_w = visible_width(&line);
            if right_pos > current_w {
                line.push_str(&" ".repeat(right_pos - current_w));
            }
            line.push_str(&right);
        }

        let current_w = visible_width(&line);
        if current_w < width_usize {
            line.push_str(&" ".repeat(width_usize - current_w));
        }

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn new_creates_empty() {
        let bar = StatusBar::new();
        assert!(bar.left.is_empty());
        assert!(bar.center.is_empty());
        assert!(bar.right.is_empty());
    }

    #[test]
    fn default_creates_empty() {
        let bar: StatusBar = Default::default();
        assert!(bar.left.is_empty());
        assert!(bar.center.is_empty());
        assert!(bar.right.is_empty());
    }

    #[test]
    fn renders_empty() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new();
            let rendered = bar.render(20).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert_eq!(visible_width(&rendered.lines[0]), 20);
        });
    }

    #[test]
    fn renders_left_only() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new().left(Segment::new("L1"));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            assert!(line.contains("L1"));
            // L1 should be at the left edge (visual position 0).
            let l1_pos = line.find("L1").unwrap();
            let visual_pos = visible_width(&line[..l1_pos]);
            assert_eq!(visual_pos, 0);
            assert_eq!(visible_width(line), 20);
        });
    }

    #[test]
    fn renders_right_only() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new().right(Segment::new("R1"));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            assert!(line.contains("R1"));
            // R1 should be near the right edge (visual position 18).
            let r1_pos = line.find("R1").unwrap();
            let visual_pos = visible_width(&line[..r1_pos]);
            assert!(visual_pos >= 16, "right segment should be near the edge");
            assert_eq!(visible_width(line), 20);
        });
    }

    #[test]
    fn renders_center_only() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new().center(Segment::new("C1"));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            assert!(line.contains("C1"));
            let c1_pos = line.find("C1").unwrap();
            // "C1" is 2 chars, centered in 20 -> visual pos around 9
            let visual_pos = visible_width(&line[..c1_pos]);
            assert!(visual_pos >= 8 && visual_pos <= 10);
            assert_eq!(visible_width(line), 20);
        });
    }

    #[test]
    fn renders_left_and_right() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new()
                .left(Segment::new("L1"))
                .right(Segment::new("R1"));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            assert!(line.contains("L1"));
            assert!(line.contains("R1"));
            let l1_pos = line.find("L1").unwrap();
            let r1_pos = line.find("R1").unwrap();
            assert!(l1_pos < r1_pos);
            assert_eq!(visible_width(line), 20);
        });
    }

    #[test]
    fn renders_all_zones() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new()
                .left(Segment::new("L1"))
                .center(Segment::new("C1"))
                .right(Segment::new("R1"));
            let rendered = bar.render(30).unwrap();
            let line = &rendered.lines[0];
            assert!(line.contains("L1"));
            assert!(line.contains("C1"));
            assert!(line.contains("R1"));
            let l1_pos = line.find("L1").unwrap();
            let c1_pos = line.find("C1").unwrap();
            let r1_pos = line.find("R1").unwrap();
            assert!(l1_pos < c1_pos);
            assert!(c1_pos < r1_pos);
            assert_eq!(visible_width(line), 30);
        });
    }

    #[test]
    fn default_style_is_secondary() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new().left(Segment::new("x"));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            // Light theme text_secondary is #666666 = 102,102,102
            assert!(line.contains("\x1b[38;2;102;102;102m"));
        });
    }

    #[test]
    fn custom_style_overrides_default() {
        Theme::with(Theme::Light, || {
            let accent_style = Style::new().fg(Theme::current().accent());
            let bar = StatusBar::new().left(Segment::new("x").styled(accent_style));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            // Light theme accent is SUNBEAM_ORANGE (#fa520f = 250,82,15)
            assert!(line.contains("\x1b[38;2;250;82;15m"));
        });
    }

    #[test]
    fn multiple_segments_joined() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new()
                .left(Segment::new("A"))
                .left(Segment::new("B"));
            let rendered = bar.render(20).unwrap();
            let line = &rendered.lines[0];
            // Plain text should contain "A  B" (two spaces between segments)
            // Just search for the two segment texts with sufficient spacing.
            let a_pos = line.find('A').unwrap();
            let b_pos = line.find('B').unwrap();
            assert!(b_pos > a_pos);
            assert!(line[a_pos..b_pos].contains("  "));
        });
    }

    #[test]
    fn truncates_when_too_wide() {
        Theme::with(Theme::Light, || {
            let bar = StatusBar::new()
                .left(Segment::new("VeryLongLeft"))
                .right(Segment::new("VeryLongRight"));
            let rendered = bar.render(15).unwrap();
            let line = &rendered.lines[0];
            assert!(visible_width(line) <= 15);
        });
    }
}
