use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::{
        Editor,
        editor::VimMode,
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
fn editor_types_and_renders() {
    let mut editor = Editor::new();
    editor.set_focused(true);
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('h')));
    editor.handle_input(&key_event(KeyCode::Char('i')));
    let r = editor.render(80).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("hi")));
}

#[test]
fn editor_cursor_moves() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&key_event(KeyCode::Left));
    assert_eq!(editor.cursor_grapheme(), 1);
}

#[test]
fn editor_newline_inserts_line() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Enter));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    let r = editor.render(80).unwrap();
    assert!(r.lines.len() >= 2);
}

#[test]
fn editor_backspace_deletes() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&key_event(KeyCode::Backspace));
    assert_eq!(editor.text(), "a");
}

#[test]
fn editor_delete_forward() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Left));
    editor.handle_input(&key_event(KeyCode::Delete));
    assert_eq!(editor.text(), "");
}

#[test]
fn editor_undo_redo() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&ctrl_event('-'));
    assert_eq!(editor.text(), "a");
}

#[test]
fn editor_kill_ring() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&key_event(KeyCode::Left));
    editor.handle_input(&key_event(KeyCode::Left));
    editor.handle_input(&ctrl_event('k'));
    assert_eq!(editor.text(), "");
    editor.handle_input(&ctrl_event('y'));
    assert_eq!(editor.text(), "ab");
}

#[test]
fn editor_history_navigation() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('f')));
    editor.handle_input(&key_event(KeyCode::Char('i')));
    editor.handle_input(&key_event(KeyCode::Char('r')));
    editor.handle_input(&key_event(KeyCode::Char('s')));
    editor.handle_input(&key_event(KeyCode::Char('t')));
    editor.push_history();
    editor.handle_input(&ctrl_event('a'));
    editor.handle_input(&ctrl_event('k'));
    editor.handle_input(&key_event(KeyCode::Char('s')));
    editor.handle_input(&key_event(KeyCode::Char('e')));
    editor.handle_input(&key_event(KeyCode::Char('c')));
    editor.handle_input(&key_event(KeyCode::Char('o')));
    editor.handle_input(&key_event(KeyCode::Char('n')));
    editor.handle_input(&key_event(KeyCode::Char('d')));
    editor.push_history();
    editor.handle_input(&key_event(KeyCode::Up));
    assert_eq!(editor.text(), "second");
    editor.handle_input(&key_event(KeyCode::Up));
    assert_eq!(editor.text(), "first");
}

#[test]
fn editor_backslash_enter_workaround() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('\\')));
    editor.handle_input(&key_event(KeyCode::Enter));
    assert_eq!(editor.text(), "a\n");
}

#[test]
fn editor_word_navigation() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('h')));
    editor.handle_input(&key_event(KeyCode::Char('e')));
    editor.handle_input(&key_event(KeyCode::Char('l')));
    editor.handle_input(&key_event(KeyCode::Char('l')));
    editor.handle_input(&key_event(KeyCode::Char('o')));
    editor.handle_input(&key_event(KeyCode::Char(' ')));
    editor.handle_input(&key_event(KeyCode::Char('w')));
    editor.handle_input(&key_event(KeyCode::Char('o')));
    editor.handle_input(&key_event(KeyCode::Char('r')));
    editor.handle_input(&key_event(KeyCode::Char('l')));
    editor.handle_input(&key_event(KeyCode::Char('d')));
    editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('b'),
        crossterm::event::KeyModifiers::ALT,
    )));
    assert_eq!(editor.cursor_grapheme(), 6);
}

#[test]
fn editor_ctrl_up_down() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Enter));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&ctrl_event('a'));
    // Ctrl+Up should move cursor up
    editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Up,
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.cursor_grapheme(), 0);
    // Ctrl+Down should move cursor down
    editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Down,
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.cursor_grapheme(), 2);
}

#[test]
fn editor_ignored_key() {
    let mut editor = Editor::new();
    let result = editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::F(1),
        crossterm::event::KeyModifiers::empty(),
    )));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn editor_non_key_event() {
    let mut editor = Editor::new();
    let result = editor.handle_input(&Event::Resize(80, 24));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn editor_as_focusable() {
    let editor = Editor::new();
    assert!(editor.as_focusable().is_some());
}

#[test]
fn editor_render_cache() {
    let mut editor = Editor::new();
    editor.set_focused(true);
    editor.set_mode(VimMode::Insert);
    for c in "hello world".chars() {
        editor.handle_input(&key_event(KeyCode::Char(c)));
    }
    // First render populates cache
    let r1 = editor.render(80).unwrap();
    // Second render with same width should hit cache
    let r2 = editor.render(80).unwrap();
    assert_eq!(r1.lines, r2.lines);
}
