use std::collections::HashMap;

use crossterm::event::KeyCode;

use crate::{
    Component,
    Event,
    Focusable,
    InputResult,
    RenderError,
    Rendered,
    theme::{
        Palette,
        Style,
        Theme,
        stylize,
    },
};

/// Callback type for table filter input.
type FilterCallback = Box<dyn Fn(&str)>;

/// A column definition for the [`Table`] component.
pub struct Column {
    /// Unique identifier for this column, used as a key into row data.
    pub key: String,
    /// Display label shown in the table header.
    pub label: String,
    /// Fixed width in columns, or `None` to distribute space automatically.
    pub width: Option<u16>,
    /// Whether the column can be sorted.
    pub sortable: bool,
}

impl Column {
    /// Create a new column with the given key and label.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            width: None,
            sortable: false,
        }
    }

    /// Set a fixed width for this column.
    pub fn width(mut self, w: u16) -> Self {
        self.width = Some(w);
        self
    }

    /// Mark this column as sortable.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

/// A single row of data in the [`Table`] component.
pub struct Row {
    cells: HashMap<String, String>,
}

impl Row {
    /// Create a new row from a map of column key to cell value.
    pub fn new(cells: HashMap<String, String>) -> Self {
        Self { cells }
    }

    /// Get the cell value for the given column key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cells.get(key).map(|s| s.as_str())
    }
}

/// A table component with sortable columns, row selection, and interactive
/// filtering.
///
/// Renders as a header row, a separator line, and data rows. The selected row
/// shows a `> ` prefix and is highlighted with the theme's accent color when
/// the table is focused.
///
/// # Hooks
///
/// - [`on_select`](Table::on_select) — fired when the user navigates rows.
/// - [`on_sort`](Table::on_sort) — fired when sort column or direction changes.
/// - [`on_filter`](Table::on_filter) — fired when the filter query changes.
/// - [`on_filter_char`](Table::on_filter_char) — transforms or rejects filter
///   characters during interactive filter mode.
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Row>,
    selected: usize,
    sort_column: Option<usize>,
    sort_ascending: bool,
    focused: bool,
    filter_query: Option<String>,
    filter_buffer: String,
    in_filter_mode: bool,
    filter_key: char,
    sort_indicator_asc: String,
    sort_indicator_desc: String,
    display_indices: Vec<usize>,
    on_select: Option<Box<dyn Fn(usize)>>,
    on_sort: Option<Box<dyn Fn(usize, bool)>>,
    on_filter: Option<FilterCallback>,
    on_filter_char: Option<Box<dyn Fn(char) -> Option<char>>>,
}

impl Table {
    /// Create a new table with the given columns and rows.
    pub fn new(columns: Vec<Column>, rows: Vec<Row>) -> Self {
        let display_indices: Vec<usize> = (0..rows.len()).collect();
        Self {
            columns,
            rows,
            selected: 0,
            sort_column: None,
            sort_ascending: true,
            focused: false,
            filter_query: None,
            filter_buffer: String::new(),
            in_filter_mode: false,
            filter_key: '/',
            sort_indicator_asc: "▲".to_string(),
            sort_indicator_desc: "▼".to_string(),
            display_indices,
            on_select: None,
            on_sort: None,
            on_filter: None,
            on_filter_char: None,
        }
    }

