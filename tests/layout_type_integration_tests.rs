use photon_ui::layout::{Margin, Offset, Position, Rect};
use proptest::prelude::*;

proptest! {
    #[test]
    fn rect_intersection_commutative(a in rect_strategy(), b in rect_strategy()) {
        prop_assert_eq!(a.intersection(b), b.intersection(a));
    }

    #[test]
    fn rect_intersects_symmetric(a in rect_strategy(), b in rect_strategy()) {
        prop_assert_eq!(a.intersects(b), b.intersects(a));
    }

    #[test]
    fn rect_union_contains_both(a in non_empty_rect_strategy(), b in non_empty_rect_strategy()) {
        let u = a.union(b);
        prop_assert!(u.intersects(a));
        prop_assert!(u.intersects(b));
    }

    #[test]
    fn rect_clamp_fits_inside(other in rect_strategy()) {
        let fixed = Rect::new(10, 10, 50, 50);
        let clamped = fixed.clamp(other);
        prop_assert!(clamped.x >= other.x);
        prop_assert!(clamped.y >= other.y);
        prop_assert!(clamped.right() <= other.right());
        prop_assert!(clamped.bottom() <= other.bottom());
    }

    #[test]
    fn rect_inner_area_less_or_equal(r in rect_strategy(), m in margin_strategy()) {
        let inner = r.inner(m);
        prop_assert!(inner.area() <= r.area());
    }

    #[test]
    fn position_add_sub_offset_roundtrip(
        p in position_strategy(),
        o in offset_strategy()
    ) {
        let result = (p + o) - o;
        // Due to clamping, roundtrip only holds when no overflow occurred
        let no_overflow_x = i32::from(p.x) + i32::from(o.x) >= 0
            && i32::from(p.x) + i32::from(o.x) <= i32::from(u16::MAX);
        let no_overflow_y = i32::from(p.y) + i32::from(o.y) >= 0
            && i32::from(p.y) + i32::from(o.y) <= i32::from(u16::MAX);
        if no_overflow_x && no_overflow_y {
            prop_assert_eq!(result, p);
        }
    }

    #[test]
    fn rect_positions_count_equals_area(r in non_empty_rect_strategy()) {
        prop_assert_eq!(r.positions().count() as u32, r.area());
    }

    #[test]
    fn rect_rows_count_equals_height(r in non_empty_rect_strategy()) {
        prop_assert_eq!(r.rows().count() as u16, r.height);
    }

    #[test]
    fn rect_columns_count_equals_width(r in non_empty_rect_strategy()) {
        prop_assert_eq!(r.columns().count() as u16, r.width);
    }
}

fn rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..200, 0u16..200, 0u16..100, 0u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn non_empty_rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..200, 0u16..200, 1u16..100, 1u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn margin_strategy() -> impl Strategy<Value = Margin> {
    (0u16..20, 0u16..20).prop_map(|(h, v)| Margin::new(h, v))
}

fn position_strategy() -> impl Strategy<Value = Position> {
    (0u16..300, 0u16..300).prop_map(|(x, y)| Position::new(x, y))
}

fn offset_strategy() -> impl Strategy<Value = Offset> {
    (-100i16..100, -100i16..100).prop_map(|(x, y)| Offset::new(x, y))
}
