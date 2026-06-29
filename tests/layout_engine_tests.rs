use std::{
    collections::hash_map::DefaultHasher,
    hash::{
        Hash,
        Hasher,
    },
};

use photon_ui::layout::{
    Constraint,
    Direction,
    Flex,
    Layout,
    Rect,
    Spacing,
};

#[test]
fn layout_split_two_equal_vertical() {
    let layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[0].width, 10);
    assert!((rects[0].height as i16 - 5).abs() <= 1);
    assert!((rects[1].height as i16 - 5).abs() <= 1);
}

#[test]
fn layout_split_three_horizontal() {
    let layout = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ]);
    let rects = layout.split(Rect::new(0, 0, 30, 5));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].width, 10);
    assert_eq!(rects[1].width, 10);
    assert_eq!(rects[2].width, 10);
}

#[test]
fn layout_split_with_margin_reduces_size() {
    let layout = Layout::vertical([Constraint::Length(5)]).margin(2);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects[0].width, 6); // 10 - 2*2
}

#[test]
fn layout_split_min_max_combined() {
    let layout = Layout::vertical([Constraint::Min(2), Constraint::Max(8)]);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert!(rects[0].height >= 2);
    assert!(rects[1].height <= 8);
}

#[test]
fn layout_split_ratio_one_to_three() {
    let layout = Layout::vertical([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)]);
    let rects = layout.split(Rect::new(0, 0, 10, 20));
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].height + rects[1].height, 20);
}

#[test]
fn layout_builder_chaining() {
    let layout = Layout::horizontal([Constraint::Length(5)])
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .margin(1)
        .flex(Flex::Center)
        .spacing(1);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects.len(), 2);
}

#[test]
fn layout_areas_const_generic() {
    let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
    let areas: [Rect; 2] = layout.areas(Rect::new(0, 0, 10, 10));
    assert_eq!(areas[0].height, 5);
    assert_eq!(areas[1].height, 5);
}

#[test]
fn layout_split_empty_returns_zeros() {
    let layout = Layout::vertical([Constraint::Length(5)]);
    let rects = layout.split(Rect::ZERO);
    assert_eq!(rects[0], Rect::ZERO);
}

#[test]
fn layout_flex_start_leaves_trailing_space() {
    let layout = Layout::vertical([Constraint::Length(3)]).flex(Flex::Start);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects[0].height, 3);
}

#[test]
fn layout_flex_space_between_no_edge_space() {
    let layout =
        Layout::horizontal([Constraint::Length(3), Constraint::Length(3)]).flex(Flex::SpaceBetween);
    let rects = layout.split(Rect::new(0, 0, 10, 1));
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].x, 0);
}

/// Regression: Min segments must expand to fill available space when
/// surrounded by fixed Length segments.
#[test]
fn layout_min_expands_to_fill() {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Length(4),
    ]);
    let rects = layout.split(Rect::new(0, 0, 80, 40));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].height, 4, "top Length(4) should be exactly 4");
    assert_eq!(
        rects[1].height, 32,
        "middle Min(3) should expand to fill remaining 32"
    );
    assert_eq!(rects[2].height, 4, "bottom Length(4) should be exactly 4");
    assert_eq!(rects[0].y, 0);
    assert_eq!(rects[1].y, 4);
    assert_eq!(rects[2].y, 36);
}

/// Regression: Min segments should stay at minimum when space is tight.
#[test]
fn layout_min_stays_at_minimum_when_constrained() {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Length(4),
    ]);
    let rects = layout.split(Rect::new(0, 0, 80, 11));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].height, 4);
    assert_eq!(
        rects[1].height, 3,
        "middle should be exactly Min(3) when no extra space"
    );
    assert_eq!(rects[2].height, 4);
}

/// Regression: Fill segments must expand to fill available space.
#[test]
fn layout_fill_expands_to_fill() {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(4),
    ]);
    let rects = layout.split(Rect::new(0, 0, 80, 40));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].height, 4);
    assert_eq!(
        rects[1].height, 32,
        "Fill(1) should expand to fill remaining 32"
    );
    assert_eq!(rects[2].height, 4);
}

