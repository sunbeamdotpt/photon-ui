use photon_ui::events::{Event, Key, Modifiers, matches_key};
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind};

#[test]
fn matches_key_char() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
    assert!(matches_key(&event, &Key::Char('a', Modifiers::none())));
    assert!(!matches_key(&event, &Key::Char('b', Modifiers::none())));
}

#[test]
fn matches_key_with_modifiers() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(matches_key(&event, &Key::ctrl('c')));
    assert!(!matches_key(&event, &Key::Char('c', Modifiers::none())));
}

#[test]
fn matches_key_code() {
    let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert!(matches_key(&event, &Key::enter()));
    assert!(!matches_key(&event, &Key::backspace()));
}

#[test]
fn matches_key_ignores_release() {
    let event = Event::Key(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert!(!matches_key(&event, &Key::Char('a', Modifiers::none())));
}

#[test]
fn matches_key_non_key_event() {
    assert!(!matches_key(&Event::Resize(10, 10), &Key::enter()));
}

#[test]
fn key_constructors() {
    assert_eq!(Key::left(), Key::Code(KeyCode::Left, Modifiers::none()));
    assert_eq!(Key::right(), Key::Code(KeyCode::Right, Modifiers::none()));
    assert_eq!(Key::up(), Key::Code(KeyCode::Up, Modifiers::none()));
    assert_eq!(Key::down(), Key::Code(KeyCode::Down, Modifiers::none()));
    assert_eq!(Key::backspace(), Key::Code(KeyCode::Backspace, Modifiers::none()));
    assert_eq!(Key::delete(), Key::Code(KeyCode::Delete, Modifiers::none()));
    assert_eq!(Key::home(), Key::Code(KeyCode::Home, Modifiers::none()));
    assert_eq!(Key::end(), Key::Code(KeyCode::End, Modifiers::none()));
    assert_eq!(Key::tab(), Key::Code(KeyCode::Tab, Modifiers::none()));
    assert_eq!(Key::esc(), Key::Code(KeyCode::Esc, Modifiers::none()));
    assert_eq!(Key::ctrl('x'), Key::Char('x', Modifiers::ctrl()));
    assert_eq!(Key::ctrl_shift('x'), Key::Char('x', Modifiers::ctrl_shift()));
    assert_eq!(Key::alt('x'), Key::Char('x', Modifiers::alt()));
}

#[test]
fn modifiers_combinations() {
    let m = Modifiers::ctrl_shift();
    assert!(m.ctrl);
    assert!(m.shift);
    assert!(!m.alt);
}
