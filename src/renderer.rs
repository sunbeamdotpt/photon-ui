use thiserror::Error;

/// Error type returned when a component produces invalid output.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RenderError {
    /// A rendered line exceeds the allowed width.
    #[error("width overflow: line width {actual} exceeds {width}")]
    WidthOverflow {
        /// The offending line content.
        line: String,
        /// The maximum allowed width.
        width: u16,
        /// The measured width of the line.
        actual: usize,
    },
}

/// Result of handling an input event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputResult {
    /// The event was consumed and handled by this component.
    Handled,
    /// The event was not relevant; the framework may propagate it.
    Ignored,
    /// The event was handled and the component requests an immediate re-render.
    RequestRender,
}

/// The output of a component's [`render`](crate::Component::render) call.
///
/// Contains the text lines to display, an optional cursor position, and
/// any terminal image commands that should be emitted.
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    /// Lines of text, each guaranteed to fit within the requested width.
    pub lines: Vec<String>,
    /// Optional cursor position as `(row, col)` in screen coordinates.
    pub cursor: Option<(usize, usize)>,
    /// Terminal image commands (Kitty / iTerm2 protocols) to emit.
    pub images: Vec<ImageCommand>,
}

/// A command to display an image in the terminal.
///
/// Images are identified by an `id` so the renderer can track which images
/// are still visible and delete stale ones. `row` and `col` are the screen
/// cell where the terminal should place the top-left corner of the image.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCommand {
    /// Unique identifier for this image.
    pub id: u32,
    /// Raw image data or protocol-specific payload.
    pub data: String,
    /// Target row (0-indexed) for the top-left corner of the image.
    pub row: u16,
    /// Target column (0-indexed) for the top-left corner of the image.
    pub col: u16,
}

impl Rendered {
    /// Create an empty rendered frame with no lines, cursor, or images.
    pub fn empty() -> Self {
        Self {
            lines: Vec::new(),
            cursor: None,
            images: Vec::new(),
        }
    }

    /// Composite this rendered content onto a target at the given offset.
    ///
    /// Lines are overwritten starting at `row` / `col`. The cursor and image
    /// commands are translated and appended to the target.
    pub fn blit_onto(&self, target: &mut Rendered, row: u16, col: u16) {
        for (i, line) in self.lines.iter().enumerate() {
            let target_row = row as usize + i;
            if target_row >= target.lines.len() {
                break;
            }
            let col_usize = col as usize;
            let target_vw = crate::utils::visible_width(&target.lines[target_row]);
            // Pad target line so the overlay has something to overwrite.
            if target_vw < col_usize {
                target.lines[target_row].push_str(&" ".repeat(col_usize - target_vw));
            }
            let source_vw = crate::utils::visible_width(line);
            let end = col_usize + source_vw;
            let target_vw_after = crate::utils::visible_width(&target.lines[target_row]);
            if end > target_vw_after {
                target.lines[target_row].push_str(&" ".repeat(end - target_vw_after));
            }
            let start_byte =
                crate::utils::byte_index_at_visual_pos(&target.lines[target_row], col_usize);
            let end_byte = crate::utils::byte_index_at_visual_pos(&target.lines[target_row], end);
            target.lines[target_row].replace_range(start_byte..end_byte, line);
        }
        if let Some((r, c)) = self.cursor {
            target.cursor = Some((row as usize + r, col as usize + c));
        }
        for image in &self.images {
            target.images.push(ImageCommand {
                id: image.id,
                data: image.data.clone(),
                row: row + image.row,
                col: col + image.col,
            });
        }
    }

