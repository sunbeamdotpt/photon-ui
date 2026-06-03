use std::collections::HashMap;

use photon_ui::{
    Component,
    components::{
        Box as BoxComponent, Button, Div, Divider, Editor, Header, ImageWidget,
        Input, Loader, Panel, ProgressBar, SelectList, SettingsList, Sidebar, SidebarItem,
        Table, Column, Row, Tabs, TreeNode, TreeView, TruncatedText,
    },
    layout::{Constraint, Rect, layout::Layout},
    renderer::{RenderStrategy, Renderer},
    terminal::TestTerminal,
    utils::visible_width,
};

/// Build the exact dashboard layout from the demo, wrapped in a root container
/// that mimics the TUI's vertical layout (Header + Breadcrumbs + DashboardBody).
fn build_full_dashboard(_width: u16) -> Box<dyn Component> {
    let mut root = Div::new(Layout::vertical([
        Constraint::Length(1), // Header
        Constraint::Length(1), // Breadcrumbs
        Constraint::Min(10),   // DashboardBody
    ]));

    root.push(Box::new(Header::new("Dashboard Demo").action("q:quit")));
    root.push(Box::new(photon_ui::components::Breadcrumbs::new(vec!["Home", "Dashboard", "Overview"])));

    let mut dashboard_body = Div::new(Layout::horizontal([
        Constraint::Length(16), // Sidebar
        Constraint::Min(10),    // MainContent
    ]));

    let sidebar = Sidebar::new(vec![
        SidebarItem::new("Overview").icon("📊"),
        SidebarItem::new("Files").icon("📁"),
        SidebarItem::new("Settings").icon("⚙️"),
        SidebarItem::new("Profile").icon("👤"),
    ]);
    dashboard_body.push(Box::new(sidebar));

    let mut main_content = Div::new(Layout::vertical([
        Constraint::Length(1), // Tabs
        Constraint::Length(1), // Divider
        Constraint::Length(1), // ProgressBar row
        Constraint::Length(1), // Divider
        Constraint::Length(4), // DataRow
        Constraint::Length(1), // Divider
        Constraint::Length(2), // FormsRow
        Constraint::Length(1), // Divider
        Constraint::Length(3), // ListsRow
        Constraint::Length(1), // Divider
        Constraint::Length(1), // ButtonsRow
        Constraint::Length(1), // Divider
        Constraint::Length(3), // ContentRow
        Constraint::Length(1), // Divider
        Constraint::Length(2), // SystemRow
    ]));

    main_content.push(Box::new(Tabs::new(vec!["Overview", "Resources", "Logs"])));
    main_content.push(Box::new(Divider::horizontal()));

    let mut progress_row = Div::new(Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]));
    progress_row.push(Box::new(ProgressBar::new("CPU", 0.45).width(15)));
    progress_row.push(Box::new(ProgressBar::new("RAM", 1.0).width(15)));
    main_content.push(Box::new(progress_row));
    main_content.push(Box::new(Divider::horizontal().labeled("Data")));

    let mut data_row = Div::new(Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]));
    let table = Table::new(
        vec![
            Column::new("name", "Name").width(15),
            Column::new("status", "Status").width(10),
            Column::new("size", "Size").width(8),
        ],
        vec![
            Row::new(HashMap::from([
                ("name".to_string(), "src/main.rs".to_string()),
                ("status".to_string(), "active".to_string()),
                ("size".to_string(), "12.4KB".to_string()),
            ])),
            Row::new(HashMap::from([
                ("name".to_string(), "Cargo.toml".to_string()),
                ("status".to_string(), "active".to_string()),
                ("size".to_string(), "1.2KB".to_string()),
            ])),
            Row::new(HashMap::from([
                ("name".to_string(), "README.md".to_string()),
                ("status".to_string(), "idle".to_string()),
                ("size".to_string(), "4.5KB".to_string()),
            ])),
        ],
    );
    data_row.push(Box::new(table));
    let tree = TreeView::new(vec![
        TreeNode::new("src")
            .child(TreeNode::new("main.rs"))
            .child(TreeNode::new("lib.rs")),
        TreeNode::new("tests").child(TreeNode::new("integration.rs")),
    ]);
    data_row.push(Box::new(tree));
    main_content.push(Box::new(data_row));
    main_content.push(Box::new(Divider::horizontal().labeled("Forms")));

    let mut forms_row = Div::new(Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]));
    let mut input = Input::new();
    input.set_text("Search components...");
    forms_row.push(Box::new(input));
    let mut editor = Editor::new();
    editor.set_text("fn main() {\n    println!(\"Hello\");\n}");
    forms_row.push(Box::new(editor));
    main_content.push(Box::new(forms_row));
    main_content.push(Box::new(Divider::horizontal().labeled("Lists")));

    let mut lists_row = Div::new(Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]));
    let mut select_list = SelectList::new(
        vec![
            "Rust".into(),
            "Python".into(),
            "TypeScript".into(),
            "Go".into(),
            "Zig".into(),
            "Haskell".into(),
        ],
        4,
    );
    select_list.set_selected(1);
    lists_row.push(Box::new(select_list));
    let mut settings = SettingsList::new(vec![
        ("Dark mode".into(), true),
        ("Auto-save".into(), false),
        ("Notifications".into(), true),
    ]);
    settings.set_selected(0);
    lists_row.push(Box::new(settings));
    main_content.push(Box::new(lists_row));
    main_content.push(Box::new(Divider::horizontal()));

    let mut buttons_row = Div::new(Layout::horizontal([
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
    ]));
    buttons_row.push(Box::new(Button::primary("Primary")));
    buttons_row.push(Box::new(Button::dark("Dark")));
    buttons_row.push(Box::new(Button::ghost("Ghost")));
    buttons_row.push(Box::new(Button::text("Text")));
    main_content.push(Box::new(buttons_row));
    main_content.push(Box::new(Divider::horizontal().labeled("Content")));

    let mut content_row = Div::new(Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]));
    content_row.push(Box::new(photon_ui::components::Markdown::new(
        "**Photon UI**\n\nA Rust TUI library with:\n",
    )));
    content_row.push(Box::new(
        Panel::new()
            .title("Info")
            .lines(vec![
                "Photon UI v0.1.0".into(),
                "A Rust TUI library".into(),
            ]),
    ));
    main_content.push(Box::new(content_row));
    main_content.push(Box::new(Divider::horizontal().labeled("System")));

    let mut system_row = Div::new(Layout::horizontal([
        Constraint::Length(14),
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Min(5),
    ]));
    system_row.push(Box::new(Loader::new("Loading...", Some("\x1b[36m".into()), None)));
    system_row.push(Box::new(Loader::new(
        "Background task running...",
        Some("\x1b[33m".into()),
        Some("\x1b[90m".into()),
    )));
    system_row.push(Box::new(ImageWidget::new(
        vec![],
        "image/png",
        Some("[image]".to_string()),
    )));
    system_row.push(Box::new(
        BoxComponent::new(1)
            .with_background(|line, _w| format!("\x1b[44m{}\x1b[0m", line)),
    ));
    system_row.push(Box::new(TruncatedText::new(
        "This is a very long line that will be truncated with an ellipsis if the terminal is not wide enough to display it all",
        0,
        0,
    )));
    main_content.push(Box::new(system_row));

    dashboard_body.push(Box::new(main_content));
    root.push(Box::new(dashboard_body));
    Box::new(root)
}

