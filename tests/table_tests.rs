use std::collections::HashMap;

use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::{
        Column,
        Row,
        Table,
    },
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

#[test]
fn table_sort_by_reorders_rows() {
    Theme::with(Theme::Light, || {
        let cols = vec![Column::new("name", "Name").sortable().width(10)];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.sort_by(0);
        let rendered = table.render(40).unwrap();
        // After sort: Alice, Bob, Charlie
        assert!(rendered.lines[2].contains("Alice"));
        assert!(rendered.lines[3].contains("Bob"));
        assert!(rendered.lines[4].contains("Charlie"));
    });
}

#[test]
fn table_sort_toggle_direction() {
    Theme::with(Theme::Light, || {
        let cols = vec![Column::new("name", "Name").sortable().width(10)];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.sort_by(0);
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines[2].contains("Alice"));

        table.sort_by(0); // toggle to descending
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines[2].contains("Bob"));
    });
}

#[test]
fn table_filter_shows_matching_rows() {
    Theme::with(Theme::Light, || {
        let cols = vec![Column::new("name", "Name").width(10)];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_filter("a");
        let rendered = table.render(40).unwrap();
        // Alice and Charlie match (case-insensitive)
        assert_eq!(rendered.lines.len(), 4); // header + sep + 2 rows
        assert!(rendered.lines[2].contains("Alice"));
        assert!(rendered.lines[3].contains("Charlie"));
    });
}

#[test]
fn table_filter_and_sort_combined() {
    Theme::with(Theme::Light, || {
        let cols = vec![Column::new("name", "Name").sortable().width(10)];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_filter("a");
        table.sort_by(0);
        let rendered = table.render(40).unwrap();
        // Filtered: Alice, Charlie; Sorted ascending: Alice, Charlie
        assert_eq!(rendered.lines.len(), 4);
        assert!(rendered.lines[2].contains("Alice"));
        assert!(rendered.lines[3].contains("Charlie"));
    });
}

#[test]
fn table_clear_filter_restores_rows() {
    let cols = vec![Column::new("name", "Name").width(10)];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.set_filter("Alice");
    assert_eq!(table.displayed_row_count(), 1);

    table.clear_filter();
    assert_eq!(table.displayed_row_count(), 2);
}

#[test]
fn table_clear_sort_restores_order() {
    let cols = vec![Column::new("name", "Name").sortable().width(10)];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.sort_by(0);
    assert_eq!(table.displayed_row_count(), 2);

    table.clear_sort();
    let rendered = table.render(40).unwrap();
    assert!(rendered.lines[2].contains("Charlie"));
    assert!(rendered.lines[3].contains("Alice"));
}

#[test]
fn table_selected_row_after_sort() {
    let cols = vec![Column::new("name", "Name").sortable().width(10)];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.sort_by(0);
    // After sort: Alice (visible 0), Charlie (visible 1)
    // selected was 0, still 0, now points to Alice
    let selected = table.selected_row();
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().get("name"), Some("Alice"));
}

#[test]
fn table_navigation_respects_filter() {
    let cols = vec![Column::new("name", "Name").width(10)];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.set_filter("Bob");
    // Only Bob is visible
    let result = table.handle_input(&Event::Key(KeyCode::Down.into()));
    assert_eq!(result, InputResult::Handled);
    assert_eq!(table.selected(), 0); // clamped, only 1 row
}

#[test]
fn table_set_rows_updates_display() {
    let cols = vec![Column::new("name", "Name").width(10)];
    let rows = vec![Row::new(HashMap::from([(
        "name".to_string(),
        "Alice".to_string(),
    )]))];
    let mut table = Table::new(cols, rows);
    assert_eq!(table.displayed_row_count(), 1);

    let new_rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
    ];
    table.set_rows(new_rows);
    assert_eq!(table.displayed_row_count(), 2);
}

#[test]
fn table_interactive_filter_mode() {
    let cols = vec![Column::new("name", "Name").width(10)];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
    ];
    let mut table = Table::new(cols, rows);
    table.set_focused(true);

    // Enter filter mode with '/'
    let result = table.handle_input(&Event::Key(KeyCode::Char('/').into()));
    assert_eq!(result, InputResult::Handled);
    assert!(table.in_filter_mode());

    // Type 'a'
    table.handle_input(&Event::Key(KeyCode::Char('a').into()));
    assert_eq!(table.filter_query(), Some("a"));
    assert_eq!(table.displayed_row_count(), 2); // Alice, Charlie

    // Press Enter to exit filter mode
    table.handle_input(&Event::Key(KeyCode::Enter.into()));
    assert!(!table.in_filter_mode());
    assert_eq!(table.filter_query(), Some("a"));
}

#[test]
fn table_hooks_fire_from_user_input() {
    use std::cell::Cell;
    use std::rc::Rc;

    let cols = vec![Column::new("name", "Name").sortable().width(10)];
    let rows = vec![
        Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
    ];

    let select_called = Rc::new(Cell::new(false));
    let sort_called = Rc::new(Cell::new(false));
    let filter_called = Rc::new(Cell::new(false));

    let sc = select_called.clone();
    let so = sort_called.clone();
    let fi = filter_called.clone();

    let mut table = Table::new(cols, rows)
        .on_select(move |_| {
            sc.set(true);
        })
        .on_sort(move |_, _| {
            so.set(true);
        })
        .on_filter(move |_| {
            fi.set(true);
        });

    table.set_focused(true);

    table.handle_input(&Event::Key(KeyCode::Down.into()));
    assert!(select_called.get());

    table.sort_by(0);
    assert!(sort_called.get());

    table.set_filter("a");
    assert!(filter_called.get());
}
