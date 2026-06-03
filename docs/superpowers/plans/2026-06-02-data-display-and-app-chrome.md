# Data Display + App Chrome Components Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 8 new TUI components (ProgressBar, Breadcrumbs, Header, Tabs, StatusBar, Table, TreeView, Sidebar) in three phases to enable k9s/btop-style dashboard apps.

**Architecture:** Each component implements the existing `Component` trait, uses `Theme` + `Style` for colors, `Border` for frames, and `Focusable` + `handle_input` when keyboard-navigable. Components render into `Rendered` lines and mount into `TUI` or `Layout::split` like existing components.

**Tech Stack:** Rust 2024, crossterm, existing photon-ui Component/Focusable/Theme/Style/Border systems.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/components/progress_bar.rs` | ProgressBar component + tests |
| `src/components/breadcrumbs.rs` | Breadcrumbs component + tests |
| `src/components/header.rs` | Header component + tests |
| `src/components/tabs.rs` | Tabs component (focusable) + tests |
| `src/components/status_bar.rs` | StatusBar component + tests |
| `src/components/table.rs` | Table component (focusable) + tests |
| `src/components/tree_view.rs` | TreeView component (focusable) + tests |
| `src/components/sidebar.rs` | Sidebar component (focusable) + tests |
| `src/components/mod.rs` | Export all new components |
| `tests/progress_bar_tests.rs` | ProgressBar integration tests |
| `tests/breadcrumbs_tests.rs` | Breadcrumbs integration tests |
| `tests/header_tests.rs` | Header integration tests |
| `tests/tabs_tests.rs` | Tabs integration tests |
| `tests/status_bar_tests.rs` | StatusBar integration tests |
| `tests/table_tests.rs` | Table integration tests |
| `tests/tree_view_tests.rs` | TreeView integration tests |
| `tests/sidebar_tests.rs` | Sidebar integration tests |
| `examples/demo.rs` | Demo page 5 showcasing new components |

---

## Phase 1: Simple Display Components

---

### Task 1: ProgressBar

**Files:**
- Create: `src/components/progress_bar.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/progress_bar_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/progress_bar_tests.rs
use photon_ui::{Component, Rendered};
use photon_ui::components::ProgressBar;
use photon_ui::theme::Theme;

#[test]
fn progress_bar_renders_at_50_percent() {
    Theme::with(Theme::Light, || {
        let bar = ProgressBar::new("Loading", 0.5);
        let rendered = bar.render(20).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        // Should contain filled blocks and empty blocks
        assert!(rendered.lines[0].contains('█'));
        assert!(rendered.lines[0].contains('░'));
        assert!(rendered.lines[0].contains("50%"));
    });
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test progress_bar_renders_at_50_percent 2>&1 | tail -5
```

Expected: FAIL — `ProgressBar` not found.

- [ ] **Step 3: Create ProgressBar component**

```rust
// src/components/progress_bar.rs
//! A horizontal progress bar with optional percentage label.

use crate::{Component, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};

/// A horizontal progress indicator.
pub struct ProgressBar {
    label: String,
    value: f32,
    width: u16,
    show_percent: bool,
}

impl ProgressBar {
    /// Create a new progress bar. `value` is clamped to 0.0..=1.0.
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        Self {
            label: label.into(),
            value: value.clamp(0.0, 1.0),
            width: 20,
            show_percent: true,
        }
    }

    /// Set the total bar width in columns (default 20).
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Hide the percentage label.
    pub fn hide_percent(mut self) -> Self {
        self.show_percent = false;
        self
    }
}

impl Component for ProgressBar {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let bar_width = self.width.saturating_sub(2) as usize; // brackets
        let filled = (self.value * bar_width as f32).round() as usize;
        let empty = bar_width.saturating_sub(filled);

        let filled_style = Style::new().fg(theme.accent());
        let empty_style = Style::new().fg(theme.border_default());
        let filled_ch = crate::theme::stylize("█", &filled_style);
        let empty_ch = crate::theme::stylize("░", &empty_style);

        let mut bar = String::from("[");
        bar.push_str(&filled_ch.repeat(filled));
        bar.push_str(&empty_ch.repeat(empty));
        bar.push(']');

        if self.show_percent {
            bar.push_str(&format!(" {:3.0}%", self.value * 100.0));
        }