    /// Index of the currently selected visible row.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected row index (clamped to valid visible range).
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.display_indices.len().saturating_sub(1));
    }

    /// Set the column used for sorting display.
    pub fn set_sort_column(&mut self, column: Option<usize>) {
        self.sort_column = column;
        self.recompute_display_indices();
    }

    /// Set whether the current sort is ascending.
    pub fn set_sort_ascending(&mut self, ascending: bool) {
        self.sort_ascending = ascending;
        self.recompute_display_indices();
    }

    /// Sort by the given column index. Toggles direction if already sorting
    /// by the same column. Does nothing if the column is not sortable.
    pub fn sort_by(&mut self, column: usize) {
        if column >= self.columns.len() {
            return;
        }
        if !self.columns[column].sortable {
            return;
        }
        if self.sort_column == Some(column) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = Some(column);
            self.sort_ascending = true;
        }
        self.recompute_display_indices();
        if let Some(ref cb) = self.on_sort {
            cb(column, self.sort_ascending);
        }
    }

    /// Clear sorting and restore original row order.
    pub fn clear_sort(&mut self) {
        self.sort_column = None;
        self.sort_ascending = true;
        self.recompute_display_indices();
    }

    /// Set a filter query. Only rows with at least one cell containing the
    /// query (case-insensitive) are displayed.
    pub fn set_filter(&mut self, query: impl Into<String>) {
        let q = query.into();
        self.filter_query = if q.is_empty() { None } else { Some(q) };
        self.filter_buffer = self.filter_query.clone().unwrap_or_default();
        self.recompute_display_indices();
        if let Some(ref cb) = self.on_filter {
            let query_str = self.filter_query.as_deref().unwrap_or("");
            cb(query_str);
        }
    }

    /// Clear the filter and show all rows.
    pub fn clear_filter(&mut self) {
        self.filter_query = None;
        self.filter_buffer.clear();
        self.recompute_display_indices();
        if let Some(ref cb) = self.on_filter {
            cb("");
        }
    }

    /// Return the current filter query, if any.
    pub fn filter_query(&self) -> Option<&str> {
        self.filter_query.as_deref()
    }

    /// Return the current sort column, if any.
    pub fn sort_column_index(&self) -> Option<usize> {
        self.sort_column
    }

    /// Return whether the current sort is ascending.
    pub fn sort_ascending(&self) -> bool {
        self.sort_ascending
    }

    /// Return `true` if the table is currently accepting interactive filter
    /// input.
    pub fn in_filter_mode(&self) -> bool {
        self.in_filter_mode
    }

    /// Return the number of rows currently displayed (after filtering).
    pub fn displayed_row_count(&self) -> usize {
        self.display_indices.len()
    }

    /// Return the original row index of the currently selected visible row.
    pub fn selected_original_index(&self) -> Option<usize> {
        self.display_indices.get(self.selected).copied()
    }

    /// Return a reference to the currently selected visible row.
    pub fn selected_row(&self) -> Option<&Row> {
        self.display_indices
            .get(self.selected)
            .map(|idx| &self.rows[*idx])
    }

    /// Replace the rows and recompute the display order.
    pub fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        self.recompute_display_indices();
    }

    /// Attach a callback invoked when the selected row changes via keyboard
    /// navigation.
    pub fn on_select(mut self, cb: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    /// Attach a callback invoked when sort column or direction changes.
    pub fn on_sort(mut self, cb: impl Fn(usize, bool) + 'static) -> Self {
        self.on_sort = Some(Box::new(cb));
        self
    }

    /// Attach a callback invoked when the filter query changes.
    pub fn on_filter(mut self, cb: impl Fn(&str) + 'static) -> Self {
        self.on_filter = Some(Box::new(cb));
        self
    }

    /// Attach a callback invoked for each character typed in interactive filter
    /// mode. Return `Some(transformed)` to accept the character (possibly
    /// modified) or `None` to reject it.
    pub fn on_filter_char(mut self, cb: impl Fn(char) -> Option<char> + 'static) -> Self {
        self.on_filter_char = Some(Box::new(cb));
        self
    }

    /// Set the key that enters interactive filter mode. Defaults to `'/'`.
    pub fn filter_key(mut self, key: char) -> Self {
        self.filter_key = key;
        self
    }

    /// Set the indicator shown next to a column header when sorted ascending.
    pub fn sort_indicator_asc(mut self, indicator: impl Into<String>) -> Self {
        self.sort_indicator_asc = indicator.into();
        self
    }

    /// Set the indicator shown next to a column header when sorted descending.
    pub fn sort_indicator_desc(mut self, indicator: impl Into<String>) -> Self {
        self.sort_indicator_desc = indicator.into();
        self
    }

    fn move_selection_down(&mut self) {
        if self.selected + 1 < self.display_indices.len() {
            self.selected += 1;
            if let Some(ref cb) = self.on_select {
                cb(self.selected);
            }
        }
    }

    fn move_selection_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if let Some(ref cb) = self.on_select {
                cb(self.selected);
            }
        }
    }

    fn recompute_display_indices(&mut self) {
        let mut indices: Vec<usize> = (0..self.rows.len()).collect();

        // Apply filter
        if let Some(ref query) = self.filter_query {
            let query_lower = query.to_lowercase();
            indices.retain(|idx| {
                let row = &self.rows[*idx];
                for col in &self.columns {
                    if let Some(val) = row.get(&col.key) &&
                        val.to_lowercase().contains(&query_lower)
                    {
                        return true;
                    }
                }
                false
            });
        }

        // Apply sort
        if let Some(sort_col) = self.sort_column &&
            sort_col < self.columns.len() &&
            self.columns[sort_col].sortable
        {
            let key = &self.columns[sort_col].key;
            let ascending = self.sort_ascending;
            indices.sort_by(|a, b| {
                let val_a = self.rows[*a].get(key).unwrap_or("");
                let val_b = self.rows[*b].get(key).unwrap_or("");
                match ascending {
                    | true => val_a.cmp(val_b),
                    | false => val_b.cmp(val_a),
                }
            });
        }

        self.display_indices = indices;
        self.selected = self
            .selected
            .min(self.display_indices.len().saturating_sub(1));
    }

    fn compute_column_widths(&self, total_width: u16) -> Vec<u16> {
        let num_cols = self.columns.len();
        if num_cols == 0 {
            return Vec::new();
        }

        let separator_width = (num_cols.saturating_sub(1)) as u16;
        let prefix_width = 2u16;
        let budget = total_width
            .saturating_sub(prefix_width)
            .saturating_sub(separator_width);

        if budget == 0 {
            return vec![0; num_cols];
        }

        let mut widths = Vec::with_capacity(num_cols);
        let mut flex_indices = Vec::new();
        let mut fixed_total = 0u16;

        for (i, col) in self.columns.iter().enumerate() {
            if let Some(w) = col.width {
                let w = w.min(budget);
                widths.push(w);
                fixed_total += w;
            } else {
                widths.push(0);
                flex_indices.push(i);
            }
        }

        if !flex_indices.is_empty() {
            let flex_budget = budget.saturating_sub(fixed_total);
            let flex_width = if flex_budget > 0 {
                flex_budget / flex_indices.len() as u16
            } else {
                1
            };
            for &i in &flex_indices {
                widths[i] = flex_width.max(1);
            }
        }

        // If total exceeds budget, scale proportionally
        let total: u16 = widths.iter().sum();
        if total > budget && budget > 0 {
            for w in &mut widths {
                *w = (*w as u32 * budget as u32 / total as u32) as u16;
            }
        }

        widths
    }
}