    /// Composite this rendered content into a target at the given rect.
    ///
    /// Lines are clipped to `rect.height`. Each line is inserted at `rect.x`
    /// and truncated to `rect.width`. The cursor and images are translated.
    pub fn blit_into_rect(&self, target: &mut Rendered, rect: Rect) {
        for (i, line) in self.lines.iter().enumerate().take(rect.height as usize) {
            let target_row = rect.y as usize + i;
            if target_row >= target.lines.len() {
                while target.lines.len() <= target_row {
                    target.lines.push(String::new());
                }
            }
            let col = rect.x as usize;
            let target_line = &mut target.lines[target_row];
            let target_vw = crate::utils::visible_width(target_line);
            if target_vw < col {
                target_line.push_str(&" ".repeat(col - target_vw));
            }
            let truncated = if crate::utils::visible_width(line) > rect.width as usize {
                Some(crate::utils::truncate_to_width(line, rect.width, ""))
            } else {
                None
            };
            let source = truncated.as_deref().unwrap_or(line);
            let vw = crate::utils::visible_width(source);
            let end = col + vw;
            let target_vw_after = crate::utils::visible_width(target_line);
            if end > target_vw_after {
                target_line.push_str(&" ".repeat(end - target_vw_after));
            }
            let mut start_byte = crate::utils::byte_index_at_visual_pos(target_line, col);
            let end_byte = crate::utils::byte_index_at_visual_pos(target_line, end);
            // Preserve ANSI reset codes (\x1b[0m) at the start boundary so
            // background colours don't bleed into the next component.
            if target_line.as_bytes().get(start_byte) == Some(&b'\x1b') &&
                target_line[start_byte..].starts_with("\x1b[0m")
            {
                start_byte = (start_byte + "\x1b[0m".len()).min(end_byte);
            }
            target_line.replace_range(start_byte..end_byte, source);
        }
        if let Some((r, c)) = self.cursor {
            target.cursor = Some((rect.y as usize + r, rect.x as usize + c));
        }
        for image in &self.images {
            target.images.push(ImageCommand {
                id: image.id,
                data: image.data.clone(),
                row: rect.y + image.row,
                col: rect.x + image.col,
            });
        }
    }
}

use std::{
    collections::HashMap,
    io,
};

macro_rules! try_io {
    ($expr:expr) => {
        match $expr {
            | Ok(v) => v,
            | Err(e) => return Err(e),
        }
    };
}

use crate::{
    layout::Rect,
    terminal::Terminal,
};

/// Extract the Kitty placement APC sequence from a full transmit-and-place
/// payload. The placement command is always appended last by `encode_kitty`.
fn extract_placement(data: &str) -> Option<String> {
    data.rfind("\x1b_Ga=p")
        .map(|start| data[start..].to_string())
}

fn append_images(
    buffer: &mut String,
    images: &[ImageCommand],
    transmitted: &mut HashMap<u32, String>,
) {
    for image in images {
        // Kitty's a=p command places the image at the current cursor position.
        // Move the cursor to the cell where the component requested the image
        // so placement stays stable across diff renders.
        buffer.push_str(&format!("\x1b[{};{}H", image.row + 1, image.col + 1));
        // Only transmit the payload once per id; subsequent frames just place
        // the existing image. This avoids re-compositing artifacts that can
        // make the display look washed out.
        if let Some(placement) = transmitted.get(&image.id) {
            buffer.push_str(placement);
        } else {
            buffer.push_str(&image.data);
            if let Some(placement) = extract_placement(&image.data) {
                transmitted.insert(image.id, placement);
            }
        }
    }
}

