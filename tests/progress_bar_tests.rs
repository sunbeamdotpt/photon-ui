use photon_ui::{
    Component,
    components::ProgressBar,
    theme::Theme,
};

#[test]
fn progress_bar_renders_with_percent() {
    Theme::with(Theme::Light, || {
        let pb = ProgressBar::new("Downloading", 0.75).width(12);
        let rendered = pb.render(80).unwrap();
        let line = &rendered.lines[0];

        assert!(line.starts_with("Downloading "));
        assert!(line.contains('['));
        assert!(line.contains(']'));
        assert!(line.contains("75%"));
    });
}

#[test]
fn progress_bar_hides_percent() {
    Theme::with(Theme::Light, || {
        let pb = ProgressBar::new("", 0.5).width(10).hide_percent();
        let rendered = pb.render(80).unwrap();
        let line = &rendered.lines[0];

        assert!(!line.contains('%'));
        assert!(line.contains('['));
        assert!(line.contains(']'));
    });
}

#[test]
fn progress_bar_empty_label() {
    Theme::with(Theme::Light, || {
        let pb = ProgressBar::new("", 0.3).width(10);
        let rendered = pb.render(80).unwrap();
        let line = &rendered.lines[0];

        // Should not start with a stray space when label is empty
        assert!(line.starts_with('['));
        assert!(line.contains("30%"));
    });
}

#[test]
fn progress_bar_clamps_value() {
    Theme::with(Theme::Light, || {
        let over = ProgressBar::new("", 2.0).width(10).render(80).unwrap();
        let under = ProgressBar::new("", -0.5).width(10).render(80).unwrap();

        assert!(over.lines[0].contains("100%"));
        assert!(under.lines[0].contains("0%"));
    });
}

#[test]
fn progress_bar_uses_theme_colors() {
    let light_line = Theme::with(Theme::Light, || {
        ProgressBar::new("", 0.5)
            .width(10)
            .render(80)
            .unwrap()
            .lines[0]
            .clone()
    });

    let dark_line = Theme::with(Theme::Dark, || {
        ProgressBar::new("", 0.5)
            .width(10)
            .render(80)
            .unwrap()
            .lines[0]
            .clone()
    });

    // Both should contain the bar structure
    assert!(light_line.contains('['));
    assert!(dark_line.contains('['));

    // Both should have ANSI codes
    assert!(light_line.starts_with('\x1b') || light_line.contains('\x1b'));
    assert!(dark_line.starts_with('\x1b') || dark_line.contains('\x1b'));
}
