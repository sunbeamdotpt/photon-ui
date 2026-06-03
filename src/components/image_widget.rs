use crate::{
    Component,
    RenderError,
    Rendered,
};

/// A placeholder component for inline terminal images.
///
/// The actual image data is passed to the renderer via
/// [`ImageCommand`](crate::renderer::ImageCommand); this widget only renders a
/// placeholder text line. Image protocol encoding (Kitty, iTerm2) is handled by
/// [`crate::image`].
pub struct ImageWidget {
    _data: Vec<u8>,
    _mime_type: String,
    placeholder: String,
}

impl ImageWidget {
    /// Create a new image widget.
    ///
    /// `data` is the raw image bytes. `placeholder` defaults to `"[image]"`.
    pub fn new(data: Vec<u8>, mime_type: impl Into<String>, placeholder: Option<String>) -> Self {
        Self {
            _data: data,
            _mime_type: mime_type.into(),
            placeholder: placeholder.unwrap_or_else(|| "[image]".to_string()),
        }
    }
}

impl Component for ImageWidget {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        Ok(Rendered {
            lines: vec![self.placeholder.clone()],
            cursor: None,
            images: Vec::new(),
        })
    }
}