impl Renderer {
    /// Write the rendered output to the terminal using the current strategy.
    ///
    /// This implementation closely follows the original TypeScript TUI
    /// renderer:
    /// - FirstRender: outputs all lines without clearing (assumes clean
    ///   alternate screen).
    /// - FullRedraw: clears screen + scrollback, then outputs all lines.
    /// - Diff: computes first/last changed line, moves cursor there, and only
    ///   rewrites the changed region using `\x1b[2K` per line.
    pub fn render(&mut self, term: &mut dyn Terminal, rendered: &Rendered) -> io::Result<()> {
        match self.strategy {
            | RenderStrategy::FirstRender => {
                let mut buffer = String::from("\x1b[?2026h\x1b[0m\x1b[2J\x1b[H");
                for (i, line) in rendered.lines.iter().enumerate() {
                    if i > 0 {
                        buffer.push_str("\r\n");
                    }
                    buffer.push_str(line);
                }
                append_images(&mut buffer, &rendered.images, &mut self.transmitted_images);
                buffer.push_str("\x1b[?2026l");
                try_io!(term.write(&buffer));
            },
            | RenderStrategy::FullRedraw => {
                // After a full redraw the terminal may have cleared image
                // placements, so re-transmit on the next appearance.
                self.transmitted_images.clear();
                let mut buffer = String::from("\x1b[?2026h\x1b[0m\x1b[2J\x1b[H\x1b[3J");
                for (i, line) in rendered.lines.iter().enumerate() {
                    if i > 0 {
                        buffer.push_str("\r\n");
                    }
                    buffer.push_str(line);
                }
                append_images(&mut buffer, &rendered.images, &mut self.transmitted_images);
                buffer.push_str("\x1b[?2026l");
                try_io!(term.write(&buffer));
            },
            | RenderStrategy::Diff => {
                if let Some(ref prev) = self.previous {
                    let mut first_diff: Option<usize> = None;
                    let mut last_diff: usize = 0;
                    let max_lines = prev.lines.len().max(rendered.lines.len());
                    for i in 0..max_lines {
                        let old = prev.lines.get(i).map(|s| s.as_str()).unwrap_or("");
                        let new = rendered.lines.get(i).map(|s| s.as_str()).unwrap_or("");
                        if old != new {
                            if first_diff.is_none() {
                                first_diff = Some(i);
                            }
                            last_diff = i;
                        }
                    }
                    let images_changed = prev.images != rendered.images;

                    // All changes are in deleted lines (nothing new to render, just clear)
                    if first_diff.is_some_and(|f| f >= rendered.lines.len()) {
                        if prev.lines.len() > rendered.lines.len() {
                            let mut buffer = String::from("\x1b[?2026h");
                            let target_row = rendered.lines.len().saturating_sub(1);
                            if target_row > 0 {
                                buffer.push_str(&format!("\x1b[{};1H", target_row + 1));
                            }
                            buffer.push('\r');
                            let extra = prev.lines.len() - rendered.lines.len();
                            if extra > 0 {
                                buffer.push_str("\x1b[1B");
                            }
                            for i in 0..extra {
                                buffer.push_str("\r\x1b[0m\x1b[2K");
                                if i < extra - 1 {
                                    buffer.push_str("\x1b[1B");
                                }
                            }
                            if extra > 0 {
                                buffer.push_str(&format!("\x1b[{}A", extra));
                            }
                            append_images(
                                &mut buffer,
                                &rendered.images,
                                &mut self.transmitted_images,
                            );
                            buffer.push_str("\x1b[?2026l");
                            try_io!(term.write(&buffer));
                        }
                    } else if let Some(start) = first_diff {
                        let mut buffer = String::from("\x1b[?2026h");
                        // Move cursor to first changed line (1-indexed row, col 1)
                        buffer.push_str(&format!("\x1b[{};1H", start + 1));
                        // Carriage return to column 0
                        buffer.push('\r');

                        let render_end = last_diff.min(rendered.lines.len().saturating_sub(1));
                        for i in start..=render_end {
                            if i > start {
                                buffer.push_str("\r\n");
                            }
                            buffer.push_str("\x1b[0m\x1b[2K");
                            buffer.push_str(&rendered.lines[i]);
                        }

                        // Previous frame had more lines: clear the extra ones
                        if prev.lines.len() > rendered.lines.len() {
                            let extra = prev.lines.len() - rendered.lines.len();
                            for _ in 0..extra {
                                buffer.push_str("\r\n\x1b[0m\x1b[2K");
                            }
                            // Move cursor back to end of new content
                            if extra > 0 {
                                buffer.push_str(&format!("\x1b[{}A", extra));
                            }
                        }

                        append_images(&mut buffer, &rendered.images, &mut self.transmitted_images);
                        buffer.push_str("\x1b[?2026l");
                        try_io!(term.write(&buffer));
                    } else if images_changed {
                        let mut buffer = String::from("\x1b[?2026h");
                        append_images(&mut buffer, &rendered.images, &mut self.transmitted_images);
                        buffer.push_str("\x1b[?2026l");
                        try_io!(term.write(&buffer));
                    }
                } else {
                    // No previous frame but Diff strategy: treat as first render
                    let mut buffer = String::from("\x1b[?2026h");
                    for (i, line) in rendered.lines.iter().enumerate() {
                        if i > 0 {
                            buffer.push_str("\r\n");
                        }
                        buffer.push_str(line);
                    }
                    append_images(&mut buffer, &rendered.images, &mut self.transmitted_images);
                    buffer.push_str("\x1b[?2026l");
                    try_io!(term.write(&buffer));
                }
            },
        }

        if let Some((row, col)) = rendered.cursor {
            try_io!(term.move_cursor(row as u16, col as u16));
        }

        self.previous = Some(rendered.clone());
        self.strategy = RenderStrategy::Diff;
        Ok(())
    }
}

