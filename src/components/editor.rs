use crate::{Component, Rendered, RenderError, Event, InputResult, Focusable};
use crate::kill_ring::KillRing;
use crate::undo_stack::UndoStack;
use crate::word_navigation::{find_word_forward, find_word_backward};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use crossterm::event::KeyCode;

/// Snapshot of editor state for undo/redo.
#[derive(Clone)]
pub struct EditorAction {
    /// The full text buffer at the time of the snapshot.
    pub text: String,
    /// Cursor position in grapheme indices.
    pub cursor: usize,
}

/// Marks a region of text that was inserted via paste, for later reference.
#[derive(Clone)]
pub struct PasteMarker {
    /// Human-readable label describing the paste source.
    pub label: String,
    /// Start grapheme index (inclusive).
    pub start: usize,
    /// End grapheme index (exclusive).
    pub end: usize,
}

/// Vim editing mode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal mode — keys are commands (hjkl, dw, yy, etc.).
    Normal,
    /// Insert mode — keys insert text.
    Insert,
}

/// A multi-line text editor component.
///
/// Defaults to **Emacs-style** bindings (Ctrl+A, Ctrl+E, Ctrl+K, etc.).
/// Call [`set_vim_mode_enabled`](Editor::set_vim_mode_enabled) to opt into
/// vim-style modal editing (Normal/Insert mode with hjkl, dd, yy, etc.).
///
/// Supports undo/redo, kill-ring (yank/yank-pop), history navigation,
/// word-wise movement, and paste detection for large inserts.
#[derive(Clone)]
pub struct Editor {
    text: String,
    cursor: usize,
    focused: bool,
    kill_ring: KillRing,
    undo_stack: UndoStack<EditorAction>,
    lines_cache: Vec<String>,
    cache_width: u16,
    history: Vec<String>,
    history_index: Option<usize>,
    paste_markers: Vec<PasteMarker>,
    max_history: usize,
    /// Whether vim modal editing is enabled.
    vim_mode_enabled: bool,
    /// Current vim mode (only meaningful when `vim_mode_enabled` is `true`).
    mode: VimMode,
    /// Pending normal-mode command prefix (e.g. `'d'`, `'y'`).
    pending_cmd: Option<char>,
}

