use crate::{Component, Rendered, RenderError};

/// A rectangular box filled with spaces, optionally with a background color.
///
/// The box renders `pad_y` rows of spaces at the requested width. A background
/// function can be applied to each line via [`with_background`](Box::with_background).
pub struct Box {
    pad_y: u16,
    bg_fn: Option<fn(&str, u16) -> String>,
}

impl Box {
    /// Create a new box with the given vertical padding.
    pub fn new(pad_y: u16) -> Self {
        Self { pad_y, bg_fn: None }
    }

    /// Apply a background color function to every line.
    ///
    /// The function receives `(line_text, width)` and should return the
    /// styled line.
    pub fn with_background(mut self, bg: fn(&str, u16) -> String) -> Self {
        self.bg_fn = Some(bg);
        self
    }
}

impl Component for Box {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let empty = " ".repeat(width as usize);
        for _ in 0..self.pad_y {
            lines.push(empty.clone());
        }
        if let Some(bg) = self.bg_fn {
            lines = lines.into_iter().map(|l| bg(&l, width)).collect();
        }
        Ok(Rendered { lines, cursor: None, images: Vec::new() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_with_background() {
        let b = Box::new(2).with_background(|line, _w| format!(">{}<", line));
        let r = b.render(5).unwrap();
        assert_eq!(r.lines.len(), 2);
    }
}
