/// A generic undo/redo stack.
///
/// Stores snapshots of state. Calling [`undo`](UndoStack::undo) moves backward
/// through history; [`redo`](UndoStack::redo) moves forward. Pushing a new
/// item after undoing discards the redo branch.
///
/// # Type parameters
///
/// `T` must implement `Clone` because the stack stores owned snapshots.
#[derive(Debug, Clone)]
pub struct UndoStack<T: Clone> {
    history: Vec<T>,
    index: usize,
}

impl<T: Clone> UndoStack<T> {
    /// Create an empty undo stack.
    pub fn new() -> Self {
        Self { history: Vec::new(), index: 0 }
    }

    /// Push a new state snapshot onto the stack.
    ///
    /// Any redo history after the current position is discarded.
    pub fn push(&mut self, item: T) {
        self.history.truncate(self.index + 1);
        self.history.push(item);
        self.index = self.history.len().saturating_sub(1);
    }

    /// Move one step backward in history and return the state at the new
    /// position.
    ///
    /// If already at the earliest state, returns that state again.
    pub fn undo(&mut self) -> Option<&T> {
        if self.index == 0 {
            self.history.first()
        } else {
            self.index -= 1;
            self.history.get(self.index)
        }
    }

    /// Move one step forward in history and return the state at the new
    /// position.
    ///
    /// Returns `None` if already at the most recent state.
    pub fn redo(&mut self) -> Option<&T> {
        if self.index + 1 < self.history.len() {
            self.index += 1;
            self.history.get(self.index)
        } else {
            None
        }
    }

    /// Return the state at the current position without changing it.
    pub fn current(&self) -> Option<&T> {
        self.history.get(self.index)
    }
}

impl<T: Clone> Default for UndoStack<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_returns_none_when_empty() {
        let stack: UndoStack<i32> = UndoStack::new();
        assert!(stack.current().is_none());
    }

    #[test]
    fn redo_at_end_returns_none() {
        let mut stack = UndoStack::new();
        stack.push(1);
        assert!(stack.redo().is_none());
    }

    #[test]
    fn undo_redo_sequence() {
        let mut stack = UndoStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.current(), Some(&3));
        assert_eq!(stack.undo(), Some(&2));
        assert_eq!(stack.undo(), Some(&1));
        assert_eq!(stack.undo(), Some(&1));
        assert_eq!(stack.redo(), Some(&2));
        assert_eq!(stack.redo(), Some(&3));
        assert_eq!(stack.redo(), None);
    }

    #[test]
    fn push_after_undo_overwrites() {
        let mut stack = UndoStack::new();
        stack.push(1);
        stack.push(2);
        stack.undo();
        stack.push(3);
        assert_eq!(stack.current(), Some(&3));
        assert_eq!(stack.redo(), None);
    }
}
