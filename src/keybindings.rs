use std::collections::HashMap;

use thiserror::Error;

use crate::events::Key;

/// Error returned when a keybinding conflicts with an existing binding.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum KeybindingError {
    /// The key is already bound to a different action.
    #[error("conflict: {0} is already bound to {1}")]
    Conflict(String, String),
}

/// Manages a bidirectional mapping between action names and keys.
///
/// Actions are human-readable strings like `"editor.cursorLeft"`. The manager
/// ensures that each key maps to at most one action.
pub struct KeybindingsManager {
    bindings: HashMap<String, Key>,
    reverse: HashMap<Key, String>,
}

/// Return the framework's default vim-style keybindings.
///
/// | Action | Key |
/// |--------|-----|
/// | `editor.cursorLeft` | `h` (normal) / Left |
/// | `editor.cursorRight` | `l` (normal) / Right |
/// | `editor.cursorUp` | `k` (normal) / Up |
/// | `editor.cursorDown` | `j` (normal) / Down |
/// | `editor.wordForward` | `w` (normal) |
/// | `editor.wordBackward` | `b` (normal) |
/// | `editor.lineStart` | `0` (normal) / Home |
/// | `editor.lineEnd` | `$` (normal) / End |
/// | `editor.deleteChar` | `x` (normal) |
/// | `editor.undo` | `u` (normal) |
/// | `editor.redo` | `Ctrl+r` (normal) |
/// | `editor.yankLine` | `yy` (normal) |
/// | `editor.paste` | `p` (normal) |
/// | `editor.insertMode` | `i` (normal) |
/// | `editor.normalMode` | `Escape` |
/// | `input.submit` | Enter |
/// | `input.backspace` | Backspace |
/// | `input.delete` | Delete |
/// | `select.next` | `j` / Down |
/// | `select.prev` | `k` / Up |
/// | `select.confirm` | Enter |
/// | `select.cancel` | `q` / Escape |
pub fn default_bindings() -> Vec<(String, Key)> {
    vec![
        ("editor.cursorLeft".into(), Key::left()),
        ("editor.cursorRight".into(), Key::right()),
        ("editor.cursorUp".into(), Key::up()),
        ("editor.cursorDown".into(), Key::down()),
        ("editor.wordForward".into(), Key::ctrl('f')),
        ("editor.wordBackward".into(), Key::ctrl('b')),
        ("editor.lineStart".into(), Key::home()),
        ("editor.lineEnd".into(), Key::end()),
        ("editor.deleteChar".into(), Key::delete()),
        ("editor.undo".into(), Key::ctrl('-')),
        ("editor.redo".into(), Key::ctrl_shift('z')),
        ("editor.yankLine".into(), Key::ctrl('y')),
        ("editor.paste".into(), Key::ctrl('v')),
        ("editor.insertMode".into(), Key::char('i')),
        ("editor.normalMode".into(), Key::esc()),
        ("input.submit".into(), Key::enter()),
        ("input.backspace".into(), Key::backspace()),
        ("input.delete".into(), Key::delete()),
        ("select.next".into(), Key::down()),
        ("select.prev".into(), Key::up()),
        ("select.confirm".into(), Key::enter()),
        ("select.cancel".into(), Key::esc()),
    ]
}

impl Default for KeybindingsManager {
    fn default() -> Self {
        let mut mgr = Self {
            bindings: HashMap::new(),
            reverse: HashMap::new(),
        };
        for (action, key) in default_bindings() {
            mgr.bindings.insert(action.clone(), key.clone());
            mgr.reverse.insert(key, action);
        }
        mgr
    }
}

impl KeybindingsManager {
    /// Create a manager pre-populated with [`default_bindings`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up the key bound to an action.
    pub fn get(&self, action: &str) -> Option<&Key> {
        self.bindings.get(action)
    }

    /// Bind `action` to `key`, replacing any previous binding for that action.
    ///
    /// Returns an error if `key` is already bound to a *different* action.
    pub fn set(&mut self, action: &str, key: Key) -> Result<(), KeybindingError> {
        if let Some(existing_action) = self.reverse.get(&key) &&
            existing_action != action
        {
            return Err(KeybindingError::Conflict(
                format!("{:?}", key),
                existing_action.clone(),
            ));
        }
        if let Some(old_key) = self.bindings.get(action) {
            self.reverse.remove(old_key);
        }
        self.bindings.insert(action.to_string(), key.clone());
        self.reverse.insert(key, action.to_string());
        Ok(())
    }

    /// Detect any duplicate keys in the binding map.
    ///
    /// Returns pairs of `(action, other_action)` that share the same key.
    pub fn detect_conflicts(&self) -> Vec<(String, String)> {
        let mut conflicts = Vec::new();
        let mut seen: HashMap<&Key, &String> = HashMap::new();
        for (action, key) in &self.bindings {
            if let Some(other) = seen.get(key) {
                conflicts.push((action.clone(), other.to_string()));
            } else {
                seen.insert(key, action);
            }
        }
        conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
