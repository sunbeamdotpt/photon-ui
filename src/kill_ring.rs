/// A kill-ring (clipboard history) for Emacs-style yank operations.
///
/// Stores deleted text in a ring buffer. [`yank`](KillRing::yank) returns the
/// most recent entry, and [`yank_pop`](KillRing::yank_pop) cycles backwards
/// through older entries.
#[derive(Debug, Clone, Default)]
pub struct KillRing {
    entries: Vec<String>,
    index: usize,
}

impl KillRing {
    /// Create an empty kill ring.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add text to the kill ring.
    ///
    /// Empty strings are ignored. The internal index is reset to the newest
    /// entry.
    pub fn push(&mut self, text: String) {
        if !text.is_empty() {
            self.entries.push(text);
            self.index = self.entries.len().saturating_sub(1);
        }
    }

    /// Return the most recently killed text.
    pub fn yank(&self) -> Option<&str> {
        self.entries.last().map(|s| s.as_str())
    }

    /// Cycle backwards through the kill ring and return the text at the new
    /// position.
    pub fn yank_pop(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        self.index = self.index.checked_sub(1).unwrap_or(self.entries.len() - 1);
        Some(&self.entries[self.index])
    }
}
