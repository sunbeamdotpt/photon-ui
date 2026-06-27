use unicode_width::UnicodeWidthChar;

/// Compute the visible display width of a string.
///
/// ANSI escape sequences (CSI `\x1b[…` and OSC `\x1b]…`) do not contribute to
/// the width. Full-width characters (e.g. CJK) count as 2 columns.
pub fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                | Some(&'[') => {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c.is_alphabetic() {
                            break;
                        }
                    }
                    continue;
                },
                | Some(&']') => {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' &&
                            let Some(&'\\') = chars.peek()
                        {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                },
                | _ => {},
            }
        }
        width += ch.width().unwrap_or(0);
    }
    width
}

/// Return the byte index in `s` that corresponds to visual position
/// `target_pos`.
///
/// ANSI escape sequences are skipped (they contribute 0 width). If `target_pos`
/// is beyond the visible width of `s`, the byte index after the last visible
/// character is returned.
pub fn byte_index_at_visual_pos(s: &str, target_pos: usize) -> usize {
    let mut width = 0;
    let mut byte_idx = 0;
    let mut chars = s.chars().peekable();

    while let Some(&ch) = chars.peek() {
        let ch_len = ch.len_utf8();
        if ch == '\x1b' {
            chars.next();
            byte_idx += ch_len;
            match chars.peek() {
                | Some(&'[') => {
                    chars.next();
                    byte_idx += '['.len_utf8();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        byte_idx += c.len_utf8();
                        if c.is_alphabetic() {
                            break;
                        }
                    }
                },
                | Some(&']') => {
                    chars.next();
                    byte_idx += ']'.len_utf8();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        byte_idx += c.len_utf8();
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' &&
                            let Some(&'\\') = chars.peek()
                        {
                            chars.next();
                            byte_idx += '\\'.len_utf8();
                            break;
                        }
                    }
                },
                | _ => {},
            }
            continue;
        }
        if width >= target_pos {
            return byte_idx;
        }
        chars.next();
        width += ch.width().unwrap_or(0);
        byte_idx += ch_len;
        if width >= target_pos {
            return byte_idx;
        }
    }
    byte_idx
}

/// Return the ANSI reset sequence, with an explicit intensity reset prefix
/// when bold or faint was active.
///
/// Some macOS terminals (Ghostty, Terminal.app) do not reliably drop the bold
/// attribute on `\x1b[0m` alone. Emitting `\x1b[22m` first turns off bold and
/// faint explicitly before the full reset.
fn sgr_reset(bold_or_faint: bool) -> &'static str {
    if bold_or_faint {
        "\x1b[22m\x1b[0m"
    } else {
        "\x1b[0m"
    }
}

/// Truncate a string so its visible width does not exceed `max_width`.
///
/// If truncation is necessary, `ellipsis` is appended at the end. The result
/// always satisfies `visible_width(result) <= max_width`.
///
/// # Example
///
/// ```
/// use photon_ui::utils::truncate_to_width;
///
/// assert_eq!(truncate_to_width("hello world", 8, "…"), "hello w…");
/// assert_eq!(truncate_to_width("hello", 10, "…"), "hello");
/// ```
pub fn truncate_to_width(s: &str, max_width: u16, ellipsis: &str) -> String {
    let max = max_width as usize;
    let ellip_width = visible_width(ellipsis);
    let total = visible_width(s);
    if total <= max {
        return s.to_string();
    }
    let target = max.saturating_sub(ellip_width);
    let mut result = String::new();
    let mut w = 0;
    let mut chars = s.chars().peekable();
    let mut tracker = AnsiCodeTracker::new();
    while let Some(ch) = chars.next() {
        // Skip ANSI escape sequences (CSI and OSC) — they contribute 0 width.
        if ch == '\x1b' {
            match chars.peek() {
                | Some(&'[') => {
                    result.push(ch);
                    chars.next(); // consume '['
                    result.push('[');
                    let mut seq = String::from("\x1b[");
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        result.push(c);
                        seq.push(c);
                        if c.is_alphabetic() {
                            break;
                        }
                    }
                    tracker.process(&seq);
                    continue;
                },
                | Some(&']') => {
                    result.push(ch);
                    chars.next(); // consume ']'
                    result.push(']');
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        result.push(c);
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' &&
                            let Some(&'\\') = chars.peek()
                        {
                            chars.next();
                            result.push('\\');
                            break;
                        }
                    }
                    continue;
                },
                | _ => {},
            }
        }
        let cw = ch.width().unwrap_or(0);
        if w + cw > target {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result.push_str(ellipsis);
    // If the original string contained ANSI codes, append a reset so that
    // truncated strings don't leave active attributes (e.g. background colours)
    // dangling. When bold or faint is active, emit an explicit intensity reset
    // first to work around terminals that don't clear bold on `\x1b[0m` alone.
    if s.contains('\x1b') {
        result.push_str(sgr_reset(tracker.bold || tracker.faint));
    }
    result
}

