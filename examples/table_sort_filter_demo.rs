use std::{
    cell::RefCell,
    rc::Rc,
};

use photon_ui::{
    Component,
    Focusable,
    components::{
        Column,
        Row,
        Table,
    },
    theme::Theme,
};

fn main() {
    Theme::with(Theme::Light, || {
        let cols = vec![
            Column::new("name", "Name").sortable().width(12),
            Column::new("role", "Role").sortable().width(14),
            Column::new("status", "Status").width(10),
        ];

        let rows = vec![
            Row::new(
                [
                    ("name".to_string(), "Charlie".to_string()),
                    ("role".to_string(), "Designer".to_string()),
                    ("status".to_string(), "Active".to_string()),
                ]
                .into(),
            ),
            Row::new(
                [
                    ("name".to_string(), "Alice".to_string()),
                    ("role".to_string(), "Engineer".to_string()),
                    ("status".to_string(), "Active".to_string()),
                ]
                .into(),
            ),
            Row::new(
                [
                    ("name".to_string(), "Bob".to_string()),
                    ("role".to_string(), "Manager".to_string()),
                    ("status".to_string(), "Away".to_string()),
                ]
                .into(),
            ),
            Row::new(
                [
                    ("name".to_string(), "Diana".to_string()),
                    ("role".to_string(), "Engineer".to_string()),
                    ("status".to_string(), "Offline".to_string()),
                ]
                .into(),
            ),
        ];

        let last_action = Rc::new(RefCell::new(String::new()));
        let la_select = last_action.clone();
        let la_sort = last_action.clone();
        let la_filter = last_action.clone();

        let mut table = Table::new(cols, rows)
            .on_select(move |idx| {
                *la_select.borrow_mut() = format!("selected row {}", idx);
            })
            .on_sort(move |col, asc| {
                *la_sort.borrow_mut() = format!(
                    "sorted by column {} {}",
                    col,
                    if asc { "asc" } else { "desc" }
                );
            })
            .on_filter(move |q| {
                *la_filter.borrow_mut() = format!("filter: '{}'", q);
            })
            .on_filter_char(|c| {
                // Only allow alphanumeric and space in filter
                if c.is_alphanumeric() || c == ' ' {
                    Some(c)
                } else {
                    None
                }
            });
        table.set_focused(true);

        println!("╔══════════════════════════════════════════╗");
        println!("║  Original order (no sort, no filter)     ║");
        println!("╚══════════════════════════════════════════╝");
        let rendered = table.render(40).unwrap();
        for line in &rendered.lines {
            println!("{}", line);
        }
        println!();

        table.sort_by(0);
        println!("╔══════════════════════════════════════════╗");
        println!("║  Sorted by Name ▲ (ascending)            ║");
        println!("║  {}  ║", pad_right(&last_action.borrow(), 36));
        println!("╚══════════════════════════════════════════╝");
        let rendered = table.render(40).unwrap();
        for line in &rendered.lines {
            println!("{}", line);
        }
        println!();

        table.sort_by(0);
        println!("╔══════════════════════════════════════════╗");
        println!("║  Sorted by Name ▼ (descending)           ║");
        println!("║  {}  ║", pad_right(&last_action.borrow(), 36));
        println!("╚══════════════════════════════════════════╝");
        let rendered = table.render(40).unwrap();
        for line in &rendered.lines {
            println!("{}", line);
        }
        println!();

        table.clear_sort();
        table.set_filter("eng");
        println!("╔══════════════════════════════════════════╗");
        println!("║  Filter: \"eng\" + Sort by Name ▲         ║");
        println!("║  {}  ║", pad_right(&last_action.borrow(), 36));
        println!("╚══════════════════════════════════════════╝");
        table.sort_by(0);
        *last_action.borrow_mut() = "filter: 'eng' + sort by column 0 asc".to_string();
        let rendered = table.render(40).unwrap();
        for line in &rendered.lines {
            println!("{}", line);
        }
        println!();

        // Simulate interactive filter mode: press '/' then type 'a'
        table.clear_sort();
        table.clear_filter();
        println!("╔══════════════════════════════════════════╗");
        println!("║  Interactive filter mode demo            ║");
        println!("╚══════════════════════════════════════════╝");
        table.handle_input(&photon_ui::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        table.handle_input(&photon_ui::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        )));
        let rendered = table.render(40).unwrap();
        for line in &rendered.lines {
            println!("{}", line);
        }
        println!();

        println!(
            "Selected row: {:?}",
            table.selected_row().unwrap().get("name")
        );
        println!("Last action: {}", last_action.borrow());
    });
}

fn pad_right(s: &str, width: usize) -> String {
    let _visible = s.chars().count().min(width);
    let mut out = s.chars().take(width).collect::<String>();
    while out.chars().count() < width {
        out.push(' ');
    }
    out
}
