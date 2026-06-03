use std::collections::HashMap;

use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::{Column, Row, Table},
    events::Event,
    theme::Theme,
};

#[test]
fn table_renders_header_separator_and_rows() {
    Theme::with(Theme::Light, || {
        let cols = vec![
            Column::new("name", "Name").width(10),
            Column::new("status", "Status").width(10),
        ];
        let rows = vec![
            Row::new(HashMap::from([
                ("name".to_string(), "Alice".to_string()),
                ("status".to_string(), "Active".to_string()),
            ])),
            Row::new(HashMap::from([
                ("name".to_string(), "Bob".to_string()),
                ("status".to_string(), "Inactive".to_string()),
            ])),
        ];
        let table = Table::new(cols, rows);
        let rendered = table.render(40).unwrap();
        assert_eq!(rendered.lines.len(), 4); // header + sep + 2 rows
        assert!(rendered.lines[0].contains("Name"));
        assert!(rendered.lines[0].contains("Status"));
        assert!(rendered.lines[1].contains("─"));
    });
}

#[test]
fn table_focused_shows_accent_on_selected() {
    Theme::with(Theme::Light, || {
        let cols = vec![Column::new("name", "Name").width(10)];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        let rendered = table.render(40).unwrap();
        // First data row (index 2) should have "> " prefix since selected=0
        assert!(rendered.lines[2].contains("> "));
        // Second data row should have "  " prefix
        assert!(!rendered.lines[3].contains("> "));
    });
}

#[test]
fn table_keyboard_navigation() {
    let cols = vec![Column::new("name", "Name")];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.set_focused(true);

    let result = table.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(result, InputResult::Handled);
    assert_eq!(table.selected(), 1);

    let result = table.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(result, InputResult::Handled);
    assert_eq!(table.selected(), 2);

    // clamped
    let result = table.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(result, InputResult::Handled);
    assert_eq!(table.selected(), 2);
}

#[test]
fn table_j_k_navigation() {
    let cols = vec![Column::new("name", "Name")];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.set_focused(true);

    let result = table.handle_input(&Event::Key(KeyCode::Char('j').into()));
    assert_eq!(result, InputResult::Handled);
    assert_eq!(table.selected(), 1);

    let result = table.handle_input(&Event::Key(KeyCode::Char('k').into()));
    assert_eq!(result, InputResult::Handled);
    assert_eq!(table.selected(), 0);
}

#[test]
fn table_sort_indicator_renders() {
    Theme::with(Theme::Light, || {
        let cols = vec![Column::new("name", "Name").sortable().width(10)];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_sort_column(Some(0));
        table.set_sort_ascending(true);
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines[0].contains("▲"));

        table.set_sort_ascending(false);
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines[0].contains("▼"));
    });
}

#[test]
fn table_set_selected_clamps() {
    let cols = vec![Column::new("name", "Name")];
    let rows = vec![Row::new(HashMap::from([(
        "name".to_string(),
        "Alice".to_string(),
    )]))];
    let mut table = Table::new(cols, rows);
    table.set_selected(100);
    assert_eq!(table.selected(), 0);
}