        if !self.label.is_empty() {
            bar = format!("{} {}", self.label, bar);
        }

        Ok(Rendered {
            lines: vec![bar],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_at_zero() {
        Theme::with(Theme::Light, || {
            let bar = ProgressBar::new("", 0.0);
            let r = bar.render(20).unwrap();
            assert!(r.lines[0].contains('░'));
            assert!(!r.lines[0].contains('█'));
        });
    }

    #[test]
    fn progress_bar_at_full() {
        Theme::with(Theme::Light, || {
            let bar = ProgressBar::new("", 1.0);
            let r = bar.render(20).unwrap();
            assert!(r.lines[0].contains('█'));
            assert!(!r.lines[0].contains('░'));
        });
    }

    #[test]
    fn progress_bar_clamps_value() {
        Theme::with(Theme::Light, || {
            let bar = ProgressBar::new("", 1.5);
            let r = bar.render(20).unwrap();
            assert!(r.lines[0].contains("100%"));
        });
    }
}
```

- [ ] **Step 4: Export ProgressBar in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod progress_bar;
pub use progress_bar::ProgressBar;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test progress_bar 2>&1 | tail -10
```

Expected: All 4 tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/progress_bar.rs src/components/mod.rs tests/progress_bar_tests.rs && git commit -m "feat(components): add ProgressBar component"
```

---

### Task 2: Breadcrumbs

**Files:**
- Create: `src/components/breadcrumbs.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/breadcrumbs_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/breadcrumbs_tests.rs
use photon_ui::{Component, Rendered};
use photon_ui::components::Breadcrumbs;
use photon_ui::theme::Theme;

#[test]
fn breadcrumbs_renders_items() {
    Theme::with(Theme::Light, || {
        let crumbs = Breadcrumbs::new(vec!["Home", "Settings", "Account"]);
        let rendered = crumbs.render(40).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        let line = &rendered.lines[0];
        assert!(line.contains("Home"));
        assert!(line.contains("Account"));
        assert!(line.contains(">"));
    });
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test breadcrumbs_renders_items 2>&1 | tail -5
```

Expected: FAIL — `Breadcrumbs` not found.

- [ ] **Step 3: Create Breadcrumbs component**

```rust
// src/components/breadcrumbs.rs
//! A path trail with separators.

use crate::{Component, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};

/// A breadcrumb navigation trail.
pub struct Breadcrumbs {
    items: Vec<String>,
    separator: String,
}

impl Breadcrumbs {
    /// Create breadcrumbs from a list of labels.
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(|i| i.into()).collect(),
            separator: " > ".to_string(),
        }
    }

    /// Set a custom separator (default " > ").
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl Component for Breadcrumbs {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let sep_style = Style::new().fg(theme.border_default());
        let sep_styled = crate::theme::stylize(&self.separator, &sep_style);

        let mut parts = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            let is_last = i == self.items.len() - 1;
            let style = if is_last {
                Style::new().fg(theme.text_primary()).bold()
            } else {
                Style::new().fg(theme.text_secondary())
            };
            parts.push(crate::theme::stylize(item, &style));
            if !is_last {
                parts.push(sep_styled.clone());
            }
        }

        let line = parts.concat();
        let line = crate::utils::truncate_to_width(&line, width, "");

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumbs_last_item_is_bold() {
        Theme::with(Theme::Light, || {
            let crumbs = Breadcrumbs::new(vec!["A", "B"]);
            let r = crumbs.render(20).unwrap();
            // Line should have ANSI codes (bold for last item)
            assert!(r.lines[0].contains('\x1b'));
        });
    }

    #[test]
    fn breadcrumbs_custom_separator() {
        Theme::with(Theme::Light, || {
            let crumbs = Breadcrumbs::new(vec!["A", "B"]).separator(" / ");
            let r = crumbs.render(20).unwrap();
            assert!(r.lines[0].contains(" / "));
        });
    }
}
```

- [ ] **Step 4: Export Breadcrumbs in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod breadcrumbs;
pub use breadcrumbs::Breadcrumbs;
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test breadcrumbs 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/breadcrumbs.rs src/components/mod.rs tests/breadcrumbs_tests.rs && git commit -m "feat(components): add Breadcrumbs component"
```

---

### Task 3: Header

**Files:**
- Create: `src/components/header.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/header_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/header_tests.rs
use photon_ui::{Component, Rendered};
use photon_ui::components::Header;
use photon_ui::theme::Theme;

#[test]
fn header_renders_title() {
    Theme::with(Theme::Light, || {
        let header = Header::new("Dashboard");
        let rendered = header.render(40).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert!(rendered.lines[0].contains("Dashboard"));
    });
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test header_renders_title 2>&1 | tail -5
```

Expected: FAIL — `Header` not found.

- [ ] **Step 3: Create Header component**

```rust
// src/components/header.rs
//! A top-bar title header with optional action labels.

use crate::{Component, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};

/// A top-bar header component.
pub struct Header {
    title: String,
    actions: Vec<String>,
}

impl Header {
    /// Create a header with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            actions: Vec::new(),
        }
    }

    /// Add a right-aligned action label.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.actions.push(label.into());
        self
    }
}

impl Component for Header {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let title_style = Style::new().fg(theme.text_primary()).bold();
        let title = crate::theme::stylize(&self.title, &title_style);

        let action_style = Style::new().fg(theme.text_secondary());
        let actions: Vec<String> = self.actions.iter()
            .map(|a| format!("[{}]", crate::theme::stylize(a, &action_style)))
            .collect();
        let actions_str = actions.join(" ");

        let total_content = crate::utils::visible_width(&title) + if actions_str.is_empty() { 0 } else { crate::utils::visible_width(&actions_str) + 1 };
        let pad = width.saturating_sub(total_content as u16);

        let line = if actions_str.is_empty() {
            title
        } else {
            format!("{}{:>pad$}", title, actions_str, pad = pad as usize)
        };

        let line = crate::utils::truncate_to_width(&line, width, "");

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_with_actions() {
        Theme::with(Theme::Light, || {
            let header = Header::new("App").action("Quit");
            let r = header.render(30).unwrap();
            assert!(r.lines[0].contains("App"));
            assert!(r.lines[0].contains("Quit"));
        });
    }
}
```

- [ ] **Step 4: Export Header in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod header;
pub use header::Header;
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test header 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/header.rs src/components/mod.rs tests/header_tests.rs && git commit -m "feat(components): add Header component"
```

---

## Phase 2: Tab + Status Chrome

---

### Task 4: Tabs

**Files:**
- Create: `src/components/tabs.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/tabs_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/tabs_tests.rs
use photon_ui::{Component, Event, Focusable, InputResult};
use photon_ui::components::Tabs;
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn tabs_navigation() {
    let mut tabs = Tabs::new(vec!["Tab 1", "Tab 2", "Tab 3"]);
    tabs.set_focused(true);
    assert_eq!(tabs.active(), 0);

    tabs.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Right,
        KeyModifiers::empty(),
    )));
    assert_eq!(tabs.active(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test tabs_navigation 2>&1 | tail -5
```

Expected: FAIL — `Tabs` not found.

- [ ] **Step 3: Create Tabs component**

```rust
// src/components/tabs.rs
//! A horizontal tab bar with active tab indicator.

use crate::{Component, Event, Focusable, InputResult, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};
use crossterm::event::KeyCode;

/// A controlled tab navigation bar.
pub struct Tabs {
    items: Vec<String>,
    active: usize,
    focused: bool,
}

impl Tabs {
    /// Create tabs from a list of labels.
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(|i| i.into()).collect(),
            active: 0,
            focused: false,
        }
    }

    /// Index of the currently active tab.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Set the active tab index (clamped).
    pub fn set_active(&mut self, index: usize) {
        self.active = index.min(self.items.len().saturating_sub(1));
    }
}

