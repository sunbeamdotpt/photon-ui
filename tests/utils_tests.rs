use photon_ui::utils::{
    AnsiCodeTracker,
    byte_index_at_visual_pos,
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

/// Regression: `\x1b[0m` must reset all tracked SGR state.
#[test]
fn tracker_reset_clears_all_sgr() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[1;3;4;31;43m");
    assert!(tracker.bold);
    assert!(tracker.italic);
    assert!(tracker.underline);
    assert!(tracker.fg_color.is_some());
    assert!(tracker.bg_color.is_some());
    tracker.process("\x1b[0m");
    assert!(!tracker.bold);
    assert!(!tracker.italic);
    assert!(!tracker.underline);
    assert!(tracker.fg_color.is_none());
    assert!(tracker.bg_color.is_none());
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

/// Regression: truncating bold text must emit an explicit bold-off before the
/// full reset so macOS terminals don't leave bold active.
#[test]
fn truncate_to_width_emits_bold_off_for_bold_text() {
    let s = "\x1b[1mhello world\x1b[0m";
    let truncated = truncate_to_width(s, 8, "…");
    assert!(truncated.ends_with("\x1b[22m\x1b[0m"));
}

/// Non-bold text should keep the plain reset suffix.
#[test]
fn truncate_to_width_uses_plain_reset_for_non_bold() {
    let s = "\x1b[31mhello world\x1b[0m";
    let truncated = truncate_to_width(s, 8, "…");
    assert!(truncated.ends_with("\x1b[0m"));
    assert!(!truncated.contains("\x1b[22m"));
}

/// Regression: wrapping bold text must close each line with an explicit
/// bold-off so the terminal doesn't leak bold into the next line.
#[test]
fn wrap_text_with_ansi_emits_bold_off_for_bold_text() {
    let lines = wrap_text_with_ansi("\x1b[1mhello world\x1b[0m", 6);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with("\x1b[22m\x1b[0m"));
    assert!(lines[1].starts_with("\x1b[1m"));
}

#[test]
fn visible_width_osc8_escape_slash_terminator() {
    assert_eq!(visible_width("\x1b]8;;https://example.com\x1b\\hello"), 5);
}

#[test]
fn visible_width_osc8_bel_terminator() {
    assert_eq!(visible_width("\x1b]8;;https://example.com\x07hello"), 5);
}

#[test]
fn visible_width_lone_escape_falls_through() {
    assert_eq!(visible_width("\x1bhello"), 5);
    assert_eq!(visible_width("\x1bXhello"), 6);
}

#[test]
fn byte_index_at_visual_pos_with_osc8_escape_slash() {
    let s = "\x1b]8;;https://example.com\x1b\\hello";
    assert_eq!(byte_index_at_visual_pos(s, 0), 26);
    assert_eq!(byte_index_at_visual_pos(s, 3), 29);
}

#[test]
fn byte_index_at_visual_pos_lone_escape_falls_through() {
    assert_eq!(byte_index_at_visual_pos("\x1bhello", 0), 1);
}

#[test]
fn truncate_to_width_preserves_osc8_hyperlink() {
    let s = "\x1b]8;;https://example.com\x1b\\hello world\x1b]8;;\x1b\\";
    let truncated = truncate_to_width(s, 8, "…");
    assert!(truncated.starts_with("\x1b]8;;https://example.com\x1b\\"));
    assert!(truncated.contains("hello w…"));
    assert!(truncated.ends_with("\x1b[0m"));
    assert_eq!(visible_width(&truncated), 8);
}

#[test]
fn truncate_to_width_preserves_osc8_hyperlink_bel() {
    let s = "\x1b]8;;https://example.com\x07hello world\x1b]8;;\x07";
    let truncated = truncate_to_width(s, 8, "…");
    assert!(truncated.starts_with("\x1b]8;;https://example.com\x07"));
    assert!(truncated.contains("hello w…"));
    assert_eq!(visible_width(&truncated), 8);
}

#[test]
fn tracker_osc8_missing_semicolon_ignored() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b]8;params\x1b\\");
    assert!(tracker.hyperlink.is_none());
}

#[test]
fn tracker_extended_color_incomplete_prefix_only() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[38m");
    assert!(tracker.fg_color.is_none());
}

#[test]
fn tracker_extended_color_incomplete_256() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[38;5m");
    assert!(tracker.fg_color.is_none());
}

#[test]
fn tracker_extended_color_incomplete_truecolor() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[38;2;1;2m");
    assert!(tracker.fg_color.is_none());
}

#[test]
fn tracker_extended_color_unknown_subtype() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[38;9;1m");
    assert!(tracker.fg_color.is_none());
}

#[test]
fn tracker_current_codes_includes_faint() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[2m");
    assert_eq!(tracker.current_codes(), "\x1b[2m");
}

#[test]
fn tracker_current_codes_includes_italic() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[3m");
    assert_eq!(tracker.current_codes(), "\x1b[3m");
}

#[test]
fn tracker_current_codes_includes_underline() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[4m");
    assert_eq!(tracker.current_codes(), "\x1b[4m");
}

#[test]
fn tracker_current_codes_includes_reverse() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[7m");
    assert_eq!(tracker.current_codes(), "\x1b[7m");
}

#[test]
fn tracker_line_end_reset_closes_reverse() {
    let mut tracker = AnsiCodeTracker::new();
    tracker.process("\x1b[7m");
    let reset = tracker.line_end_reset();
    assert!(reset.contains("\x1b[27m"));
}

#[test]
fn wrap_text_with_ansi_lone_escape_treated_as_text() {
    let lines = wrap_text_with_ansi("\x1bhello world", 6);
    assert_eq!(lines, vec!["\x1bhello ", "world"]);
}

#[test]
fn wrap_text_with_ansi_newline_with_underline_reset() {
    let lines = wrap_text_with_ansi("\x1b[4mhello\nworld\x1b[0m", 20);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\x1b[24m"));
    assert!(lines[1].starts_with("\x1b[4m"));
}

#[test]
fn truncate_to_width_lone_escape_falls_through() {
    let truncated = truncate_to_width("\x1bhello world", 8, "…");
    assert_eq!(visible_width(&truncated), 8);
    assert!(truncated.contains("hello w…"));
}