/// An active OSC 8 hyperlink tracked by [`AnsiCodeTracker`].
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveHyperlink {
    /// Hyperlink parameters (e.g. `id` or empty string).
    pub params: String,
    /// The target URL.
    pub url: String,
    /// The original terminator sequence (`\x1b\\` or `\x07`).
    pub terminator: String,
}

/// Tracks active ANSI SGR and OSC 8 state across line breaks.
///
/// When wrapping styled text, styles must be closed at the end of each
/// physical line and reopened at the start of the next. This struct records
/// which attributes are currently active and can emit the corresponding
/// escape sequences.
///
/// # Example
///
/// ```
/// use photon_ui::utils::AnsiCodeTracker;
///
/// let mut tracker = AnsiCodeTracker::new();
/// tracker.process("\x1b[1m"); // bold on
/// tracker.process("\x1b[31m"); // red fg
/// assert_eq!(tracker.current_codes(), "\x1b[1;31m");
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AnsiCodeTracker {
    /// Bold (SGR 1) is active.
    pub bold: bool,
    /// Italic (SGR 3) is active.
    pub italic: bool,
    /// Underline (SGR 4) is active.
    pub underline: bool,
    /// Faint / dim (SGR 2) is active.
    pub faint: bool,
    /// Reverse video (SGR 7) is active.
    pub reverse: bool,
    /// Active foreground color SGR parameter, e.g. `"31"` or `"38;5;240"`.
    pub fg_color: Option<String>,
    /// Active background color SGR parameter, e.g. `"41"` or `"48;5;240"`.
    pub bg_color: Option<String>,
    /// Active OSC 8 hyperlink, if any.
    pub hyperlink: Option<ActiveHyperlink>,
}

impl AnsiCodeTracker {
    /// Create a tracker with no active codes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an OSC 8 hyperlink sequence.
    ///
    /// Returns `Some(Some(link))` on open, `Some(None)` on close, and
    /// `None` if the sequence is not a valid OSC 8 hyperlink.
    fn parse_osc8(seq: &str) -> Option<Option<ActiveHyperlink>> {
        let body = match seq.strip_prefix("\x1b]") {
            | Some(b) => b,
            | None => return None,
        };
        let (body, terminator) = if let Some(body) = body.strip_suffix("\x1b\\") {
            (body, "\x1b\\".to_string())
        } else if let Some(body) = body.strip_suffix('\x07') {
            (body, "\x07".to_string())
        } else {
            return None;
        };
        let rest = match body.strip_prefix("8;") {
            | Some(r) => r,
            | None => return None,
        };
        let sep = match rest.find(';') {
            | Some(s) => s,
            | None => return None,
        };
        let params = rest[..sep].to_string();
        let url = rest[sep + 1..].to_string();
        if url.is_empty() {
            Some(None)
        } else {
            Some(Some(ActiveHyperlink {
                params,
                url,
                terminator,
            }))
        }
    }