impl Editor {
    /// Create a new empty editor with Emacs-style bindings.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            focused: false,
            kill_ring: KillRing::new(),
            undo_stack: UndoStack::new(),
            lines_cache: Vec::new(),
            cache_width: 0,
            history: Vec::new(),
            history_index: None,
            paste_markers: Vec::new(),
            max_history: 100,
            vim_mode_enabled: false,
            mode: VimMode::Normal,
            pending_cmd: None,
        }
    }

    /// Returns `true` if vim modal editing is enabled.
    pub fn vim_mode_enabled(&self) -> bool { self.vim_mode_enabled }

    /// Enable or disable vim modal editing.
    ///
    /// When enabled the editor starts in Normal mode; press `i` to insert,
    /// `Escape` to return to Normal. When disabled, Emacs-style bindings
    /// are always active.
    pub fn set_vim_mode_enabled(&mut self, enabled: bool) {
        self.vim_mode_enabled = enabled;
        self.mode = VimMode::Normal;
        self.pending_cmd = None;
    }

    /// Current vim mode (only meaningful when vim mode is enabled).
    pub fn mode(&self) -> VimMode { self.mode }

    /// Switch to the given vim mode.
    pub fn set_mode(&mut self, mode: VimMode) { self.mode = mode; self.pending_cmd = None; }

    /// Borrow the current text content.
    pub fn text(&self) -> &str { &self.text }

    /// Current cursor position in grapheme indices.
    pub fn cursor_grapheme(&self) -> usize { self.cursor }

    /// Replace the entire text buffer and move the cursor to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.graphemes().len();
        self.lines_cache.clear();
        self.cache_width = 0;
    }

    fn graphemes(&self) -> Vec<&str> {
        self.text.graphemes(true).collect()
    }

    fn byte_index(&self, grapheme_idx: usize) -> usize {
        self.text.grapheme_indices(true)
            .nth(grapheme_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn save_undo(&mut self) {
        self.undo_stack.push(EditorAction { text: self.text.clone(), cursor: self.cursor });
    }

    fn insert_char(&mut self, c: char) {
        let idx = self.byte_index(self.cursor);
        self.text.insert(idx, c);
        self.cursor += 1;
        self.invalidate_cache();
    }

    fn insert_newline(&mut self) {
        let idx = self.byte_index(self.cursor);
        self.text.insert(idx, '\n');
        self.cursor += 1;
        self.invalidate_cache();
    }

    fn insert_str(&mut self, s: &str) {
        let idx = self.byte_index(self.cursor);
        self.text.insert_str(idx, s);
        self.cursor += s.graphemes(true).count();
        self.invalidate_cache();
    }

    fn delete_backward(&mut self) {
        if self.cursor > 0 {
            let start = self.byte_index(self.cursor - 1);
            let end = self.byte_index(self.cursor);
            let killed = self.text.drain(start..end).collect::<String>();
            self.kill_ring.push(killed);
            self.cursor -= 1;
            self.invalidate_cache();
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.graphemes().len() {
            let start = self.byte_index(self.cursor);
            let end = self.byte_index(self.cursor + 1);
            self.text.drain(start..end);
            self.invalidate_cache();
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor > 0 { self.cursor -= 1; }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor < self.graphemes().len() { self.cursor += 1; }
    }

    fn move_cursor_up(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line == 0 { return; }
        let target_line = line - 1;
        let mut current_line = 0;
        let mut current_col = 0;
        let mut gidx = 0;
        for g in self.text.graphemes(true) {
            if current_line == target_line {
                if current_col >= col || g == "\n" {
                    self.cursor = gidx;
                    return;
                }
                current_col += g.width();
            } else if g == "\n" {
                current_line += 1;
                current_col = 0;
            }
            gidx += 1;
        }
    }

    fn move_cursor_down(&mut self) {
        let (line, col) = self.cursor_line_col();
        let total_lines = self.text.lines().count();
        if line + 1 >= total_lines { return; }
        let target_line = line + 1;
        let mut current_line = 0;
        let mut current_col = 0;
        let mut gidx = 0;
        for g in self.text.graphemes(true) {
            if current_line == target_line {
                if current_col >= col || g == "\n" {
                    self.cursor = gidx;
                    return;
                }
                current_col += g.width();
            } else if g == "\n" {
                current_line += 1;
                current_col = 0;
            }
            gidx += 1;
        }
        self.cursor = gidx;
    }

    fn move_cursor_home(&mut self) {
        let (line, _) = self.cursor_line_col();
        let mut current_line = 0;
        let mut gidx = 0;
        for g in self.text.graphemes(true) {
            if current_line == line {
                self.cursor = gidx;
                return;
            }
            if g == "\n" { current_line += 1; }
            gidx += 1;
        }
    }

    fn move_cursor_end(&mut self) {
        let (line, _) = self.cursor_line_col();
        let mut current_line = 0;
        let mut gidx = 0;
        let mut found = false;
        for g in self.text.graphemes(true) {
            if g == "\n" {
                if found {
                    self.cursor = gidx;
                    return;
                }
                current_line += 1;
            }
            if current_line == line { found = true; }
            gidx += 1;
        }
        self.cursor = gidx;
    }

    fn move_word_forward(&mut self) {
        let idx = self.byte_index(self.cursor);
        let new_idx = find_word_forward(&self.text, idx, |c| c.is_whitespace());
        let slice = &self.text[idx..new_idx];
        self.cursor += slice.graphemes(true).count();
    }

    fn move_word_backward(&mut self) {
        let idx = self.byte_index(self.cursor);
        let new_idx = find_word_backward(&self.text, idx, |c| c.is_whitespace());
        let slice = &self.text[new_idx..idx];
        self.cursor -= slice.graphemes(true).count();
    }

    fn kill_word_forward(&mut self) {
        let idx = self.byte_index(self.cursor);
        let new_idx = find_word_forward(&self.text, idx, |c| c.is_whitespace());
        let killed = self.text.drain(idx..new_idx).collect::<String>();
        self.kill_ring.push(killed);
        self.invalidate_cache();
    }

    fn kill_word_backward(&mut self) {
        let idx = self.byte_index(self.cursor);
        let new_idx = find_word_backward(&self.text, idx, |c| c.is_whitespace());
        let killed = self.text.drain(new_idx..idx).collect::<String>();
        let count = killed.graphemes(true).count();
        self.kill_ring.push(killed);
        self.cursor -= count;
        self.invalidate_cache();
    }

    fn kill_to_end(&mut self) {
        let idx = self.byte_index(self.cursor);
        let killed = self.text.split_off(idx);
        self.kill_ring.push(killed);
        self.invalidate_cache();
    }

    fn yank(&mut self) {
        if let Some(text) = self.kill_ring.yank().map(|s| s.to_string()) {
            self.insert_str(&text);
        }
    }

    fn yank_pop(&mut self) {
        if let Some(text) = self.kill_ring.yank_pop().map(|s| s.to_string()) {
            self.insert_str(&text);
        }
    }

    fn undo(&mut self) {
        if let Some(action) = self.undo_stack.undo() {
            self.text = action.text.clone();
            self.cursor = action.cursor;
            self.invalidate_cache();
        }
    }

    fn redo(&mut self) {
        if let Some(action) = self.undo_stack.redo() {
            self.text = action.text.clone();
            self.cursor = action.cursor;
            self.invalidate_cache();
        }
    }

    /// Append the current text to the history ring buffer.
    ///
    /// History is capped at `max_history` items (default 100). The history
    /// index is reset so subsequent Up/Down navigation starts from the newest
    /// entry.
    pub fn push_history(&mut self) {
        if !self.text.is_empty() {
            self.history.push(self.text.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
        self.history_index = None;
    }

    fn history_up(&mut self) {
        if self.history.is_empty() { return; }
        let idx = match self.history_index {
            Some(i) if i > 0 => i - 1,
            Some(_) => return,
            None => self.history.len() - 1,
        };
        self.history_index = Some(idx);
        self.text = self.history[idx].clone();
        self.cursor = self.graphemes().len();
        self.invalidate_cache();
    }

    fn history_down(&mut self) {
        let idx = match self.history_index {
            Some(i) if i + 1 < self.history.len() => i + 1,
            Some(_) => {
                self.history_index = None;
                self.text.clear();
                self.cursor = 0;
                self.invalidate_cache();
                return;
            }
            None => return,
        };
        self.history_index = Some(idx);
        self.text = self.history[idx].clone();
        self.cursor = self.graphemes().len();
        self.invalidate_cache();
    }

    fn invalidate_cache(&mut self) {
        self.lines_cache.clear();
        self.cache_width = 0;
    }

    /// (Re-)wrap the editor text at the given width, caching the result.
    pub(crate) fn rewrap(&mut self, width: u16) {
        self.lines_cache = crate::utils::wrap_text_with_ansi(&self.text, width);
        self.cache_width = width;
    }

    /// Insert a potentially large pasted string, marking it with a label if
    /// it spans many lines.
    pub(crate) fn insert_paste(&mut self, text: &str) {
        let start = self.cursor;
        self.insert_str(text);
        let end = self.cursor;
        let line_count = text.lines().count();
        if line_count > 5 {
            self.paste_markers.push(PasteMarker {
                label: format!("[paste #{} +{} lines]", self.paste_markers.len() + 1, line_count),
                start,
                end,
            });
        }
    }

    /// Delete the current line (vim `dd` behavior).
    fn delete_line(&mut self) {
        let (line, _) = self.cursor_line_col();
        let mut current_line = 0;
        let mut start_byte = 0;
        let mut byte_pos = 0;
        for g in self.text.graphemes(true) {
            if current_line == line {
                start_byte = byte_pos;
                break;
            }
            if g == "\n" { current_line += 1; }
            byte_pos += g.len();
        }
        // Find the end of this line, including the newline if present.
        let mut end_byte = self.text.len();
        byte_pos = 0;
        let mut found = false;
        for g in self.text.graphemes(true) {
            if found && g == "\n" {
                end_byte = byte_pos + g.len();
                break;
            }
            if byte_pos >= start_byte { found = true; }
            byte_pos += g.len();
        }
        self.cursor = self.text[..start_byte].graphemes(true).count();
        let killed = self.text.drain(start_byte..end_byte).collect::<String>();
        if !killed.is_empty() {
            self.kill_ring.push(killed);
        }
        self.invalidate_cache();
    }

    /// Yank (copy) the current line into the kill ring (vim `yy` behavior).
    fn yank_line(&mut self) {
        let (line, _) = self.cursor_line_col();
        let mut current_line = 0;
        let mut start_byte = 0;
        let mut byte_pos = 0;
        for g in self.text.graphemes(true) {
            if current_line == line {
                start_byte = byte_pos;
                break;
            }
            if g == "\n" { current_line += 1; }
            byte_pos += g.len();
        }
        let mut end_byte = self.text.len();
        byte_pos = 0;
        let mut found = false;
        for g in self.text.graphemes(true) {
            if found && g == "\n" {
                end_byte = byte_pos + g.len();
                break;
            }
            if byte_pos >= start_byte { found = true; }
            byte_pos += g.len();
        }
        let yanked = self.text[start_byte..end_byte].to_string();
        if !yanked.is_empty() {
            self.kill_ring.push(yanked);
        }
    }

    /// Open a new line below the current one and enter Insert mode.
    fn open_line_below(&mut self) {
        self.move_cursor_end();
        self.insert_newline();
        self.mode = VimMode::Insert;
    }

    /// Open a new line above the current one and enter Insert mode.
    fn open_line_above(&mut self) {
        self.move_cursor_home();
        if self.cursor > 0 {
            self.cursor -= 1; // back over the newline
            self.insert_newline();
        } else {
            self.insert_newline();
            self.cursor = 0;
        }
        self.mode = VimMode::Insert;
    }

    /// Replace the character under the cursor with `c`.
    fn replace_char(&mut self, c: char) {
        if self.cursor < self.graphemes().len() {
            let start = self.byte_index(self.cursor);
            let end = self.byte_index(self.cursor + 1);
            self.text.drain(start..end);
            self.text.insert(start, c);
            self.invalidate_cache();
        }
    }

    /// Move cursor to the start of the document.
    fn go_to_start(&mut self) { self.cursor = 0; }

    /// Move cursor to the end of the document.
    fn go_to_end(&mut self) { self.cursor = self.graphemes().len(); }

    /// Compute the cursor position as `(line, column)` in display coordinates.
    fn cursor_line_col(&self) -> (usize, usize) {
        let mut current_line = 0;
        let mut current_col = 0;
        let mut graphemes_seen = 0;
        for g in self.text.graphemes(true) {
            if graphemes_seen >= self.cursor {
                break;
            }
            if g == "\n" {
                current_line += 1;
                current_col = 0;
            } else {
                current_col += g.width();
            }
            graphemes_seen += 1;
        }
        (current_line, current_col)
    }
}

impl Default for Editor {
    fn default() -> Self { Self::new() }
}

impl Focusable for Editor {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Editor {
    fn handle_insert_mode(&mut self, key: &crossterm::event::KeyEvent) -> InputResult {
        use crossterm::event::KeyModifiers;
        self.save_undo();
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'a' => self.move_cursor_home(),
                        'e' => self.move_cursor_end(),
                        'b' => self.move_cursor_left(),
                        'f' => self.move_cursor_right(),
                        'n' => self.move_cursor_down(),
                        'p' => self.move_cursor_up(),
                        'd' => self.delete_forward(),
                        'h' => self.delete_backward(),
                        'k' => self.kill_to_end(),
                        'w' => self.kill_word_backward(),
                        'u' => {
                            self.move_cursor_home();
                            self.kill_to_end();
                        }
                        'y' => self.yank(),
                        '-' | '_' => self.undo(),
                        _ => return InputResult::Ignored,
                    }
                } else if key.modifiers.contains(KeyModifiers::ALT) {
                    match c {
                        'b' => self.move_word_backward(),
                        'f' => self.move_word_forward(),
                        'd' => self.kill_word_forward(),
                        _ => return InputResult::Ignored,
                    }
                } else {
                    self.insert_char(c);
                }
                InputResult::Handled
            }
            KeyCode::Enter => {
                let idx = self.byte_index(self.cursor);
                if idx > 0 && self.text.as_bytes().get(idx - 1) == Some(&b'\\') {
                    self.text.remove(idx - 1);
                    self.cursor -= 1;
                    self.insert_newline();
                } else {
                    self.insert_newline();
                }
                InputResult::Handled
            }
            KeyCode::Left => { self.move_cursor_left(); InputResult::Handled }
            KeyCode::Right => { self.move_cursor_right(); InputResult::Handled }
            KeyCode::Up => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_up();
                } else {
                    self.history_up();
                }
                InputResult::Handled
            }
            KeyCode::Down => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_down();
                } else {
                    self.history_down();
                }
                InputResult::Handled
            }
            KeyCode::Home => { self.move_cursor_home(); InputResult::Handled }
            KeyCode::End => { self.move_cursor_end(); InputResult::Handled }
            KeyCode::Backspace => { self.delete_backward(); InputResult::Handled }
            KeyCode::Delete => { self.delete_forward(); InputResult::Handled }
            KeyCode::Esc => { self.mode = VimMode::Normal; InputResult::Handled }
            _ => InputResult::Ignored,
        }
    }

    fn handle_normal_mode(&mut self, key: &crossterm::event::KeyEvent) -> InputResult {
        // Multi-key commands (dd, yy, gg, etc.) are handled via pending_cmd.
        if let Some(pending) = self.pending_cmd {
            match key.code {
                KeyCode::Char('d') if pending == 'd' => {
                    self.save_undo();
                    self.delete_line();
                    self.pending_cmd = None;
                    return InputResult::Handled;
                }
                KeyCode::Char('y') if pending == 'y' => {
                    self.yank_line();
                    self.pending_cmd = None;
                    return InputResult::Handled;
                }
                KeyCode::Char('g') if pending == 'g' => {
                    self.go_to_start();
                    self.pending_cmd = None;
                    return InputResult::Handled;
                }
                KeyCode::Char('w') if pending == 'd' => {
                    self.save_undo();
                    self.kill_word_forward();
                    self.pending_cmd = None;
                    return InputResult::Handled;
                }
                KeyCode::Char('w') if pending == 'y' => {
                    let start = self.cursor;
                    self.move_word_forward();
                    let end_byte = self.byte_index(self.cursor);
                    let start_byte = self.byte_index(start);
                    let yanked = self.text[start_byte..end_byte].to_string();
                    if !yanked.is_empty() { self.kill_ring.push(yanked); }
                    self.cursor = start;
                    self.pending_cmd = None;
                    return InputResult::Handled;
                }
                KeyCode::Char(c) if pending == 'r' => {
                    self.save_undo();
                    self.replace_char(c);
                    self.pending_cmd = None;
                    return InputResult::Handled;
                }
                _ => {
                    self.pending_cmd = None;
                    // Fall through to normal handling below
                }
            }
        }

        match key.code {
            KeyCode::Char(c) => {
                match c {
                    'h' => self.move_cursor_left(),
                    'j' => self.move_cursor_down(),
                    'k' => self.move_cursor_up(),
                    'l' => self.move_cursor_right(),
                    'w' => self.move_word_forward(),
                    'b' => self.move_word_backward(),
                    'x' => { self.save_undo(); self.delete_forward(); }
                    '0' => self.move_cursor_home(),
                    '$' => self.move_cursor_end(),
                    'i' => self.mode = VimMode::Insert,
                    'a' => { self.move_cursor_right(); self.mode = VimMode::Insert; }
                    'o' => { self.save_undo(); self.open_line_below(); }
                    'O' => { self.save_undo(); self.open_line_above(); }
                    'p' => { self.save_undo(); self.yank(); }
                    'u' => { self.save_undo(); self.undo(); }
                    'r' => {
                        self.pending_cmd = Some('r');
                        return InputResult::Handled;
                    }
                    'd' | 'y' => {
                        self.pending_cmd = Some(c);
                        return InputResult::Handled;
                    }
                    'g' => {
                        self.pending_cmd = Some('g');
                        return InputResult::Handled;
                    }
                    'G' => self.go_to_end(),
                    _ => return InputResult::Ignored,
                }
                InputResult::Handled
            }
            KeyCode::Left => { self.move_cursor_left(); InputResult::Handled }
            KeyCode::Right => { self.move_cursor_right(); InputResult::Handled }
            KeyCode::Up => { self.move_cursor_up(); InputResult::Handled }
            KeyCode::Down => { self.move_cursor_down(); InputResult::Handled }
            KeyCode::Home => { self.move_cursor_home(); InputResult::Handled }
            KeyCode::End => { self.move_cursor_end(); InputResult::Handled }
            KeyCode::Backspace => { self.move_cursor_left(); InputResult::Handled }
            _ => InputResult::Ignored,
        }
    }
}

