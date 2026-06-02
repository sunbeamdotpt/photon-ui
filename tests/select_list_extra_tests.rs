use photon_ui::components::SelectList;
use photon_ui::{Component, Focusable, Event};
use crossterm::event::KeyCode;

fn key_event(code: KeyCode) -> Event {
    Event::Key(code.into())
}

#[test]
fn select_list_empty() {
    let list = SelectList::new(vec![], 5);
    let r = list.render(80).unwrap();
    assert!(r.lines.is_empty());
}

#[test]
fn select_list_focusable() {
    let mut list = SelectList::new(vec!["a".into()], 5);
    assert!(!list.focused());
    list.set_focused(true);
    assert!(list.focused());
}

#[test]
fn select_list_up_at_top() {
    let mut list = SelectList::new(vec!["a".into(), "b".into()], 5);
    list.set_focused(true);
    list.handle_input(&key_event(KeyCode::Up));
    let r = list.render(80).unwrap();
    assert!(r.lines[0].contains("> "));
}

#[test]
fn select_list_enter_requests_render() {
    let mut list = SelectList::new(vec!["a".into(), "b".into()], 5);
    list.set_focused(true);
    let result = list.handle_input(&key_event(KeyCode::Enter));
    assert!(matches!(result, photon_ui::InputResult::RequestRender));
}

#[test]
fn select_list_ignored_key() {
    let mut list = SelectList::new(vec!["a".into()], 5);
    list.set_focused(true);
    let result = list.handle_input(&key_event(KeyCode::Char('x')));
    assert!(matches!(result, photon_ui::InputResult::Ignored));
}

#[test]
fn select_list_resize_event() {
    let mut list = SelectList::new(vec!["a".into()], 5);
    let result = list.handle_input(&Event::Resize(80, 24));
    assert!(matches!(result, photon_ui::InputResult::Ignored));
}
