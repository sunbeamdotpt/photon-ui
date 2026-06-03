use crossterm::event::KeyCode;

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
};
use crate::theme::{
    Palette,
    Style,
    Theme,
    stylize,
};

/// A single node in a tree view.
///
/// Nodes can have children and can be expanded or collapsed.
pub struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
    expanded: bool,
}

impl TreeNode {
    /// Create a new tree node with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Add a child node and return self for chaining.
    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }
}

/// A tree view component with collapsible nodes and keyboard navigation.
///
/// Renders with tree-drawing characters. The selected node is highlighted
/// with the theme's accent color and bold.
pub struct TreeView {
    nodes: Vec<TreeNode>,
    selected: Vec<usize>,
    focused: bool,
}

impl TreeView {
    /// Create a new tree view with the given root nodes.
    ///
    /// Defaults to selecting the first root node (if any).
    pub fn new(nodes: Vec<TreeNode>) -> Self {
        let selected = if nodes.is_empty() {
            Vec::new()
        } else {
            vec![0]
        };
        Self {
            nodes,
            selected,
            focused: false,
        }
    }

    /// Flatten the visible tree into a list of entries with depth, node
    /// reference, path, and whether the node is the last child of its parent.
    fn flatten(&self) -> Vec<(usize, &TreeNode, Vec<usize>, bool)> {
        let mut result = Vec::new();
        let len = self.nodes.len();
        for (i, node) in self.nodes.iter().enumerate() {
            Self::flatten_node(node, 0, vec![i], i == len - 1, &mut result);
        }
        result
    }