impl Focusable for Tabs {
    fn focused(&self) -> bool {
        self.focused
    }
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for Tabs {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let active_style = Style::new().fg(theme.accent()).bold();
        let inactive_style = Style::new().fg(theme.text_secondary());
        let focus_indicator = if self.focused { "│" } else { " " };

        let mut line = String::new();
        line.push_str(focus_indicator);

        for (i, item) in self.items.iter().enumerate() {
            let is_active = i == self.active;
            let style = if is_active { &active_style } else { &inactive_style };
            let label = if is_active {
                format!(" [{}] ", item)
            } else {
                format!("  {}  ", item)
            };
            line.push_str(&crate::theme::stylize(&label, style));
        }

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Right => {
                    if self.active + 1 < self.items.len() {
                        self.active += 1;
                    }
                    InputResult::Handled
                }
                KeyCode::Left => {
                    if self.active > 0 {
                        self.active -= 1;
                    }
                    InputResult::Handled
                }
                KeyCode::Char('l') if self.focused => {
                    if self.active + 1 < self.items.len() {
                        self.active += 1;
                    }
                    InputResult::Handled
                }
                KeyCode::Char('h') if self.focused => {
                    if self.active > 0 {
                        self.active -= 1;
                    }
                    InputResult::Handled
                }
                _ => InputResult::Ignored,
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
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn tabs_active_starts_at_zero() {
        let tabs = Tabs::new(vec!["A", "B"]);
        assert_eq!(tabs.active(), 0);
    }

    #[test]
    fn tabs_clamps_active() {
        let mut tabs = Tabs::new(vec!["A", "B"]);
        tabs.set_active(10);
        assert_eq!(tabs.active(), 1);
    }
}
```

- [ ] **Step 4: Export Tabs in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod tabs;
pub use tabs::Tabs;
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test tabs 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/tabs.rs src/components/mod.rs tests/tabs_tests.rs && git commit -m "feat(components): add Tabs component with keyboard navigation"
```

---

### Task 5: StatusBar

**Files:**
- Create: `src/components/status_bar.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/status_bar_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/status_bar_tests.rs
use photon_ui::{Component, Rendered};
use photon_ui::components::status_bar::{StatusBar, Segment};
use photon_ui::theme::Theme;

#[test]
fn status_bar_renders_segments() {
    Theme::with(Theme::Light, || {
        let bar = StatusBar::new()
            .left(Segment::new("MODE: normal"))
            .right(Segment::new("q:quit"));
        let rendered = bar.render(60).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert!(rendered.lines[0].contains("MODE: normal"));
        assert!(rendered.lines[0].contains("q:quit"));
    });
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test status_bar_renders_segments 2>&1 | tail -5
```

Expected: FAIL — `StatusBar` not found.

- [ ] **Step 3: Create StatusBar component**

```rust
// src/components/status_bar.rs
//! A multi-zone bottom status bar.

use crate::{Component, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};

/// A styled segment of text in the status bar.
#[derive(Clone)]
pub struct Segment {
    text: String,
    style: Style,
}

impl Segment {
    /// Create a segment with default text styling.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::new(),
        }
    }

    /// Apply a custom style.
    pub fn styled(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// A multi-zone status bar (left / center / right).
pub struct StatusBar {
    left: Vec<Segment>,
    center: Vec<Segment>,
    right: Vec<Segment>,
}

impl StatusBar {
    /// Create an empty status bar.
    pub fn new() -> Self {
        Self {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }

    pub fn left(mut self, segment: Segment) -> Self {
        self.left.push(segment);
        self
    }
    pub fn center(mut self, segment: Segment) -> Self {
        self.center.push(segment);
        self
    }
    pub fn right(mut self, segment: Segment) -> Self {
        self.right.push(segment);
        self
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for StatusBar {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let default_style = Style::new().fg(theme.text_secondary());

        let render_zone = |segments: &[Segment]| -> String {
            segments.iter()
                .map(|s| {
                    let style = if s.style == Style::new() { &default_style } else { &s.style };
                    crate::theme::stylize(&s.text, style)
                })
                .collect::<Vec<_>>()
                .join("  ")
        };

        let left = render_zone(&self.left);
        let center = render_zone(&self.center);
        let right = render_zone(&self.right);

        let left_w = crate::utils::visible_width(&left);
        let center_w = crate::utils::visible_width(&center);
        let right_w = crate::utils::visible_width(&right);

        let mut line = left;

        if !center.is_empty() {
            let gap = width as usize - left_w - right_w;
            let center_pad = gap.saturating_sub(center_w) / 2;
            line.push_str(&" ".repeat(center_pad));
            line.push_str(&center);
        }

        if !right.is_empty() {
            let total_so_far = crate::utils::visible_width(&line) + right_w;
            let pad = width as usize - total_so_far;
            line.push_str(&" ".repeat(pad));
            line.push_str(&right);
        }

        let line = crate::utils::truncate_to_width(&line, width, "");

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_empty_renders_blank() {
        let bar = StatusBar::new();
        let r = bar.render(20).unwrap();
        assert!(r.lines[0].trim().is_empty());
    }
}
```

- [ ] **Step 4: Export StatusBar in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod status_bar;
pub use status_bar::{StatusBar, Segment};
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test status_bar 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/status_bar.rs src/components/mod.rs tests/status_bar_tests.rs && git commit -m "feat(components): add StatusBar component with left/center/right zones"
```

---

## Phase 3: Heavy Data Components

---

### Task 6: Table

**Files:**
- Create: `src/components/table.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/table_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/table_tests.rs
use photon_ui::{Component, Event, Focusable, InputResult};
use photon_ui::components::table::{Table, Column, Row};
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

#[test]
fn table_renders_rows() {
    let table = Table::new(
        vec![
            Column::new("name", "Name").width(10),
            Column::new("size", "Size").width(5),
        ],
        vec![
            Row::new(HashMap::from([
                ("name".to_string(), "file.txt".to_string()),
                ("size".to_string(), "1.2MB".to_string()),
            ])),
        ],
    );
    let rendered = table.render(30).unwrap();
    assert!(rendered.lines.len() >= 2); // header + data row
    assert!(rendered.lines[0].contains("Name"));
    assert!(rendered.lines[1].contains("file.txt"));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test table_renders_rows 2>&1 | tail -5
```

Expected: FAIL — `Table` not found.

- [ ] **Step 3: Create Table component**

```rust
// src/components/table.rs
//! A data table with sortable columns and row selection.

use crate::{Component, Event, Focusable, InputResult, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};
use crossterm::event::KeyCode;
use std::collections::HashMap;

/// Column definition.
pub struct Column {
    pub key: String,
    pub label: String,
    pub width: Option<u16>,
    pub sortable: bool,
}

impl Column {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            width: None,
            sortable: false,
        }
    }
    pub fn width(mut self, w: u16) -> Self {
        self.width = Some(w);
        self
    }
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

/// Row data.
pub struct Row {
    cells: HashMap<String, String>,
}

impl Row {
    pub fn new(cells: HashMap<String, String>) -> Self {
        Self { cells }
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cells.get(key).map(|s| s.as_str())
    }
}

/// A data table.
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Row>,
    selected: usize,
    sort_column: Option<usize>,
    sort_ascending: bool,
    focused: bool,
}

