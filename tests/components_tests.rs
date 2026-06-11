use photon_ui::{
    Component,
    InputResult,
    components::{
        Box as BoxComp,
        CancellableLoader,
        Loader,
        Spacer,
        Text,
        TruncatedText,
    },
    events::Event,
};

#[test]
fn spacer_renders_empty_lines() {
    let s = Spacer::new(3);
    let r = s.render(80).unwrap();
    assert_eq!(r.lines.len(), 3);
    assert!(r.lines.iter().all(|l| l.is_empty()));
}

#[test]
fn text_renders_with_padding() {
    let t = Text::new("hello", 2, 1);
    let r = t.render(80).unwrap();
    assert_eq!(r.lines.len(), 3);
    assert!(r.lines[1].contains("hello"));
}

#[test]
fn truncated_text_fits_width() {
    let t = TruncatedText::new("hello world", 0, 0);
    let r = t.render(10).unwrap();
    assert!(photon_ui::utils::visible_width(&r.lines[0]) <= 10);
}

#[test]
fn box_renders_with_padding() {
    let b = BoxComp::new(1);
    let r = b.render(10).unwrap();
    assert_eq!(r.lines.len(), 1);
    assert_eq!(r.lines[0].len(), 10);
}

#[test]
fn loader_renders_spinner() {
    let mut l = Loader::new("Loading…", None, None);
    let r = l.render(20).unwrap();
    assert!(r.lines[0].contains("Loading"));
    l.tick();
    let r2 = l.render(20).unwrap();
    assert_ne!(r.lines[0], r2.lines[0]);
}

#[test]
fn cancellable_loader_cancels() {
    let mut l = CancellableLoader::new("Loading…", None, None);
    assert!(!l.is_cancelled());
    let result = l.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('c'),
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert_eq!(result, InputResult::Handled);
    assert!(l.is_cancelled());
}