impl Focusable for Table {
    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for Table {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();

        if self.columns.is_empty() {
            return Ok(Rendered {
                lines: Vec::new(),
                cursor: None,
                images: Vec::new(),
            });
        }

        let separator_count = self.columns.len().saturating_sub(1) as u16;
        let min_width = 2u16 + separator_count;
        if width < min_width {
            return Ok(Rendered {
                lines: Vec::new(),
                cursor: None,
                images: Vec::new(),
            });
        }

        let widths = self.compute_column_widths(width);
        let mut lines = Vec::new();

        // Filter input line (when in interactive filter mode)
        if self.in_filter_mode {
            let filter_style = Style::new().fg(theme.text_secondary());
            let filter_text = format!("/{}", self.filter_buffer);
            let filter_line = crate::utils::truncate_to_width(&filter_text, width, "…");
            lines.push(stylize(&filter_line, &filter_style));
        }

        // Header row
        let header_style = Style::new().fg(theme.text_primary()).bold();
        let mut header_parts = vec![stylize("  ", &header_style)];
        for (i, col) in self.columns.iter().enumerate() {
            let mut label = col.label.clone();
            if let Some(sort_idx) = self.sort_column &&
                sort_idx == i &&
                col.sortable
            {
                let indicator = if self.sort_ascending {
                    &self.sort_indicator_asc
                } else {
                    &self.sort_indicator_desc
                };
                label.push_str(indicator);
            }

            let cell_width = widths.get(i).copied().unwrap_or(0);
            let cell = if cell_width == 0 {
                String::new()
            } else {
                let truncated = crate::utils::truncate_to_width(&label, cell_width, "…");
                format!("{:<width$}", truncated, width = cell_width as usize)
            };
            header_parts.push(stylize(&cell, &header_style));

            if i + 1 < self.columns.len() {
                header_parts.push(" ".to_string());
            }
        }
        lines.push(header_parts.concat());

        // Separator line
        let sep_line = "─".repeat(width as usize);
        let sep_style = Style::new().fg(theme.border_default());
        lines.push(stylize(&sep_line, &sep_style));

        // Data rows
        let accent_style = Style::new().fg(theme.accent()).bold();
        let text_style = Style::new().fg(theme.text_primary());

        for (visible_idx, &row_idx) in self.display_indices.iter().enumerate() {
            let is_selected = visible_idx == self.selected;
            let row_style = if is_selected && self.focused {
                &accent_style
            } else {
                &text_style
            };

            let prefix = if is_selected && self.focused {
                stylize("> ", row_style)
            } else {
                "  ".to_string()
            };

            let row = &self.rows[row_idx];
            let mut row_parts = vec![prefix];
            for (col_idx, col) in self.columns.iter().enumerate() {
                let cell_width = widths.get(col_idx).copied().unwrap_or(0);
                let cell_text = row.get(&col.key).unwrap_or("");
                let cell = if cell_width == 0 {
                    String::new()
                } else {
                    let truncated = crate::utils::truncate_to_width(cell_text, cell_width, "…");
                    format!("{:<width$}", truncated, width = cell_width as usize)
                };
                row_parts.push(stylize(&cell, row_style));

                if col_idx + 1 < self.columns.len() {
                    row_parts.push(" ".to_string());
                }
            }
            lines.push(row_parts.concat());
        }

        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        use crossterm::event::KeyModifiers;
        if let Event::Key(key) = event {
            if self.in_filter_mode {
                match key.code {
                    | KeyCode::Esc => {
                        self.in_filter_mode = false;
                        self.filter_buffer.clear();
                        InputResult::Handled
                    },
                    | KeyCode::Enter => {
                        self.in_filter_mode = false;
                        if self.filter_buffer.is_empty() {
                            self.clear_filter();
                        }
                        InputResult::Handled
                    },
                    | KeyCode::Backspace => {
                        if self.filter_buffer.is_empty() {
                            self.in_filter_mode = false;
                            self.clear_filter();
                        } else {
                            self.filter_buffer.pop();
                            let buf = self.filter_buffer.clone();
                            self.set_filter(&buf);
                        }
                        InputResult::Handled
                    },
                    | KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let ch = if let Some(ref cb) = self.on_filter_char {
                            match cb(c) {
                                | Some(transformed) => transformed,
                                | None => return InputResult::Handled,
                            }
                        } else {
                            c
                        };
                        self.filter_buffer.push(ch);
                        let buf = self.filter_buffer.clone();
                        self.set_filter(&buf);
                        InputResult::Handled
                    },
                    | _ => InputResult::Ignored,
                }
            } else {
                match key.code {
                    | KeyCode::Down => {
                        self.move_selection_down();
                        InputResult::Handled
                    },
                    | KeyCode::Up => {
                        self.move_selection_up();
                        InputResult::Handled
                    },
                    | KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.move_selection_down();
                        InputResult::Handled
                    },
                    | KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.move_selection_up();
                        InputResult::Handled
                    },
                    | KeyCode::Char(c)
                        if c == self.filter_key &&
                            !key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.in_filter_mode = true;
                        self.filter_buffer = self.filter_query.clone().unwrap_or_default();
                        InputResult::Handled
                    },
                    | _ => InputResult::Ignored,
                }
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> {
        Some(self)
    }

    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{
            Cell,
            RefCell,
        },
        collections::HashMap,
        rc::Rc,
    };

    use crossterm::event::KeyCode;

    use super::*;
    use crate::Event;

    #[test]
    fn table_new() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let table = Table::new(cols, rows);
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn table_set_selected_clamps() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_selected(100);
        assert_eq!(table.selected(), 1);
    }

    #[test]
    fn table_renders_header_and_rows() {
        Theme::with(Theme::Light, || {
            let cols = vec![Column::new("name", "Name")];
            let rows = vec![Row::new(HashMap::from([(
                "name".to_string(),
                "Alice".to_string(),
            )]))];
            let table = Table::new(cols, rows);
            let rendered = table.render(40).unwrap();
            assert_eq!(rendered.lines.len(), 3); // header, sep, 1 row
            assert!(rendered.lines[0].contains("Name"));
        });
    }

    #[test]
    fn table_selected_row_focused() {
        Theme::with(Theme::Light, || {
            let cols = vec![Column::new("name", "Name")];
            let rows = vec![Row::new(HashMap::from([(
                "name".to_string(),
                "Alice".to_string(),
            )]))];
            let mut table = Table::new(cols, rows);
            table.set_focused(true);
            let rendered = table.render(40).unwrap();
            assert!(rendered.lines[2].contains("> "));
        });
    }

    #[test]
    fn table_navigation() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.selected(), 1);
    }

    #[test]
    fn table_j_k_navigation() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.selected(), 1);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('k'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn table_sort_indicator() {
        Theme::with(Theme::Light, || {
            let cols = vec![Column::new("name", "Name").sortable()];
            let rows = vec![Row::new(HashMap::from([(
                "name".to_string(),
                "Alice".to_string(),
            )]))];
            let mut table = Table::new(cols, rows);
            table.set_sort_column(Some(0));
            table.set_sort_ascending(true);
            let rendered = table.render(40).unwrap();
            assert!(rendered.lines[0].contains("▲"));
        });
    }

    #[test]
    fn table_empty_columns() {
        let cols: Vec<Column> = vec![];
        let rows: Vec<Row> = vec![];
        let table = Table::new(cols, rows);
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines.is_empty());
    }

    #[test]
    fn table_unfocused_no_accent_prefix() {
        Theme::with(Theme::Light, || {
            let cols = vec![Column::new("name", "Name")];
            let rows = vec![Row::new(HashMap::from([(
                "name".to_string(),
                "Alice".to_string(),
            )]))];
            let table = Table::new(cols, rows);
            let rendered = table.render(40).unwrap();
            assert!(!rendered.lines[2].contains("> "));
        });
    }

    #[test]
    fn table_sort_by_reorders_rows() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.sort_by(0);
        assert_eq!(table.displayed_row_count(), 3);
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines[2].contains("Alice"));
        assert!(rendered.lines[3].contains("Bob"));
        assert!(rendered.lines[4].contains("Charlie"));
    }

    #[test]
    fn table_filter_rows() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_filter("a");
        assert_eq!(table.displayed_row_count(), 2);
    }

    #[test]
    fn table_clear_filter() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_filter("Alice");
        assert_eq!(table.displayed_row_count(), 1);
        table.clear_filter();
        assert_eq!(table.displayed_row_count(), 2);
    }

    #[test]
    fn table_selected_row_returns_correct_data() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Charlie".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.sort_by(0);
        let row = table.selected_row();
        assert!(row.is_some());
        assert_eq!(row.unwrap().get("name"), Some("Alice"));
        assert_eq!(table.selected_original_index(), Some(1));
    }

    #[test]
    fn table_on_select_fires() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let selected = Rc::new(Cell::new(99usize));
        let sc = selected.clone();
        let mut table = Table::new(cols, rows).on_select(move |idx| {
            sc.set(idx);
        });
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(selected.get(), 1);
    }

    #[test]
    fn table_on_sort_fires() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let sort_col = Rc::new(Cell::new(99usize));
        let sort_asc = Rc::new(Cell::new(false));
        let sc = sort_col.clone();
        let sa = sort_asc.clone();
        let mut table = Table::new(cols, rows).on_sort(move |col, asc| {
            sc.set(col);
            sa.set(asc);
        });
        table.sort_by(0);
        assert_eq!(sort_col.get(), 0);
        assert!(sort_asc.get());
    }

    #[test]
    fn table_on_filter_fires() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let filter = Rc::new(RefCell::new(String::new()));
        let fi = filter.clone();
        let mut table = Table::new(cols, rows).on_filter(move |q| {
            *fi.borrow_mut() = q.to_string();
        });
        table.set_filter("Alice");
        assert_eq!(filter.borrow().as_str(), "Alice");
        table.clear_filter();
        assert_eq!(filter.borrow().as_str(), "");
    }

    #[test]
    fn table_on_filter_char_rejects() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows)
            .on_filter_char(|c| if c.is_alphabetic() { Some(c) } else { None });
        table.set_focused(true);
        // Enter filter mode
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(table.in_filter_mode());
        // Type a digit — should be rejected
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('1'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.filter_query(), None);
        // Type a letter — should be accepted
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.filter_query(), Some("a"));
    }

    #[test]
    fn table_filter_mode_enter_and_exit() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);

        // Press '/' to enter filter mode
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(table.in_filter_mode());

        // Type 'a'
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.filter_query(), Some("a"));

        // Press Enter to exit filter mode
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(!table.in_filter_mode());
        assert_eq!(table.filter_query(), Some("a"));
    }

    #[test]
    fn table_filter_mode_esc_clears_buffer() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);

        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.filter_query(), Some("a"));

        // Esc exits mode but keeps the applied filter
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(!table.in_filter_mode());
        assert_eq!(table.filter_query(), Some("a"));
    }

    #[test]
    fn table_filter_mode_backspace_exits_when_empty() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);

        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(table.in_filter_mode());

        // Backspace on empty buffer exits and clears filter
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(!table.in_filter_mode());
        assert_eq!(table.filter_query(), None);
    }

    #[test]
    fn table_filter_mode_renders_input_line() {
        Theme::with(Theme::Light, || {
            let cols = vec![Column::new("name", "Name").width(10)];
            let rows = vec![Row::new(HashMap::from([(
                "name".to_string(),
                "Alice".to_string(),
            )]))];
            let mut table = Table::new(cols, rows);
            table.set_focused(true);
            table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('/'),
                crossterm::event::KeyModifiers::empty(),
            )));
            table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('a'),
                crossterm::event::KeyModifiers::empty(),
            )));
            let rendered = table.render(40).unwrap();
            // Filter line + header + sep + 1 row = 4 lines
            assert_eq!(rendered.lines.len(), 4);
            assert!(rendered.lines[0].contains("/a"));
        });
    }

    #[test]
    fn table_custom_filter_key() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows).filter_key('f');
        table.set_focused(true);

        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('f'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(table.in_filter_mode());
    }

    #[test]
    fn table_custom_sort_indicators() {
        Theme::with(Theme::Light, || {
            let cols = vec![Column::new("name", "Name").sortable().width(10)];
            let rows = vec![Row::new(HashMap::from([(
                "name".to_string(),
                "Alice".to_string(),
            )]))];
            let mut table = Table::new(cols, rows)
                .sort_indicator_asc("^")
                .sort_indicator_desc("v");
            table.set_sort_column(Some(0));
            table.set_sort_ascending(true);
            let rendered = table.render(40).unwrap();
            assert!(rendered.lines[0].contains("^"));
            assert!(!rendered.lines[0].contains("▲"));

            table.set_sort_ascending(false);
            let rendered = table.render(40).unwrap();
            assert!(rendered.lines[0].contains("v"));
            assert!(!rendered.lines[0].contains("▼"));
        });
    }

    #[test]
    fn table_sort_by_out_of_bounds() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.sort_by(99);
        assert_eq!(table.sort_column_index(), None);
    }

    #[test]
    fn table_sort_by_unsortable_column() {
        let cols = vec![
            Column::new("name", "Name").sortable(),
            Column::new("status", "Status"),
        ];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.sort_by(1);
        assert_eq!(table.sort_column_index(), None);
    }

    #[test]
    fn table_clear_filter_fires_hook() {
        let filter = Rc::new(RefCell::new(String::from("init")));
        let fi = filter.clone();
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows).on_filter(move |q| {
            *fi.borrow_mut() = q.to_string();
        });
        table.set_filter("Alice");
        table.clear_filter();
        assert_eq!(filter.borrow().as_str(), "");
    }

    #[test]
    fn table_getters() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        assert_eq!(table.sort_column_index(), None);
        assert!(table.sort_ascending());
        assert!(!table.in_filter_mode());
        assert_eq!(table.displayed_row_count(), 2);
        assert_eq!(table.selected_original_index(), Some(0));

        table.sort_by(0);
        assert_eq!(table.sort_column_index(), Some(0));
        assert_eq!(table.selected_original_index(), Some(0));

        table.set_filter("Bob");
        assert_eq!(table.displayed_row_count(), 1);
        assert_eq!(table.selected_original_index(), Some(1));
    }

    #[test]
    fn table_selected_row_none_when_empty() {
        let cols = vec![Column::new("name", "Name")];
        let rows: Vec<Row> = vec![];
        let table = Table::new(cols, rows);
        assert!(table.selected_row().is_none());
    }

    #[test]
    fn table_render_tiny_width_returns_empty() {
        let cols = vec![Column::new("a", "A"), Column::new("b", "B")];
        let rows = vec![Row::new(HashMap::from([(
            "a".to_string(),
            "x".to_string(),
        )]))];
        let table = Table::new(cols, rows);
        let rendered = table.render(1).unwrap();
        assert!(rendered.lines.is_empty());
    }

    #[test]
    fn table_render_zero_budget_columns() {
        Theme::with(Theme::Light, || {
            let cols = vec![
                Column::new("a", "A").width(5),
                Column::new("b", "B").width(5),
            ];
            let rows = vec![Row::new(HashMap::from([(
                "a".to_string(),
                "x".to_string(),
            )]))];
            let table = Table::new(cols, rows);
            let rendered = table.render(4).unwrap();
            // Width is enough to pass min_width but columns scale to 0
            assert_eq!(rendered.lines.len(), 3); // header + sep + 1 row
            // Cells are empty because column widths are 0
            assert!(!rendered.lines[0].contains("A"));
        });
    }

    #[test]
    fn table_set_rows_with_active_filter() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_filter("Alice");
        assert_eq!(table.displayed_row_count(), 1);

        let new_rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alison".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alex".to_string())])),
        ];
        table.set_rows(new_rows);
        // Filter "Alice" still applies; only "Alice" matches
        assert_eq!(table.displayed_row_count(), 1);
    }

    #[test]
    fn table_filter_mode_enter_empty_buffer() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(table.in_filter_mode());
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(!table.in_filter_mode());
        assert_eq!(table.filter_query(), None);
    }

    #[test]
    fn table_filter_mode_unhandled_key_ignored() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        let result = table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(result, InputResult::Ignored);
        assert!(table.in_filter_mode());
    }

    #[test]
    fn table_non_key_event_ignored() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        let result = table.handle_input(&Event::Resize(80, 24));
        assert_eq!(result, InputResult::Ignored);
    }

    #[test]
    fn table_as_focusable_returns_some() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        assert!(table.as_focusable().is_some());
        assert!(table.as_focusable_mut().is_some());
        table.set_focused(true);
        assert!(table.focused());
    }

    #[test]
    fn table_set_selected_empty_rows() {
        let cols = vec![Column::new("name", "Name")];
        let rows: Vec<Row> = vec![];
        let mut table = Table::new(cols, rows);
        table.set_selected(5);
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn table_navigation_clamped_at_edges() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        // Already at top, Up should not move
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Up,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.selected(), 0);
        // Already at bottom (same row), Down should not move
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn table_filter_char_transforms() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows).on_filter_char(|c| Some(c.to_ascii_uppercase()));
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.filter_query(), Some("A"));
    }

    #[test]
    fn table_set_sort_column_and_ascending() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_sort_column(Some(0));
        table.set_sort_ascending(false);
        assert_eq!(table.sort_column_index(), Some(0));
        assert!(!table.sort_ascending());
        let rendered = table.render(40).unwrap();
        assert!(rendered.lines[2].contains("Bob"));
    }

    #[test]
    fn table_filter_mode_ctrl_char_ignored() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![Row::new(HashMap::from([(
            "name".to_string(),
            "Alice".to_string(),
        )]))];
        let mut table = Table::new(cols, rows);
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        )));
        let result = table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('c'),
            crossterm::event::KeyModifiers::CONTROL,
        )));
        assert_eq!(result, InputResult::Ignored);
    }

    #[test]
    fn table_sort_toggle_same_column() {
        let cols = vec![Column::new("name", "Name").sortable()];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.sort_by(0);
        assert!(table.sort_ascending());
        table.sort_by(0);
        assert!(!table.sort_ascending());
        table.sort_by(0);
        assert!(table.sort_ascending());
    }

    #[test]
    fn table_filter_rejects_no_matches() {
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![
            Row::new(HashMap::from([("name".to_string(), "Alice".to_string())])),
            Row::new(HashMap::from([("name".to_string(), "Bob".to_string())])),
        ];
        let mut table = Table::new(cols, rows);
        table.set_filter("zzz");
        assert_eq!(table.displayed_row_count(), 0);
        let rendered = table.render(40).unwrap();
        assert_eq!(rendered.lines.len(), 2); // header + sep only
    }

    #[test]
    fn table_cell_width_zero_in_header_and_row() {
        Theme::with(Theme::Light, || {
            let cols = vec![
                Column::new("name", "Name").width(0),
                Column::new("status", "Status").width(0),
            ];
            let rows = vec![Row::new(HashMap::from([
                ("name".to_string(), "Alice".to_string()),
                ("status".to_string(), "Active".to_string()),
            ]))];
            let table = Table::new(cols, rows);
            let rendered = table.render(40).unwrap();
            // Cells are empty because width is 0, but header/row structure still renders
            assert_eq!(rendered.lines.len(), 3); // header + sep + 1 row
            assert!(!rendered.lines[0].contains("Name"));
            assert!(!rendered.lines[2].contains("Alice"));
        });
    }
}
