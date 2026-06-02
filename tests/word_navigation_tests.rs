use photon_ui::word_navigation::{find_word_forward, find_word_backward};

#[test]
fn find_word_forward_basic() {
    assert_eq!(find_word_forward("hello world", 0, |c| c.is_whitespace()), 5);
    assert_eq!(find_word_forward("hello world", 6, |c| c.is_whitespace()), 11);
}

#[test]
fn find_word_backward_basic() {
    assert_eq!(find_word_backward("hello world", 11, |c| c.is_whitespace()), 6);
    assert_eq!(find_word_backward("hello world", 5, |c| c.is_whitespace()), 0);
}
