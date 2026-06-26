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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacer_renders_requested_lines() {
        let spacer = Spacer::new(3);
        let rendered = spacer.render(80).unwrap();
        assert_eq!(rendered.lines.len(), 3);
        assert!(rendered.lines.iter().all(|line| line.is_empty()));
    }

    #[test]
    fn spacer_zero_lines() {
        let spacer = Spacer::new(0);
        let rendered = spacer.render(80).unwrap();
        assert!(rendered.lines.is_empty());
    }
}