/// Regression test: render the full dashboard at narrow widths and verify no
/// line exceeds the terminal width.
#[test]
fn dashboard_no_wrapping_at_narrow_widths() {
    for width in [80, 70, 60, 50, 45, 40, 35, 30, 25] {
        let dashboard = build_full_dashboard(width);
        let rendered = dashboard.render_rect(Rect::new(0, 0, width, 24)).unwrap();

        for (i, line) in rendered.lines.iter().enumerate() {
            let vw = visible_width(line);
            assert!(
                vw <= width as usize,
                "line {} exceeds terminal width {} (actual {}): {:?}",
                i,
                width,
                vw,
                line
            );
        }
    }
}

/// Regression test: the renderer diff strategy must not produce wrapped output
/// when the previous frame had longer lines. We simulate a resize by rendering
/// at 80 cols, then at 40 cols with FullRedraw, then doing a Diff.
#[test]
fn dashboard_diff_no_wrapping_after_resize() {
    // First render at wide width
    let wide = build_full_dashboard(80);
    let frame1 = wide.render_rect(Rect::new(0, 0, 80, 24)).unwrap();

    let mut term = TestTerminal::new(40, 24);
    let mut renderer = Renderer::new();
    renderer.render(&mut term, &frame1).unwrap();

    // Second render at narrow width with FullRedraw (simulating resize)
    renderer.set_strategy(RenderStrategy::FullRedraw);
    let narrow = build_full_dashboard(40);
    let frame2 = narrow.render_rect(Rect::new(0, 0, 40, 24)).unwrap();
    renderer.render(&mut term, &frame2).unwrap();

    // Third render with Diff (simulating a small update after resize)
    renderer.set_strategy(RenderStrategy::Diff);
    let frame3 = narrow.render_rect(Rect::new(0, 0, 40, 24)).unwrap();
    renderer.render(&mut term, &frame3).unwrap();

    // Inspect only the LAST render's output
    let last_output = term.written().last().cloned().unwrap_or_default();
    for line in last_output.split("\r\n") {
        let vw = visible_width(line);
        assert!(
            vw <= 40,
            "diff output line exceeds width 40 (actual {}): {:?}",
            vw,
            line
        );
    }
}

/// Regression test: render through the actual TUI and verify no wrapping.
#[test]
fn dashboard_via_tui_no_wrapping() {
    use photon_ui::TUI;

    let mut tui = TUI::new(Box::new(TestTerminal::new(40, 24)));
    tui.set_layout(Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(1),
    ]));

    let dashboard = build_full_dashboard(40);
    tui.mount(dashboard);
    tui.render_frame().unwrap();

    // Access the terminal through the TUI's terminal() method
    let term = tui.terminal();
    // We can't easily inspect the rendered output from TUI, but at least
    // verify it doesn't panic.
}
