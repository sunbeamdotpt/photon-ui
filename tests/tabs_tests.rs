use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::Tabs,
    events::Event,
    theme::Theme,
};

#[test]
fn tabs_renders_with_accent_for_active() {
    Theme::with(Theme::Light, || {
        let tabs = Tabs::new(vec!["Tab 1", "Tab 2", "Tab 3"]);
        let rendered = tabs.render(80).unwrap();
        let line = &rendered.lines[0];
        // Light theme accent is SUNBEAM_ORANGE (#fa520f = 250,82,15)
        assert!(line.contains("\x1b[38;2;250;82;15m"));
    });
}

#[test]
fn tabs_renders_focused_prefix() {
    Theme::with(Theme::Light, || {
        let mut tabs = Tabs::new(vec!["Tab 1", "Tab 2"]);
        tabs.set_focused(true);
        let rendered = tabs.render(80).unwrap();
        assert!(rendered.lines[0].starts_with('│'));
    });
}

#[test]
fn tabs_keyboard_navigation() {
    let mut tabs = Tabs::new(vec!["A", "B", "C"]);
    tabs.set_focused(true);

    tabs.handle_input(&Event::Key(KeyCode::Right.into()));
    assert_eq!(tabs.active(), 1);

    tabs.handle_input(&Event::Key(KeyCode::Right.into()));
    assert_eq!(tabs.active(), 2);

    tabs.handle_input(&Event::Key(KeyCode::Right.into()));
    assert_eq!(tabs.active(), 2); // clamped
}

#[test]
fn tabs_inactive_uses_secondary_color() {
    Theme::with(Theme::Light, || {
        let tabs = Tabs::new(vec!["Tab 1", "Tab 2"]);
        let rendered = tabs.render(80).unwrap();
        let line = &rendered.lines[0];
        // Light theme text_secondary is #666666 = 102,102,102
        assert!(line.contains("\x1b[38;2;102;102;102m"));
    });
}

#[test]
fn tabs_h_and_l_navigation() {
    let mut tabs = Tabs::new(vec!["A", "B", "C"]);
    tabs.set_focused(true);
    tabs.set_active(2);

    tabs.handle_input(&Event::Key(KeyCode::Char('h').into()));
    assert_eq!(tabs.active(), 1);

    tabs.handle_input(&Event::Key(KeyCode::Char('l').into()));
    assert_eq!(tabs.active(), 2);
}

#[test]
fn tabs_returns_handled_on_switch() {
    let mut tabs = Tabs::new(vec!["A", "B"]);
    tabs.set_focused(true);

    let result = tabs.handle_input(&Event::Key(KeyCode::Right.into()));
    assert_eq!(result, InputResult::Handled);

    let result = tabs.handle_input(&Event::Key(KeyCode::Left.into()));
    assert_eq!(result, InputResult::Handled);
}
