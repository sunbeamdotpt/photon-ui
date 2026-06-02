use photon_ui::fuzzy::{fuzzy_match, fuzzy_filter};

#[test]
fn fuzzy_match_scores() {
    let score = fuzzy_match("abc", "alphabetic");
    assert!(score > 0);
}

#[test]
fn fuzzy_match_no_match() {
    let score = fuzzy_match("xyz", "alphabet");
    assert_eq!(score, 0);
}

#[test]
fn fuzzy_filter_basic() {
    let items = vec!["hello", "world", "help"];
    let filtered = fuzzy_filter(&items, "hel", |s| *s);
    assert!(filtered.contains(&&"hello"));
    assert!(filtered.contains(&&"help"));
    assert!(!filtered.contains(&&"world"));
}
