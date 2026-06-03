use photon_ui::layout::{
    Offset,
    Position,
};

#[test]
fn position_add_offset_roundtrip() {
    let p = Position::new(50, 50);
    let o = Offset::new(10, -5);
    assert_eq!((p + o) - o, p);
}

#[test]
fn position_add_offset_clamps_at_max() {
    let p = Position::MAX;
    let o = Offset::new(1, 1);
    assert_eq!(p + o, Position::MAX);
}

#[test]
fn position_sub_offset_clamps_at_min() {
    let p = Position::ORIGIN;
    let o = Offset::new(1, 1);
    assert_eq!(p - o, Position::ORIGIN);
}

#[test]
fn position_add_large_positive_offset_clamps() {
    let p = Position::new(65000, 65000);
    let o = Offset::new(1000, 1000);
    let result = p + o;
    assert_eq!(result, Position::MAX);
}

#[test]
fn position_sub_large_negative_offset_clamps() {
    let p = Position::new(65000, 65000);
    let o = Offset::new(-1000, -1000);
    let result = p - o;
    assert_eq!(result, Position::MAX);
}

#[test]
fn position_add_sub_assign() {
    let mut p = Position::new(10, 10);
    p += Offset::new(5, 3);
    assert_eq!(p, Position::new(15, 13));
    p -= Offset::new(2, 4);
    assert_eq!(p, Position::new(13, 9));
}

#[test]
fn position_origin_plus_offset() {
    assert_eq!(Position::ORIGIN + Offset::new(5, 7), Position::new(5, 7));
}