/// Regression: Min-only layout should fill the entire area.
#[test]
fn layout_min_only_fills_area() {
    let layout = Layout::vertical([Constraint::Min(3)]);
    let rects = layout.split(Rect::new(0, 0, 80, 40));
    assert_eq!(rects.len(), 1);
    assert_eq!(
        rects[0].height, 40,
        "single Min(3) should fill entire 40-row area"
    );
}

#[test]
fn layout_horizontal_margin_builder() {
    let layout = Layout::vertical([Constraint::Length(5)])
        .horizontal_margin(2)
        .vertical_margin(1);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects[0].x, 2);
    assert_eq!(rects[0].y, 1);
    assert_eq!(rects[0].width, 6);
}

#[test]
fn layout_vertical_margin_builder() {
    let layout = Layout::horizontal([Constraint::Length(5)])
        .vertical_margin(2)
        .horizontal_margin(1);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects[0].x, 1);
    assert_eq!(rects[0].y, 2);
    assert_eq!(rects[0].height, 6);
}

#[test]
fn layout_hash_equal_layouts_match() {
    fn hash_layout(layout: &Layout) -> u64 {
        let mut hasher = DefaultHasher::new();
        layout.hash(&mut hasher);
        hasher.finish()
    }

    let a = Layout::vertical([Constraint::Length(5), Constraint::Length(5)])
        .margin(1)
        .flex(Flex::Center)
        .spacing(1);
    let b = Layout::vertical([Constraint::Length(5), Constraint::Length(5)])
        .margin(1)
        .flex(Flex::Center)
        .spacing(1);
    assert_eq!(hash_layout(&a), hash_layout(&b));
}

#[test]
fn layout_hash_different_layouts_differ() {
    fn hash_layout(layout: &Layout) -> u64 {
        let mut hasher = DefaultHasher::new();
        layout.hash(&mut hasher);
        hasher.finish()
    }

    let a = Layout::vertical([Constraint::Length(5)]);
    let b = Layout::horizontal([Constraint::Length(5)]);
    assert_ne!(hash_layout(&a), hash_layout(&b));
}

#[test]
fn layout_areas_const_generic_fills_zeros() {
    let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
    let areas: [Rect; 3] = layout.areas(Rect::new(0, 0, 10, 10));
    assert_eq!(areas[0].height, 5);
    assert_eq!(areas[1].height, 5);
    assert_eq!(areas[2], Rect::ZERO);
}

#[test]
fn layout_flex_legacy_excess_goes_to_last() {
    let layout = Layout::horizontal([Constraint::Length(3)]).flex(Flex::Legacy);
    let rects = layout.split(Rect::new(0, 0, 10, 1));
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[0].width, 10);
}

#[test]
fn layout_flex_end_aligns_to_end() {
    let layout = Layout::horizontal([Constraint::Length(3), Constraint::Length(3)])
        .flex(Flex::End)
        .spacing(0);
    let rects = layout.split(Rect::new(0, 0, 10, 1));
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[1].x + rects[1].width, 10);
    assert!(rects[0].x > 0);
}

#[test]
fn layout_flex_space_between_with_three_segments() {
    let layout = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .flex(Flex::SpaceBetween)
    .spacing(0);
    let rects = layout.split(Rect::new(0, 0, 12, 1));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[1].x, 5);
    assert_eq!(rects[2].x, 10);
}

#[test]
fn layout_flex_space_around() {
    let layout = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .flex(Flex::SpaceAround)
    .spacing(0);
    let rects = layout.split(Rect::new(0, 0, 12, 1));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].x, 1);
    assert_eq!(rects[1].x, 5);
    assert_eq!(rects[2].x, 9);
}

#[test]
fn layout_flex_space_evenly() {
    let layout = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .flex(Flex::SpaceEvenly)
    .spacing(0);
    let rects = layout.split(Rect::new(0, 0, 14, 1));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].x, 2);
    assert_eq!(rects[1].x, 6);
    assert_eq!(rects[2].x, 10);
}

#[test]
fn layout_spacing_overlap() {
    let layout = Layout::horizontal([Constraint::Length(3), Constraint::Length(3)])
        .spacing(Spacing::Overlap(2));
    let rects = layout.split(Rect::new(0, 0, 10, 1));
    assert_eq!(rects.len(), 2);
}