/// Strategy used by [`Renderer`] to draw a frame.
#[derive(Default)]
pub enum RenderStrategy {
    /// Full draw with no previous state; clears and redraws everything.
    #[default]
    FirstRender,
    /// Force a complete screen clear and redraw.
    FullRedraw,
    /// Compute a minimal diff against the previous frame and only redraw
    /// changed lines.
    Diff,
}

/// Differential terminal renderer.
///
/// Tracks the previous frame to enable efficient redrawing. The strategy
/// is automatically reset to [`Diff`](RenderStrategy::Diff) after each render.
#[derive(Default)]
pub struct Renderer {
    previous: Option<Rendered>,
    strategy: RenderStrategy,
    transmitted_images: HashMap<u32, String>,
}

impl Renderer {
    /// Create a new renderer with no previous frame and
    /// [`FirstRender`](RenderStrategy::FirstRender) strategy.
    pub fn new() -> Self {
        Self {
            previous: None,
            strategy: RenderStrategy::FirstRender,
            transmitted_images: HashMap::new(),
        }
    }

    /// Override the strategy for the next render call.
    pub fn set_strategy(&mut self, strategy: RenderStrategy) {
        self.strategy = strategy;
    }

    /// Access the previously rendered frame, if any.
    pub fn previous(&self) -> Option<&Rendered> {
        self.previous.as_ref()
    }

    /// Forget a previously-transmitted image id.
    ///
    /// Call this when the image is deleted from the terminal so that a future
    /// image with the same id will be re-transmitted.
    pub fn forget_image(&mut self, id: u32) {
        self.transmitted_images.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::TestTerminal;

    #[test]
    fn first_render_strategy() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();
        let rendered = Rendered {
            lines: vec!["hello".into()],
            cursor: None,
            images: vec![ImageCommand {
                id: 1,
                data: "img".into(),
                row: 0,
                col: 0,
            }],
        };
        renderer.render(&mut term, &rendered).unwrap();
        let written = term.written().join("");
        assert!(written.contains("hello"));
        assert!(written.contains("img"));
        assert!(written.contains("\x1b[?2026h"));
        // First render clears screen and homes cursor
        assert!(written.contains("\x1b[H"));
        assert!(written.contains("\x1b[2J"));
        assert!(!written.contains("\x1b[2K"));
    }

