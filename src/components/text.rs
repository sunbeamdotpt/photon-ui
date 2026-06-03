use crate::{
    Component,
    RenderError,
    Rendered,
};

/// Static text with optional horizontal and vertical padding.
///
/// Each line of the input text is prefixed with `pad_x` spaces, truncated to
/// fit within the requested width, and padded with trailing spaces so that old
/// terminal content on the same line is fully overwritten. `pad_y` empty lines
/// are added before and after the content.
pub struct Text {
    text: String,
    pad_x: u16,
    pad_y: u16,
}

impl Text {
    /// Create a new text component.
    pub fn new(text: impl Into<String>, pad_x: u16, pad_y: u16) -> Self {
        Self {
            text: text.into(),
            pad_x,
            pad_y,
        }
    }
}

impl Component for Text {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let empty_line = " ".repeat(width as usize);
        for _ in 0..self.pad_y {
            lines.push(empty_line.clone());
        }
        let pad = " ".repeat(self.pad_x as usize);
        for line in self.text.lines() {
            let padded = format!("{}{}", pad, line);
            let truncated = crate::utils::truncate_to_width(&padded, width, "…");
            let vw = crate::utils::visible_width(&truncated);
            let mut final_line = truncated;
            if vw < width as usize {
                final_line.push_str(&" ".repeat(width as usize - vw));
            }
            lines.push(final_line);
        }
        for _ in 0..self.pad_y {
            lines.push(empty_line.clone());
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
    fn text_truncates_to_width() {
        let text = Text::new("hello world", 0, 0);
        let rendered = text.render(5).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert_eq!(rendered.lines[0].trim_end(), "hell…"); // ellipsis is 3 cols wide
    }

    #[test]
    fn text_pads_to_width() {
        let text = Text::new("hi", 0, 0);
        let rendered = text.render(10).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert_eq!(rendered.lines[0].len(), 10);
        assert_eq!(rendered.lines[0], "hi        ");
    }

    #[test]
    fn text_pad_y_adds_blank_lines() {
        let text = Text::new("a", 0, 2);
        let rendered = text.render(10).unwrap();
        assert_eq!(rendered.lines.len(), 5); // 2 + 1 + 2
        assert_eq!(rendered.lines[0], "          "); // padded to width
        assert_eq!(rendered.lines[1], "          ");
        assert!(rendered.lines[2].starts_with("a"));
        assert_eq!(rendered.lines[3], "          ");
        assert_eq!(rendered.lines[4], "          ");
    }

    #[test]
    fn text_pad_x_prefixes_spaces() {
        let text = Text::new("x", 3, 0);
        let rendered = text.render(10).unwrap();
        assert_eq!(rendered.lines[0], "   x      "); // 3 pad + 1 text + 6 spaces = 10
    }

    #[test]
    fn text_long_line_with_pad_x_truncates() {
        let text = Text::new("abcdefghij", 5, 0);
        let rendered = text.render(10).unwrap();
        // 5 spaces + 10 chars = 15, truncated to 10 with 3-col ellipsis
        assert!(rendered.lines[0].starts_with("     ")); // 5 pad spaces preserved
        assert!(rendered.lines[0].trim_end().ends_with("abcd…"));
    }
}
