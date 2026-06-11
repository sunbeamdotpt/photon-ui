use unicode_width::UnicodeWidthChar;

use crate::{
    layer::Shadow,
    layout::Rect,
    renderer::{
        ImageCommand,
        Rendered,
    },
    utils::AnsiCodeTracker,
};

/// Normalized style state for a single terminal cell.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellStyle {
    /// Bold (SGR 1) is active.
    pub bold: bool,
    /// Faint / dim (SGR 2) is active.
    pub faint: bool,
    /// Italic (SGR 3) is active.
    pub italic: bool,
    /// Underline (SGR 4) is active.
    pub underline: bool,
    /// Reverse video (SGR 7) is active.
    pub reverse: bool,
    /// Active foreground color SGR parameter.
    pub fg_color: Option<String>,
    /// Active background color SGR parameter.
    pub bg_color: Option<String>,
    /// Active OSC 8 hyperlink, if any.
    pub hyperlink: Option<crate::utils::ActiveHyperlink>,
    /// Unrecognized ANSI sequences that should be emitted verbatim before this
    /// cell's symbol.
    pub prefix: String,
}

impl CellStyle {
    fn from_tracker(tracker: &AnsiCodeTracker, prefix: &str) -> Self {
        Self {
            bold: tracker.bold,
            faint: tracker.faint,
            italic: tracker.italic,
            underline: tracker.underline,
            reverse: tracker.reverse,
            fg_color: tracker.fg_color.clone(),
            bg_color: tracker.bg_color.clone(),
            hyperlink: tracker.hyperlink.clone(),
            prefix: prefix.to_string(),
        }
    }

    fn has_sgr(&self) -> bool {
        self.bold ||
            self.faint ||
            self.italic ||
            self.underline ||
            self.reverse ||
            self.fg_color.is_some() ||
            self.bg_color.is_some() ||
            !self.prefix.is_empty()
    }

    fn sgr_sequence(&self) -> String {
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
        if parts.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", parts.join(";"))
        }
    }
}

/// A single display cell in the compositor's internal grid.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cell {
    /// The visual symbol for this cell. Empty when this cell is the second half
    /// of a full-width character.
    pub symbol: String,
    /// The normalized style for this cell.
    pub style: CellStyle,
    /// Display width: `0` for a continuation spacer, `1` for normal, `2` for
    /// the first half of a full-width character.
    pub width: u8,
    /// Whether this cell is transparent padding beyond the line's real content.
    pub transparent: bool,
}

fn is_opaque(cell: &Cell) -> bool {
    cell.width > 0 && !cell.transparent
}

/// Region over which a shadow is applied to lower layers.
#[derive(Debug, Clone, PartialEq)]
enum ShadowRegion {
    /// Every cell except those covered by the layer that cast the shadow.
    Complement(Vec<Vec<bool>>),
    /// A concrete rectangle.
    Rect(Rect),
}

/// A shadow cast by one layer onto layers below it.
#[derive(Debug, Clone, PartialEq)]
struct ShadowMask {
    region: ShadowRegion,
    style: CellStyle,
}

impl ShadowMask {
    fn covers(&self, row: usize, col: usize) -> bool {
        match &self.region {
            | ShadowRegion::Complement(covered) => {
                if let Some(row_mask) = covered.get(row) &&
                    let Some(cell) = row_mask.get(col)
                {
                    return !cell;
                }
                false
            },
            | ShadowRegion::Rect(rect) => {
                let r = row as u16;
                let c = col as u16;
                r >= rect.y &&
                    r < rect.y.saturating_add(rect.height) &&
                    c >= rect.x &&
                    c < rect.x.saturating_add(rect.width)
            },
        }
    }
}

/// Composites multiple full-terminal-size layer buffers front-to-back.
///
/// Layers are added from highest index (front) to lowest index (back). Each
/// layer is rendered independently; cells already covered by a higher layer are
/// stripped from lower layers.
pub struct Compositor {
    width: usize,
    height: usize,
    output: Vec<Vec<Cell>>,
    covered: Vec<Vec<bool>>,
    shadows: Vec<ShadowMask>,
    cursor: Option<(usize, usize)>,
    images: Vec<ImageCommand>,
}

impl Compositor {
    /// Create a compositor for the given terminal size.
    pub fn new(width: u16, height: u16) -> Self {
        let w = width as usize;
        let h = height as usize;
        Self {
            width: w,
            height: h,
            output: vec![vec![Cell::default(); w]; h],
            covered: vec![vec![false; w]; h],
            shadows: Vec::new(),
            cursor: None,
            images: Vec::new(),
        }
    }

