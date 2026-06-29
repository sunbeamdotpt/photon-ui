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

fn alt_event(c: char) -> Event {
    Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char(c),
        crossterm::event::KeyModifiers::ALT,
    ))
}

fn ignored_key_event() -> Event {
    Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::F(1),
        crossterm::event::KeyModifiers::empty(),
    ))
}

#[test]
fn editor_default_and_focus_methods() {
    let mut editor: Editor = Default::default();
    assert!(!editor.focused());
    editor.set_focused(true);
    assert!(editor.focused());
    editor.set_focused(false);
    assert!(!editor.focused());
}

#[test]
fn editor_as_focusable_mut() {
    let mut editor = Editor::new();
    let focusable = editor.as_focusable_mut();
    assert!(focusable.is_some());
}

#[test]
fn editor_vim_mode_enabled_getter() {
    let mut editor = Editor::new();
    assert!(!editor.vim_mode_enabled());
    editor.set_vim_mode_enabled(true);
    assert!(editor.vim_mode_enabled());
}

#[test]
fn editor_insert_mode_ctrl_navigation() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&key_event(KeyCode::Enter));
    editor.handle_input(&key_event(KeyCode::Char('c')));
    editor.handle_input(&ctrl_event('a'));
    assert_eq!(editor.cursor_grapheme(), 3);
    editor.handle_input(&ctrl_event('e'));
    assert_eq!(editor.cursor_grapheme(), 4);
    editor.handle_input(&ctrl_event('b'));
    assert_eq!(editor.cursor_grapheme(), 3);
    editor.handle_input(&ctrl_event('f'));
    assert_eq!(editor.cursor_grapheme(), 4);
    editor.handle_input(&ctrl_event('p'));
    assert_eq!(editor.cursor_grapheme(), 1);
    editor.handle_input(&ctrl_event('n'));
    assert_eq!(editor.cursor_grapheme(), 4);
}

#[test]
fn editor_insert_mode_ctrl_delete() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('a')));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    editor.handle_input(&key_event(KeyCode::Left));
    editor.handle_input(&ctrl_event('d'));
    assert_eq!(editor.text(), "a");
    editor.handle_input(&ctrl_event('h'));
    assert_eq!(editor.text(), "");
}

#[test]
fn editor_insert_mode_ctrl_kill_undo() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab");
    editor.handle_input(&ctrl_event('a'));
    editor.handle_input(&ctrl_event('k'));
    assert_eq!(editor.text(), "");
    editor.handle_input(&ctrl_event('-'));
    assert_eq!(editor.text(), "ab");
}

#[test]
fn editor_insert_mode_ctrl_redo_handles() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    // With no redo history the handler simply returns Handled without changes.
    let result = editor.handle_input(&ctrl_event('r'));
    assert!(matches!(result, InputResult::Handled));
}

#[test]
fn editor_insert_mode_ignored_ctrl() {
    let mut editor = Editor::new();
    let result = editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char('q'),
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn editor_insert_mode_alt_navigation() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Char('h')));
    editor.handle_input(&key_event(KeyCode::Char('i')));
    editor.handle_input(&alt_event('b'));
    assert_eq!(editor.cursor_grapheme(), 0);
    editor.handle_input(&alt_event('f'));
    assert_eq!(editor.cursor_grapheme(), 2);
}

#[test]
fn editor_insert_mode_alt_kill() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("hi there");
    editor.handle_input(&alt_event('b'));
    editor.handle_input(&alt_event('d'));
    assert_eq!(editor.text(), "hi ");
    editor.handle_input(&ctrl_event('a'));
    editor.handle_input(&alt_event('d'));
    assert_eq!(editor.text(), " ");
}

#[test]
fn editor_insert_mode_yank_pop() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello world");
    editor.handle_input(&ctrl_event('a'));
    editor.handle_input(&ctrl_event('k'));
    editor.handle_input(&ctrl_event('y'));
    assert_eq!(editor.text(), "hello world");
    editor.handle_input(&alt_event('y'));
    assert_eq!(editor.text(), "hello worldhello world");
}

#[test]
fn editor_insert_mode_right_arrow() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab");
    editor.handle_input(&key_event(KeyCode::Left));
    editor.handle_input(&key_event(KeyCode::Left));
    editor.handle_input(&key_event(KeyCode::Right));
    assert_eq!(editor.cursor_grapheme(), 1);
}

#[test]
fn editor_insert_mode_down_history() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("first");
    editor.push_history();
    editor.set_text("second");
    editor.push_history();
    editor.handle_input(&key_event(KeyCode::Up));
    editor.handle_input(&key_event(KeyCode::Up));
    assert_eq!(editor.text(), "first");
    editor.handle_input(&key_event(KeyCode::Down));
    assert_eq!(editor.text(), "second");
    editor.handle_input(&key_event(KeyCode::Down));
    assert_eq!(editor.text(), "");
}

#[test]
fn editor_insert_mode_history_up_empty() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&key_event(KeyCode::Up));
    assert_eq!(editor.text(), "");
}

#[test]
fn editor_insert_mode_ignored_alt() {
    let mut editor = Editor::new();
    let result = editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char('q'),
        crossterm::event::KeyModifiers::ALT,
    )));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn editor_move_up_from_first_line() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab\ncd");
    // Cursor is at end of line 1; move to start of line 1.
    editor.handle_input(&ctrl_event('a'));
    // Ctrl+Up moves cursor up; plain Up navigates history.
    editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Up,
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.cursor_grapheme(), 0);
}

