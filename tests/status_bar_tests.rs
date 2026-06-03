use photon_ui::{
    Component,
    components::{
        Segment,
        StatusBar,
    },
    theme::{
        Palette,
        Theme,
    },
    utils::visible_width,
};

#[test]
fn status_bar_renders_single_line() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new()
            .left(Segment::new("mode: normal"))
            .right(Segment::new("Ln 1, Col 1"));
        let rendered = bar.render(40).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert_eq!(visible_width(&rendered.lines[0]), 40);
    });
}

#[test]
fn status_bar_aligns_zones() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new()
            .left(Segment::new("LEFT"))
            .center(Segment::new("CENTER"))
            .right(Segment::new("RIGHT"));
        let rendered = bar.render(40).unwrap();
        let line = &rendered.lines[0];

        let left_pos = line.find("LEFT").unwrap();
        let center_pos = line.find("CENTER").unwrap();
        let right_pos = line.find("RIGHT").unwrap();

        assert!(left_pos < center_pos);
        assert!(center_pos < right_pos);

        // Center should be roughly in the middle (40 - 6) / 2 = 17 (visual)
        let center_visual = visible_width(&line[..center_pos]);
        assert!(center_visual >= 15 && center_visual <= 19);
    });
}

#[test]
fn status_bar_uses_default_style() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new().left(Segment::new("info"));
        let rendered = bar.render(20).unwrap();
        let line = &rendered.lines[0];
        // Light theme text_secondary is #666666 = 102,102,102
        assert!(line.contains("\x1b[38;2;102;102;102m"));
    });
}

#[test]
fn status_bar_custom_style_overrides() {
    Theme::with(Theme::Light, || {
        let style = photon_ui::theme::Style::new().fg(Theme::current().accent());
        let bar = StatusBar::new().left(Segment::new("accent").styled(style));
        let rendered = bar.render(20).unwrap();
        let line = &rendered.lines[0];
        // Light theme accent is SUNBEAM_ORANGE (#fa520f = 250,82,15)
        assert!(line.contains("\x1b[38;2;250;82;15m"));
    });
}

#[test]
fn status_bar_truncates_content() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new()
            .left(Segment::new("this is a very long left side"))
            .right(Segment::new("right"));
        let rendered = bar.render(20).unwrap();
        let line = &rendered.lines[0];
        assert!(visible_width(line) <= 20);
        // Right side should still be visible (it has priority after left is capped)
        assert!(line.contains("right"));
    });
}

#[test]
fn status_bar_empty_renders_full_width() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new();
        let rendered = bar.render(25).unwrap();
        assert_eq!(visible_width(&rendered.lines[0]), 25);
    });
}

#[test]
fn status_bar_multiple_segments_per_zone() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new()
            .left(Segment::new("A"))
            .left(Segment::new("B"))
            .right(Segment::new("C"))
            .right(Segment::new("D"));
        let rendered = bar.render(40).unwrap();
        let line = &rendered.lines[0];

        assert!(line.contains("A"));
        assert!(line.contains("B"));
        assert!(line.contains("C"));
        assert!(line.contains("D"));

        let a_pos = line.find("A").unwrap();
        let b_pos = line.find("B").unwrap();
        let c_pos = line.find("C").unwrap();
        let d_pos = line.find("D").unwrap();

        assert!(a_pos < b_pos);
        assert!(c_pos < d_pos);
        assert!(b_pos < c_pos);
    });
}

#[test]
fn status_bar_theme_colors_differ_between_themes() {
    let light_line = Theme::with(Theme::Light, || {
        StatusBar::new()
            .left(Segment::new("x"))
            .render(20)
            .unwrap()
            .lines[0]
            .clone()
    });

    let dark_line = Theme::with(Theme::Dark, || {
        StatusBar::new()
            .left(Segment::new("x"))
            .render(20)
            .unwrap()
            .lines[0]
            .clone()
    });

    assert!(light_line.contains('\x1b'));
    assert!(dark_line.contains('\x1b'));
    assert_ne!(light_line, dark_line);
}
