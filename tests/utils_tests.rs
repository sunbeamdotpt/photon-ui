use photon_ui::utils::{
    AnsiCodeTracker,
    truncate_to_width,
    visible_width,
    wrap_text_with_ansi,
};

#[test]
fn visible_width_ascii() {
    assert_eq!(visible_width("hello"), 5);
}

#[test]
fn visible_width_cjk() {
    assert_eq!(visible_width("你好"), 4);
}

#[test]
fn truncate_to_width_fits() {
    assert_eq!(truncate_to_width("hello", 10, "…"), "hello");
}

#[test]
fn truncate_to_width_truncates() {
    assert_eq!(truncate_to_width("hello world", 8, "…"), "hello w…");
}

#[test]
fn truncate_to_width_cjk() {
    assert_eq!(truncate_to_width("你好世界", 5, "…"), "你好…");
}

#[test]
fn tracker_tracks_bold() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[1m");
    assert!(tracker.bold);
    tracker.process("\x1b[22m");
    assert!(!tracker.bold);
}

#[test]
fn tracker_tracks_color() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[31m");
    assert_eq!(tracker.fg_color, Some("31".to_string()));
}

#[test]
fn wrap_plain_text() {
    let lines = wrap_text_with_ansi("hello world", 6);
    assert_eq!(lines, vec!["hello ", "world"]);
}

#[test]
fn wrap_preserves_ansi() {
    let lines = wrap_text_with_ansi("\x1b[31mhello world\x1b[0m", 6);
    assert_eq!(lines[0], "\x1b[31mhello \x1b[0m");
    assert!(lines[1].contains("world"));
}

#[test]
fn visible_width_emoji() {
    use photon_ui::utils::visible_width;
    assert_eq!(visible_width("📊"), 2);
    assert_eq!(visible_width("👤"), 2);
}

#[test]
fn truncate_to_width_with_emoji() {
    use photon_ui::utils::truncate_to_width;
    let s = "> 📊 Overview";
    assert_eq!(visible_width(s), 13);
    let truncated = truncate_to_width(s, 10, "…");
    assert_eq!(visible_width(&truncated), 10);
}
