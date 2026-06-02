use photon_ui::autocomplete::{AutocompleteProvider, SimpleAutocomplete, CombinedAutocompleteProvider};

#[test]
fn simple_autocomplete_suggests() {
    let provider = SimpleAutocomplete::new(vec!["hello".into(), "help".into(), "world".into()]);
    let suggestions = provider.suggest("hel");
    assert_eq!(suggestions.len(), 2);
    assert!(suggestions.contains(&"hello".to_string()));
    assert!(suggestions.contains(&"help".to_string()));
}

#[test]
fn combined_slash_commands() {
    let provider = CombinedAutocompleteProvider::new(
        vec!["open".into(), "close".into()],
        "/tmp"
    );
    let suggestions = provider.suggest("/op");
    assert!(suggestions.contains(&"/open".to_string()));
}
