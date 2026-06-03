# Data Display + App Chrome Components — Design Spec

## Goal

Add 8 new components to photon-ui to support complex TUI applications in the style of k9s, btop, and similar dashboard-style terminal apps. The components split into two categories:

- **Data Display**: Table, TreeView, Tabs, ProgressBar
- **App Chrome**: StatusBar, Header, Breadcrumbs, Sidebar

## Build Order

| Phase | Components | Rationale |
|-------|-----------|-----------|
| 1 | ProgressBar, Breadcrumbs, Header | Simplest; establishes chrome styling patterns |
| 2 | Tabs, StatusBar | Tab chrome + bottom chrome; enables framed layout demos |
| 3 | Table, TreeView, Sidebar | Heaviest components; Table and TreeView share row-rendering logic |

## Architecture

Every component:

- Implements `Component` (`render`, optional `handle_input`)
- Implements `Focusable` when keyboard-navigable
- Uses `Theme` + `Style` for theming, `Border` for frames
- Renders into `Rendered` lines; stacks in `TUI` or fits into `Layout::split`
- Follows builder API convention (`new()` + chainable setters)

## Component Specs

---

### Phase 1

#### ProgressBar

A horizontal progress indicator with an optional percentage label.

```rust
pub struct ProgressBar {
    label: String,
    value: f32,      // 0.0 ..= 1.0
    width: u16,      // total bar width in columns
    show_percent: bool,
}
```

- Renders as `[██████░░░░]  60%` or just the bar if `show_percent` is false
- Filled segment uses `theme.accent()`; empty segment uses `theme.border_default()`
- Value clamped to 0.0..=1.0
- Not focusable (pure display)

#### Breadcrumbs

A path trail with separators.

```rust
pub struct Breadcrumbs {
    items: Vec<String>,
    separator: String, // defaults to " > "
}
```

- Renders as `Home > Settings > Account`
- Last item uses `theme.text_primary()` + bold; earlier items use `theme.text_secondary()`
- Separator uses `theme.border_default()`
- Not focusable

#### Header

A top-bar title bar with optional action buttons.

```rust
pub struct Header {
    title: String,
    actions: Vec<String>, // right-aligned action labels
}
```

- Renders as `  Title                    [Action1] [Action2]  `
- Title left-aligned, actions right-aligned
- Uses `theme.bg_page()` background (inverse style: dark bg, light text) if terminal supports it; otherwise bold + border underline
- Not focusable

---

### Phase 2

#### Tabs

A horizontal tab bar with active tab indicator.

```rust
pub struct Tabs {
    items: Vec<String>,
    active: usize,
    focused: bool,
}
```

- Renders as `  [ Tab 1 ]  [ Tab 2 ]  [ Tab 3 ]  ` with active tab underlined/bold
- Active tab uses `theme.accent()`; inactive uses `theme.text_secondary()`
- Keyboard: `←`/`→` or `h`/`l` to switch tabs
- `handle_input` returns `InputResult::Handled` on tab switch
- Focusable

#### StatusBar

A multi-zone bottom bar for contextual info and key hints.

```rust
pub struct StatusBar {
    left: Vec<Segment>,
    center: Vec<Segment>,
    right: Vec<Segment>,
}

pub struct Segment {
    text: String,
    style: Style,
}
```

- Renders as `  left info          center info          right info  `
- Zones separated by padding; each segment has independent styling
- Default segments use `theme.bg_page()` inverse style
- Not focusable

---

### Phase 3

#### Table

A data table with rows, columns, sortable headers, and optional row selection.

```rust
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Row>,
    selected: usize,           // row index
    sort_column: Option<usize>,
    sort_ascending: bool,
    focused: bool,
}

pub struct Column {
    key: String,
    label: String,
    width: Option<u16>,        // None = flex
    sortable: bool,
}

pub struct Row {
    cells: HashMap<String, String>,
}
```

- Header row: labels with optional `▲`/`▼` sort indicator
- Data rows: cells truncated to column widths
- Selected row: `> ` prefix + accent background or bold
- Keyboard: `↑`/`↓` or `j`/`k` to move selection; `Enter` to select; `s` on header to sort
- Focusable
- Column widths: explicit width, or flex proportional to content

#### TreeView

A collapsible hierarchical tree.

```rust
pub struct TreeView {
    nodes: Vec<TreeNode>,
    selected: Vec<usize>, // path indices from root
    focused: bool,
}

pub struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
    expanded: bool,
}
```

- Renders with indentation: `  ▼ parent` / `    ├─ child` / `    └─ child`
- Collapsed nodes show `▶`; expanded show `▼`
- Keyboard: `↑`/`↓` or `j`/`k` to navigate; `→`/`Enter` to expand; `←` to collapse
- Focusable
- Selection is a path (e.g. `[0, 2]` = root[0].children[2])

#### Sidebar

A vertical navigation list with active item indicator.

```rust
pub struct Sidebar {
    items: Vec<SidebarItem>,
    selected: usize,
    focused: bool,
}

pub struct SidebarItem {
    label: String,
    icon: Option<String>, // single-char prefix like "⚡" or "📁"
}
```

- Renders as a vertical list with `> ` prefix on the active item
- Active item uses `theme.accent()` + bold
- Keyboard: `↑`/`↓` or `j`/`k` to navigate; `Enter` to activate
- Focusable
- Optional left border (uses `Border::LEFT`)

## Testing Strategy

- Every component gets unit tests in `src/components/<name>.rs` or `tests/<name>_tests.rs`
- Tests cover: render output shape, keyboard navigation, theme awareness, edge cases (empty data, overflow)
- Target: maintain >90% coverage for new component code
- Demo app: add a new page (page 5) showcasing Table + TreeView + Tabs in a framed layout

## Shared Utilities

Phase 3 introduces shared row-rendering logic used by both Table and TreeView:

- `truncate_to_width` (already exists) for cell content
- `draw_border` (already exists) for panel frames
- **New**: `render_row` helper for consistent row styling (selected vs unselected, padding, truncation)

## Open Questions

1. Should Table support column resizing via keyboard? (Defer to v2; fixed widths for now)
2. Should TreeView support multi-select? (Defer to v2; single-select for now)
3. Should Sidebar support nested sections? (Defer to v2; flat list for now)