    /// Process an ANSI escape sequence, updating internal state.
    ///
    /// Supports:
    /// - OSC 8 hyperlink open / close (`\x1b]8;;URL\x1b\\`, `\x1b]8;;\x1b\\`)
    /// - SGR codes (`\x1b[…m`) for bold, italic, underline, and colors,
    ///   including 256-color (`38;5;N` / `48;5;N`) and 24-bit truecolor
    ///   (`38;2;R;G;B` / `48;2;R;G;B`) forms.
    pub fn process(&mut self, seq: &str) {
        if let Some(parsed) = Self::parse_osc8(seq) {
            self.hyperlink = parsed;
            return;
        }

        let body = seq.strip_prefix("\x1b[").unwrap_or(seq);
        let body = body.strip_suffix('m').unwrap_or(body);
        let codes: Vec<&str> = body.split(';').collect();
        let mut i = 0;
        while i < codes.len() {
            let code = codes[i];
            match code {
                | "0" => *self = Self::default(),
                | "1" => self.bold = true,
                | "2" => self.faint = true,
                | "3" => self.italic = true,
                | "4" => self.underline = true,
                | "7" => self.reverse = true,
                | "22" => {
                    self.bold = false;
                    self.faint = false;
                },
                | "23" => self.italic = false,
                | "24" => self.underline = false,
                | "27" => self.reverse = false,
                | "39" => self.fg_color = None,
                | "49" => self.bg_color = None,
                | "38" => {
                    self.fg_color = Self::parse_extended_color(&codes, &mut i, "38");
                    continue;
                },
                | "48" => {
                    self.bg_color = Self::parse_extended_color(&codes, &mut i, "48");
                    continue;
                },
                | c if c.starts_with('3') && c.len() >= 2 => self.fg_color = Some(c.to_string()),
                | c if c.starts_with('4') && c.len() >= 2 => self.bg_color = Some(c.to_string()),
                | _ => {},
            }
            i += 1;
        }
    }

    /// Parse an extended color specification that follows `38` or `48`.
    ///
    /// The `prefix` is `"38"` for foreground or `"48"` for background. The
    /// returned string includes the prefix so it can be emitted directly as an
    /// SGR parameter (e.g. `"38;2;250;82;15"`). `i` is advanced past the
    /// consumed codes; incomplete specifications return `None`.
    fn parse_extended_color(codes: &[&str], i: &mut usize, prefix: &str) -> Option<String> {
        *i += 1;
        if *i >= codes.len() {
            return None;
        }
        match codes[*i] {
            | "5" => {
                *i += 1;
                if *i >= codes.len() {
                    return None;
                }
                let idx = codes[*i];
                *i += 1;
                Some(format!("{};5;{}", prefix, idx))
            },
            | "2" => {
                *i += 1;
                if *i + 2 >= codes.len() {
                    return None;
                }
                let r = codes[*i];
                let g = codes[*i + 1];
                let b = codes[*i + 2];
                *i += 3;
                Some(format!("{};2;{};{};{}", prefix, r, g, b))
            },
            | _ => None,
        }
    }

    /// Return the escape sequences needed to restore all active codes.
    ///
    /// This is used to reopen styles at the beginning of a continuation line.
    pub fn current_codes(&self) -> String {
        let mut parts = Vec::new();
        if self.bold {
            parts.push("1");
        }
        if self.faint {
            parts.push("2");
        }
        if self.italic {
            parts.push("3");
        }
        if self.underline {
            parts.push("4");
        }
        if self.reverse {
            parts.push("7");
        }
        if let Some(ref fg) = self.fg_color {
            parts.push(fg.as_str());
        }
        if let Some(ref bg) = self.bg_color {
            parts.push(bg.as_str());
        }
        let mut result = if parts.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", parts.join(";"))
        };
        if let Some(ref link) = self.hyperlink {
            result.push_str(&format!(
                "\x1b]8;{};{}{}",
                link.params, link.url, link.terminator
            ));
        }
        result
    }

    /// Return the escape sequences needed to close active codes at a line end.
    ///
    /// Unlike a full SGR reset, this only closes attributes that would bleed
    /// into padding or subsequent lines (underline and hyperlinks). The caller
    /// is responsible for emitting `\x1b[0m` when a full SGR reset is needed.
    pub fn line_end_reset(&self) -> String {
        let mut result = String::new();
        if self.underline {
            result.push_str("\x1b[24m");
        }
        if self.reverse {
            result.push_str("\x1b[27m");
        }
        if let Some(ref link) = self.hyperlink {
            result.push_str(&format!("\x1b]8;;{}", link.terminator));
        }
        result
    }

    /// Returns `true` if any SGR or OSC 8 code is currently active.
    pub fn has_active_codes(&self) -> bool {
        self.bold ||
            self.faint ||
            self.italic ||
            self.underline ||
            self.reverse ||
            self.fg_color.is_some() ||
            self.bg_color.is_some() ||
            self.hyperlink.is_some()
    }
}

