use std::sync::atomic::{
    AtomicU32,
    Ordering,
};

use crate::{
    Component,
    RenderError,
    Rendered,
    image::ImageProtocol,
    layout::Rect,
    renderer::ImageCommand,
};

static NEXT_IMAGE_ID: AtomicU32 = AtomicU32::new(1);

/// Default assumed terminal cell size in pixels.
///
/// These values are only used to convert the image's pixel dimensions into a
/// reasonable default cell size. The actual terminal cell size may differ, but
/// Kitty scales the image to fit the requested `c=`/`r=` cell rectangle.
const DEFAULT_CELL_WIDTH_PX: u32 = 10;
const DEFAULT_CELL_HEIGHT_PX: u32 = 20;

/// Maximum default cell width for an image. Images wider than this are scaled
/// down so they do not dominate the terminal screen.
const DEFAULT_MAX_COLS: u16 = 40;

/// Maximum default cell height for an image.
const DEFAULT_MAX_ROWS: u16 = 20;

/// A widget that displays an inline terminal image.
///
/// The component renders placeholder text lines that reserve the same number
/// of cells the terminal will use for the image, and emits an
/// [`ImageCommand`](crate::renderer::ImageCommand) so the renderer can upload
/// the image using the selected terminal graphics protocol (Kitty or iTerm2).
pub struct ImageWidget {
    id: u32,
    data: Vec<u8>,
    mime_type: String,
    placeholder: String,
    protocol: ImageProtocol,
    cols: u16,
    rows: u16,
}

impl ImageWidget {
    /// Create a new image widget.
    ///
    /// `data` is the raw image bytes. `placeholder` defaults to `"[image]"`.
    /// The widget is assigned a unique image id and uses the Kitty protocol by
    /// default. The display size in cells is inferred from the image's pixel
    /// dimensions when possible, falling back to `20×10` cells.
    pub fn new(data: Vec<u8>, mime_type: impl Into<String>, placeholder: Option<String>) -> Self {
        let mime_type = mime_type.into();
        let (cols, rows) = compute_default_size(&data, &mime_type);
        Self {
            id: NEXT_IMAGE_ID.fetch_add(1, Ordering::Relaxed),
            data,
            mime_type,
            placeholder: placeholder.unwrap_or_else(|| "[image]".to_string()),
            protocol: ImageProtocol::default(),
            cols,
            rows,
        }
    }

    /// Override the terminal image protocol used by this widget.
    pub fn with_protocol(mut self, protocol: ImageProtocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Set the on-screen size in terminal cells.
    ///
    /// This determines how many placeholder lines the widget renders and the
    /// `c=`/`r=` values passed to the Kitty graphics protocol. The terminal
    /// scales the image to fit this cell rectangle.
    pub fn with_size(mut self, cols: u16, rows: u16) -> Self {
        self.cols = cols.max(1);
        self.rows = rows.max(1);
        self
    }

    /// Build a placeholder line that fills the requested visual width.
    fn placeholder_line(&self, width: u16) -> String {
        let target = width as usize;
        let placeholder_vw = crate::utils::visible_width(&self.placeholder);
        if placeholder_vw >= target {
            return crate::utils::truncate_to_width(&self.placeholder, width, "");
        }
        let pad = target - placeholder_vw;
        let mut line = self.placeholder.clone();
        line.push_str(&" ".repeat(pad));
        line
    }
}

impl Component for ImageWidget {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let display_cols = self.cols.min(width);
        let encoded = match self.protocol {
            | ImageProtocol::Kitty => {
                crate::image::encode_kitty(self.id, &self.data, display_cols, self.rows)
            },
            | ImageProtocol::Iterm2 => crate::image::encode_iterm2(&self.data, &self.mime_type),
        };
        let images = if encoded.is_empty() {
            Vec::new()
        } else {
            vec![ImageCommand {
                id: self.id,
                data: encoded,
                row: 0,
                col: 0,
            }]
        };

        let mut lines = Vec::new();
        if self.rows > 0 {
            lines.push(self.placeholder_line(display_cols));
        }
        for _ in 1..self.rows {
            lines.push(" ".repeat(display_cols as usize));
        }

        Ok(Rendered {
            lines,
            cursor: None,
            images,
        })
    }

    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError> {
        let mut rendered = match self.render(rect.width) {
            | Ok(r) => r,
            | Err(e) => return Err(e),
        };
        // Clip placeholder lines to the allocated rect so the widget composes
        // cleanly inside Cassowary layouts.
        rendered.lines.truncate(rect.height as usize);
        Ok(rendered)
    }
}

/// Compute a default cell size from the image's pixel dimensions.
///
/// The image is scaled to fit within [`DEFAULT_MAX_COLS`] while preserving its
/// aspect ratio under the assumption of a `10×20` pixel terminal cell. Returns
/// a fallback `20×10` size when dimensions cannot be parsed.
fn compute_default_size(data: &[u8], mime_type: &str) -> (u16, u16) {
    let dims = match mime_type {
        | "image/png" => crate::image::get_png_dimensions(data),
        | "image/jpeg" | "image/jpg" => crate::image::get_jpeg_dimensions(data),
        | "image/gif" => crate::image::get_gif_dimensions(data),
        | "image/webp" => crate::image::get_webp_dimensions(data),
        | _ => None,
    };
    let (pixel_w, pixel_h) = match dims {
        | Some(d) => d,
        | None => return (20, 10),
    };
    if pixel_w == 0 || pixel_h == 0 {
        return (20, 10);
    }

    let max_cols = DEFAULT_MAX_COLS as u32;
    let max_rows = DEFAULT_MAX_ROWS as u32;
    let cols = (pixel_w / DEFAULT_CELL_WIDTH_PX).clamp(1, max_cols);
    let rows = ((pixel_h * cols * DEFAULT_CELL_WIDTH_PX) / (pixel_w * DEFAULT_CELL_HEIGHT_PX))
        .clamp(1, max_rows) as u16;
    let cols = cols as u16;
    (cols, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_widget_default_size_from_png_dimensions() {
        // PNG signature + IHDR chunk with 100×200 pixels.
        let mut data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&100u32.to_be_bytes());
        data.extend_from_slice(&200u32.to_be_bytes());
        let widget = ImageWidget::new(data, "image/png", None);
        // 100px / 10px per cell = 10 cols; aspect-preserving height = 10 rows.
        assert_eq!(widget.cols, 10);
        assert_eq!(widget.rows, 10);
    }

    #[test]
    fn image_widget_with_size_override() {
        let widget = ImageWidget::new(vec![], "image/png", None).with_size(15, 8);
        assert_eq!(widget.cols, 15);
        assert_eq!(widget.rows, 8);
    }

    #[test]
    fn image_widget_render_reserves_multiple_lines() {
        let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_size(10, 3);
        let rendered = widget.render(80).unwrap();
        assert_eq!(rendered.lines.len(), 3);
        assert!(rendered.lines[0].starts_with("[image]"));
        assert_eq!(crate::utils::visible_width(&rendered.lines[1]), 10);
    }

    #[test]
    fn image_widget_render_rect_clips_to_height() {
        let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_size(5, 5);
        let rect = Rect::new(0, 0, 80, 2);
        let rendered = widget.render_rect(rect).unwrap();
        assert_eq!(rendered.lines.len(), 2);
    }

    #[test]
    fn image_widget_render_includes_kitty_dimensions() {
        let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_size(12, 6);
        let rendered = widget.render(80).unwrap();
        assert_eq!(rendered.images.len(), 1);
        let data = &rendered.images[0].data;
        assert!(data.contains("c=12"));
        assert!(data.contains("r=6"));
    }
}
