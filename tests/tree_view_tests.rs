use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Focusable,
    InputResult,
    components::{
        TreeNode,
        TreeView,
    },
    events::Event,
    theme::Theme,
};

#[test]
fn tree_view_renders_collapsed_roots() {
    Theme::with(Theme::Light, || {
        let view = TreeView::new(vec![
            TreeNode::new("A").child(TreeNode::new("A1")),
            TreeNode::new("B"),
        ]);
        let rendered = view.render(80).unwrap();
        assert_eq!(rendered.lines.len(), 2);
        assert!(rendered.lines[0].contains("A"));
        assert!(rendered.lines[0].contains("▶"));
        assert!(rendered.lines[1].contains("B"));
    });
}

#[test]
fn tree_view_renders_expanded_with_children() {
    Theme::with(Theme::Light, || {
        let mut view = TreeView::new(vec![
            TreeNode::new("A").child(TreeNode::new("A1")),
            TreeNode::new("B"),
        ]);
        view.set_focused(true);
        view.handle_input(&Event::Key(KeyCode::Right.into()));
        let rendered = view.render(80).unwrap();
        assert_eq!(rendered.lines.len(), 3);
        assert!(rendered.lines[0].contains("▼"));
        assert!(rendered.lines[1].contains("A1"));
        assert!(rendered.lines[1].contains("└─"));
    });
}

#[test]
fn tree_view_focused_selected_prefix() {
    Theme::with(Theme::Light, || {
        let mut view = TreeView::new(vec![TreeNode::new("A")]);
        view.set_focused(true);
        let rendered = view.render(80).unwrap();
        // Selected + focused root gets "> " prefix; because the line is
        // styled, ANSI codes precede the visible prefix.
        assert!(rendered.lines[0].contains("> "));
    });
}

#[test]
fn tree_view_selected_uses_accent() {
    Theme::with(Theme::Light, || {
        let mut view = TreeView::new(vec![TreeNode::new("A"), TreeNode::new("B")]);
        view.set_focused(true);
        let rendered = view.render(80).unwrap();
        // Accent color for light theme: \x1b[38;2;250;82;15m
        assert!(rendered.lines[0].contains("\x1b[38;2;250;82;15m"));
    });
}

#[test]
fn tree_view_keyboard_down_up() {
    let mut view = TreeView::new(vec![TreeNode::new("A"), TreeNode::new("B")]);
    view.set_focused(true);
    view.handle_input(&Event::Key(KeyCode::Down.into()));
    let rendered = view.render(80).unwrap();
    // Second line should be selected (bold)
    assert!(rendered.lines[1].contains("\x1b[1m"));
}

#[test]
fn tree_view_toggle_expand_collapse() {
    let mut view = TreeView::new(vec![TreeNode::new("A").child(TreeNode::new("A1"))]);
    view.set_focused(true);

    let result = view.handle_input(&Event::Key(KeyCode::Right.into()));
    assert_eq!(result, InputResult::Handled);

    let rendered = view.render(80).unwrap();
    assert_eq!(rendered.lines.len(), 2);

    let result = view.handle_input(&Event::Key(KeyCode::Right.into()));
    assert_eq!(result, InputResult::Handled);

    let rendered = view.render(80).unwrap();
    assert_eq!(rendered.lines.len(), 1);
}

#[test]
fn tree_view_enter_toggles() {
    let mut view = TreeView::new(vec![TreeNode::new("A").child(TreeNode::new("A1"))]);
    view.set_focused(true);

    let result = view.handle_input(&Event::Key(KeyCode::Enter.into()));
    assert_eq!(result, InputResult::Handled);
}

#[test]
fn tree_view_left_collapses() {
    let mut view = TreeView::new(vec![TreeNode::new("A").child(TreeNode::new("A1"))]);
    view.set_focused(true);

    view.handle_input(&Event::Key(KeyCode::Right.into()));
    view.handle_input(&Event::Key(KeyCode::Left.into()));

    let rendered = view.render(80).unwrap();
    assert_eq!(rendered.lines.len(), 1);
    assert!(rendered.lines[0].contains("▶"));
}

#[test]
fn tree_view_left_navigates_to_parent() {
    let mut view = TreeView::new(vec![TreeNode::new("A").child(TreeNode::new("A1"))]);
    view.set_focused(true);

    view.handle_input(&Event::Key(KeyCode::Right.into()));
    view.handle_input(&Event::Key(KeyCode::Down.into()));

    let rendered = view.render(80).unwrap();
    assert!(rendered.lines[1].contains("\x1b[1m"));

    view.handle_input(&Event::Key(KeyCode::Left.into()));

    let rendered = view.render(80).unwrap();
    assert!(rendered.lines[0].contains("\x1b[1m"));
}

#[test]
fn tree_view_j_k_navigation() {
    let mut view = TreeView::new(vec![TreeNode::new("A"), TreeNode::new("B")]);
    view.set_focused(true);

    view.handle_input(&Event::Key(KeyCode::Char('j').into()));
    let rendered = view.render(80).unwrap();
    assert!(rendered.lines[1].contains("\x1b[1m"));

    view.handle_input(&Event::Key(KeyCode::Char('k').into()));
    let rendered = view.render(80).unwrap();
    assert!(rendered.lines[0].contains("\x1b[1m"));
}
