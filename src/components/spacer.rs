use crate::{
    Component,
    RenderError,
    Rendered,
};

/// A component that renders empty lines for layout spacing.
pub struct Spacer {
    lines: usize,
}

impl Spacer {
    /// Create a spacer that renders `lines` empty rows.
    pub fn new(lines: usize) -> Self {
        Self { lines }
    }
}

impl Component for Spacer {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        Ok(Rendered {
            lines: vec!["".to_string(); self.lines],
            cursor: None,
            images: Vec::new(),
        })
    }
}
