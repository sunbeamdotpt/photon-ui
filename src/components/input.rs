use std::cell::Cell;

use crossterm::event::KeyCode;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    kill_ring::KillRing,
    undo_stack::UndoStack,
};

/// Snapshot of input state for undo/redo.
#[derive(Clone)]
pub struct EditAction {
    /// The full text buffer at the time of the snapshot.
    pub text: String,
    /// Cursor position in grapheme indices.
    pub cursor: usize,
}

/// Vim editing mode state for single-line input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputVimMode {
    /// Normal mode — keys are commands (hl, x, i, etc.).
    Normal,
    /// Insert mode — keys insert text.
    Insert,
}

/// Single-line text input with horizontal scrolling.
///
/// Defaults to **Emacs-style** bindings (Ctrl+A, Ctrl+E, Ctrl+K, etc.).
/// Call [`set_vim_mode_enabled`](Input::set_vim_mode_enabled) to opt into
/// vim-style modal editing (Normal/Insert mode with hl, x, i, etc.).
///
/// The input scrolls horizontally when the text exceeds the render width so
/// that the cursor always remains visible. Supports undo, yank, and kill-ring.
pub struct Input {
    text: String,
    cursor: usize,
    focused: bool,
    kill_ring: KillRing,
    undo_stack: UndoStack<EditAction>,
    scroll: Cell<usize>,
    vim_mode_enabled: bool,
    mode: InputVimMode,
}

