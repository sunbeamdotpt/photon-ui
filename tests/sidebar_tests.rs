use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::{Sidebar, SidebarItem},
    events::Event,
    theme::Theme,
};

#[test]
fn sidebar_renders_with_accent_for_selected() {
    Theme::with(Theme::Light, || {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        sidebar.set_focused(true);
        let rendered = sidebar.render(80).unwrap();
        let line = &rendered.lines[0];
        // Light theme accent is SUNBEAM_ORANGE (#fa520f = 250,82,15)
        assert!(line.contains("\x1b[38;2;250;82;15m"));
    });
}

#[test]
fn sidebar_renders_secondary_for_unselected() {
    Theme::with(Theme::Light, || {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        sidebar.set_focused(true);
        let rendered = sidebar.render(80).unwrap();
        let line = &rendered.lines[1];
        // Light theme text_secondary is #666666 = 102,102,102
        assert!(line.contains("\x1b[38;2;102;102;102m"));
    });
}

#[test]
fn sidebar_unfocused_no_accent() {
    Theme::with(Theme::Light, || {
        let sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        // not focused
        let rendered = sidebar.render(80).unwrap();
        let line = &rendered.lines[0];
        // Should use text_secondary, not accent
        assert!(line.contains("\x1b[38;2;102;102;102m"));
        assert!(!line.contains("\x1b[38;2;250;82;15m"));
    });
}

#[test]
fn sidebar_keyboard_navigation() {
    let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B"), SidebarItem::new("C")]);
    sidebar.set_focused(true);

    sidebar.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(sidebar.selected(), 1);

    sidebar.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(sidebar.selected(), 2);

    sidebar.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(sidebar.selected(), 2); // clamped
}

#[test]
fn sidebar_up_navigation() {
    let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B"), SidebarItem::new("C")]);
    sidebar.set_focused(true);
    sidebar.set_selected(2);

    sidebar.handle_input(&Event::Key(KeyCode::Up.into()));
    assert_eq!(sidebar.selected(), 1);

    sidebar.handle_input(&Event::Key(KeyCode::Up.into()));
    assert_eq!(sidebar.selected(), 0);

    sidebar.handle_input(&Event::Key(KeyCode::Up.into()));
    assert_eq!(sidebar.selected(), 0); // clamped
}

#[test]
fn sidebar_j_and_k_navigation() {
    let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B"), SidebarItem::new("C")]);
    sidebar.set_focused(true);

    sidebar.handle_input(&Event::Key(KeyCode::Char('j').into()));
    assert_eq!(sidebar.selected(), 1);

    sidebar.handle_input(&Event::Key(KeyCode::Char('k').into()));
    assert_eq!(sidebar.selected(), 0);
}

#[test]
fn sidebar_returns_handled_on_navigate() {
    let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
    sidebar.set_focused(true);

    let result = sidebar.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(result, InputResult::Handled);

    let result = sidebar.handle_input(&Event::Key(KeyCode::Up.into()));
    assert_eq!(result, InputResult::Handled);
}

#[test]
fn sidebar_left_border_by_default() {
    Theme::with(Theme::Light, || {
        let sidebar = Sidebar::new(vec![SidebarItem::new("A")]);
        let rendered = sidebar.render(80).unwrap();
        assert!(rendered.lines[0].contains('▐'));
    });
}

#[test]
fn sidebar_hide_border_removes_border() {
    Theme::with(Theme::Light, || {
        let sidebar = Sidebar::new(vec![SidebarItem::new("A")]).hide_border();
        let rendered = sidebar.render(80).unwrap();
        assert!(!rendered.lines[0].contains('▐'));
    });
}

#[test]
fn sidebar_selected_prefix() {
    Theme::with(Theme::Light, || {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
        sidebar.set_focused(true);
        let rendered = sidebar.render(80).unwrap();
        // First line (selected) should have "> " prefix after border
        assert!(rendered.lines[0].contains("> "));
        // Second line (unselected) should have "  " prefix after border
        assert!(rendered.lines[1].contains("  "));
    });
}

#[test]
fn sidebar_icon_renders() {
    Theme::with(Theme::Light, || {
        let sidebar = Sidebar::new(vec![
            SidebarItem::new("Home").icon("🏠"),
            SidebarItem::new("Settings").icon("⚙"),
        ]);
        let rendered = sidebar.render(80).unwrap();
        assert!(rendered.lines[0].contains("🏠"));
        assert!(rendered.lines[0].contains("Home"));
        assert!(rendered.lines[1].contains("⚙"));
        assert!(rendered.lines[1].contains("Settings"));
    });
}

#[test]
fn sidebar_set_selected_clamps() {
    let mut sidebar = Sidebar::new(vec![SidebarItem::new("A"), SidebarItem::new("B")]);
    sidebar.set_selected(100);
    assert_eq!(sidebar.selected(), 1);
}