    /// Add a layer's rendered buffer. Layers must be added front-to-back.
    pub fn add_layer(&mut self, rendered: &Rendered, shadow: &Shadow) {
        let grid = parse_rendered(rendered, self.width, self.height);
        let mut layer_covered = vec![vec![false; self.width]; self.height];

        for (r, row) in grid.iter().enumerate() {
            let mut col = 0usize;
            for cell in row {
                if cell.width == 0 {
                    continue;
                }
                let w = cell.width as usize;
                let end = col + w;
                let opaque = is_opaque(cell);
                if opaque && end <= self.width && !self.is_covered(r, col, w) {
                    let style = self.apply_shadows(r, col, &cell.style);
                    self.output[r][col] = Cell {
                        symbol: cell.symbol.clone(),
                        style: style.clone(),
                        width: cell.width,
                        transparent: false,
                    };
                    if w == 2 {
                        self.output[r][col + 1] = Cell {
                            symbol: String::new(),
                            style,
                            width: 0,
                            transparent: false,
                        };
                    }
                    self.mark_covered(r, col, w);
                }
                if opaque {
                    layer_covered[r][col] = true;
                    if w == 2 {
                        layer_covered[r][col + 1] = true;
                    }
                }
                col += w;
            }
        }

        if self.cursor.is_none() &&
            let Some((r, c)) = rendered.cursor &&
            r < self.height &&
            c < self.width
        {
            self.cursor = Some((r, c));
        }

        self.images.extend(rendered.images.clone());
        self.add_shadow(shadow, &layer_covered);
    }

    /// Consume the compositor and produce the final composite frame.
    pub fn finalize(self) -> Rendered {
        let mut lines = Vec::with_capacity(self.height);
        for row in self.output {
            lines.push(encode_cells_to_line(&row));
        }
        Rendered {
            lines,
            cursor: self.cursor,
            images: self.images,
        }
    }

    fn is_covered(&self, row: usize, col: usize, width: usize) -> bool {
        if let Some(row_mask) = self.covered.get(row) {
            for c in col..col + width {
                if let Some(true) = row_mask.get(c) {
                    return true;
                }
            }
        }
        false
    }

    fn mark_covered(&mut self, row: usize, col: usize, width: usize) {
        if let Some(row_mask) = self.covered.get_mut(row) {
            for c in col..col + width {
                if let Some(cell) = row_mask.get_mut(c) {
                    *cell = true;
                }
            }
        }
    }

    fn apply_shadows(&self, row: usize, col: usize, style: &CellStyle) -> CellStyle {
        let mut result = style.clone();
        for shadow in &self.shadows {
            if shadow.covers(row, col) {
                result = merge_style(result, &shadow.style);
            }
        }
        result
    }

    fn add_shadow(&mut self, shadow: &Shadow, layer_covered: &[Vec<bool>]) {
        match shadow {
            | Shadow::None => {},
            | Shadow::Dim { style } => {
                let cell_style = parse_style_string(style);
                self.shadows.push(ShadowMask {
                    region: ShadowRegion::Complement(layer_covered.to_vec()),
                    style: cell_style,
                });
            },
            | Shadow::Drop {
                style,
                offset_x,
                offset_y,
            } => {
                let cell_style = parse_style_string(style);
                if let Some(rect) = compute_bbox(layer_covered) {
                    let x = (rect.x as i16).saturating_add(*offset_x);
                    let y = (rect.y as i16).saturating_add(*offset_y);
                    let shadow_rect =
                        Rect::new(x.max(0) as u16, y.max(0) as u16, rect.width, rect.height);
                    self.shadows.push(ShadowMask {
                        region: ShadowRegion::Rect(shadow_rect),
                        style: cell_style,
                    });
                }
            },
        }
    }
}

fn merge_style(mut target: CellStyle, source: &CellStyle) -> CellStyle {
    target.bold = target.bold || source.bold;
    target.faint = target.faint || source.faint;
    target.italic = target.italic || source.italic;
    target.underline = target.underline || source.underline;
    target.reverse = target.reverse || source.reverse;
    if target.fg_color.is_none() && source.fg_color.is_some() {
        target.fg_color = source.fg_color.clone();
    }
    if target.bg_color.is_none() && source.bg_color.is_some() {
        target.bg_color = source.bg_color.clone();
    }
    if target.hyperlink.is_none() && source.hyperlink.is_some() {
        target.hyperlink = source.hyperlink.clone();
    }
    if !source.prefix.is_empty() {
        target.prefix.push_str(&source.prefix);
    }
    target
}

