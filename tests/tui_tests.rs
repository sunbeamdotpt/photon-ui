use photon_ui::{TUI, Overlay, OverlayPosition, OverlayConstraints, Anchor, Component, Focusable, Rendered, RenderError};
use photon_ui::tui::Rect;
use photon_ui::terminal::TestTerminal;
use photon_ui::events::Event;
use crossterm::event::KeyCode;

struct DummyComponent {
    focused: bool,
    render_result: Vec<String>,
}

impl DummyComponent {
    fn new(lines: Vec<String>) -> Self {
        Self { focused: false, render_result: lines }
    }
}

impl Focusable for DummyComponent {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Component for DummyComponent {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        Ok(Rendered {
            lines: self.render_result.clone(),
            cursor: None,
            images: Vec::new(),
        })
    }
    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}

#[test]
fn tui_mount_sets_focus() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(DummyComponent::new(vec!["a".into()])));
    // First mounted component should be focused
    assert!(tui.terminal().size().is_ok());
}

#[test]
fn tui_set_focus_switches() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    let mut c1 = DummyComponent::new(vec!["a".into()]);
    let mut c2 = DummyComponent::new(vec!["b".into()]);
    c1.set_focused(true);
    tui.mount(Box::new(c1));
    tui.mount(Box::new(c2));
    // Should not panic
}

#[test]
fn tui_render_frame() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(DummyComponent::new(vec!["hello".into()])));
    tui.render_frame().unwrap();
    // Should not panic
}

#[test]
fn tui_handle_input() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(DummyComponent::new(vec!["a".into()])));
    tui.handle_input(&Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('x'), crossterm::event::KeyModifiers::empty())));
}

#[test]
fn overlay_anchor_center() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::Anchor(Anchor::Center),
        constraints: OverlayConstraints {
            min_width: 10,
            max_height: 5,
            margin: 1,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        },
    };
    let rect = overlay.compute_position(80, 24, 20, 3).unwrap();
    assert_eq!(rect.y, (24 - 3) / 2);
    assert_eq!(rect.x, (80 - 20) / 2);
    assert_eq!(rect.width, 20);
    assert_eq!(rect.height, 3);
}

#[test]
fn overlay_anchor_top_left() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::Anchor(Anchor::TopLeft),
        constraints: OverlayConstraints {
            min_width: 5,
            max_height: 5,
            margin: 2,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        },
    };
    let rect = overlay.compute_position(80, 24, 10, 3).unwrap();
    assert_eq!(rect.y, 2);
    assert_eq!(rect.x, 2);
}

#[test]
fn overlay_at_position() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::At(5, 10),
        constraints: OverlayConstraints {
            min_width: 1,
            max_height: 10,
            margin: 0,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        },
    };
    let rect = overlay.compute_position(80, 24, 10, 3).unwrap();
    assert_eq!(rect.y, 5);
    assert_eq!(rect.x, 10);
}

#[test]
fn overlay_percent_position() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::Percent("50%".into(), "50%".into()),
        constraints: OverlayConstraints {
            min_width: 1,
            max_height: 10,
            margin: 0,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        },
    };
    let rect = overlay.compute_position(100, 100, 10, 10).unwrap();
    assert_eq!(rect.y, 50);
    assert_eq!(rect.x, 50);
}

#[test]
fn overlay_not_visible() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::Anchor(Anchor::Center),
        constraints: OverlayConstraints {
            min_width: 1,
            max_height: 10,
            margin: 0,
            offset_x: 0,
            offset_y: 0,
            visible: Some(|_w, _h| false),
        },
    };
    assert!(overlay.compute_position(80, 24, 10, 3).is_none());
}

#[test]
fn overlay_with_offset() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::Anchor(Anchor::TopLeft),
        constraints: OverlayConstraints {
            min_width: 1,
            max_height: 10,
            margin: 0,
            offset_x: 5,
            offset_y: 3,
            visible: None,
        },
    };
    let rect = overlay.compute_position(80, 24, 10, 3).unwrap();
    assert_eq!(rect.y, 3);
    assert_eq!(rect.x, 5);
}

#[test]
fn overlay_clamped_to_terminal() {
    let overlay = Overlay {
        content: Box::new(DummyComponent::new(vec!["test".into()])),
        position: OverlayPosition::Anchor(Anchor::BottomRight),
        constraints: OverlayConstraints {
            min_width: 100,
            max_height: 10,
            margin: 0,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        },
    };
    let rect = overlay.compute_position(80, 24, 10, 3).unwrap();
    assert!(rect.width <= 80);
    assert!(rect.height <= 24);
}