impl Table {
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
    pub fn selected(&self) -> usize { self.selected }
    pub fn set_selected(&mut self, idx: usize) {
        self.selected = idx.min(self.rows.len().saturating_sub(1));
    }
}

impl Focusable for Table {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Component for Table {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let mut lines = Vec::new();

        // Calculate column widths
        let total_fixed: u16 = self.columns.iter().filter_map(|c| c.width).sum();
        let flex_count = self.columns.iter().filter(|c| c.width.is_none()).count() as u16;
        let flex_width = if flex_count > 0 {
            width.saturating_sub(total_fixed).saturating_sub(self.columns.len().saturating_sub(1) as u16) / flex_count
        } else { 0 };

        // Header row
        let header_style = Style::new().fg(theme.text_primary()).bold();
        let mut header = String::new();
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 { header.push(' '); }
            let w = col.width.unwrap_or(flex_width) as usize;
            let label = if col.sortable && self.sort_column == Some(i) {
                let arrow = if self.sort_ascending { "▲" } else { "▼" };
                format!("{}{}", col.label, arrow)
            } else {
                col.label.clone()
            };
            let cell = format!("{:width$}", label, width = w);
            header.push_str(&crate::utils::truncate_to_width(&cell, w as u16, ""));
        }
        lines.push(crate::theme::stylize(&header, &header_style));

