/// Score a query against a text using a simple fuzzy matching algorithm.
///
/// Returns a positive score if all query characters appear in order within
/// the text, or `0` if they do not. Matches at word boundaries (after
/// non-alphanumeric characters) score higher.
///
/// The comparison is case-insensitive.
///
/// # Example
///
/// ```
/// use photon_ui::fuzzy::fuzzy_match;
///
/// assert!(fuzzy_match("hw", "hello world") > 0);
/// assert_eq!(fuzzy_match("xyz", "hello world"), 0);
/// ```
pub fn fuzzy_match(query: &str, text: &str) -> i32 {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut score = 0;
    let mut qi = 0;
    let query_chars: Vec<char> = query_lower.chars().collect();

    for (ti, tc) in text_lower.chars().enumerate() {
        if qi < query_chars.len() && tc == query_chars[qi] {
            score += 10;
            if ti == 0 ||
                !text_lower
                    .chars()
                    .nth(ti.saturating_sub(1))
                    .unwrap_or(' ')
                    .is_alphanumeric()
            {
                score += 5;
            }
            qi += 1;
        }
    }

    if qi < query_chars.len() { 0 } else { score }
}

/// Filter and rank a slice of items by fuzzy match against a query.
///
/// Only items with a positive score are returned, sorted highest-first.
///
/// # Example
///
/// ```
/// use photon_ui::fuzzy::fuzzy_filter;
///
/// let items = vec!["hello", "world", "help"];
/// let results = fuzzy_filter(&items, "he", |s| *s);
/// assert_eq!(results.len(), 2); // "hello" and "help"
/// ```
pub fn fuzzy_filter<'a, T>(items: &'a [T], query: &str, get_text: fn(&T) -> &str) -> Vec<&'a T> {
    let mut scored: Vec<(i32, &T)> = items
        .iter()
        .map(|item| {
            let text = get_text(item);
            let score = fuzzy_match(query, text);
            (score, item)
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    scored.sort_by_key(|b| std::cmp::Reverse(b.0));
    scored.into_iter().map(|(_, item)| item).collect()
}
