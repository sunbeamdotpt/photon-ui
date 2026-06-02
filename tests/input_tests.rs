use photon_ui::components::Input;
use photon_ui::components::input::InputVimMode;
use photon_ui::events::Event;
use photon_ui::{Component, InputResult, Focusable};
use crossterm::event::KeyCode;

fn key_event(code: KeyCode) -> Event {
    Event::Key(code.into())
}

fn ctrl_event(c: char) -> Event {
    Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char(c),
        crossterm::event::KeyModifiers::CONTROL,
    ))
}

#[test]
fn input_types_text() {
    let mut input = Input::new();
    input.set_focused(true);
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    let r = input.render(80).unwrap();
    assert!(r.lines[0].contains("a"));
}

#[test]
fn input_backspace_deletes() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Backspace));
    let r = input.render(80).unwrap();
    assert!(!r.lines[0].contains("a"));
}

#[test]
fn input_cursor_moves() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&key_event(KeyCode::Left));
    assert_eq!(input.cursor(), 1);
}

#[test]
fn input_kill_ring() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&key_event(KeyCode::Left));
    input.handle_input(&key_event(KeyCode::Left));
    input.handle_input(&ctrl_event('k'));
    assert_eq!(input.text(), "");
    input.handle_input(&ctrl_event('y'));
    assert_eq!(input.text(), "ab");
}

#[test]
fn input_undo() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&ctrl_event('-'));
    assert_eq!(input.text(), "a");
}

#[test]
fn input_home_end() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&ctrl_event('a'));
    assert_eq!(input.cursor(), 0);
    input.handle_input(&ctrl_event('e'));
    assert_eq!(input.cursor(), 2);
}