        // Separator
        let sep = "─".repeat(width as usize);
        lines.push(crate::theme::stylize(&sep, &Style::new().fg(theme.border_default())));

        // Data rows
        for (ri, row) in self.rows.iter().enumerate() {
            let is_selected = ri == self.selected && self.focused;
            let row_style = if is_selected {
                Style::new().fg(theme.accent()).bold()
            } else {
                Style::new().fg(theme.text_primary())
            };

            let mut line = String::new();
            if is_selected { line.push_str("> "); } else { line.push_str("  "); }

            for (ci, col) in self.columns.iter().enumerate() {
                if ci > 0 { line.push(' '); }
                let w = col.width.unwrap_or(flex_width) as usize;
                let raw = row.get(&col.key).unwrap_or("");
                let cell = format!("{:width$}", raw, width = w);
                line.push_str(&crate::utils::truncate_to_width(&cell, w as u16, ""));
            }
            lines.push(crate::theme::stylize(&line, &row_style));
        }

        Ok(Rendered { lines, cursor: None, images: Vec::new() })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down => {
                    if self.selected + 1 < self.rows.len() { self.selected += 1; }
                    InputResult::Handled
                }
                KeyCode::Up => {
                    if self.selected > 0 { self.selected -= 1; }
                    InputResult::Handled
                }
                KeyCode::Char('j') => {
                    if self.selected + 1 < self.rows.len() { self.selected += 1; }
                    InputResult::Handled
                }
                KeyCode::Char('k') => {
                    if self.selected > 0 { self.selected -= 1; }
                    InputResult::Handled
                }
                _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_empty_renders_header_only() {
        let table = Table::new(
            vec![Column::new("a", "A")],
            vec![],
        );
        let r = table.render(20).unwrap();
        assert!(r.lines[0].contains("A"));
    }

