use crate::components::Loader;
use crate::{Component, Rendered, RenderError, Event, InputResult};
use std::sync::Arc;

/// A [`Loader`] that can be cancelled via Ctrl-C.
///
/// The cancellation signal is an [`AtomicBool`](std::sync::atomic::AtomicBool)
/// behind an [`Arc`], so it can be shared across threads or tasks. The loader
/// itself does not stop rendering when cancelled; the application should check
/// [`is_cancelled`](CancellableLoader::is_cancelled) and take action.
pub struct CancellableLoader {
    loader: Loader,
    signal: Arc<std::sync::atomic::AtomicBool>,
}

impl CancellableLoader {
    /// Create a new cancellable loader with the given message.
    pub fn new(message: impl Into<String>, spinner_color: Option<String>, message_color: Option<String>) -> Self {
        Self {
            loader: Loader::new(message, spinner_color, message_color),
            signal: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Set the cancellation flag.
    pub fn cancel(&self) {
        self.signal.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Returns `true` if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.signal.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Advance the underlying spinner frame.
    pub fn tick(&mut self) {
        self.loader.tick();
    }
}

impl Component for CancellableLoader {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        self.loader.render(width)
    }
    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crate::events::matches_key;
        if matches_key(event, &crate::events::Key::ctrl('c')) {
            self.cancel();
            InputResult::Handled
        } else {
            InputResult::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn render_delegates() {
        let cl = CancellableLoader::new("test", None, None);
        let r = cl.render(80).unwrap();
        assert!(r.lines[0].contains("test"));
    }

    #[test]
    fn ignored_key() {
        let mut cl = CancellableLoader::new("test", None, None);
        let result = cl.handle_input(&Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())));
        assert!(matches!(result, InputResult::Ignored));
        assert!(!cl.is_cancelled());
    }
}
