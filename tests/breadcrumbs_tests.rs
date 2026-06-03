use photon_ui::{
    Component,
    components::Breadcrumbs,
    theme::Theme,
};

#[test]
fn breadcrumbs_renders_single_item() {
    Theme::with(Theme::Light, || {
        let bc = Breadcrumbs::new(vec!["Home"]);
        let rendered = bc.render(80).unwrap();
        let line = &rendered.lines[0];

        assert!(line.contains("Home"));
        assert!(!line.contains(" > "));
    });
}

#[test]
fn breadcrumbs_renders_trail() {
    Theme::with(Theme::Light, || {
        let bc = Breadcrumbs::new(vec!["Home", "Settings", "Account"]);
        let rendered = bc.render(80).unwrap();
        let line = &rendered.lines[0];

        assert!(line.contains("Home"));
        assert!(line.contains("Settings"));
        assert!(line.contains("Account"));
        assert!(line.contains(" > "));
    });
}

#[test]
fn breadcrumbs_custom_separator() {
    Theme::with(Theme::Light, || {
        let bc = Breadcrumbs::new(vec!["A", "B", "C"]).separator(" / ");
        let rendered = bc.render(80).unwrap();
        let line = &rendered.lines[0];

        assert!(line.contains(" / "));
        assert!(!line.contains(" > "));
    });
}

#[test]
fn breadcrumbs_theme_colors_differ_between_themes() {
    let light_line = Theme::with(Theme::Light, || {
        Breadcrumbs::new(vec!["X", "Y"]).render(80).unwrap().lines[0].clone()
    });

    let dark_line = Theme::with(Theme::Dark, || {
        Breadcrumbs::new(vec!["X", "Y"]).render(80).unwrap().lines[0].clone()
    });

    assert!(light_line.contains('\x1b'));
    assert!(dark_line.contains('\x1b'));
    // Light and dark secondary colors differ
    assert_ne!(light_line, dark_line);
}

#[test]
fn breadcrumbs_empty_items() {
    Theme::with(Theme::Light, || {
        let bc = Breadcrumbs::new(Vec::<String>::new());
        let rendered = bc.render(80).unwrap();
        assert_eq!(rendered.lines[0], "");
    });
}