fn parse_style_string(style: &str) -> CellStyle {
    let mut tracker = AnsiCodeTracker::new();
    let mut prefix = String::new();
    let mut chars = style.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' &&
            let Some(seq) = extract_sequence(&mut chars)
        {
            let before = tracker.clone();
            tracker.process(&seq);
            if tracker == before {
                prefix.push_str(&seq);
            }
        }
    }
    CellStyle::from_tracker(&tracker, &prefix)
}

fn compute_bbox(covered: &[Vec<bool>]) -> Option<Rect> {
    let mut min_row: Option<usize> = None;
    let mut max_row: Option<usize> = None;
    let mut min_col: Option<usize> = None;
    let mut max_col: Option<usize> = None;

    for (r, row) in covered.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            if *cell {
                if min_row.is_none() {
                    min_row = Some(r);
                }
                max_row = Some(r);
                if min_col.is_none_or(|m| c < m) {
                    min_col = Some(c);
                }
                if max_col.is_none_or(|m| c > m) {
                    max_col = Some(c);
                }
            }
        }
    }

    match (min_row, max_row, min_col, max_col) {
        | (Some(min_r), Some(max_r), Some(min_c), Some(max_c)) => Some(Rect::new(
            min_c as u16,
            min_r as u16,
            (max_c - min_c + 1) as u16,
            (max_r - min_r + 1) as u16,
        )),
        | _ => None,
    }
}

fn parse_rendered(rendered: &Rendered, width: usize, height: usize) -> Vec<Vec<Cell>> {
    let mut grid = vec![Vec::new(); height];
    for (r, line) in rendered.lines.iter().enumerate() {
        if r >= height {
            break;
        }
        grid[r] = parse_line_to_cells(line, width);
    }
    grid
}

fn parse_line_to_cells(line: &str, max_width: usize) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    let mut tracker = AnsiCodeTracker::new();
    let mut prefix = String::new();
    let mut visible_width = 0usize;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some(seq) = extract_sequence(&mut chars) {
                let before = tracker.clone();
                tracker.process(&seq);
                if tracker == before {
                    prefix.push_str(&seq);
                }
            }
            continue;
        }

        let w = ch.width().unwrap_or(0);
        if w == 0 {
            if let Some(last) = cells.last_mut() {
                last.symbol.push(ch);
            }
            continue;
        }

        if visible_width + w > max_width {
            break;
        }

        let style = CellStyle::from_tracker(&tracker, &prefix);
        prefix.clear();
        cells.push(Cell {
            symbol: ch.to_string(),
            style,
            width: w as u8,
            transparent: false,
        });
        if w == 2 {
            cells.push(Cell {
                symbol: String::new(),
                style: CellStyle::default(),
                width: 0,
                transparent: false,
            });
        }
        visible_width += w;
    }

    // Compute the real content range: columns between the first and last
    // non-whitespace / styled character. Plain whitespace outside this range is
    // transparent padding; plain whitespace inside the range is intentional
    // spacing and remains opaque.
    let mut first_content_col: Option<usize> = None;
    let mut last_content_col = 0usize;
    let mut col = 0usize;
    for cell in &cells {
        if cell.width == 0 {
            continue;
        }
        if !cell.symbol.trim().is_empty() || cell.style.has_sgr() {
            if first_content_col.is_none() {
                first_content_col = Some(col);
            }
            last_content_col = col + cell.width as usize;
        }
        col += cell.width as usize;
    }

    let first = first_content_col.unwrap_or(0);
    let mut col = 0usize;
    for cell in &mut cells {
        if cell.width == 0 {
            continue;
        }
        if (col < first || col >= last_content_col) && !cell.style.has_sgr() {
            cell.transparent = true;
        }
        col += cell.width as usize;
    }

    cells
}

