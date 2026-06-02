use photon_ui::undo_stack::UndoStack;

#[test]
fn push_undo_redo() {
    let mut stack = UndoStack::new();
    stack.push(1);
    stack.push(2);
    assert_eq!(stack.undo(), Some(&1));
    assert_eq!(stack.redo(), Some(&2));
}

#[test]
fn undo_at_start_returns_none() {
    let mut stack: UndoStack<i32> = UndoStack::new();
    assert_eq!(stack.undo(), None);
}