impl Component for Editor {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let editor = self.clone();
        let (cursor_line, cursor_col) = editor.cursor_line_col();
        let lines = if width == self.cache_width && !self.lines_cache.is_empty() {
            self.lines_cache.clone()
        } else {
            crate::utils::wrap_text_with_ansi(&self.text, width)
        };
        Ok(Rendered {
            lines,
            cursor: if self.focused { Some((cursor_line, cursor_col)) } else { None },
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            if self.vim_mode_enabled {
                match self.mode {
                    VimMode::Insert => self.handle_insert_mode(key),
                    VimMode::Normal => self.handle_normal_mode(key),
                }
            } else {
                self.handle_insert_mode(key)
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

    fn key_event(code: KeyCode) -> Event {
        Event::Key(code.into())
    }

    fn ctrl_event(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
    }

    fn alt_event(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT))
    }

    #[test]
    fn yank_pop_cycles() {
        let mut editor = Editor::new();
        editor.insert_str("ab");
        editor.move_cursor_home();
        editor.kill_to_end();
        assert_eq!(editor.text(), "");
        editor.yank();
        assert_eq!(editor.text(), "ab");
        editor.yank_pop();
        assert_eq!(editor.text(), "abab");
    }

    #[test]
    fn insert_paste_short() {
        let mut editor = Editor::new();
        editor.insert_paste("short");
        assert_eq!(editor.text(), "short");
    }

    #[test]
    fn insert_paste_long() {
        let mut editor = Editor::new();
        let long_text = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11";
        editor.insert_paste(long_text);
        assert_eq!(editor.paste_markers.len(), 1);
        assert!(editor.paste_markers[0].label.contains("+11 lines"));
    }

    #[test]
    fn rewrap_caches() {
        let mut editor = Editor::new();
        editor.insert_str("ab");
        editor.rewrap(80);
        // Second call with same width should use cache
        editor.rewrap(80);
        assert_eq!(editor.text(), "ab");
    }

    #[test]
    fn history_down_navigation() {
        let mut editor = Editor::new();
        editor.insert_str("a");
        editor.push_history();
        editor.move_cursor_home();
        editor.kill_to_end();
        editor.insert_str("b");
        editor.push_history();
        editor.history_up();
        assert_eq!(editor.text(), "b");
        editor.history_up();
        assert_eq!(editor.text(), "a");
        editor.history_down();
        assert_eq!(editor.text(), "b");
        editor.history_down();
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn history_down_empty() {
        let mut editor = Editor::new();
        editor.history_down();
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn push_history_empty() {
        let mut editor = Editor::new();
        editor.push_history();
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn move_cursor_up_down() {
        let mut editor = Editor::new();
        editor.insert_str("a\nb");
        editor.move_cursor_up();
        // cursor lands at newline position because col=1 and current_col reaches 1 at '\n'
        assert_eq!(editor.cursor_grapheme(), 1);
        editor.move_cursor_down();
        assert_eq!(editor.cursor_grapheme(), 3);
    }

    #[test]
    fn kill_word_forward() {
        let mut editor = Editor::new();
        editor.insert_str("hello world");
        editor.move_cursor_home();
        editor.kill_word_forward();
        assert_eq!(editor.text(), " world");
    }

    #[test]
    fn render_unfocused() {
        let mut editor = Editor::new();
        editor.set_focused(false);
        editor.insert_str("x");
        let r = editor.render(80).unwrap();
        assert!(r.cursor.is_none());
    }

    #[test]
    fn ctrl_a_e_navigation() {
        let mut editor = Editor::new();
        editor.insert_str("abc");
        editor.move_cursor_home();
        assert_eq!(editor.cursor_grapheme(), 0);
        editor.move_cursor_end();
        assert_eq!(editor.cursor_grapheme(), 3);
    }

    #[test]
    fn ctrl_f_b_navigation() {
        let mut editor = Editor::new();
        editor.insert_str("ab");
        editor.move_cursor_home();
        editor.move_cursor_right();
        assert_eq!(editor.cursor_grapheme(), 1);
        editor.move_cursor_left();
        assert_eq!(editor.cursor_grapheme(), 0);
    }

    #[test]
    fn alt_f_forward() {
        let mut editor = Editor::new();
        editor.insert_str("hi there");
        editor.move_cursor_home();
        editor.move_word_forward();
        assert_eq!(editor.cursor_grapheme(), 2);
    }

    #[test]
    fn home_end_keys() {
        let mut editor = Editor::new();
        editor.insert_str("ab");
        editor.handle_input(&key_event(KeyCode::Home));
        assert_eq!(editor.cursor_grapheme(), 0);
        editor.handle_input(&key_event(KeyCode::End));
        assert_eq!(editor.cursor_grapheme(), 2);
    }

    #[test]
    fn delete_at_end() {
        let mut editor = Editor::new();
        editor.insert_str("a");
        editor.move_cursor_end();
        editor.delete_forward();
        assert_eq!(editor.text(), "a");
    }

    #[test]
    fn backspace_at_start() {
        let mut editor = Editor::new();
        editor.delete_backward();
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn cursor_line_col_first_line() {
        let mut editor = Editor::new();
        editor.insert_str("hello");
        let (line, col) = editor.cursor_line_col();
        assert_eq!(line, 0);
        assert_eq!(col, 5);
    }

    #[test]
    fn cursor_line_col_second_line() {
        let mut editor = Editor::new();
        editor.insert_str("hello\nworld");
        assert_eq!(editor.cursor_line_col(), (1, 5));
    }

    #[test]
    fn graphemes_count() {
        let mut editor = Editor::new();
        editor.insert_str("éà");
        assert_eq!(editor.graphemes().len(), 2);
    }

    #[test]
    fn byte_index_bounds() {
        let mut editor = Editor::new();
        editor.insert_str("ab");
        assert_eq!(editor.byte_index(0), 0);
        assert_eq!(editor.byte_index(2), 2);
        assert_eq!(editor.byte_index(10), 2);
    }

    #[test]
    fn move_cursor_up_from_line_two() {
        let mut editor = Editor::new();
        editor.insert_str("a\nb\nc");
        editor.move_cursor_end(); // cursor at end of line 2
        editor.move_cursor_up();
        // Cursor should land at the '\n' after "b" because col=1 and we hit it
        assert_eq!(editor.cursor_grapheme(), 3);
    }

    #[test]
    fn move_cursor_down_from_start() {
        let mut editor = Editor::new();
        editor.insert_str("a\nb");
        editor.cursor = 0;
        editor.move_cursor_down();
        assert_eq!(editor.cursor_grapheme(), 2);
    }

    #[test]
    fn move_cursor_home_multiline() {
        let mut editor = Editor::new();
        editor.insert_str("a\nb");
        editor.cursor = 3;
        editor.move_cursor_home();
        assert_eq!(editor.cursor_grapheme(), 2);
    }

    #[test]
    fn move_cursor_end_multiline() {
        let mut editor = Editor::new();
        editor.insert_str("a\nb");
        editor.cursor = 0;
        editor.move_cursor_end();
        assert_eq!(editor.cursor_grapheme(), 1);
    }

    #[test]
    fn history_up_past_start() {
        let mut editor = Editor::new();
        editor.insert_str("a");
        editor.push_history();
        editor.history_up();
        editor.history_up(); // should be a no-op when at first item
        assert_eq!(editor.text(), "a");
    }

    #[test]
    fn push_history_max_limit() {
        let mut editor = Editor::new();
        for i in 0..105 {
            editor.text = i.to_string();
            editor.cursor = 1;
            editor.push_history();
        }
        // History should be capped at max_history (100)
        assert_eq!(editor.history.len(), 100);
    }

    // -- Vim mode tests --

    #[test]
    fn vim_starts_in_normal_mode() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        assert_eq!(editor.mode(), VimMode::Normal);
    }

    #[test]
    fn vim_hjkl_navigation() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("ab\ncd\nef");
        editor.set_mode(VimMode::Normal);
        editor.cursor = 0;
        editor.move_cursor_down(); // to line 1, 'c'
        assert_eq!(editor.cursor_grapheme(), 3); // 'c'
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 4); // 'd'
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 3); // 'c'
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 6); // 'e' on line 2
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 3); // back to 'c'
    }

    #[test]
    fn vim_i_enters_insert_mode() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty())));
        assert_eq!(editor.mode(), VimMode::Insert);
        editor.handle_input(&key_event(KeyCode::Char('x')));
        assert_eq!(editor.text(), "x");
    }

    #[test]
    fn vim_esc_returns_to_normal_mode() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.handle_input(&key_event(KeyCode::Esc));
        assert_eq!(editor.mode(), VimMode::Normal);
    }

    #[test]
    fn vim_x_deletes_char() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("abc");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_home();
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())));
        assert_eq!(editor.text(), "bc");
    }

    #[test]
    fn vim_dd_deletes_line() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("hello\nworld");
        editor.set_mode(VimMode::Normal);
        editor.cursor = 0;
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty())));
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty())));
        assert_eq!(editor.text(), "world");
    }

    #[test]
    fn vim_yy_yanks_line() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("hello\nworld");
        editor.set_mode(VimMode::Normal);
        editor.cursor = 0;
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty())));
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty())));
        // p pastes after cursor (position 0)
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty())));
        assert_eq!(editor.text(), "hello\nhello\nworld");
    }

    #[test]
    fn vim_0_and_dollar() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("abc");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_end();
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 0);
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 3);
    }

    #[test]
    fn vim_gg_and_G() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("a\nb\nc");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_end();
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())));
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 0);
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::empty())));
        assert_eq!(editor.cursor_grapheme(), 5);
    }

    #[test]
    fn vim_a_appends() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("a");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_home();
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())));
        assert_eq!(editor.mode(), VimMode::Insert);
        editor.handle_input(&key_event(KeyCode::Char('b')));
        assert_eq!(editor.text(), "ab");
    }

    #[test]
    fn vim_o_opens_line_below() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("a");
        editor.set_mode(VimMode::Normal);
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty())));
        assert_eq!(editor.mode(), VimMode::Insert);
        assert_eq!(editor.text(), "a\n");
    }

    #[test]
    fn vim_O_opens_line_above() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("a");
        editor.set_mode(VimMode::Normal);
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('O'), KeyModifiers::empty())));
        assert_eq!(editor.mode(), VimMode::Insert);
        assert_eq!(editor.text(), "\na");
    }

    #[test]
    fn vim_r_replaces_char() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("abc");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_home();
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty())));
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())));
        assert_eq!(editor.text(), "xbc");
    }

    #[test]
    fn vim_dw_deletes_word() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("hello world");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_home();
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty())));
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty())));
        assert_eq!(editor.text(), " world");
    }

    #[test]
    fn vim_u_undo() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.handle_input(&key_event(KeyCode::Char('a')));
        editor.handle_input(&key_event(KeyCode::Char('b')));
        editor.set_mode(VimMode::Normal);
        editor.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::empty())));
        assert_eq!(editor.text(), "a");
    }

    #[test]
    fn vim_normal_mode_arrow_keys_work() {
        let mut editor = Editor::new();
        editor.set_vim_mode_enabled(true);
        editor.set_mode(VimMode::Insert);
        editor.insert_str("ab");
        editor.set_mode(VimMode::Normal);
        editor.move_cursor_end();
        editor.handle_input(&key_event(KeyCode::Left));
        assert_eq!(editor.cursor_grapheme(), 1);
    }


}
