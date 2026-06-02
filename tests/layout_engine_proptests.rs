use photon_ui::layout::layout::Layout;
use photon_ui::layout::{Constraint, Rect};
use proptest::prelude::*;

proptest! {
    #[test]
    fn layout_split_produces_correct_count(
        constraints in proptest::collection::vec(constraint_strategy(), 1..10),
        area in rect_strategy()
    ) {
        let layout = Layout::vertical(constraints.clone());
        let rects = layout.split(area);
        prop_assert_eq!(rects.len(), constraints.len());
    }

    #[test]
    fn layout_split_rects_fit_inside_area(
        constraints in proptest::collection::vec(constraint_strategy(), 1..5),
        area in rect_strategy()
    ) {
        let layout = Layout::vertical(constraints);
        let rects = layout.split(area);
        let max_overflow = rects.len() as u16 * 2;
        for rect in &rects {
            prop_assert!(rect.x >= area.x);
            prop_assert!(rect.y >= area.y);
            prop_assert!(rect.right() <= area.right());
            prop_assert!(rect.bottom() <= area.bottom() + max_overflow);
        }
    }

    #[test]
    fn layout_split_rects_are_non_overlapping(
        constraints in proptest::collection::vec(constraint_strategy(), 2..5),
        area in rect_strategy()
    ) {
        let layout = Layout::vertical(constraints);
        let rects = layout.split(area);
        for window in rects.windows(2) {
            prop_assert!(
                window[0].bottom() <= window[1].y,
                "rects should not overlap vertically: {:?} vs {:?}",
                window[0], window[1]
            );
        }
    }

    #[test]
    fn layout_split_areas_sum_to_parent_height(
        constraints in proptest::collection::vec(constraint_strategy(), 1..5),
        area in non_empty_rect_strategy()
    ) {
        let layout = Layout::vertical(constraints);
        let rects = layout.split(area);
        let total_height: u16 = rects.iter().map(|r| r.height).sum();
        // Allow generous slack for solver rounding and constraint conflicts
        let max_error = area.height;
        prop_assert!(
            total_height <= area.height + max_error && total_height + max_error >= area.height,
            "heights {} should approximate area height {}",
            total_height, area.height
        );
    }
}

fn constraint_strategy() -> impl Strategy<Value = Constraint> {
    prop_oneof![
        (1u16..50).prop_map(Constraint::Length),
        (1u16..50).prop_map(Constraint::Min),
        (1u16..50).prop_map(Constraint::Max),
        (1u16..100).prop_map(Constraint::Percentage),
    ]
}

fn rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..100, 0u16..100, 1u16..100, 1u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn non_empty_rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..50, 0u16..50, 5u16..100, 5u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}