impl Input {
    /// Create a new empty input field with Emacs-style bindings.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            focused: false,
            kill_ring: KillRing::new(),
            undo_stack: UndoStack::new(),
            scroll: Cell::new(0),
            vim_mode_enabled: false,
            mode: InputVimMode::Normal,
        }
    }

    /// Returns `true` if vim modal editing is enabled.
    pub fn vim_mode_enabled(&self) -> bool {
        self.vim_mode_enabled
    }

    /// Enable or disable vim modal editing.
    ///
    /// When enabled the input starts in Normal mode; press `i` to insert,
    /// `Escape` to return to Normal. When disabled, Emacs-style bindings
    /// are always active.
    pub fn set_vim_mode_enabled(&mut self, enabled: bool) {
        self.vim_mode_enabled = enabled;
        self.mode = InputVimMode::Normal;
    }

    /// Current vim mode (only meaningful when vim mode is enabled).
    pub fn mode(&self) -> InputVimMode {
        self.mode
    }

    /// Switch to the given vim mode.
    pub fn set_mode(&mut self, mode: InputVimMode) {
        self.mode = mode;
    }

    /// Borrow the current text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Current cursor position in grapheme indices.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Current horizontal scroll offset in grapheme indices.
    pub fn scroll(&self) -> usize {
        self.scroll.get()
    }

    /// Replace the entire text buffer and move the cursor to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.save_undo();
        self.text = text.into();
        self.cursor = self.graphemes().len();
        self.scroll.set(0);
    }

    fn save_undo(&mut self) {
        self.undo_stack.push(EditAction {
            text: self.text.clone(),
            cursor: self.cursor,
        });
    }

    fn graphemes(&self) -> Vec<&str> {
        self.text.graphemes(true).collect()
    }

    fn byte_index(&self, grapheme_idx: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .nth(grapheme_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn insert_char(&mut self, c: char) {
        let idx = self.byte_index(self.cursor);
        self.text.insert(idx, c);
        self.cursor += 1;
    }

    fn delete_backward(&mut self) {
        if self.cursor > 0 {
            let start = self.byte_index(self.cursor - 1);
            let end = self.byte_index(self.cursor);
            let killed = self.text.drain(start..end).collect::<String>();
            self.kill_ring.push(killed);
            self.cursor -= 1;
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.graphemes().len() {
            let start = self.byte_index(self.cursor);
            let end = self.byte_index(self.cursor + 1);
            self.text.drain(start..end);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor < self.graphemes().len() {
            self.cursor += 1;
        }
    }

    fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    fn move_cursor_end(&mut self) {
        self.cursor = self.graphemes().len();
    }

    fn yank(&mut self) {
        if let Some(text) = self.kill_ring.yank() {
            let idx = self.byte_index(self.cursor);
            self.text.insert_str(idx, text);
            self.cursor += text.graphemes(true).count();
        }
    }

    fn undo(&mut self) {
        if let Some(action) = self.undo_stack.undo() {
            self.text = action.text.clone();
            self.cursor = action.cursor;
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Focusable for Input {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Input {
    fn handle_insert_mode(&mut self, key: &crossterm::event::KeyEvent) -> InputResult {
        use crossterm::event::KeyModifiers;
        self.save_undo();
        match key.code {
            | KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        | 'a' => self.move_cursor_home(),
                        | 'e' => self.move_cursor_end(),
                        | 'b' => self.move_cursor_left(),
                        | 'f' => self.move_cursor_right(),
                        | 'd' => self.delete_forward(),
                        | 'h' => self.delete_backward(),
                        | 'k' => {
                            let idx = self.byte_index(self.cursor);
                            let killed = self.text.split_off(idx);
                            self.kill_ring.push(killed);
                        },
                        | 'y' => self.yank(),
                        | '-' | '_' => self.undo(),
                        | _ => return InputResult::Ignored,
                    }
                } else {
                    self.insert_char(c);
                }
                InputResult::Handled
            },
            | KeyCode::Left => {
                self.move_cursor_left();
                InputResult::Handled
            },
            | KeyCode::Right => {
                self.move_cursor_right();
                InputResult::Handled
            },
            | KeyCode::Home => {
                self.move_cursor_home();
                InputResult::Handled
            },
            | KeyCode::End => {
                self.move_cursor_end();
                InputResult::Handled
            },
            | KeyCode::Backspace => {
                self.delete_backward();
                InputResult::Handled
            },
            | KeyCode::Delete => {
                self.delete_forward();
                InputResult::Handled
            },
            | KeyCode::Esc => {
                self.mode = InputVimMode::Normal;
                InputResult::Handled
            },
            | _ => InputResult::Ignored,
        }
    }

    fn handle_normal_mode(&mut self, key: &crossterm::event::KeyEvent) -> InputResult {
        match key.code {
            | KeyCode::Char(c) => {
                match c {
                    | 'h' => self.move_cursor_left(),
                    | 'l' => self.move_cursor_right(),
                    | 'x' => {
                        self.save_undo();
                        self.delete_forward();
                    },
                    | '0' => self.move_cursor_home(),
                    | '$' => self.move_cursor_end(),
                    | 'i' => self.mode = InputVimMode::Insert,
                    | 'a' => {
                        self.move_cursor_right();
                        self.mode = InputVimMode::Insert;
                    },
                    | 'p' => {
                        self.save_undo();
                        self.yank();
                    },
                    | 'u' => {
                        self.save_undo();
                        self.undo();
                    },
                    | _ => return InputResult::Ignored,
                }
                InputResult::Handled
            },
            | KeyCode::Left => {
                self.move_cursor_left();
                InputResult::Handled
            },
            | KeyCode::Right => {
                self.move_cursor_right();
                InputResult::Handled
            },
            | KeyCode::Home => {
                self.move_cursor_home();
                InputResult::Handled
            },
            | KeyCode::End => {
                self.move_cursor_end();
                InputResult::Handled
            },
            | KeyCode::Backspace => {
                self.move_cursor_left();
                InputResult::Handled
            },
            | _ => InputResult::Ignored,
        }
    }
}

impl Component for Input {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let w = width as usize;
        let mut scroll = self.scroll.get();
        if self.cursor > scroll.saturating_add(w.saturating_sub(1)) {
            scroll = self.cursor.saturating_sub(w.saturating_sub(1));
        } else if self.cursor < scroll {
            scroll = self.cursor;
        }
        self.scroll.set(scroll);

        let visible = self
            .graphemes()
            .iter()
            .skip(scroll)
            .take(w)
            .copied()
            .collect::<String>();
        let mut line = visible;
        if line.len() < w {
            line.push_str(&" ".repeat(w.saturating_sub(line.len())));
        }
        let cursor_col = self.cursor.saturating_sub(scroll);
        Ok(Rendered {
            lines: vec![line],
            cursor: if self.focused {
                Some((0, cursor_col))
            } else {
                None
            },
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            if self.vim_mode_enabled {
                match self.mode {
                    | InputVimMode::Insert => self.handle_insert_mode(key),
                    | InputVimMode::Normal => self.handle_normal_mode(key),
                }
            } else {
                self.handle_insert_mode(key)
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> {
        Some(self)
    }

    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{
        KeyCode,
        KeyEvent,
        KeyModifiers,
    };

    use super::*;

    fn key_event(code: KeyCode) -> Event {
        Event::Key(code.into())
    }

    fn ctrl_event(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
    }

    #[test]
    fn input_delete_forward_at_end() {
        let mut input = Input::new();
        input.insert_char('a');
        input.move_cursor_end();
        input.delete_forward();
        assert_eq!(input.text(), "a");
    }

    #[test]
    fn input_scrolls_when_long() {
        let mut input = Input::new();
        input.set_focused(true);
        input.set_mode(InputVimMode::Insert);
        for _ in 0..20 {
            input.handle_input(&key_event(KeyCode::Char('x')));
        }
        let r = input.render(10).unwrap();
        assert_eq!(r.lines[0].len(), 10);
        assert!(r.cursor.is_some());
        assert_eq!(input.scroll(), 11);
        assert_eq!(r.cursor, Some((0, 9)));
    }

    #[test]
    fn input_scrolls_back_left() {
        let mut input = Input::new();
        input.set_focused(true);
        input.set_mode(InputVimMode::Insert);
        for _ in 0..20 {
            input.handle_input(&key_event(KeyCode::Char('x')));
        }
        input.render(10).unwrap();
        assert_eq!(input.scroll(), 11);
        // Move cursor to the start (Ctrl+A in insert mode)
        input.handle_input(&ctrl_event('a'));
        let r = input.render(10).unwrap();
        assert_eq!(input.scroll(), 0);
        assert_eq!(r.cursor, Some((0, 0)));
    }

    #[test]
    fn input_render_unfocused() {
        let mut input = Input::new();
        input.set_focused(false);
        input.insert_char('a');
        let r = input.render(10).unwrap();
        assert!(r.cursor.is_none());
    }

    #[test]
    fn input_yank() {
        let mut input = Input::new();
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.insert_char('b');
        input.move_cursor_home();
        input.handle_input(&ctrl_event('k'));
        assert_eq!(input.text(), "");
        input.yank();
        assert_eq!(input.text(), "ab");
    }

    #[test]
    fn input_ctrl_d_delete() {
        let mut input = Input::new();
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.insert_char('b');
        input.move_cursor_home();
        input.handle_input(&ctrl_event('d'));
        assert_eq!(input.text(), "b");
    }

    #[test]
    fn input_ignored_ctrl_key() {
        let mut input = Input::new();
        let result = input.handle_input(&ctrl_event('z'));
        assert!(matches!(result, InputResult::Ignored));
    }

    #[test]
    fn input_resize_ignored() {
        let mut input = Input::new();
        let result = input.handle_input(&Event::Resize(80, 24));
        assert!(matches!(result, InputResult::Ignored));
    }

    #[test]
    fn input_delete_backward_at_start() {
        let mut input = Input::new();
        input.delete_backward();
        assert_eq!(input.text(), "");
    }

    #[test]
    fn input_move_past_bounds() {
        let mut input = Input::new();
        input.move_cursor_left();
        assert_eq!(input.cursor(), 0);
        input.insert_char('a');
        input.move_cursor_right();
        input.move_cursor_right();
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn input_enter_ignored() {
        let mut input = Input::new();
        let result = input.handle_input(&key_event(KeyCode::Enter));
        assert!(matches!(result, InputResult::Ignored));
    }

    #[test]
    fn input_tab_ignored() {
        let mut input = Input::new();
        let result = input.handle_input(&key_event(KeyCode::Tab));
        assert!(matches!(result, InputResult::Ignored));
    }

    #[test]
    fn input_render_pads_with_spaces() {
        let mut input = Input::new();
        input.set_focused(true);
        input.insert_char('a');
        let r = input.render(10).unwrap();
        assert_eq!(r.lines[0].len(), 10);
    }

    // -- Vim mode tests --

    #[test]
    fn vim_input_starts_in_normal_mode() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        assert_eq!(input.mode(), InputVimMode::Normal);
    }

    #[test]
    fn vim_input_hl_navigation() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.insert_char('b');
        input.set_mode(InputVimMode::Normal);
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.cursor(), 1);
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn vim_input_i_enters_insert() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('i'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.mode(), InputVimMode::Insert);
        input.handle_input(&key_event(KeyCode::Char('x')));
        assert_eq!(input.text(), "x");
    }

    #[test]
    fn vim_input_esc_returns_to_normal() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.handle_input(&key_event(KeyCode::Esc));
        assert_eq!(input.mode(), InputVimMode::Normal);
    }

    #[test]
    fn vim_input_x_deletes() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.insert_char('b');
        input.set_mode(InputVimMode::Normal);
        input.move_cursor_home();
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.text(), "b");
    }

    #[test]
    fn vim_input_a_appends() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.set_mode(InputVimMode::Normal);
        input.move_cursor_home();
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.mode(), InputVimMode::Insert);
        input.handle_input(&key_event(KeyCode::Char('b')));
        assert_eq!(input.text(), "ab");
    }

    #[test]
    fn vim_input_0_and_dollar() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.insert_char('b');
        input.set_mode(InputVimMode::Normal);
        input.move_cursor_end();
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('0'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.cursor(), 0);
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('$'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn vim_input_p_paste() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.insert_char('a');
        input.insert_char('b');
        input.move_cursor_home();
        input.handle_input(&ctrl_event('k'));
        assert_eq!(input.text(), "");
        input.set_mode(InputVimMode::Normal);
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.text(), "ab");
    }

    #[test]
    fn vim_input_u_undo() {
        let mut input = Input::new();
        input.set_vim_mode_enabled(true);
        input.set_mode(InputVimMode::Insert);
        input.handle_input(&key_event(KeyCode::Char('a')));
        input.handle_input(&key_event(KeyCode::Char('b')));
        input.set_mode(InputVimMode::Normal);
        input.handle_input(&Event::Key(KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::empty(),
        )));
        assert_eq!(input.text(), "a");
    }
}
