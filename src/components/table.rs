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

/// A column definition for the [`Table`] component.
pub struct Column {
    pub key: String,
    pub label: String,
    pub width: Option<u16>,
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

/// A table component with sortable columns and row selection.
///
/// Renders as a header row, a separator line, and data rows. The selected row
/// shows a `> ` prefix and is highlighted with the theme's accent color when
/// the table is focused.
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Row>,
    selected: usize,
    sort_column: Option<usize>,
    sort_ascending: bool,
    focused: bool,
}

impl Table {
    /// Create a new table with the given columns and rows.
    pub fn new(columns: Vec<Column>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            rows,
            selected: 0,
            sort_column: None,
            sort_ascending: true,
            focused: false,
        }
    }

    /// Index of the currently selected row.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected row index (clamped to valid range).
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.rows.len().saturating_sub(1));
    }

    /// Set the column used for sorting display.
    pub fn set_sort_column(&mut self, column: Option<usize>) {
        self.sort_column = column;
    }

    /// Set whether the current sort is ascending.
    pub fn set_sort_ascending(&mut self, ascending: bool) {
        self.sort_ascending = ascending;
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

        // Header row
        let header_style = Style::new().fg(theme.text_primary()).bold();
        let mut header_parts = vec![stylize("  ", &header_style)];
        for (i, col) in self.columns.iter().enumerate() {
            let mut label = col.label.clone();
            if let Some(sort_idx) = self.sort_column &&
                sort_idx == i &&
                col.sortable
            {
                let indicator = if self.sort_ascending { "▲" } else { "▼" };
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

        for (row_idx, row) in self.rows.iter().enumerate() {
            let is_selected = row_idx == self.selected;
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
            match key.code {
                | KeyCode::Down => {
                    if self.selected + 1 < self.rows.len() {
                        self.selected += 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.selected + 1 < self.rows.len() {
                        self.selected += 1;
                    }
                    InputResult::Handled
                },
                | KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    InputResult::Handled
                },
                | _ => InputResult::Ignored,
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
    use std::collections::HashMap;

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
}
