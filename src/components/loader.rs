use crate::{
    Component,
    RenderError,
    Rendered,
};

/// Braille spinner frames used by [`Loader`].
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// An animated loading spinner with a message.
///
/// Call [`tick`](Loader::tick) to advance the spinner frame. The component
/// does not auto-animate; the application must drive it via a timer loop.
#[derive(Clone)]
pub struct Loader {
    message: String,
    frame: usize,
    spinner_color: Option<String>,
    message_color: Option<String>,
}

impl Loader {
    /// Create a new loader with the given message and optional color codes.
    pub fn new(
        message: impl Into<String>,
        spinner_color: Option<String>,
        message_color: Option<String>,
    ) -> Self {
        Self {
            message: message.into(),
            frame: 0,
            spinner_color,
            message_color,
        }
    }

    /// Advance to the next spinner frame.
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
    }
}

impl Component for Loader {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let spinner = SPINNER_FRAMES[self.frame];
        let spinner_styled = self
            .spinner_color
            .as_ref()
            .map(|c| format!("{}{}\x1b[0m", c, spinner))
            .unwrap_or_else(|| spinner.to_string());
        let message_styled = self
            .message_color
            .as_ref()
            .map(|c| format!("{}{}\x1b[0m", c, self.message))
            .unwrap_or_else(|| self.message.clone());
        let line = format!("{} {}", spinner_styled, message_styled);
        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}
