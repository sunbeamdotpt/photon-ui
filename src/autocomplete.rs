use crate::fuzzy::fuzzy_filter;

/// Trait for components that provide completion suggestions.
pub trait AutocompleteProvider {
    /// Return suggestions for the given query string.
    fn suggest(&self, query: &str) -> Vec<String>;
}

/// A simple autocomplete provider backed by a static list of strings.
///
/// Uses fuzzy filtering to rank results. An empty query returns all items.
pub struct SimpleAutocomplete {
    items: Vec<String>,
}

impl SimpleAutocomplete {
    /// Create a provider from a list of candidate strings.
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl AutocompleteProvider for SimpleAutocomplete {
    fn suggest(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self.items.clone();
        }
        let filtered = fuzzy_filter(&self.items, query, |s| s.as_str());
        filtered.into_iter().map(|s| s.clone()).collect()
    }
}

/// Combined autocomplete provider that handles slash-prefixed commands
/// differently from plain text.
///
/// If the query starts with `/`, the slash is stripped before matching
/// against the command list, then prepended back onto each result.
pub struct CombinedAutocompleteProvider {
    commands: Vec<String>,
    _base_path: String,
}

impl CombinedAutocompleteProvider {
    /// Create a combined provider.
    pub fn new(commands: Vec<String>, base_path: impl Into<String>) -> Self {
        Self { commands, _base_path: base_path.into() }
    }
}

impl AutocompleteProvider for CombinedAutocompleteProvider {
    fn suggest(&self, query: &str) -> Vec<String> {
        if query.starts_with('/') {
            fuzzy_filter(&self.commands, &query[1..], |s| s.as_str())
                .into_iter()
                .map(|s| format!("/{}", s))
                .collect()
        } else {
            fuzzy_filter(&self.commands, query, |s| s.as_str())
                .into_iter()
                .map(|s| s.clone())
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_autocomplete_empty_query() {
        let auto = SimpleAutocomplete::new(vec!["apple".into(), "banana".into()]);
        let suggestions = auto.suggest("");
        assert_eq!(suggestions.len(), 2);
    }

    #[test]
    fn combined_autocomplete_non_slash() {
        let auto = CombinedAutocompleteProvider::new(vec!["cmd".into()], "/base");
        let suggestions = auto.suggest("c");
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "cmd");
    }

    #[test]
    fn combined_autocomplete_slash() {
        let auto = CombinedAutocompleteProvider::new(vec!["cmd".into()], "/base");
        let suggestions = auto.suggest("/c");
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "/cmd");
    }
}
