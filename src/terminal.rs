use std::io;

macro_rules! try_io {
    ($expr:expr) => {
        match $expr {
            | Ok(v) => v,
            | Err(e) => return Err(e),
        }
    };
}

/// Abstraction over a terminal device.
///
/// Both real terminals ([`ProcessTerminal`]) and test doubles
/// ([`TestTerminal`]) implement this trait so the rest of the framework remains
/// agnostic to the underlying I/O mechanism.
pub trait Terminal {
    /// Enter raw mode, alternate screen, and hide the cursor.
    fn start(&mut self) -> io::Result<()>;
    /// Leave raw mode, alternate screen, and show the cursor.
    fn stop(&mut self) -> io::Result<()>;
    /// Write raw data to the terminal.
    fn write(&mut self, data: &str) -> io::Result<()>;
    /// Return the current terminal size as `(cols, rows)`.
    fn size(&self) -> io::Result<(u16, u16)>;
    /// Move the hardware cursor to `(row, col)`.
    fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()>;
    /// Hide the hardware cursor.
    fn hide_cursor(&mut self) -> io::Result<()>;
    /// Show the hardware cursor.
    fn show_cursor(&mut self) -> io::Result<()>;
}

/// In-memory terminal double for testing.
///
/// Records all writes, cursor moves, and cursor visibility changes so tests
/// can assert on the exact ANSI sequences emitted by the renderer.
pub struct TestTerminal {
    cols: u16,
    rows: u16,
    buffer: Vec<String>,
    cursor_moves: Vec<(u16, u16)>,
    cursor_hidden: bool,
}

impl TestTerminal {
    /// Create a new test terminal with the given dimensions.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            buffer: Vec::new(),
            cursor_moves: Vec::new(),
            cursor_hidden: false,
        }
    }

    /// All strings written via [`Terminal::write`] since creation.
    pub fn written(&self) -> &Vec<String> {
        &self.buffer
    }

    /// All cursor positions passed to [`Terminal::move_cursor`] since creation.
    pub fn cursor_moves(&self) -> &Vec<(u16, u16)> {
        &self.cursor_moves
    }

    /// Whether the cursor was most recently hidden.
    pub fn is_cursor_hidden(&self) -> bool {
        self.cursor_hidden
    }
}

impl Terminal for TestTerminal {
    fn start(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        self.buffer.push(data.to_string());
        Ok(())
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        Ok((self.cols, self.rows))
    }

    fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()> {
        self.cursor_moves.push((row, col));
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.cursor_hidden = true;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.cursor_hidden = false;
        Ok(())
    }
}

/// Real terminal backed by stdout.
///
/// Uses [`crossterm`] for raw-mode management, alternate screen, and cursor
/// control. When constructed via `new_test` (available under `#[cfg(test)]`),
/// it writes to an in-memory buffer and reports a fixed size of 80×24, which is
/// useful for unit-testing code paths that require a [`Terminal`] but do not
/// need a real TTY.
pub struct ProcessTerminal {
    stdout: Box<dyn std::io::Write>,
    is_tty: bool,
}

impl Default for ProcessTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTerminal {
    /// Create a terminal connected to the process stdout.
    pub fn new() -> Self {
        Self {
            stdout: Box::new(std::io::stdout()),
            is_tty: true,
        }
    }

    /// Create a test terminal that writes to an in-memory buffer.
    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            stdout: Box::new(Vec::new()),
            is_tty: false,
        }
    }
}

impl Terminal for ProcessTerminal {
    fn start(&mut self) -> io::Result<()> {
        if self.is_tty {
            try_io!(crossterm::terminal::enable_raw_mode());
        }
        try_io!(crossterm::execute!(
            self.stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        ));
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        try_io!(crossterm::execute!(
            self.stdout,
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        ));
        if self.is_tty {
            try_io!(crossterm::terminal::disable_raw_mode());
        }
        Ok(())
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        use std::io::Write;
        try_io!(self.stdout.write_all(data.as_bytes()));
        try_io!(self.stdout.flush());
        Ok(())
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        if self.is_tty {
            crossterm::terminal::size()
        } else {
            Ok((80, 24))
        }
    }

    fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()> {
        try_io!(crossterm::execute!(
            self.stdout,
            crossterm::cursor::MoveTo(col, row)
        ));
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        try_io!(crossterm::execute!(self.stdout, crossterm::cursor::Hide));
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        try_io!(crossterm::execute!(self.stdout, crossterm::cursor::Show));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_terminal_all_methods() {
        let mut term = ProcessTerminal::new_test();
        term.start().unwrap();
        term.write("hello").unwrap();
        assert_eq!(term.size().unwrap(), (80, 24));
        term.move_cursor(5, 10).unwrap();
        term.hide_cursor().unwrap();
        term.show_cursor().unwrap();
        term.stop().unwrap();
    }

    #[test]
    fn process_terminal_new_does_not_panic() {
        let _term = ProcessTerminal::new();
    }
}
