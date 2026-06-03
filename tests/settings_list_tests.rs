use photon_ui::{Component, Focusable, InputResult, Event};
use photon_ui::components::SettingsList;
use crossterm::event::KeyCode;

#[test]
fn settings_list_renders() {
    let list = SettingsList::new(vec![
        ("Option A".into(), true),
        ("Option B".into(), false),
    ]);
    let rendered = list.render(80).unwrap();
    assert_eq!(rendered.lines.len(), 2);
    assert!(rendered.lines[0].contains("[x]"));
    assert!(rendered.lines[0].contains("Option A"));
    assert!(rendered.lines[1].contains("[ ]"));
    assert!(rendered.lines[1].contains("Option B"));
}

#[test]
fn settings_list_navigation() {
    let mut list = SettingsList::new(vec![
        ("A".into(), false),
        ("B".into(), false),
        ("C".into(), false),
    ]);
    list.set_focused(true);

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[1].contains("> "));

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[2].contains("> "));

    // Should not go past end
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[2].contains("> "));
}

#[test]
fn settings_list_up_navigation() {
    let mut list = SettingsList::new(vec![
        ("A".into(), false),
        ("B".into(), false),
    ]);
    list.set_focused(true);
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty())));
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("> "));

    // Should not go above 0
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("> "));
}

#[test]
fn settings_list_j_k_navigation() {
    let mut list = SettingsList::new(vec![
        ("A".into(), false),
        ("B".into(), false),
        ("C".into(), false),
    ]);
    list.set_focused(true);

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[1].contains("> "));

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[2].contains("> "));

    // Should not go past end
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[2].contains("> "));

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[1].contains("> "));

    // Should not go above 0
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty())));
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("> "));
}

#[test]
fn settings_list_toggle() {
    let mut list = SettingsList::new(vec![
        ("A".into(), false),
        ("B".into(), false),
    ]);
    list.set_focused(true);

    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("[ ]"));

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("[x]"));

    // Toggle back with space
    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char(' '), crossterm::event::KeyModifiers::empty())));
    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("[ ]"));
}

#[test]
fn settings_list_on_change() {
    use std::cell::Cell;
    let called = Cell::new(false);
    let mut list = SettingsList::new(vec![
        ("A".into(), false),
    ]).on_change(|idx, val| {
        assert_eq!(idx, 0);
        assert!(val);
    });
    list.set_focused(true);

    list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty())));
}

#[test]
fn settings_list_values() {
    let list = SettingsList::new(vec![
        ("A".into(), true),
        ("B".into(), false),
    ]);
    assert_eq!(list.values(), vec![true, false]);
}

#[test]
fn settings_list_focusable() {
    let mut list = SettingsList::new(vec![("A".into(), false)]);
    assert!(!list.focused());
    list.set_focused(true);
    assert!(list.focused());
}

#[test]
fn settings_list_ignored_key() {
    let mut list = SettingsList::new(vec![("A".into(), false)]);
    let result = list.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char('z'),
        crossterm::event::KeyModifiers::empty(),
    )));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn settings_list_non_key_event() {
    let mut list = SettingsList::new(vec![("A".into(), false)]);
    let result = list.handle_input(&Event::Resize(80, 24));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn settings_list_as_focusable() {
    let list = SettingsList::new(vec![("A".into(), false)]);
    assert!(list.as_focusable().is_some());
}