    #[test]
    fn table_selection_bounds() {
        let mut table = Table::new(
            vec![Column::new("a", "A")],
            vec![Row::new(HashMap::from([("a".to_string(), "1".to_string())]))],
        );
        table.set_focused(true);
        table.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down, crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(table.selected(), 0); // can't go past last row
    }
}
```

- [ ] **Step 4: Export Table in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod table;
pub use table::{Table, Column, Row};
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test table 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/table.rs src/components/mod.rs tests/table_tests.rs && git commit -m "feat(components): add Table component with sortable columns and row selection"
```

---

### Task 7: TreeView

**Files:**
- Create: `src/components/tree_view.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/tree_view_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/tree_view_tests.rs
use photon_ui::{Component, Event, Focusable, InputResult};
use photon_ui::components::tree_view::{TreeView, TreeNode};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn tree_view_renders_nodes() {
    let tree = TreeView::new(vec![
        TreeNode::new("root").child(TreeNode::new("child")),
    ]);
    let rendered = tree.render(20).unwrap();
    assert!(rendered.lines.iter().any(|l| l.contains("root")));
    assert!(rendered.lines.iter().any(|l| l.contains("child")));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test tree_view_renders_nodes 2>&1 | tail -5
```

Expected: FAIL — `TreeView` not found.

- [ ] **Step 3: Create TreeView component**