/// Wrap text into lines that fit within `width` columns, preserving ANSI codes.
///
/// ANSI SGR sequences (`\x1b[…m`) and OSC 8 hyperlink sequences (`\x1b]8;…`)
/// are parsed and carried across line boundaries so that styles remain
/// continuous. Newlines in the input produce new lines in the output.
///
/// # Example
///
/// ```
/// use photon_ui::utils::wrap_text_with_ansi;
///
/// let lines = wrap_text_with_ansi("hello world", 6);
/// assert_eq!(lines, vec!["hello ", "world"]);
/// ```
pub fn wrap_text_with_ansi(text: &str, width: u16) -> Vec<String> {
    let w = width as usize;
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    let mut tracker = AnsiCodeTracker::new();

    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                | Some(&'[') => {
                    chars.next();
                    let mut seq = String::from("\x1b[");
                    while let Some(&c) = chars.peek() {
                        seq.push(c);
                        chars.next();
                        if c.is_alphabetic() {
                            break;
                        }
                    }
                    tracker.process(&seq);
                    current.push_str(&seq);
                    continue;
                },
                | Some(&']') => {
                    chars.next();
                    let mut seq = String::from("\x1b]");
                    while let Some(&c) = chars.peek() {
                        seq.push(c);
                        chars.next();
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' &&
                            let Some(&'\\') = chars.peek()
                        {
                            seq.push('\\');
                            chars.next();
                            break;
                        }
                    }
                    tracker.process(&seq);
                    current.push_str(&seq);
                    continue;
                },
                | _ => {},
            }
        }

        if ch == '\n' {
            if tracker.bold ||
                tracker.faint ||
                tracker.italic ||
                tracker.underline ||
                tracker.fg_color.is_some() ||
                tracker.bg_color.is_some()
            {
                current.push_str(sgr_reset(tracker.bold || tracker.faint));
            }
            let reset = tracker.line_end_reset();
            if !reset.is_empty() {
                current.push_str(&reset);
            }
            lines.push(current);
            current = tracker.current_codes();
            current_width = 0;
            continue;
        }

        let cw = ch.width().unwrap_or(0);
        if current_width + cw > w && !current.is_empty() {
            if tracker.bold ||
                tracker.faint ||
                tracker.italic ||
                tracker.underline ||
                tracker.fg_color.is_some() ||
                tracker.bg_color.is_some()
            {
                current.push_str(sgr_reset(tracker.bold || tracker.faint));
            }
            let reset = tracker.line_end_reset();
            if !reset.is_empty() {
                current.push_str(&reset);
            }
            lines.push(current);
            current = tracker.current_codes();
            current_width = 0;
        }
        current.push(ch);
        current_width += cw;
    }

    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_tracks_hyperlink() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]8;;https://example.com\x1b\\");
        assert!(tracker.hyperlink.is_some());
        assert_eq!(
            tracker.hyperlink.as_ref().unwrap().url,
            "https://example.com"
        );
        assert_eq!(tracker.hyperlink.as_ref().unwrap().terminator, "\x1b\\");
    }

    #[test]
    fn tracker_hyperlink_bel_terminator() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]8;;https://example.com\x07");
        assert!(tracker.hyperlink.is_some());
        assert_eq!(tracker.hyperlink.as_ref().unwrap().terminator, "\x07");
    }

    #[test]
    fn tracker_hyperlink_close() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]8;;https://example.com\x1b\\");
        assert!(tracker.hyperlink.is_some());
        tracker.process("\x1b]8;;\x1b\\");
        assert!(tracker.hyperlink.is_none());
    }

    #[test]
    fn current_codes_includes_hyperlink() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]8;;https://example.com\x1b\\");
        let codes = tracker.current_codes();
        assert!(codes.contains("\x1b]8;;https://example.com\x1b\\"));
    }

    #[test]
    fn line_end_reset_closes_hyperlink() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]8;;https://example.com\x1b\\");
        let reset = tracker.line_end_reset();
        assert!(reset.contains("\x1b]8;;\x1b\\"));
    }

    #[test]
    fn wrap_preserves_hyperlink_across_lines() {
        let text = "\x1b]8;;https://example.com\x1b\\hello world\x1b]8;;\x1b\\";
        let lines = wrap_text_with_ansi(text, 6);
        assert_eq!(lines.len(), 2);
        // First line should close hyperlink at end
        assert!(lines[0].contains("\x1b]8;;\x1b\\"));
        // Second line should reopen hyperlink
        assert!(lines[1].contains("\x1b]8;;https://example.com\x1b\\"));
    }

    #[test]
    fn has_active_codes_with_hyperlink() {
        let mut tracker = AnsiCodeTracker::new();
        assert!(!tracker.has_active_codes());
        tracker.process("\x1b]8;;https://example.com\x1b\\");
        assert!(tracker.has_active_codes());
    }

    #[test]
    fn line_end_reset_with_underline() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[4m");
        let reset = tracker.line_end_reset();
        assert!(reset.contains("\x1b[24m"));
    }

    #[test]
    fn wrap_hyperlink_bel_terminator() {
        let text = "\x1b]8;;https://example.com\x07hello world\x1b]8;;\x07";
        let lines = wrap_text_with_ansi(text, 6);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\x1b]8;;\x07"));
        assert!(lines[1].contains("\x1b]8;;https://example.com\x07"));
    }

    #[test]
    fn wrap_newline_with_active_sgr() {
        let text = "\x1b[31mhello\nworld\x1b[0m";
        let lines = wrap_text_with_ansi(text, 20);
        assert_eq!(lines.len(), 2);
        // First line should have SGR reset and hyperlink reset at end
        assert!(lines[0].contains("\x1b[0m"));
        // Second line should reopen the SGR code
        assert!(lines[1].starts_with("\x1b[31m"));
    }

    #[test]
    fn tracker_invalid_osc_ignored() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]8;;url");
        assert!(tracker.hyperlink.is_none());
    }

    #[test]
    fn tracker_invalid_osc_no_prefix() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b]9;;url\x1b\\");
        assert!(tracker.hyperlink.is_none());
    }

    #[test]
    fn has_active_codes_with_sgr() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[1m");
        assert!(tracker.has_active_codes());
    }

    /// Regression: 24-bit truecolor foreground must be preserved in full.
    #[test]
    fn tracker_preserves_truecolor_foreground() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[38;2;250;82;15m");
        assert_eq!(tracker.fg_color, Some("38;2;250;82;15".to_string()));
        assert_eq!(tracker.current_codes(), "\x1b[38;2;250;82;15m");
    }

    /// Regression: 24-bit truecolor background must be preserved in full.
    #[test]
    fn tracker_preserves_truecolor_background() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[48;2;42;42;42m");
        assert_eq!(tracker.bg_color, Some("48;2;42;42;42".to_string()));
        assert_eq!(tracker.current_codes(), "\x1b[48;2;42;42;42m");
    }

    /// Regression: 256-color foreground must be preserved.
    #[test]
    fn tracker_preserves_256_foreground() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[38;5;196m");
        assert_eq!(tracker.fg_color, Some("38;5;196".to_string()));
    }

    /// Regression: mixed truecolor and attribute codes must all be tracked.
    #[test]
    fn tracker_mixed_truecolor_and_attributes() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[1;38;2;250;82;15;48;2;0;0;0m");
        assert!(tracker.bold);
        assert_eq!(tracker.fg_color, Some("38;2;250;82;15".to_string()));
        assert_eq!(tracker.bg_color, Some("48;2;0;0;0".to_string()));
        assert_eq!(tracker.current_codes(), "\x1b[1;38;2;250;82;15;48;2;0;0;0m");
    }

    /// Regression: default foreground/background codes must still clear state.
    #[test]
    fn tracker_default_colors_clear_state() {
        let mut tracker = AnsiCodeTracker::new();
        tracker.process("\x1b[38;2;250;82;15;48;2;0;0;0m");
        tracker.process("\x1b[39;49m");
        assert!(tracker.fg_color.is_none());
        assert!(tracker.bg_color.is_none());
    }

    #[test]
    fn truncate_jk_text_demo() {
        let text = "  j/k = navigate list   Tab = switch focus   i = insert mode   Esc = normal mode   q = quit";
        let truncated = truncate_to_width(text, 80, "…");
        let vw = visible_width(&truncated);
        eprintln!("original vw: {}", visible_width(text));
        eprintln!("truncated: {:?}", truncated);
        eprintln!("truncated vw: {}", vw);
        assert!(vw <= 80, "truncated width {} exceeds 80", vw);
        assert!(truncated.ends_with("…"));
    }

    #[test]
    fn truncate_to_width_preserves_ansi_prefix() {
        let s = "\x1b[44mhello\x1b[0m";
        let truncated = truncate_to_width(s, 3, "…");
        // Should preserve the ANSI prefix, truncate visible text, add ellipsis,
        // and append a reset so attributes don't bleed.
        assert!(truncated.starts_with("\x1b[44m"));
        assert!(truncated.contains("…"));
        assert!(truncated.ends_with("\x1b[0m"));
        assert_eq!(visible_width(&truncated), 3);
    }

    #[test]
    fn truncate_to_width_preserves_ansi_infix() {
        let s = "hi\x1b[31mred\x1b[0mlo";
        let truncated = truncate_to_width(s, 4, "…");
        assert_eq!(visible_width(&truncated), 4);
        // The ANSI sequence should be fully preserved, not split mid-sequence.
        assert!(truncated.contains("\x1b[31m"));
        assert!(truncated.contains("\x1b[0m"));
    }

    #[test]
    fn truncate_to_width_no_truncation_when_fits() {
        let s = "\x1b[44mhi\x1b[0m";
        let truncated = truncate_to_width(s, 5, "…");
        // visible width is 2, which fits in 5, so return as-is
        assert_eq!(truncated, s);
    }

    #[test]
    fn byte_index_at_visual_pos_plain() {
        assert_eq!(byte_index_at_visual_pos("hello", 0), 0);
        assert_eq!(byte_index_at_visual_pos("hello", 3), 3);
        assert_eq!(byte_index_at_visual_pos("hello", 5), 5);
        assert_eq!(byte_index_at_visual_pos("hello", 10), 5);
    }

    #[test]
    fn byte_index_at_visual_pos_with_ansi_prefix() {
        let s = "\x1b[31mhello\x1b[0m";
        // "\x1b[31m" is 5 bytes, visible width 0
        assert_eq!(byte_index_at_visual_pos(s, 0), 5);
        assert_eq!(byte_index_at_visual_pos(s, 3), 8);
        assert_eq!(byte_index_at_visual_pos(s, 5), 10);
        // Past end → byte index after last visible char (including trailing ANSI)
        assert_eq!(byte_index_at_visual_pos(s, 10), 14);
    }

    #[test]
    fn byte_index_at_visual_pos_with_ansi_infix() {
        let s = "hi\x1b[31mred\x1b[0mlo";
        // visible: h i r e d l o = 7
        assert_eq!(byte_index_at_visual_pos(s, 0), 0);
        assert_eq!(byte_index_at_visual_pos(s, 2), 2);
        // Position 3 is 'e' which starts at byte 8 (after "hi\x1b[31mr")
        assert_eq!(byte_index_at_visual_pos(s, 3), 8);
        // Past end
        assert_eq!(byte_index_at_visual_pos(s, 7), 16);
    }

    #[test]
    fn byte_index_at_visual_pos_with_hyperlink() {
        let s = "\x1b]8;;https://example.com\x07hello";
        // OSC hyperlink is 25 bytes, visible width 0
        assert_eq!(byte_index_at_visual_pos(s, 0), 25);
        assert_eq!(byte_index_at_visual_pos(s, 3), 28);
    }
}