fn extract_sequence(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    let mut seq = String::from('\x1b');
    match chars.peek() {
        | Some(&'[') => {
            chars.next();
            seq.push('[');
            while let Some(&c) = chars.peek() {
                chars.next();
                seq.push(c);
                if c.is_alphabetic() {
                    return Some(seq);
                }
            }
        },
        | Some(&']') => {
            chars.next();
            seq.push(']');
            while let Some(&c) = chars.peek() {
                chars.next();
                seq.push(c);
                if c == '\x07' {
                    return Some(seq);
                }
                if c == '\x1b' &&
                    let Some(&'\\') = chars.peek()
                {
                    chars.next();
                    seq.push('\\');
                    return Some(seq);
                }
            }
        },
        | _ => {},
    }
    None
}

fn encode_cells_to_line(cells: &[Cell]) -> String {
    let mut line = String::new();
    let mut current = CellStyle::default();

    for cell in cells {
        if cell.width == 0 {
            continue;
        }

        if cell.style != current {
            // Close hyperlink if it is changing.
            if current.hyperlink != cell.style.hyperlink &&
                let Some(ref link) = current.hyperlink
            {
                line.push_str(&format!("\x1b]8;;{}", link.terminator));
            }
            // Reset SGR when any parsed SGR attribute is currently active.
            if current.has_sgr() {
                line.push_str("\x1b[0m");
            }
            // Apply new SGR.
            let sgr = cell.style.sgr_sequence();
            if !sgr.is_empty() {
                line.push_str(&sgr);
            }
            // Open new hyperlink if it differs from current.
            if cell.style.hyperlink != current.hyperlink &&
                let Some(ref link) = cell.style.hyperlink
            {
                line.push_str(&format!(
                    "\x1b]8;{};{}{}",
                    link.params, link.url, link.terminator
                ));
            }
            // Emit any unrecognized prefix sequences.
            if !cell.style.prefix.is_empty() {
                line.push_str(&cell.style.prefix);
            }
            current = cell.style.clone();
        }

        line.push_str(&cell.symbol);
    }

    if let Some(ref link) = current.hyperlink {
        line.push_str(&format!("\x1b]8;;{}", link.terminator));
    }
    if current.has_sgr() {
        line.push_str("\x1b[0m");
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered_from(lines: &[&str]) -> Rendered {
        Rendered {
            lines: lines.iter().map(|s| s.to_string()).collect(),
            cursor: None,
            images: Vec::new(),
        }
    }

    #[test]
    fn cell_parse_empty_line() {
        let cells = parse_line_to_cells("", 10);
        assert!(cells.is_empty());
    }

    #[test]
    fn cell_parse_plain_text() {
        let cells = parse_line_to_cells("abc", 10);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].symbol, "a");
        assert_eq!(cells[1].symbol, "b");
        assert_eq!(cells[2].symbol, "c");
        assert_eq!(cells[0].width, 1);
    }

    #[test]
    fn cell_parse_ansi_bold() {
        let cells = parse_line_to_cells("\x1b[1mhi\x1b[0m", 10);
        assert_eq!(cells.len(), 2);
        assert!(cells[0].style.bold);
        // The reset comes after 'i', so 'i' is still bold at parse time.
        assert!(cells[1].style.bold);
    }

    #[test]
    fn cell_parse_ansi_colors() {
        let cells = parse_line_to_cells("\x1b[31;44mX", 10);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].style.fg_color, Some("31".to_string()));
        assert_eq!(cells[0].style.bg_color, Some("44".to_string()));
    }

    #[test]
    fn cell_parse_hyperlink() {
        let cells = parse_line_to_cells("\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\", 10);
        assert_eq!(cells.len(), 4);
        // The hyperlink is active for all visible characters; the close
        // sequence follows the last character.
        assert!(cells[0].style.hyperlink.is_some());
        assert!(cells[3].style.hyperlink.is_some());
    }

    #[test]
    fn cell_parse_cjk_and_emoji() {
        let cells = parse_line_to_cells("漢a", 10);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].symbol, "漢");
        assert_eq!(cells[0].width, 2);
        assert_eq!(cells[1].width, 0);
        assert_eq!(cells[2].symbol, "a");
    }

    #[test]
    fn cell_encode_plain_text() {
        let cells = parse_line_to_cells("abc", 10);
        let line = encode_cells_to_line(&cells);
        assert_eq!(line, "abc");
    }

    #[test]
    fn cell_roundtrip_preserves_visible_width() {
        let original = "\x1b[31mred\x1b[0m \x1b[1mbold\x1b[0m";
        let cells = parse_line_to_cells(original, 20);
        let encoded = encode_cells_to_line(&cells);
        assert_eq!(
            crate::utils::visible_width(&encoded),
            crate::utils::visible_width(original)
        );
    }

    #[test]
    fn cell_encode_resets_at_line_end() {
        let cells = parse_line_to_cells("\x1b[31mred", 10);
        let line = encode_cells_to_line(&cells);
        assert!(line.ends_with("\x1b[0m"));
    }

    #[test]
    fn compositor_empty_layers() {
        let mut comp = Compositor::new(10, 2);
        comp.add_layer(&rendered_from(&["", ""]), &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.lines.len(), 2);
        assert_eq!(out.lines[0], "");
    }

    #[test]
    fn compositor_single_layer_passthrough() {
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(&rendered_from(&["hello"]), &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.lines[0], "hello");
    }

    #[test]
    fn compositor_two_layers_full_occlusion() {
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(&rendered_from(&["WORLD"]), &Shadow::None);
        comp.add_layer(&rendered_from(&["hello"]), &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.lines[0], "WORLD");
    }

    #[test]
    fn compositor_three_layers_partial_occlusion() {
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(&rendered_from(&["ABC  "]), &Shadow::None);
        comp.add_layer(&rendered_from(&["  XYZ"]), &Shadow::None);
        comp.add_layer(&rendered_from(&["12345"]), &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.lines[0], "ABCYZ");
    }

    #[test]
    fn compositor_cursor_topmost_wins() {
        let mut top = rendered_from(&["top"]);
        top.cursor = Some((0, 2));
        let mut bottom = rendered_from(&["bottom"]);
        bottom.cursor = Some((0, 1));
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(&top, &Shadow::None);
        comp.add_layer(&bottom, &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.cursor, Some((0, 2)));
    }

    #[test]
    fn compositor_images_merged_from_all_layers() {
        let mut top = rendered_from(&["top"]);
        top.images.push(ImageCommand {
            id: 1,
            data: "a".into(),
        });
        let mut bottom = rendered_from(&["bottom"]);
        bottom.images.push(ImageCommand {
            id: 2,
            data: "b".into(),
        });
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(&top, &Shadow::None);
        comp.add_layer(&bottom, &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.images.len(), 2);
    }

    #[test]
    fn compositor_dim_shadow_applies_to_exposed_lower_cells() {
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(
            &rendered_from(&[" ABC "]),
            &Shadow::Dim {
                style: "\x1b[2m".into(),
            },
        );
        comp.add_layer(&rendered_from(&["12345"]), &Shadow::None);
        let out = comp.finalize();
        // Columns 0 and 4 are exposed and should be dimmed.
        assert!(out.lines[0].starts_with("\x1b[2m1"));
        // Columns 1-3 are covered by the top layer.
        assert!(out.lines[0].contains("ABC"));
    }

    #[test]
    fn compositor_drop_shadow_offset_positive() {
        let mut comp = Compositor::new(6, 3);
        comp.add_layer(
            &rendered_from(&["", " AB ", ""]),
            &Shadow::Drop {
                style: "\x1b[2m".into(),
                offset_x: 1,
                offset_y: 1,
            },
        );
        comp.add_layer(
            &rendered_from(&["XXXXXX", "XXXXXX", "XXXXXX"]),
            &Shadow::None,
        );
        let out = comp.finalize();
        // Shadow should appear at row 2, columns 2-4.
        assert!(out.lines[2].contains("\x1b[2m"));
    }

    #[test]
    fn compositor_shadow_none_is_identity() {
        let mut comp = Compositor::new(5, 1);
        comp.add_layer(&rendered_from(&["hello"]), &Shadow::None);
        let out = comp.finalize();
        assert_eq!(out.lines[0], "hello");
    }

    #[test]
    fn compositor_ansi_reset_preserved_at_boundaries() {
        let mut comp = Compositor::new(10, 1);
        comp.add_layer(&rendered_from(&["     world"]), &Shadow::None);
        comp.add_layer(
            &rendered_from(&["\x1b[44mhello\x1b[0m     "]),
            &Shadow::None,
        );
        let out = comp.finalize();
        // The blue background from the bottom layer must be reset before the
        // top layer's plain text begins.
        assert!(out.lines[0].contains("\x1b[0m"));
    }

    #[test]
    fn compositor_no_panic_on_oversized_layer() {
        let mut comp = Compositor::new(3, 1);
        comp.add_layer(&rendered_from(&["hello world"]), &Shadow::None);
        let out = comp.finalize();
        assert!(out.lines[0].len() <= 11);
    }
}
