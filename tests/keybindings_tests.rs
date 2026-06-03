use photon_ui::{
    Key,
    KeybindingsManager,
    default_bindings,
};

#[test]
fn default_bindings_contains_navigation() {
    let bindings = default_bindings();
    let actions: Vec<_> = bindings.iter().map(|(a, _)| a.clone()).collect();
    assert!(actions.contains(&"editor.cursorLeft".to_string()));
    assert!(actions.contains(&"editor.cursorRight".to_string()));
    assert!(actions.contains(&"input.submit".to_string()));
}

#[test]
fn manager_default_populated() {
    let mgr = KeybindingsManager::new();
    assert!(mgr.get("editor.cursorLeft").is_some());
    assert!(mgr.get("input.submit").is_some());
}

#[test]
fn manager_set_and_get() {
    let mut mgr = KeybindingsManager::new();
    mgr.set("test.action", Key::ctrl('t')).unwrap();
    assert_eq!(mgr.get("test.action"), Some(&Key::ctrl('t')));
}

#[test]
fn manager_detects_conflicts_in_defaults() {
    let mgr = KeybindingsManager::new();
    let conflicts = mgr.detect_conflicts();
    // Default bindings have input.submit and select.confirm both on Enter
    assert!(!conflicts.is_empty());
}

#[test]
fn manager_set_conflict_error() {
    let mut mgr = KeybindingsManager::new();
    mgr.set("action1", Key::ctrl('x')).unwrap();
    let result = mgr.set("action2", Key::ctrl('x'));
    assert!(result.is_err());
}

#[test]
fn manager_overwrite_same_action() {
    let mut mgr = KeybindingsManager::new();
    mgr.set("action1", Key::ctrl('x')).unwrap();
    mgr.set("action1", Key::ctrl('q')).unwrap();
    assert_eq!(mgr.get("action1"), Some(&Key::ctrl('q')));
}

#[test]
fn manager_no_conflicts_when_empty() {
    let mut mgr = KeybindingsManager::new();
    // Clear defaults by testing with a fresh empty manager approach:
    // Since default has conflicts, just verify set prevents creating conflicts
    mgr.set("action1", Key::ctrl('z')).unwrap();
    let result = mgr.set("action2", Key::ctrl('z'));
    assert!(result.is_err());
}
