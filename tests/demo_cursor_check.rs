use std::{
    cell::RefCell,
    io,
    rc::Rc,
};

use photon_ui::{
    TUI,
    Terminal,
    TestTerminal,
    components::{
        Input,
        Spacer,
        Text,
    },
    theme::Theme,
    utils::EDIT_CURSOR,
};

struct SharedTerm(Rc<RefCell<TestTerminal>>);

impl Terminal for SharedTerm {
    fn start(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        self.0.borrow_mut().write(data)
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        self.0.borrow().size()
    }

    fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()> {
        self.0.borrow_mut().move_cursor(row, col)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.0.borrow_mut().hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.0.borrow_mut().show_cursor()
    }
}

#[test]
fn demo_page_two_input_renders_cursor() {
    Theme::set(Theme::Dark);
    let term = Rc::new(RefCell::new(TestTerminal::new(80, 24)));
    let mut tui = TUI::new(Box::new(SharedTerm(term.clone())));

    // Mimic demo page 2 mount order
    tui.mount(Box::new(Text::new(
        " Photon UI Demo  |  Page 2/6  |  1-6=pages  Tab=focus  q=quit ",
        0,
        0,
    )));
    tui.mount(Box::new(Spacer::new(1)));
    tui.mount(Box::new(Text::new(
        "Input: Emacs mode, orange block cursor",
        0,
        0,
    )));

    let mut input = Input::new();
    input.set_text("sdasdasdasdasdasdasdasdasdasd");
    tui.mount(Box::new(input));

    tui.render_frame().unwrap();
    let output = term.borrow().written().concat();
    assert!(
        output.contains(EDIT_CURSOR),
        "expected block cursor in rendered output:\n{}",
        output
    );
}
