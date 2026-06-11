use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    components::SelectList,
    events::Event,
};

#[test]
fn select_list_navigation() {
    let mut list = SelectList::new(vec!["a".into(), "b".into(), "c".into()], 5);
    list.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(list.selected(), 1);
    list.handle_input(&Event::Key(KeyCode::Up.into()));
    assert_eq!(list.selected(), 0);
}

#[test]
fn select_list_scrolls() {
    let mut list = SelectList::new(
        vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
        3,
    );
    list.handle_input(&Event::Key(KeyCode::Down.into()));
    list.handle_input(&Event::Key(KeyCode::Down.into()));
    list.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(list.selected(), 3);
}

#[test]
fn select_list_renders() {
    let list = SelectList::new(vec!["a".into(), "b".into()], 5);
    let r = list.render(10).unwrap();
    assert_eq!(r.lines.len(), 2);
    assert!(r.lines[0].contains("> a"));
}
