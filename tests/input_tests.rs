use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::{
        Input,
        input::InputVimMode,
    },
    events::Event,
};

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

#[test]
fn input_default_construction() {
    let input = Input::default();
    assert_eq!(input.text(), "");
    assert!(!input.vim_mode_enabled());
}

#[test]
fn input_focused_getter() {
    let mut input = Input::new();
    input.set_focused(true);
    assert!(input.focused());
    input.set_focused(false);
    assert!(!input.focused());
}

#[test]
fn input_vim_mode_enabled_getter() {
    let mut input = Input::new();
    assert!(!input.vim_mode_enabled());
    input.set_vim_mode_enabled(true);
    assert!(input.vim_mode_enabled());
}

#[test]
fn input_ctrl_b_and_ctrl_f_move_cursor() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&ctrl_event('a'));
    input.handle_input(&ctrl_event('f'));
    assert_eq!(input.cursor(), 1);
    input.handle_input(&ctrl_event('b'));
    assert_eq!(input.cursor(), 0);
}

#[test]
fn input_ctrl_h_deletes_backward() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&ctrl_event('h'));
    assert_eq!(input.text(), "a");
}

#[test]
fn input_keycode_right_home_end_delete() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.handle_input(&key_event(KeyCode::Home));
    assert_eq!(input.cursor(), 0);
    input.handle_input(&key_event(KeyCode::Right));
    assert_eq!(input.cursor(), 1);
    input.handle_input(&key_event(KeyCode::End));
    assert_eq!(input.cursor(), 2);
    input.handle_input(&key_event(KeyCode::Home));
    input.handle_input(&key_event(KeyCode::Delete));
    assert_eq!(input.text(), "b");
}

#[test]
fn input_normal_mode_ignored_char() {
    let mut input = Input::new();
    input.set_vim_mode_enabled(true);
    input.set_mode(InputVimMode::Normal);
    let result = input.handle_input(&key_event(KeyCode::Char('z')));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn input_normal_mode_navigation_keys() {
    let mut input = Input::new();
    input.set_vim_mode_enabled(true);
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&key_event(KeyCode::Char('a')));
    input.handle_input(&key_event(KeyCode::Char('b')));
    input.set_mode(InputVimMode::Normal);
    input.handle_input(&key_event(KeyCode::End));
    assert_eq!(input.cursor(), 2);
    input.handle_input(&key_event(KeyCode::Left));
    assert_eq!(input.cursor(), 1);
    input.handle_input(&key_event(KeyCode::Right));
    assert_eq!(input.cursor(), 2);
    input.handle_input(&key_event(KeyCode::Home));
    assert_eq!(input.cursor(), 0);
    input.handle_input(&key_event(KeyCode::Backspace));
    assert_eq!(input.cursor(), 0);
}

#[test]
fn input_normal_mode_unhandled_key_ignored() {
    let mut input = Input::new();
    input.set_vim_mode_enabled(true);
    input.set_mode(InputVimMode::Normal);
    let result = input.handle_input(&key_event(KeyCode::Esc));
    assert!(matches!(result, InputResult::Ignored));
}
