/// Find the byte index of the start of the next word after `start`.
///
/// A word boundary is determined by `is_boundary`. The function skips any
/// boundary characters at `start`, then finds the next boundary after
/// non-boundary characters. Returns `text.len()` if no next word is found.
///
/// # Example
///
/// ```
/// use photon_ui::word_navigation::find_word_forward;
///
/// let text = "hello world";
/// assert_eq!(find_word_forward(text, 0, |c| c.is_whitespace()), 5);
/// assert_eq!(find_word_forward(text, 6, |c| c.is_whitespace()), 11);
/// ```
pub fn find_word_forward(text: &str, start: usize, is_boundary: fn(char) -> bool) -> usize {
    let mut found_non_boundary = false;
    for (i, c) in text.char_indices().skip(start) {
        if is_boundary(c) {
            if found_non_boundary {
                return i;
            }
        } else {
            found_non_boundary = true;
        }
    }
    text.len()
}

/// Find the byte index of the start of the previous word before `start`.
///
/// A word boundary is determined by `is_boundary`. The function skips any
/// boundary characters before `start`, then finds the previous boundary
/// before non-boundary characters. Returns `0` if no previous word is found.
///
/// # Example
///
/// ```
/// use photon_ui::word_navigation::find_word_backward;
///
/// let text = "hello world";
/// assert_eq!(find_word_backward(text, 11, |c| c.is_whitespace()), 6);
/// assert_eq!(find_word_backward(text, 5, |c| c.is_whitespace()), 0);
/// ```
pub fn find_word_backward(text: &str, start: usize, is_boundary: fn(char) -> bool) -> usize {
    let mut found_non_boundary = false;
    let chars: Vec<(usize, char)> = text.char_indices().take(start).collect();
    for &(i, c) in chars.iter().rev() {
        if is_boundary(c) {
            if found_non_boundary {
                return i + 1;
            }
        } else {
            found_non_boundary = true;
        }
    }
    0
}
