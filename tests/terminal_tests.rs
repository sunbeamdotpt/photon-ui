use photon_ui::terminal::{
    Terminal,
    TestTerminal,
};

#[test]
fn test_terminal_records_writes() {
    let mut term = TestTerminal::new(80, 24);
    term.write("hello").unwrap();
    term.write("world").unwrap();
    assert_eq!(
        term.written(),
        &vec!["hello".to_string(), "world".to_string()]
    );
}

#[test]
fn test_terminal_size() {
    let term = TestTerminal::new(100, 50);
    assert_eq!(term.size().unwrap(), (100, 50));
}

#[test]
fn test_terminal_cursor_moves() {
    let mut term = TestTerminal::new(80, 24);
    term.move_cursor(5, 10).unwrap();
    term.move_cursor(0, 0).unwrap();
    assert_eq!(term.cursor_moves(), &vec![(5, 10), (0, 0)]);
}

#[test]
fn test_terminal_cursor_hide_show() {
    let mut term = TestTerminal::new(80, 24);
    assert!(!term.is_cursor_hidden());
    term.hide_cursor().unwrap();
    assert!(term.is_cursor_hidden());
    term.show_cursor().unwrap();
    assert!(!term.is_cursor_hidden());
}

#[test]
fn test_terminal_start_stop() {
    let mut term = TestTerminal::new(80, 24);
    term.start().unwrap();
    term.stop().unwrap();
    // Should not panic
}