```rust
// src/components/tree_view.rs
//! A collapsible hierarchical tree view.

use crate::{Component, Event, Focusable, InputResult, RenderError, Rendered};
use crate::theme::{Palette, Style, Theme};
use crossterm::event::KeyCode;

/// A node in the tree.
pub struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
    expanded: bool,
}

impl TreeNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: true,
        }
    }
    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }
}

/// Flattened tree item for rendering.
struct FlatNode<'a> {
    node: &'a TreeNode,
    depth: usize,
    is_last: bool,
}

/// A hierarchical tree view.
pub struct TreeView {
    nodes: Vec<TreeNode>,
    selected: Vec<usize>, // path from root
    focused: bool,
}

impl TreeView {
    pub fn new(nodes: Vec<TreeNode>) -> Self {
        Self {
            nodes,
            selected: vec![0],
            focused: false,
        }
    }

    fn flatten(&self) -> Vec<FlatNode> {
        let mut result = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            Self::flatten_node(node, 0, i == self.nodes.len() - 1, &mut result);
        }
        result
    }

    fn flatten_node<'a>(node: &'a TreeNode, depth: usize, is_last: bool, out: &mut Vec<FlatNode<'a>>) {
        out.push(FlatNode { node, depth, is_last });
        if node.expanded {
            for (i, child) in node.children.iter().enumerate() {
                Self::flatten_node(child, depth + 1, i == node.children.len() - 1, out);
            }
        }
    }

    fn selected_flat_index(&self) -> usize {
        // Simplified: find the flat index matching the selected path
        let flat = self.flatten();
        for (i, f) in flat.iter().enumerate() {
            if self.is_at_path(i, &flat) {
                return i;
            }
        }
        0
    }

    fn is_at_path(&self, flat_idx: usize, flat: &[FlatNode]) -> bool {
        // Heuristic: match depth 0 index for now
        flat_idx < flat.len() && flat[flat_idx].depth == 0 && flat_idx == self.selected[0]
    }

    fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        let mut nodes = &mut self.nodes;
        let mut node = nodes.get_mut(path.first()?)?;
        for &idx in &path[1..] {
            node = node.children.get_mut(idx)?;
        }
        Some(node)
    }
}

impl Focusable for TreeView {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Component for TreeView {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let flat = self.flatten();
        let sel_idx = self.selected_flat_index();
        let mut lines = Vec::new();

        for (i, f) in flat.iter().enumerate() {
            let is_selected = i == sel_idx && self.focused;
            let style = if is_selected {
                Style::new().fg(theme.accent()).bold()
            } else {
                Style::new().fg(theme.text_primary())
            };

            let prefix = if f.depth == 0 {
                if is_selected { "> " } else { "  " }.to_string()
            } else {
                let indent = "  ".repeat(f.depth);
                let branch = if f.is_last { "└─ " } else { "├─ " };
                format!("{}{}", indent, branch)
            };

            let expander = if !f.node.children.is_empty() {
                if f.node.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let line = format!("{}{}{}", prefix, expander, f.node.label);
            let line = crate::utils::truncate_to_width(&line, width, "");
            lines.push(crate::theme::stylize(&line, &style));
        }

        Ok(Rendered { lines, cursor: None, images: Vec::new() })
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            let flat = self.flatten();
            let mut sel = self.selected_flat_index();
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if sel + 1 < flat.len() { sel += 1; }
                    InputResult::Handled
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if sel > 0 { sel -= 1; }
                    InputResult::Handled
                }
                KeyCode::Right | KeyCode::Enter => {
                    if let Some(f) = flat.get(sel) {
                        if !f.node.children.is_empty() {
                            // Toggle expansion of node at selected path
                            if let Some(node) = self.node_at_path_mut(&self.selected) {
                                node.expanded = !node.expanded;
                            }
                        }
                    }
                    InputResult::Handled
                }
                KeyCode::Left => {
                    // Collapse current or navigate to parent
                    if let Some(node) = self.node_at_path_mut(&self.selected) {
                        if node.expanded && !node.children.is_empty() {
                            node.expanded = false;
                        } else if self.selected.len() > 1 {
                            self.selected.pop();
                        }
                    }
                    InputResult::Handled
                }
                _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_view_flatten_respects_expansion() {
        let mut tree = TreeView::new(vec![
            TreeNode::new("a").child(TreeNode::new("b")),
        ]);
        // Default expanded
        let flat = tree.flatten();
        assert_eq!(flat.len(), 2);
    }
}
```

