use photon_ui::{
    Component,
    Layer,
    RenderError,
    Rendered,
    Shadow,
};

struct StaticComponent {
    lines: Vec<String>,
}

impl StaticComponent {
    fn new(lines: &[&str]) -> Self {
        Self {
            lines: lines.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Component for StaticComponent {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        Ok(Rendered {
            lines: self.lines.clone(),
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[test]
fn layer_renders_to_full_terminal_size() {
    let mut layer = Layer::new();
    layer.mount(Box::new(StaticComponent::new(&["hello"])));
    let rendered = layer.render(80, 24, None);
    assert_eq!(rendered.lines.len(), 24);
    assert!(rendered.lines[0].starts_with("hello"));
}

#[test]
fn layer_with_layout_splits_components() {
    use photon_ui::layout::{
        Constraint,
        Direction,
        Layout,
    };

    let mut layer = Layer::new();
    layer.mount(Box::new(StaticComponent::new(&["top"])));
    layer.mount(Box::new(StaticComponent::new(&["bottom"])));
    layer.set_layout(
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)]),
    );

    let rendered = layer.render(80, 24, None);
    assert!(rendered.lines[0].starts_with("top"));
    assert!(rendered.lines[1].starts_with("bottom"));
}

#[test]
fn layer_shadow_variants_are_distinct() {
    let dim = Shadow::Dim {
        style: "\x1b[2m".into(),
    };
    let drop = Shadow::Drop {
        style: "\x1b[2m".into(),
        offset_x: 1,
        offset_y: 1,
    };
    assert_ne!(dim, drop);
    assert_ne!(dim, Shadow::None);
}

#[test]
fn layer_visibility_skips_rendering() {
    let mut layer = Layer::new();
    layer.mount(Box::new(StaticComponent::new(&["visible"])));
    layer.visible = false;
    let rendered = layer.render(80, 24, None);
    assert_eq!(rendered.lines[0], "");
    assert_eq!(rendered.lines.len(), 24);
}

#[test]
fn layer_preserves_cursor_from_focused_component() {
    struct CursorComponent;

    impl Component for CursorComponent {
        fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
            Ok(Rendered {
                lines: vec!["cursor".into()],
                cursor: Some((0, 2)),
                images: Vec::new(),
            })
        }
    }

    let mut layer = Layer::new();
    layer.mount(Box::new(StaticComponent::new(&["a"])));
    layer.mount(Box::new(CursorComponent));
    let rendered = layer.render(80, 24, Some(1));
    assert_eq!(rendered.cursor, Some((1, 2)));
}
