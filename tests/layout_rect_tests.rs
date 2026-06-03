use photon_ui::layout::{
    Margin,
    Offset,
    Position,
    Rect,
    Size,
};

#[test]
fn rect_inner_preserves_area_when_no_margin() {
    let r = Rect::new(0, 0, 10, 10);
    assert_eq!(r.inner(Margin::new(0, 0)), r);
}

#[test]
fn rect_outer_then_inner_roundtrip() {
    let r = Rect::new(10, 10, 20, 20);
    let m = Margin::new(2, 3);
    assert_eq!(r.outer(m).inner(m), r);
}

#[test]
fn rect_union_with_self() {
    let r = Rect::new(1, 2, 3, 4);
    assert_eq!(r.union(r), r);
}

#[test]
fn rect_intersection_with_self() {
    let r = Rect::new(1, 2, 3, 4);
    assert_eq!(r.intersection(r), r);
}

#[test]
fn rect_intersection_commutative() {
    let a = Rect::new(0, 0, 5, 5);
    let b = Rect::new(3, 3, 5, 5);
    assert_eq!(a.intersection(b), b.intersection(a));
}

#[test]
fn rect_contains_corners() {
    let r = Rect::new(0, 0, 3, 3);
    assert!(r.contains(Position::new(0, 0)));
    assert!(r.contains(Position::new(2, 2)));
    assert!(!r.contains(Position::new(3, 3)));
}

#[test]
fn rect_clamp_already_inside() {
    let outer = Rect::new(0, 0, 100, 100);
    let inner = Rect::new(10, 10, 5, 5);
    assert_eq!(inner.clamp(outer), inner);
}

#[test]
fn rect_rows_count_matches_height() {
    let r = Rect::new(0, 0, 5, 3);
    assert_eq!(r.rows().count(), 3);
}

#[test]
fn rect_columns_count_matches_width() {
    let r = Rect::new(0, 0, 4, 6);
    assert_eq!(r.columns().count(), 4);
}

#[test]
fn rect_positions_count_matches_area() {
    let r = Rect::new(0, 0, 3, 4);
    assert_eq!(r.positions().count(), 12);
}

#[test]
fn rect_resize_to_zero() {
    let r = Rect::new(5, 5, 10, 10).resize(Size::ZERO);
    assert_eq!(r, Rect::new(5, 5, 0, 0));
}

#[test]
fn rect_offset_by_zero_is_unchanged() {
    let r = Rect::new(1, 2, 3, 4);
    assert_eq!(r.offset(Offset::new(0, 0)), r);
}

#[test]
fn rect_intersects_edge_case_touching() {
    let a = Rect::new(0, 0, 2, 2);
    let b = Rect::new(2, 0, 2, 2);
    assert!(!a.intersects(b));
}
