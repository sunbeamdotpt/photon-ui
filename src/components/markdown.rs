use pulldown_cmark::{
    Event as MdEvent,
    Parser,
    Tag,
    TagEnd,
};

use crate::{
    Component,
    RenderError,
    Rendered,
};

/// Renders CommonMark / Markdown text as styled terminal output.
///
/// Supports headings (bold + underline, no `#` prefix), bold, italic,
/// inline code (configurable style, no backticks), lists with bullet markers,
/// soft/hard breaks, and raw HTML passthrough. Text is automatically wrapped
/// to the requested width via
/// [`wrap_text_with_ansi`](crate::utils::wrap_text_with_ansi).
pub struct Markdown {
    text: String,
    code_style: Option<fn(&str) -> String>,
}

impl Markdown {
    /// Create a new Markdown component from the given text.
    ///
    /// Defaults: headings are bold+underlined, inline code is cyan, bold is
    /// bright white, italic is slanted.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            code_style: None,
        }
    }

    /// Override the inline code styling.
    ///
    /// The function receives the raw code text and should return the styled
    /// string (including any ANSI reset). Defaults to cyan foreground.
    ///
    /// # Example
    ///
    /// ```
    /// use photon_ui::components::Markdown;
    ///
    /// let md = Markdown::new("`hello`").with_code_style(|s| format!("\x1b[48;5;240m{}\x1b[0m", s));
    /// ```
    pub fn with_code_style(mut self, style: fn(&str) -> String) -> Self {
        self.code_style = Some(style);
        self
    }
}

impl Component for Markdown {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let parser = Parser::new(&self.text);
        let mut current_line = String::new();
        let mut in_bold = false;
        let mut in_italic = false;
        let mut pending_bullet = false;

        // Helper: prepend a bullet to `current_line` if this is the first
        // line of a list item, then push it to `lines` and clear.
        let push_line = |line: &mut String, bullet: &mut bool, dest: &mut Vec<String>| {
            if !line.is_empty() {
                if *bullet {
                    *line = format!("- {}", line);
                    *bullet = false;
                }
                dest.push(line.clone());
                line.clear();
            }
        };

        for event in parser {
            match event {
                | MdEvent::Start(tag) => match tag {
                    | Tag::Heading { .. } => {},
                    | Tag::Strong => in_bold = true,
                    | Tag::Emphasis => in_italic = true,
                    | Tag::Item => pending_bullet = true,
                    | _ => {},
                },
                | MdEvent::End(tag_end) => {
                    match tag_end {
                        | TagEnd::Heading(_) => {
                            if !current_line.is_empty() {
                                // Headings: bold + underline, no Markdown # prefix
                                let styled = format!("\x1b[1m\x1b[4m{}\x1b[0m", current_line);
                                if pending_bullet {
                                    lines.push(format!("- {}", styled));
                                    pending_bullet = false;
                                } else {
                                    lines.push(styled);
                                }
                                current_line.clear();
                            }
                        },
                        | TagEnd::Paragraph => {
                            push_line(&mut current_line, &mut pending_bullet, &mut lines);
                            lines.push("".to_string());
                        },
                        | TagEnd::Item => {
                            push_line(&mut current_line, &mut pending_bullet, &mut lines);
                        },
                        | TagEnd::Strong => in_bold = false,
                        | TagEnd::Emphasis => in_italic = false,
                        | _ => {},
                    }
                },
                | MdEvent::Text(text) => {
                    let mut styled = text.to_string();
                    if in_bold {
                        // Bold: bright white for visibility
                        styled = format!("\x1b[1m\x1b[97m{}\x1b[0m", styled);
                    }
                    if in_italic {
                        styled = format!("\x1b[3m{}\x1b[23m", styled);
                    }
                    current_line.push_str(&styled);
                },
                | MdEvent::Code(code) => {
                    let styled = if let Some(style) = self.code_style {
                        style(&code)
                    } else {
                        format!("\x1b[36m{}\x1b[0m", code)
                    };
                    current_line.push_str(&styled);
                },
                | MdEvent::SoftBreak | MdEvent::HardBreak => {
                    push_line(&mut current_line, &mut pending_bullet, &mut lines);
                },
                | MdEvent::Html(html) => {
                    current_line.push_str(&html);
                },
                | _ => {},
            }
        }

        if !current_line.is_empty() {
            if pending_bullet {
                current_line = format!("- {}", current_line);
            }
            lines.push(current_line);
        }