    fn flatten_node<'a>(
        node: &'a TreeNode,
        depth: usize,
        path: Vec<usize>,
        is_last: bool,
        result: &mut Vec<(usize, &'a TreeNode, Vec<usize>, bool)>,
    ) {
        result.push((depth, node, path.clone(), is_last));
        if node.expanded {
            let child_len = node.children.len();
            for (i, child) in node.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                Self::flatten_node(child, depth + 1, child_path, i == child_len - 1, result);
            }
        }
    }

    /// Find the flat index of the currently selected node by comparing paths.
    fn selected_flat_index(
        &self,
        flat: &[(usize, &TreeNode, Vec<usize>, bool)],
    ) -> Option<usize> {
        flat.iter().position(|(_, _, path, _)| path == &self.selected)
    }

    /// Navigate to the node at the given mutable path.
    fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let mut node = match self.nodes.get_mut(path[0]) {
            Some(n) => n,
            None => return None,
        };
        for &index in &path[1..] {
            node = match node.children.get_mut(index) {
                Some(n) => n,
                None => return None,
            };
        }
        Some(node)
    }

    fn navigate_down(&mut self) {
        let new_path = {
            let flat = self.flatten();
            if let Some(idx) = self.selected_flat_index(&flat) {
                flat.get(idx + 1).map(|entry| entry.2.clone())
            } else {
                None
            }
        };
        if let Some(path) = new_path {
            self.selected = path;
        }
    }

    fn navigate_up(&mut self) {
        let new_path = {
            let flat = self.flatten();
            if let Some(idx) = self.selected_flat_index(&flat) {
                if idx > 0 {
                    flat.get(idx - 1).map(|entry| entry.2.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(path) = new_path {
            self.selected = path;
        }
    }
}

impl Focusable for TreeView {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for TreeView {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let accent_style = Style::new().fg(theme.accent()).bold();
        let normal_style = Style::new().fg(theme.text_primary());

        let flat = self.flatten();
        let selected_index = self.selected_flat_index(&flat);

        let mut lines = Vec::new();
        for (flat_i, (depth, node, _path, is_last)) in flat.iter().enumerate() {
            let is_selected = selected_index == Some(flat_i);

            let mut line = String::new();
            if *depth == 0 {
                if is_selected && self.focused {
                    line.push_str("> ");
                } else {
                    line.push_str("  ");
                }
            } else {
                line.push_str(&"  ".repeat(*depth));
                if *is_last {
                    line.push_str("└─ ");
                } else {
                    line.push_str("├─ ");
                }
            }

            if !node.children.is_empty() {
                if node.expanded {
                    line.push_str("▼ ");
                } else {
                    line.push_str("▶ ");
                }
            } else {
                line.push_str("  ");
            }

            line.push_str(&node.label);

            let truncated = crate::utils::truncate_to_width(&line, width, "…");
            let styled = if is_selected {
                stylize(&truncated, &accent_style)
            } else {
                stylize(&truncated, &normal_style)
            };
            lines.push(styled);
        }

        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crossterm::event::KeyModifiers;
        if self.nodes.is_empty() {
            return InputResult::Ignored;
        }

        if let Event::Key(key) = event {
            match key.code {
                | KeyCode::Down => {
                    self.navigate_down();
                    InputResult::Handled
                },
                | KeyCode::Up => {
                    self.navigate_up();
                    InputResult::Handled
                },
                | KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.navigate_down();
                    InputResult::Handled
                },
                | KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.navigate_up();
                    InputResult::Handled
                },
                | KeyCode::Right | KeyCode::Enter => {
                    let path = self.selected.clone();
                    if let Some(node) = self.node_at_path_mut(&path)
                        && !node.children.is_empty()
                    {
                        node.expanded = !node.expanded;
                        return InputResult::Handled;
                    }
                    InputResult::Ignored
                },
                | KeyCode::Left => {
                    let path = self.selected.clone();
                    if let Some(node) = self.node_at_path_mut(&path)
                        && node.expanded
                        && !node.children.is_empty()
                    {
                        node.expanded = false;
                        return InputResult::Handled;
                    }
                    if self.selected.len() > 1 {
                        self.selected.pop();
                        return InputResult::Handled;
                    }
                    InputResult::Ignored
                },
                | _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> {
        Some(self)
    }

    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;

    use super::*;

    #[test]
    fn tree_node_builder() {
        let node = TreeNode::new("root").child(TreeNode::new("child"));
        assert_eq!(node.label, "root");
        assert_eq!(node.children.len(), 1);
    }

    #[test]
    fn tree_view_new() {
        let view = TreeView::new(vec![TreeNode::new("a"), TreeNode::new("b")]);
        assert_eq!(view.selected, vec![0]);
    }

    #[test]
    fn tree_view_new_empty() {
        let view = TreeView::new(Vec::new());
        assert!(view.selected.is_empty());
    }

    #[test]
    fn tree_view_focusable() {
        let mut view = TreeView::new(vec![TreeNode::new("a")]);
        assert!(!view.focused());
        view.set_focused(true);
        assert!(view.focused());
    }

    #[test]
    fn tree_view_render() {
        Theme::with(Theme::Light, || {
            let view = TreeView::new(vec![TreeNode::new("root")]);
            let rendered = view.render(80).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert!(rendered.lines[0].contains("root"));
        });
    }

    #[test]
    fn tree_view_navigation_down() {
        let mut view = TreeView::new(vec![TreeNode::new("a"), TreeNode::new("b")]);
        view.set_focused(true);
        view.handle_input(&Event::Key(KeyCode::Down.into()));
        assert_eq!(view.selected, vec![1]);
    }

    #[test]
    fn tree_view_navigation_up() {
        let mut view = TreeView::new(vec![TreeNode::new("a"), TreeNode::new("b")]);
        view.set_focused(true);
        view.selected = vec![1];
        view.handle_input(&Event::Key(KeyCode::Up.into()));
        assert_eq!(view.selected, vec![0]);
    }

    #[test]
    fn tree_view_toggle_expansion() {
        let mut view = TreeView::new(vec![TreeNode::new("root").child(TreeNode::new("child"))]);
        view.set_focused(true);
        let flat = view.flatten();
        assert_eq!(flat.len(), 1);

        view.handle_input(&Event::Key(KeyCode::Right.into()));
        let flat = view.flatten();
        assert_eq!(flat.len(), 2);

        view.handle_input(&Event::Key(KeyCode::Right.into()));
        let flat = view.flatten();
        assert_eq!(flat.len(), 1);
    }

    #[test]
    fn tree_view_left_navigates_to_parent() {
        let mut view = TreeView::new(vec![TreeNode::new("root").child(TreeNode::new("child"))]);
        view.set_focused(true);
        view.handle_input(&Event::Key(KeyCode::Right.into()));
        view.handle_input(&Event::Key(KeyCode::Down.into()));
        assert_eq!(view.selected, vec![0, 0]);

        view.handle_input(&Event::Key(KeyCode::Left.into()));
        assert_eq!(view.selected, vec![0]);
    }

    #[test]
    fn tree_view_left_collapses() {
        let mut view = TreeView::new(vec![TreeNode::new("root").child(TreeNode::new("child"))]);
        view.set_focused(true);
        view.handle_input(&Event::Key(KeyCode::Right.into()));
        assert!(view.nodes[0].expanded);

        view.handle_input(&Event::Key(KeyCode::Left.into()));
        assert!(!view.nodes[0].expanded);
    }

    #[test]
    fn tree_view_j_k_navigation() {
        let mut view = TreeView::new(vec![TreeNode::new("a"), TreeNode::new("b")]);
        view.set_focused(true);
        view.handle_input(&Event::Key(KeyCode::Char('j').into()));
        assert_eq!(view.selected, vec![1]);
        view.handle_input(&Event::Key(KeyCode::Char('k').into()));
        assert_eq!(view.selected, vec![0]);
    }

    #[test]
    fn tree_view_enter_toggles() {
        let mut view = TreeView::new(vec![TreeNode::new("root").child(TreeNode::new("child"))]);
        view.set_focused(true);
        let result = view.handle_input(&Event::Key(KeyCode::Enter.into()));
        assert_eq!(result, InputResult::Handled);
        assert!(view.nodes[0].expanded);
    }

    #[test]
    fn tree_view_leaf_ignores_right() {
        let mut view = TreeView::new(vec![TreeNode::new("leaf")]);
        view.set_focused(true);
        let result = view.handle_input(&Event::Key(KeyCode::Right.into()));
        assert_eq!(result, InputResult::Ignored);
    }

    #[test]
    fn tree_view_root_left_ignored_when_collapsed() {
        let mut view = TreeView::new(vec![TreeNode::new("root").child(TreeNode::new("child"))]);
        view.set_focused(true);
        let result = view.handle_input(&Event::Key(KeyCode::Left.into()));
        assert_eq!(result, InputResult::Ignored);
    }
}
