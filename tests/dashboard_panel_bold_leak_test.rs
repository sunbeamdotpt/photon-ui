use std::{
    cell::RefCell,
    io,
    rc::Rc,
};

use photon_ui::{
    Component,
    TUI,
    Terminal,
    TestTerminal,
    components::Panel,
    theme::Theme,
};

/// Wrapper that lets us keep a reference to a [`TestTerminal`] after handing it
/// to a [`TUI`].
struct SharedTerminal {
    inner: Rc<RefCell<TestTerminal>>,
}

impl Terminal for SharedTerminal {
    fn size(&self) -> io::Result<(u16, u16)> {
        self.inner.borrow().size()
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        self.inner.borrow_mut().write(data)
    }

    fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()> {
        self.inner.borrow_mut().move_cursor(row, col)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().show_cursor()
    }

    fn start(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().start()
    }

    fn stop(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().stop()
    }
}

/// Regression: plain content lines following an explicit bold span must not
/// inherit bold styling after compositing and rendering.
#[test]
fn panel_plain_line_after_bold_span_is_not_bold() {
    Theme::with(Theme::Dark, || {
        let panel = Panel::new().title("Info").lines(vec![
            "\x1b[1m\x1b[97mPhoton UI v0.1.0\x1b[0m".into(),
            "A Rust TUI library".into(),
            "\x1b[31mBuilt with ♥\x1b[0m".into(),
        ]);
        let rendered = panel.render(40).unwrap();

        // Find the plain content line.
        let plain_line = rendered
            .lines
            .iter()
            .find(|l| l.contains("A Rust TUI library"))
            .expect("plain content line should exist");

        // After the explicit reset in the previous line, this line should not
        // contain any bold-on code except as part of the border (which is not
        // bold). The line is built from plain text, so the only ANSI codes
        // should be the border color prefixes/suffixes.
        assert!(
            !plain_line.contains("\x1b[1m"),
            "plain line should not contain bold-on code: {:?}",
            plain_line.replace('\x1b', "\\x1b")
        );
    });
}

/// Full pipeline: render the same Panel through a TUI and verify the terminal
/// output does not leak bold onto the plain line.
#[test]
fn tui_panel_plain_line_after_bold_span_is_not_bold() {
    Theme::with(Theme::Dark, || {
        let term = Rc::new(RefCell::new(TestTerminal::new(40, 10)));
        let mut tui = TUI::new(Box::new(SharedTerminal {
            inner: Rc::clone(&term),
        }));
        tui.mount(Box::new(Panel::new().title("Info").lines(vec![
            "\x1b[1m\x1b[97mPhoton UI v0.1.0\x1b[0m".into(),
            "A Rust TUI library".into(),
            "\x1b[31mBuilt with ♥\x1b[0m".into(),
        ])));
        tui.render_frame().unwrap();

        let written = term.borrow().written().join("");

        // The plain line should not contain a bold-on sequence. Split by lines
        // and look for the one containing "A Rust".
        let plain_segment = written
            .split("\r\n")
            .find(|s| s.contains("A Rust TUI library"))
            .expect("plain line should be written");

        assert!(
            !plain_segment.contains("\x1b[1m"),
            "terminal output for plain line should not contain bold-on: {:?}",
            plain_segment.replace('\x1b', "\\x1b")
        );
    });
}