        let mut wrapped = Vec::new();
        for line in lines {
            if crate::utils::visible_width(&line) > width as usize {
                wrapped.extend(crate::utils::wrap_text_with_ansi(&line, width));
            } else {
                wrapped.push(line);
            }
        }

        Ok(Rendered {
            lines: wrapped,
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_bold() {
        let md = Markdown::new("**bold**");
        let r = md.render(80).unwrap();
        assert!(r.lines[0].contains("\x1b[1m"));
        assert!(r.lines[0].contains("\x1b[97m"));
    }

    #[test]
    fn markdown_italic() {
        let md = Markdown::new("*italic*");
        let r = md.render(80).unwrap();
        assert!(r.lines[0].contains("\x1b[3m"));
    }

    #[test]
    fn markdown_inline_code() {
        let md = Markdown::new("`code`");
        let r = md.render(80).unwrap();
        // Default: cyan foreground, no backticks
        assert!(r.lines[0].contains("\x1b[36m"));
        assert!(!r.lines[0].contains("`code`"));
        assert!(r.lines[0].contains("code"));
    }

    #[test]
    fn markdown_inline_code_custom_style() {
        let md = Markdown::new("`code`").with_code_style(|s| format!(">{}<", s));
        let r = md.render(80).unwrap();
        assert!(r.lines[0].contains(">code<"));
    }

    #[test]
    fn markdown_heading_no_hash() {
        let md = Markdown::new("# Hello");
        let r = md.render(80).unwrap();
        assert!(!r.lines[0].contains("# Hello"));
        assert!(r.lines[0].contains("Hello"));
        assert!(r.lines[0].contains("\x1b[1m"));
        assert!(r.lines[0].contains("\x1b[4m"));
    }

    #[test]
    fn markdown_soft_break() {
        let md = Markdown::new("line1\nline2");
        let r = md.render(80).unwrap();
        assert!(r.lines.iter().any(|l| l.contains("line1")));
    }

    #[test]
    fn markdown_html_passthrough() {
        let md = Markdown::new("<div>text</div>\n\nmore");
        let r = md.render(80).unwrap();
        assert!(!r.lines.is_empty());
    }

    #[test]
    fn markdown_list_items_separate_lines() {
        let md = Markdown::new("- item one\n- item two\n- item three");
        let r = md.render(80).unwrap();
        let item_lines: Vec<&String> = r.lines.iter().filter(|l| l.contains("item")).collect();
        assert_eq!(
            item_lines.len(),
            3,
            "each list item should be on its own line: {:?}",
            r.lines
        );
    }

    #[test]
    fn markdown_list_has_bullets() {
        let md = Markdown::new("- first\n- second");
        let r = md.render(80).unwrap();
        assert!(
            r.lines.iter().any(|l| l.contains("- first")),
            "expected bullets: {:?}",
            r.lines
        );
        assert!(
            r.lines.iter().any(|l| l.contains("- second")),
            "expected bullets: {:?}",
            r.lines
        );
    }

    #[test]
    fn markdown_list_with_styling() {
        let md = Markdown::new("- *italic* item\n- **bold** item");
        let r = md.render(80).unwrap();
        let italic_line = r.lines.iter().find(|l| l.contains("italic")).unwrap();
        assert!(
            italic_line.contains("- "),
            "expected bullet: {}",
            italic_line
        );
        assert!(italic_line.contains("\x1b[3m"));

        let bold_line = r.lines.iter().find(|l| l.contains("bold")).unwrap();
        assert!(bold_line.contains("- "), "expected bullet: {}", bold_line);
        assert!(bold_line.contains("\x1b[1m"));
    }

    #[test]
    fn markdown_no_unnecessary_wrapping_for_wide_chars() {
        // "中文" is 6 bytes but only 4 visible columns wide.
        // A byte-length check would trigger unnecessary wrapping at width 5.
        let md = Markdown::new("中文");
        let r = md.render(5).unwrap();
        // Markdown paragraphs produce a text line followed by an empty line.
        let text_lines: Vec<&String> = r.lines.iter().filter(|l| !l.is_empty()).collect();
        assert_eq!(
            text_lines.len(),
            1,
            "CJK text with visible_width 4 should fit in width 5: {:?}",
            r.lines
        );
        assert!(
            text_lines[0].contains("中文"),
            "text should be intact: {:?}",
            text_lines
        );
    }
}