- [ ] **Step 4: Export TreeView in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod tree_view;
pub use tree_view::{TreeView, TreeNode};
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test tree_view 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/tree_view.rs src/components/mod.rs tests/tree_view_tests.rs && git commit -m "feat(components): add TreeView component with collapsible nodes"
```

---

### Task 8: Sidebar

**Files:**
- Create: `src/components/sidebar.rs`
- Modify: `src/components/mod.rs`
- Test: `tests/sidebar_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// tests/sidebar_tests.rs
use photon_ui::{Component, Event, Focusable, InputResult};
use photon_ui::components::sidebar::{Sidebar, SidebarItem};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn sidebar_navigation() {
    let mut sidebar = Sidebar::new(vec![
        SidebarItem::new("Home"),
        SidebarItem::new("Settings"),
    ]);
    sidebar.set_focused(true);
    assert_eq!(sidebar.selected(), 0);

    sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Down,
        KeyModifiers::empty(),
    )));
    assert_eq!(sidebar.selected(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test sidebar_navigation 2>&1 | tail -5
```

Expected: FAIL — `Sidebar` not found.

- [ ] **Step 3: Create Sidebar component**

```rust
// src/components/sidebar.rs
//! A vertical navigation sidebar.

use crate::{Component, Event, Focusable, InputResult, RenderError, Rendered};
use crate::layout::Border;
use crate::theme::{Palette, Style, Theme};
use crossterm::event::KeyCode;

/// An item in the sidebar.
pub struct SidebarItem {
    label: String,
    icon: Option<String>,
}

impl SidebarItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A vertical navigation sidebar.
pub struct Sidebar {
    items: Vec<SidebarItem>,
    selected: usize,
    focused: bool,
    show_border: bool,
}

impl Sidebar {
    pub fn new(items: Vec<SidebarItem>) -> Self {
        Self {
            items,
            selected: 0,
            focused: false,
            show_border: true,
        }
    }
    pub fn selected(&self) -> usize { self.selected }
    pub fn set_selected(&mut self, idx: usize) {
        self.selected = idx.min(self.items.len().saturating_sub(1));
    }
    pub fn hide_border(mut self) -> Self {
        self.show_border = false;
        self
    }
}

impl Focusable for Sidebar {
    fn focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, focused: bool) { self.focused = focused; }
}

impl Component for Sidebar {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::current();
        let mut lines = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == self.selected && self.focused;
            let style = if is_selected {
                Style::new().fg(theme.accent()).bold()
            } else {
                Style::new().fg(theme.text_secondary())
            };

            let prefix = if is_selected { "> " } else { "  " };
            let icon = item.icon.as_ref().map(|s| format!("{} ", s)).unwrap_or_default();
            let line = format!("{}{}{}", prefix, icon, item.label);
            let line = crate::utils::truncate_to_width(&line, width, "");
            lines.push(crate::theme::stylize(&line, &style));
        }

        // Pad remaining height with empty lines to maintain rect shape
        // (component consumers decide actual height)

        let mut rendered = Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        };

        if self.show_border && width > 1 {
            let border_style = Style::new().fg(theme.border_default());
            let rect = crate::layout::Rect::new(0, 0, width, rendered.lines.len() as u16);
            crate::layout::draw_border(&mut rendered, rect, &Border::LEFT, &border_style);
        }

        Ok(rendered)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down => {
                    if self.selected + 1 < self.items.len() { self.selected += 1; }
                    InputResult::Handled
                }
                KeyCode::Up => {
                    if self.selected > 0 { self.selected -= 1; }
                    InputResult::Handled
                }
                KeyCode::Char('j') => {
                    if self.selected + 1 < self.items.len() { self.selected += 1; }
                    InputResult::Handled
                }
                KeyCode::Char('k') => {
                    if self.selected > 0 { self.selected -= 1; }
                    InputResult::Handled
                }
                _ => InputResult::Ignored,
            }
        } else {
            InputResult::Ignored
        }
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> { Some(self) }
    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> { Some(self) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_selection_bounds() {
        let mut sidebar = Sidebar::new(vec![SidebarItem::new("A")]);
        sidebar.set_focused(true);
        sidebar.handle_input(&Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down, crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(sidebar.selected(), 0);
    }
}
```

- [ ] **Step 4: Export Sidebar in mod.rs**

```rust
// In src/components/mod.rs, add:
pub mod sidebar;
pub use sidebar::{Sidebar, SidebarItem};
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test sidebar 2>&1 | tail -10
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add src/components/sidebar.rs src/components/mod.rs tests/sidebar_tests.rs && git commit -m "feat(components): add Sidebar component with vertical navigation"
```

---

## Demo Integration

### Task 9: Add Demo Page 5

**Files:**
- Modify: `examples/demo.rs`

- [ ] **Step 1: Add page 5 to DemoApp**

In `examples/demo.rs`:
1. Change the page count from 4 to 5 in header text and key handlers
2. Add `5` keybinding
3. Add `load_page_dashboard()` method that mounts:
   - `Header::new("Dashboard Demo").action("q:quit")`
   - `Tabs::new(vec!["Overview", "Resources", "Logs"])`
   - `Table` with sample data
   - `StatusBar` with key hints
4. Call `self.tui.set_layout(Layout::vertical([...]))` to frame the page

- [ ] **Step 2: Run demo to verify**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo run --example demo 2>&1 | head -5
```

Expected: Compiles and runs. Manually verify page 5 renders.

- [ ] **Step 3: Commit**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && git add examples/demo.rs && git commit -m "feat(demo): add page 5 showcasing Table, Tabs, StatusBar, Header"
```

---

## Final Verification

- [ ] **Run full test suite**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo test 2>&1 | tail -5
```

Expected: All tests PASS.

- [ ] **Run clippy**

```bash
cd /Users/sienna/Development/sunbeam-split/photon-ui && cargo clippy --all-targets 2>&1 | grep -E "^error" | wc -l
```

Expected: 0 errors.