    #[test]
    fn full_redraw_clears_screen() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();
        renderer.set_strategy(RenderStrategy::FullRedraw);
        let rendered = Rendered {
            lines: vec!["test".into()],
            cursor: Some((0, 1)),
            images: Vec::new(),
        };
        renderer.render(&mut term, &rendered).unwrap();
        assert!(term.cursor_moves().contains(&(0, 1)));
        let written = term.written().join("");
        assert!(written.contains("\x1b[2J"));
        assert!(written.contains("\x1b[3J"));
    }

    #[test]
    fn diff_clears_changed_lines() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();

        // First render
        let frame1 = Rendered {
            lines: vec!["long old line content".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame1).unwrap();

        // Second render with shorter line — diff must clear old trailing chars
        renderer.set_strategy(RenderStrategy::Diff);
        let frame2 = Rendered {
            lines: vec!["short".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame2).unwrap();

        let written = term.written().join("");
        assert!(
            written.contains("\x1b[2K"),
            "diff must clear each changed line"
        );
    }

    #[test]
    fn diff_skips_unchanged_lines() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();

        let frame1 = Rendered {
            lines: vec!["a".into(), "b".into(), "c".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame1).unwrap();

        renderer.set_strategy(RenderStrategy::Diff);
        let frame2 = Rendered {
            lines: vec!["a".into(), "B".into(), "c".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame2).unwrap();

        let written = term.written().join("");
        // Should move cursor to line 2 and only rewrite from there
        assert!(
            written.contains("\x1b[2;1H"),
            "cursor should jump to first changed line"
        );
        // Should use \r (not \r\n) after positioning
        assert!(
            written.contains("\x1b[2;1H\r\x1b[0m\x1b[2K"),
            "should use \\r after positioning"
        );
        // Should NOT rewrite line 3 (unchanged)
        let after_line2 = written.split("\x1b[2;1H").nth(1).unwrap_or("");
        assert!(
            !after_line2.contains("\r\nc"),
            "should not rewrite unchanged line 3"
        );
    }

    #[test]
    fn diff_no_previous_treats_as_first_render() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();
        renderer.set_strategy(RenderStrategy::Diff);
        let rendered = Rendered {
            lines: vec!["test".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &rendered).unwrap();
        let written = term.written().join("");
        // No previous frame: treat as first render, no screen clear
        assert!(!written.contains("\x1b[2J"));
        assert!(written.contains("test"));
    }

    #[test]
    fn diff_clears_deleted_lines() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();

        let frame1 = Rendered {
            lines: vec!["a".into(), "b".into(), "c".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame1).unwrap();

        renderer.set_strategy(RenderStrategy::Diff);
        let frame2 = Rendered {
            lines: vec!["a".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame2).unwrap();

        let written = term.written().join("");
        // Should clear the 2 extra lines from previous frame
        assert!(written.contains("\x1b[2K"), "should clear deleted lines");
    }

    #[test]
    fn diff_emits_image_commands() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();

        let frame1 = Rendered {
            lines: vec!["a".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame1).unwrap();

        renderer.set_strategy(RenderStrategy::Diff);
        let frame2 = Rendered {
            lines: vec!["a".into()],
            cursor: None,
            images: vec![ImageCommand {
                id: 5,
                data: "img".into(),
                row: 0,
                col: 0,
            }],
        };
        renderer.render(&mut term, &frame2).unwrap();

        let written = term.written().join("");
        assert!(written.contains("img"), "diff must emit image commands");
    }

    #[test]
    fn diff_reuses_transmitted_image_without_payload() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();

        let payload = "\x1b_Ga=t,f=100,i=9,q=1,m=0;DATA\x1b\\\x1b_Ga=p,i=9,c=2,r=1,q=1\x1b\\";
        let frame1 = Rendered {
            lines: vec!["a".into()],
            cursor: None,
            images: vec![ImageCommand {
                id: 9,
                data: payload.into(),
                row: 0,
                col: 0,
            }],
        };
        renderer.render(&mut term, &frame1).unwrap();
        let first = term.written().last().map(String::as_str).unwrap_or("");
        assert!(first.contains("a=t"));
        assert!(first.contains("a=p"));

        renderer.set_strategy(RenderStrategy::Diff);
        let frame2 = Rendered {
            lines: vec!["b".into()],
            cursor: None,
            images: vec![ImageCommand {
                id: 9,
                data: payload.into(),
                row: 0,
                col: 0,
            }],
        };
        renderer.render(&mut term, &frame2).unwrap();
        let second = term.written().last().map(String::as_str).unwrap_or("");
        assert!(
            !second.contains("a=t"),
            "image payload should not be re-transmitted on subsequent frames"
        );
        assert!(
            second.contains("a=p"),
            "image placement should still be emitted"
        );
    }

    #[test]
    fn blit_onto_with_images() {
        let mut target = Rendered {
            lines: vec!["hello world".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into()],
            cursor: Some((0, 1)),
            images: vec![ImageCommand {
                id: 1,
                data: "img".into(),
                row: 0,
                col: 0,
            }],
        };
        source.blit_onto(&mut target, 0, 6);
        assert_eq!(target.images.len(), 1);
        assert_eq!(target.images[0].row, 0);
        assert_eq!(target.images[0].col, 6);
    }

    #[test]
    fn blit_into_rect_basic() {
        let mut target = Rendered {
            lines: vec!["hello world".into(), "second line".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into(), "Z".into()],
            cursor: Some((0, 1)),
            images: vec![ImageCommand {
                id: 1,
                data: "img".into(),
                row: 0,
                col: 0,
            }],
        };
        source.blit_into_rect(&mut target, Rect::new(6, 0, 10, 2));
        assert_eq!(target.lines[0], "hello XYrld");
        assert_eq!(target.lines[1], "secondZline");
        assert_eq!(target.cursor, Some((0, 7)));
        assert_eq!(target.images.len(), 1);
        assert_eq!(target.images[0].row, 0);
        assert_eq!(target.images[0].col, 6);
    }

    #[test]
    fn blit_into_rect_clips_height() {
        let mut target = Rendered {
            lines: vec!["aaaaaaaaaa".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["1".into(), "2".into(), "3".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, Rect::new(0, 0, 10, 1));
        assert_eq!(target.lines[0], "1aaaaaaaaa");
        assert_eq!(target.lines.len(), 1);
    }

    #[test]
    fn blit_into_rect_clips_width() {
        let mut target = Rendered {
            lines: vec!["aaaaaaaaaa".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["1234567890ABCDEF".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, Rect::new(0, 0, 5, 1));
        assert_eq!(target.lines[0], "12345aaaaa");
    }

    #[test]
    fn blit_into_rect_pads_short_target() {
        let mut target = Rendered {
            lines: vec!["hi".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, Rect::new(5, 0, 10, 1));
        assert_eq!(target.lines[0], "hi   XY");
    }

    /// Regression: blit_into_rect must use visible width, not byte length,
    /// so ANSI-coded lines aren't incorrectly truncated.
    #[test]
    fn blit_into_rect_preserves_ansi_reset() {
        let mut target = Rendered::empty();
        // 10 visible chars but 19 bytes (9 ANSI + 10 text + reset)
        let source = Rendered {
            lines: vec!["\x1b[44mhello     \x1b[0m".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, Rect::new(0, 0, 10, 1));
        // Must NOT truncate the \x1b[0m reset
        assert!(
            target.lines[0].contains("\x1b[0m"),
            "reset code should survive blit"
        );
        // Visible width should be exactly 10
        assert_eq!(crate::utils::visible_width(&target.lines[0]), 10);
    }

    /// Regression: blit_into_rect must not panic when target contains ANSI
    /// codes.
    #[test]
    fn blit_into_rect_ansi_target() {
        let mut target = Rendered {
            lines: vec!["\x1b[31mred text here\x1b[0m".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into()],
            cursor: None,
            images: Vec::new(),
        };
        // Blit at visual position 4 — byte index would be inside the ANSI prefix
        source.blit_into_rect(&mut target, Rect::new(4, 0, 10, 1));
        assert!(target.lines[0].contains("XY"));
        assert_eq!(crate::utils::visible_width(&target.lines[0]), 13);
    }

    /// Regression: blit_into_rect must preserve ANSI reset codes at the start
    /// boundary so background colours don't bleed into adjacent components.
    #[test]
    fn blit_into_rect_preserves_ansi_reset_at_boundary() {
        let mut target = Rendered::empty();
        // Blue background spanning visual columns 0–7
        let blue_box = Rendered {
            lines: vec!["\x1b[44m        \x1b[0m".into()],
            cursor: None,
            images: Vec::new(),
        };
        blue_box.blit_into_rect(&mut target, Rect::new(0, 0, 8, 1));

        // Plain text blitted immediately after the blue box (column 8)
        let text = Rendered {
            lines: vec!["hello".into()],
            cursor: None,
            images: Vec::new(),
        };
        text.blit_into_rect(&mut target, Rect::new(8, 0, 5, 1));

        // The reset code must survive so "hello" doesn't pick up the blue bg
        assert!(
            target.lines[0].contains("\x1b[0mhello"),
            "reset should be preserved before hello: {}",
            target.lines[0]
        );
        assert_eq!(crate::utils::visible_width(&target.lines[0]), 13);
    }

    /// Regression: blit_onto must not panic when target contains ANSI codes.
    #[test]
    fn blit_onto_ansi_target() {
        let mut target = Rendered {
            lines: vec!["\x1b[31mred text\x1b[0m".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into()],
            cursor: None,
            images: Vec::new(),
        };
        // Overlay at visual column 4 — byte index is inside ANSI prefix
        source.blit_onto(&mut target, 0, 4);
        assert!(target.lines[0].contains("XY"));
        assert_eq!(crate::utils::visible_width(&target.lines[0]), 8);
    }

    /// Regression: diff mode must reset ANSI attributes before clearing lines.
    #[test]
    fn diff_resets_ansi_before_clear() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();

        let frame1 = Rendered {
            lines: vec!["\x1b[41mred bg\x1b[0m".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame1).unwrap();

        renderer.set_strategy(RenderStrategy::Diff);
        let frame2 = Rendered {
            lines: vec!["plain".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &frame2).unwrap();

        let written = term.written().join("");
        // Every \x1b[2K must be preceded by \x1b[0m
        for chunk in written.split("\x1b[2K") {
            if !chunk.is_empty() && chunk.contains("\x1b[") {
                assert!(
                    chunk.ends_with("\x1b[0m") || !chunk.contains("\x1b[2K"),
                    "clear must be preceded by reset: {}",
                    chunk
                );
            }
        }
    }

    /// Regression: FirstRender must reset ANSI attributes before clearing.
    #[test]
    fn first_render_resets_before_clear() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();
        let rendered = Rendered {
            lines: vec!["hello".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &rendered).unwrap();
        let written = term.written().join("");
        assert!(
            written.contains("\x1b[0m\x1b[2J"),
            "reset must precede screen clear"
        );
    }

    /// Regression: FullRedraw must reset ANSI attributes before clearing.
    #[test]
    fn full_redraw_resets_before_clear() {
        let mut term = TestTerminal::new(80, 24);
        let mut renderer = Renderer::new();
        renderer.set_strategy(RenderStrategy::FullRedraw);
        let rendered = Rendered {
            lines: vec!["hello".into()],
            cursor: None,
            images: Vec::new(),
        };
        renderer.render(&mut term, &rendered).unwrap();
        let written = term.written().join("");
        assert!(
            written.contains("\x1b[0m\x1b[2J"),
            "reset must precede screen clear"
        );
    }
}
