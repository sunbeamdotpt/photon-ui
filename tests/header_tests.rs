use photon_ui::{
    Component,
    components::Header,
    theme::Theme,
    utils::visible_width,
};

#[test]
fn header_renders_title_only() {
    Theme::with(Theme::Light, || {
        let header = Header::new("My App");
        let rendered = header.render(20).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert!(rendered.lines[0].contains("My App"));
    });
}

#[test]
fn header_renders_actions_right_aligned() {
    Theme::with(Theme::Light, || {
        let header = Header::new("My App").action("Save").action("Delete");
        let rendered = header.render(40).unwrap();
        let line = &rendered.lines[0];

        assert!(line.contains("My App"));
        assert!(line.contains("[Save]"));
        assert!(line.contains("[Delete]"));

        let title_pos = line.find("My App").unwrap();
        let save_pos = line.find("[Save]").unwrap();
        assert!(save_pos > title_pos);
    });
}

#[test]
fn header_uses_primary_color_for_title() {
    Theme::with(Theme::Light, || {
        let header = Header::new("Title");
        let rendered = header.render(20).unwrap();
        let line = &rendered.lines[0];
        // Light theme text_primary is #1f1f1f = 31,31,31
        assert!(line.contains("\x1b[38;2;31;31;31m"));
    });
}

#[test]
fn header_uses_secondary_color_for_actions() {
    Theme::with(Theme::Light, || {
        let header = Header::new("Title").action("Help");
        let rendered = header.render(20).unwrap();
        let line = &rendered.lines[0];
        // Light theme text_secondary is #666666 = 102,102,102
        assert!(line.contains("\x1b[38;2;102;102;102m"));
    });
}

#[test]
fn header_truncates_when_too_wide() {
    Theme::with(Theme::Light, || {
        let header = Header::new("Very Long Title Indeed").action("X");
        let rendered = header.render(15).unwrap();
        let line = &rendered.lines[0];
        assert!(visible_width(line) <= 15);
        assert!(line.contains("[X]"));
    });
}

#[test]
fn header_empty_actions() {
    Theme::with(Theme::Light, || {
        let header = Header::new("Only Title");
        let rendered = header.render(20).unwrap();
        assert!(rendered.lines[0].contains("Only Title"));
        // No action labels like [Save] should appear
        assert!(!rendered.lines[0].contains("[Only Title]"));
    });
}

#[test]
fn header_theme_colors_differ_between_themes() {
    let light_line = Theme::with(Theme::Light, || {
        Header::new("X").action("Y").render(20).unwrap().lines[0].clone()
    });

    let dark_line = Theme::with(Theme::Dark, || {
        Header::new("X").action("Y").render(20).unwrap().lines[0].clone()
    });

    assert!(light_line.contains('\x1b'));
    assert!(dark_line.contains('\x1b'));
    assert_ne!(light_line, dark_line);
}
