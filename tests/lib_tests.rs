use photon_ui::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
};

struct MinimalComponent;

impl Component for MinimalComponent {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        Ok(Rendered::empty())
    }
}

#[test]
fn component_default_methods() {
    let mut c = MinimalComponent;
    // Default handle_input returns Ignored
    assert_eq!(c.handle_input(&Event::Resize(10, 10)), InputResult::Ignored);
    // Default wants_key_release returns false
    assert!(!c.wants_key_release());
    // Default as_focusable returns None
    assert!(c.as_focusable().is_none());
    assert!(c.as_focusable_mut().is_none());
}

struct FocusableComponent {
    focused: bool,
}

impl Focusable for FocusableComponent {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for FocusableComponent {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        Ok(Rendered::empty())
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> {
        Some(self)
    }

    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        Some(self)
    }
}

#[test]
fn focusable_component_methods() {
    let mut c = FocusableComponent { focused: false };
    assert!(!c.focused());
    c.set_focused(true);
    assert!(c.focused());
    assert!(c.as_focusable().is_some());
    assert!(c.as_focusable_mut().is_some());
}
