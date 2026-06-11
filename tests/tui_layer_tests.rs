use photon_ui::{
    Component,
    Layer,
    RenderError,
    Rendered,
    Shadow,
    TUI,
    TestTerminal,
    components::Text,
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
fn tui_adds_base_layer_automatically() {
    let term = TestTerminal::new(80, 24);
    let tui = TUI::new(Box::new(term));
    assert_eq!(tui.layer_count(), 1);
}

#[test]
fn tui_mount_adds_to_base_layer() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(Text::new("hello", 0, 0)));
    assert_eq!(tui.layer_count(), 1);
}

#[test]
fn tui_add_layer_returns_index() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    let idx = tui.add_layer(Layer::new());
    assert_eq!(idx, 1);
    assert_eq!(tui.layer_count(), 2);
}

#[test]
fn tui_insert_layer_at_bottom() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    let mut bottom = Layer::new();
    bottom.mount(Box::new(StaticComponent::new(&["bottom"])));
    tui.insert_layer(0, bottom);
    assert_eq!(tui.layer_count(), 2);
}

#[test]
fn tui_remove_layer_shifts_focus() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.add_layer(Layer::new());
    tui.set_focused_layer(1);
    let removed = tui.remove_layer(1);
    assert!(removed.is_some());
    assert_eq!(tui.layer_count(), 1);
}

#[test]
fn tui_layer_mut_allows_modification() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    let idx = tui.add_layer(Layer::new());
    if let Some(layer) = tui.layer_mut(idx) {
        layer.mount(Box::new(StaticComponent::new(&["modified"])));
    }
    assert!(tui.render_frame().is_ok());
}

#[test]
fn tui_top_layer_render_does_not_panic() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));

    let mut bottom = Layer::new();
    bottom.mount(Box::new(StaticComponent::new(&["bottom content"])));
    tui.add_layer(bottom);

    let mut top = Layer::new();
    top.mount(Box::new(StaticComponent::new(&["top content"])));
    tui.add_layer(top);

    assert!(tui.render_frame().is_ok());
}

#[test]
fn tui_layer_dim_shadow_render_does_not_panic() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));

    let mut bottom = Layer::new();
    bottom.mount(Box::new(StaticComponent::new(&["lower text"])));
    tui.add_layer(bottom);

    let mut top = Layer::new();
    top.mount(Box::new(StaticComponent::new(&["ABC"])));
    top.shadow = Shadow::Dim {
        style: "\x1b[2m".into(),
    };
    tui.add_layer(top);

    assert!(tui.render_frame().is_ok());
}

#[test]
fn tui_layer_drop_shadow_render_does_not_panic() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));

    let mut bottom = Layer::new();
    bottom.mount(Box::new(StaticComponent::new(&["background"])));
    tui.add_layer(bottom);

    let mut top = Layer::new();
    top.mount(Box::new(StaticComponent::new(&["panel"])));
    top.shadow = Shadow::Drop {
        style: "\x1b[2m".into(),
        offset_x: 1,
        offset_y: 1,
    };
    tui.add_layer(top);

    assert!(tui.render_frame().is_ok());
}

#[test]
fn tui_layer_api_does_not_break_existing_overlay_modal() {
    use photon_ui::{
        Anchor,
        Overlay,
        OverlayConstraints,
        OverlayPosition,
    };

    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(Text::new("background", 0, 0)));
    let overlay = Overlay {
        content: Box::new(Text::new("popup", 0, 0)),
        position: OverlayPosition::Anchor(Anchor::Center),
        constraints: OverlayConstraints {
            min_width: 5,
            max_height: 3,
            margin: 1,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        },
    };
    tui.add_overlay(overlay);
    tui.show_modal(Box::new(Text::new("modal", 0, 0)));
    assert!(tui.render_frame().is_ok());
    tui.dismiss_modal();
    tui.clear_overlays();
    assert!(tui.render_frame().is_ok());
}
