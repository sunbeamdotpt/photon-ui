use photon_ui::kill_ring::KillRing;

#[test]
fn push_and_yank() {
    let mut ring = KillRing::new();
    ring.push("hello".to_string());
    assert_eq!(ring.yank(), Some("hello"));
}

#[test]
fn yank_pop() {
    let mut ring = KillRing::new();
    ring.push("first".to_string());
    ring.push("second".to_string());
    assert_eq!(ring.yank(), Some("second"));
    assert_eq!(ring.yank_pop(), Some("first"));
    assert_eq!(ring.yank_pop(), Some("second"));
}
