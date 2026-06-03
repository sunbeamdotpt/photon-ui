use crate::{
    Component,
    RenderError,
    Rendered,
    utils::truncate_to_width,
};

/// Static text that is truncated with an ellipsis if it exceeds the available
/// width.
///
/// Each line is padded with `pad_x` spaces and truncated independently.
/// `pad_y` empty lines are added before and after the content.
pub struct TruncatedText {
    text: String,
    pad_x: u16,
    pad_y: u16,
}

impl TruncatedText {
    /// Create a new truncated text component.
    pub fn new(text: impl Into<String>, pad_x: u16, pad_y: u16) -> Self {
        Self {
            text: text.into(),
            pad_x,
            pad_y,
        }
    }
}

impl Component for TruncatedText {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        for _ in 0..self.pad_y {
            lines.push("".to_string());
        }
        let pad = " ".repeat(self.pad_x as usize);
        let avail = width.saturating_sub(self.pad_x * 2);
        for line in self.text.lines() {
            let truncated = truncate_to_width(line, avail, "…");
            lines.push(format!("{}{}", pad, truncated));
        }
        for _ in 0..self.pad_y {
            lines.push("".to_string());
        }
        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_text_with_padding() {
        let t = TruncatedText::new("hello", 2, 1);
        let r = t.render(20).unwrap();
        assert_eq!(r.lines.len(), 3); // pad_y top + content + pad_y bottom
        assert!(r.lines[0].is_empty());
        assert!(r.lines[1].starts_with("  hello"));
        assert!(r.lines[2].is_empty());
    }

    #[test]
    fn truncated_text_empty_with_padding() {
        let t = TruncatedText::new("", 0, 2);
        let r = t.render(10).unwrap();
        assert_eq!(r.lines.len(), 4);
    }
}