#[test]
fn editor_move_down_past_last_line() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab\ncd");
    editor.handle_input(&ctrl_event('e'));
    editor.handle_input(&key_event(KeyCode::Down));
    assert_eq!(editor.cursor_grapheme(), 5);
}

#[test]
fn editor_ctrl_w_kill_word_backward() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello world");
    editor.handle_input(&ctrl_event('w'));
    assert_eq!(editor.text(), "hello ");
}

#[test]
fn editor_vim_normal_arrow_keys() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab\ncd");
    editor.set_mode(VimMode::Normal);
    // Cursor is at end-of-document; jump to the start.
    editor.handle_input(&key_event(KeyCode::Char('g')));
    editor.handle_input(&key_event(KeyCode::Char('g')));
    assert_eq!(editor.cursor_grapheme(), 0);
    editor.handle_input(&key_event(KeyCode::Right));
    assert_eq!(editor.cursor_grapheme(), 1);
    editor.handle_input(&key_event(KeyCode::Left));
    assert_eq!(editor.cursor_grapheme(), 0);
    editor.handle_input(&key_event(KeyCode::End));
    assert_eq!(editor.cursor_grapheme(), 2);
    // Down from line 0 col 2 lands at the end of the shorter line 1.
    editor.handle_input(&key_event(KeyCode::Down));
    assert_eq!(editor.cursor_grapheme(), 5);
    editor.handle_input(&key_event(KeyCode::Home));
    assert_eq!(editor.cursor_grapheme(), 3);
    editor.handle_input(&key_event(KeyCode::Up));
    assert_eq!(editor.cursor_grapheme(), 0);
    editor.handle_input(&key_event(KeyCode::Backspace));
    assert_eq!(editor.cursor_grapheme(), 0);
}

#[test]
fn editor_vim_normal_ignored_char() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    let result = editor.handle_input(&key_event(KeyCode::Char('z')));
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn editor_vim_normal_ignored_key() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    let result = editor.handle_input(&ignored_key_event());
    assert!(matches!(result, InputResult::Ignored));
}

#[test]
fn editor_vim_yw_yanks_word() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello world");
    editor.set_mode(VimMode::Normal);
    editor.handle_input(&key_event(KeyCode::Home));
    editor.handle_input(&key_event(KeyCode::Char('y')));
    editor.handle_input(&key_event(KeyCode::Char('w')));
    // p pastes the yanked word after cursor (position 0)
    editor.handle_input(&key_event(KeyCode::Char('p')));
    assert_eq!(editor.text(), "hellohello world");
}

#[test]
fn editor_vim_pending_cmd_fallback() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello");
    editor.set_mode(VimMode::Normal);
    // Start a pending 'd' command, then cancel it with an unrecognized key.
    editor.handle_input(&key_event(KeyCode::Char('d')));
    let result = editor.handle_input(&key_event(KeyCode::Char('x')));
    assert!(matches!(result, InputResult::Handled));
    assert_eq!(editor.text(), "hello");
}

#[test]
fn editor_vim_normal_word_forward() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("hi there");
    editor.set_mode(VimMode::Normal);
    editor.handle_input(&key_event(KeyCode::Home));
    editor.handle_input(&key_event(KeyCode::Char('w')));
    assert_eq!(editor.cursor_grapheme(), 2);
}

#[test]
fn editor_insert_mode_ctrl_u() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello world");
    editor.handle_input(&ctrl_event('u'));
    assert_eq!(editor.text(), "");
}

#[test]
fn editor_move_down_past_last_line_with_ctrl() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab\ncd");
    editor.handle_input(&ctrl_event('e'));
    editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Down,
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.cursor_grapheme(), 5);
}

#[test]
fn editor_vim_normal_word_backward() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("hi there");
    editor.set_mode(VimMode::Normal);
    editor.handle_input(&key_event(KeyCode::End));
    editor.handle_input(&key_event(KeyCode::Char('b')));
    assert_eq!(editor.cursor_grapheme(), 3);
}

#[test]
fn editor_vim_open_line_above_mid_document() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("a\nb");
    editor.set_mode(VimMode::Normal);
    editor.handle_input(&key_event(KeyCode::Char('O')));
    assert_eq!(editor.mode(), VimMode::Insert);
    assert_eq!(editor.text(), "a\n\nb");
}

#[test]
fn editor_move_up_on_first_line_is_noop() {
    let mut editor = Editor::new();
    editor.set_mode(VimMode::Insert);
    editor.set_text("ab");
    editor.handle_input(&ctrl_event('a'));
    editor.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Up,
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.cursor_grapheme(), 0);
}

#[test]
fn editor_vim_dd_deletes_mid_line() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello\nworld");
    editor.set_mode(VimMode::Normal);
    editor.handle_input(&key_event(KeyCode::Down));
    editor.handle_input(&key_event(KeyCode::Char('d')));
    editor.handle_input(&key_event(KeyCode::Char('d')));
    assert_eq!(editor.text(), "hello\n");
}

#[test]
fn editor_vim_yy_yanks_mid_line() {
    let mut editor = Editor::new();
    editor.set_vim_mode_enabled(true);
    editor.set_mode(VimMode::Insert);
    editor.set_text("hello\nworld");
    editor.set_mode(VimMode::Normal);
    editor.handle_input(&key_event(KeyCode::Down));
    editor.handle_input(&key_event(KeyCode::Char('y')));
    editor.handle_input(&key_event(KeyCode::Char('y')));
    editor.handle_input(&key_event(KeyCode::Char('p')));
    // The last line has no trailing newline, so the yanked text is "world".
    assert_eq!(editor.text(), "hello\nworldworld");
}
