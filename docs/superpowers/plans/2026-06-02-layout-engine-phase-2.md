# Layout Engine (Phase 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Cassowary-based layout engine (`Layout::split`) that divides a `Rect` into sub-rects according to constraints, enabling components to render into assigned rects via a new `render_rect` API.

**Architecture:** Port ratatui's constraint solver approach using the `kasuari` crate. A `Layout` struct holds direction, constraints, margin, flex, and spacing. `Layout::split(area)` creates a fresh `kasuari::Solver`, adds constraints with specific strengths, fetches changes, and converts solver variables back into `Vec<Rect>`. The `Component` trait gains `render_rect(&self, rect: Rect)` with a default impl calling `render(rect.width)`, preserving backward compatibility. `TUI` gains an optional `layout` field for top-level splitting. A new `Container` component enables nested layouts.

**Tech Stack:** Rust 2024 edition, `kasuari = "0.4"` (Cassowary constraint solver), standard library.

---

## File Structure

**New files:**
- `src/layout/layout.rs` — `Layout` struct, builder API, `split`/`areas`, solver orchestration
- `src/layout/strengths.rs` — Solver strength constants (private module)
- `src/components/container.rs` — `Container` component for nested layouts
- `tests/layout_engine_tests.rs` — Integration tests for `Layout::split`
- `tests/layout_engine_proptests.rs` — Property-based tests for layout invariants

**Modified files:**
- `Cargo.toml` — add `kasuari` dependency
- `src/layout/mod.rs` — re-export `Layout`
- `src/lib.rs` — expose `Container` in `components` module (or add `pub use`)
- `src/lib.rs` — add `render_rect` to `Component` trait
- `src/tui.rs` — add optional `layout` field, use `blit_into_rect` in `render_frame`
- `examples/demo.rs` — add page 6 demonstrating `Layout::split`

---

### Task 1: Add `kasuari` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add dependency**

Add to `[dependencies]` in `Cargo.toml`:

```toml
kasuari = "0.4"
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully (no errors from new dependency)

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "build: add kasuari Cassowary solver dependency"
```

---

### Task 2: Add `render_rect` to `Component` trait

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Add `render_rect` method to `Component` trait**

Add the following method to the `Component` trait in `src/lib.rs`, after the existing `render` method:

```rust
    /// Render this component into a specific rectangular area.
    ///
    /// The default implementation delegates to [`render`](Component::render)
    /// with the rect's width, ignoring height bounds. Components that want
    /// to be layout-aware (e.g. clip to height, scroll, center vertically)
    /// should override this.
    fn render_rect(&self, rect: crate::layout::Rect) -> Result<Rendered, RenderError> {
        self.render(rect.width)
    }
```

The trait should now read:

```rust
pub trait Component {
    fn render(&self, width: u16) -> Result<Rendered, RenderError>;

    fn render_rect(&self, rect: crate::layout::Rect) -> Result<Rendered, RenderError> {
        self.render(rect.width)
    }

    fn handle_input(&mut self, _event: &events::Event) -> InputResult {
        InputResult::Ignored
    }

    fn wants_key_release(&self) -> bool {
        false
    }

    fn as_focusable(&self) -> Option<&dyn Focusable> {
        None
    }

    fn as_focusable_mut(&mut self) -> Option<&mut dyn Focusable> {
        None
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test`
Expected: PASS (all existing tests pass; no breaking changes)

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat(component): add render_rect with backward-compatible default"
```

---

### Task 3: Add solver strength constants

**Files:**
- Create: `src/layout/strengths.rs`

- [ ] **Step 1: Write the file**

```rust
//! Cassowary solver strength constants for layout constraints.
//!
//! Higher strength = higher priority. The solver tries to satisfy stronger
//! constraints before weaker ones. Strengths are expressed as floating-point
//! values for kasuari.

use kasuari::strength::*;

/// REQUIRED strength — must be satisfied absolutely.
pub const REQUIRED: f64 = REQUIRED;

/// Strong strength — area bounds and variable ordering.
pub const STRONG: f64 = STRONG;

/// Medium strength — size equality constraints (Length, Percentage, Ratio).
pub const MEDIUM: f64 = MEDIUM;

/// Weak strength — grow constraints (Fill, flex distribution).
pub const WEAK: f64 = WEAK;

/// Spacer size equality (highest non-required).
pub const SPACER_SIZE_EQ: f64 = REQUIRED * 0.999;

/// Minimum size ≥ constraint.
pub const MIN_SIZE_GE: f64 = STRONG * 1.5;

/// Maximum size ≤ constraint.
pub const MAX_SIZE_LE: f64 = STRONG * 1.4;

/// Exact length.
pub const LENGTH_SIZE_EQ: f64 = MEDIUM * 1.5;

/// Percentage of parent area.
pub const PERCENTAGE_SIZE_EQ: f64 = MEDIUM * 1.4;

/// Ratio of parent area.
pub const RATIO_SIZE_EQ: f64 = MEDIUM * 1.3;

/// Maximum size equality (for Max constraint).
pub const MAX_SIZE_EQ: f64 = MEDIUM * 1.2;

/// Minimum size equality (for Min constraint).
pub const MIN_SIZE_EQ: f64 = MEDIUM * 1.1;

/// Fill grow — lowest constraint priority.
pub const FILL_GROW: f64 = WEAK * 1.5;

/// Generic grow.
pub const GROW: f64 = WEAK * 1.4;

/// Space grow — flex distribution.
pub const SPACE_GROW: f64 = WEAK * 1.3;

/// All segments grow equally.
pub const ALL_SEGMENT_GROW: f64 = WEAK * 1.0;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/layout/strengths.rs
git commit -m "feat(layout): add Cassowary solver strength constants"
```

---

### Task 4: Add `Layout` struct and `split` method

**Files:**
- Create: `src/layout/layout.rs`

- [ ] **Step 1: Write the file**

```rust
use std::hash::{Hash, Hasher};

use kasuari::{Solver, Variable};

use super::{Constraint, Direction, Flex, Margin, Rect, Size, Spacing};
use super::strengths;

const FLOAT_PRECISION_MULTIPLIER: f64 = 100.0;

/// A layout configuration that splits a [`Rect`] into sub-rects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: Margin,
    flex: Flex,
    spacing: Spacing,
}

impl Layout {
    /// Create a new layout with the given direction and constraints.
    pub fn new<I>(direction: Direction, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self {
            direction,
            constraints: constraints.into_iter().map(Into::into).collect(),
            margin: Margin::new(0, 0),
            flex: Flex::default(),
            spacing: Spacing::default(),
        }
    }

    /// Shorthand for a vertical layout.
    pub fn vertical<I>(constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self::new(Direction::Vertical, constraints)
    }

    /// Shorthand for a horizontal layout.
    pub fn horizontal<I>(constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self::new(Direction::Horizontal, constraints)
    }

    /// Set the direction.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set the constraints.
    pub fn constraints<I>(mut self, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.constraints = constraints.into_iter().map(Into::into).collect();
        self
    }

    /// Set uniform margin.
    pub fn margin(mut self, margin: u16) -> Self {
        self.margin = Margin::new(margin, margin);
        self
    }

    /// Set horizontal margin only.
    pub fn horizontal_margin(mut self, margin: u16) -> Self {
        self.margin.horizontal = margin;
        self
    }

    /// Set vertical margin only.
    pub fn vertical_margin(mut self, margin: u16) -> Self {
        self.margin.vertical = margin;
        self
    }

    /// Set flex distribution mode.
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Set spacing between segments.
    pub fn spacing<T: Into<Spacing>>(mut self, spacing: T) -> Self {
        self.spacing = spacing.into();
        self
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::default(),
            constraints: Vec::new(),
            margin: Margin::default(),
            flex: Flex::default(),
            spacing: Spacing::default(),
        }
    }
}

impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.direction.hash(state);
        self.constraints.hash(state);
        self.margin.hash(state);
        self.flex.hash(state);
        self.spacing.hash(state);
    }
}

impl Layout {
    /// Split the given area into sub-rects according to this layout's constraints.
    ///
    /// Returns a `Vec<Rect>` with one element per constraint.
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        self.try_split(area).unwrap_or_default()
    }

    /// Split the given area, returning an empty vec on solver failure.
    fn try_split(&self, area: Rect) -> Option<Vec<Rect>> {
        let inner = area.inner(self.margin);
        if inner.is_empty() {
            return Some(vec![Rect::ZERO; self.constraints.len()]);
        }

        let mut solver = Solver::new();
        let segment_count = self.constraints.len();
        let spacer_count = segment_count.saturating_add(1);

        // Create variables for segment sizes and spacer sizes
        let segment_vars: Vec<Variable> = (0..segment_count).map(|_| Variable::new()).collect();
        let spacer_vars: Vec<Variable> = (0..spacer_count).map(|_| Variable::new()).collect();

        // Apply area bounds
        let total_size = match self.direction {
            Direction::Horizontal => inner.width,
            Direction::Vertical => inner.height,
        };
        let total = (total_size as f64 * FLOAT_PRECISION_MULTIPLIER) as i64;

        // Sum of all segments + spacers == total area
        let mut expr = kasuari::Expression::from(0.0);
        for &v in &segment_vars {
            expr = expr + v;
        }
        for &v in &spacer_vars {
            expr = expr + v;
        }
        solver.add_constraint(expr | EQ(REQUIRED) | total as f64).ok()?;

        // All segment and spacer variables must be ≥ 0
        for &v in segment_vars.iter().chain(spacer_vars.iter()) {
            solver.add_constraint(v | GE(REQUIRED) | 0.0).ok()?;
        }

        // Spacer equality — all spacers equal (will be overridden by flex)
        if spacer_count > 1 {
            for window in spacer_vars.windows(2) {
                solver.add_constraint(window[0] | EQ(strengths::SPACER_SIZE_EQ) | window[1]).ok()?;
            }
        }

        // Apply constraint-specific constraints
        for (i, constraint) in self.constraints.iter().enumerate() {
            let var = segment_vars[i];
            match *constraint {
                Constraint::Min(min) => {
                    let val = (min as f64 * FLOAT_PRECISION_MULTIPLIER) as i64;
                    solver.add_constraint(var | GE(strengths::MIN_SIZE_GE) | val as f64).ok()?;
                }
                Constraint::Max(max) => {
                    let val = (max as f64 * FLOAT_PRECISION_MULTIPLIER) as i64;
                    solver.add_constraint(var | LE(strengths::MAX_SIZE_LE) | val as f64).ok()?;
                    solver.add_constraint(var | EQ(strengths::MAX_SIZE_EQ) | val as f64).ok()?;
                }
                Constraint::Length(len) => {
                    let val = (len as f64 * FLOAT_PRECISION_MULTIPLIER) as i64;
                    solver.add_constraint(var | EQ(strengths::LENGTH_SIZE_EQ) | val as f64).ok()?;
                }
                Constraint::Percentage(pct) => {
                    let val = (total_size as f64 * pct as f64 / 100.0 * FLOAT_PRECISION_MULTIPLIER) as i64;
                    solver.add_constraint(var | EQ(strengths::PERCENTAGE_SIZE_EQ) | val as f64).ok()?;
                }
                Constraint::Ratio(num, den) => {
                    let val = (total_size as f64 * num as f64 / den as f64 * FLOAT_PRECISION_MULTIPLIER) as i64;
                    solver.add_constraint(var | EQ(strengths::RATIO_SIZE_EQ) | val as f64).ok()?;
                }
                Constraint::Fill(_) => {
                    // Fill constraints are handled by flex/grow below
                }
            }
        }

        // Flex distribution
        self.configure_flex(&mut solver, &segment_vars, &spacer_vars, total)?;

        // Fill proportion constraints
        self.configure_fill(&mut solver, &segment_vars)?;

        // Solve
        solver.fetch_changes();

        // Convert solver values back to rects
        Some(self.changes_to_rects(&segment_vars, &spacer_vars, inner))
    }

    fn configure_flex(
        &self,
        solver: &mut Solver,
        segment_vars: &[Variable],
        spacer_vars: &[Variable],
        total: i64,
    ) -> Option<()> {
        match self.flex {
            Flex::Legacy => {
                // Last segment absorbs all excess space
                if let Some(&last) = segment_vars.last() {
                    solver.add_constraint(last | EQ(strengths::GROW) | total as f64).ok()?;
                }
            }
            Flex::Start => {
                // Spacers after the last segment absorb space
                if let Some(&last_spacer) = spacer_vars.last() {
                    solver.add_constraint(last_spacer | EQ(strengths::SPACE_GROW) | total as f64).ok()?;
                }
            }
            Flex::End => {
                // First spacer absorbs space
                if let Some(&first_spacer) = spacer_vars.first() {
                    solver.add_constraint(first_spacer | EQ(strengths::SPACE_GROW) | total as f64).ok()?;
                }
            }
            Flex::Center => {
                // First and last spacers absorb equally
                if spacer_vars.len() >= 2 {
                    solver.add_constraint(spacer_vars[0] | EQ(strengths::SPACE_GROW) | spacer_vars[spacer_vars.len() - 1]).ok()?;
                }
            }
            Flex::SpaceBetween => {
                // Internal spacers equal, edge spacers = 0
                for &v in [spacer_vars.first()?, spacer_vars.last()?] {
                    solver.add_constraint(v | EQ(REQUIRED) | 0.0).ok()?;
                }
                if spacer_vars.len() > 2 {
                    for window in spacer_vars[1..spacer_vars.len() - 1].windows(2) {
                        solver.add_constraint(window[0] | EQ(strengths::SPACE_GROW) | window[1]).ok()?;
                    }
                }
            }
            Flex::SpaceAround => {
                // Edge spacers = half of internal spacers
                if spacer_vars.len() >= 3 {
                    solver.add_constraint(spacer_vars[0] | EQ(strengths::SPACE_GROW) | spacer_vars[1] / 2.0).ok()?;
                    solver.add_constraint(spacer_vars[spacer_vars.len() - 1] | EQ(strengths::SPACE_GROW) | spacer_vars[1] / 2.0).ok()?;
                    for window in spacer_vars[1..spacer_vars.len() - 1].windows(2) {
                        solver.add_constraint(window[0] | EQ(strengths::SPACE_GROW) | window[1]).ok()?;
                    }
                }
            }
            Flex::SpaceEvenly => {
                // All spacers equal
                if spacer_vars.len() > 1 {
                    for window in spacer_vars.windows(2) {
                        solver.add_constraint(window[0] | EQ(strengths::SPACE_GROW) | window[1]).ok()?;
                    }
                }
            }
        }
        Some(())
    }

    fn configure_fill(&self, solver: &mut Solver, segment_vars: &[Variable]) -> Option<()> {
        let fill_indices: Vec<usize> = self.constraints
            .iter()
            .enumerate()
            .filter_map(|(i, c)| matches!(c, Constraint::Fill(_)).then_some(i))
            .collect();

        if fill_indices.len() >= 2 {
            // Make Fill segments proportional to their requested fill amounts
            let first = fill_indices[0];
            let first_amount = match self.constraints[first] {
                Constraint::Fill(n) => n.max(1) as f64,
                _ => 1.0,
            };
            for &idx in &fill_indices[1..] {
                let amount = match self.constraints[idx] {
                    Constraint::Fill(n) => n.max(1) as f64,
                    _ => 1.0,
                };
                let ratio = amount / first_amount;
                solver.add_constraint(
                    segment_vars[idx] | EQ(strengths::FILL_GROW) | (segment_vars[first] * ratio)
                ).ok()?;
            }
        } else if fill_indices.len() == 1 {
            // Single fill: grow to absorb remaining space
            let idx = fill_indices[0];
            solver.add_constraint(segment_vars[idx] | EQ(strengths::FILL_GROW) | 1_000_000.0).ok()?;
        }

        Some(())
    }

    fn changes_to_rects(
        &self,
        segment_vars: &[Variable],
        spacer_vars: &[Variable],
        inner: Rect,
    ) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(segment_vars.len());
        let mut offset = 0.0;

        for (i, &var) in segment_vars.iter().enumerate() {
            let size = (var.value() / FLOAT_PRECISION_MULTIPLIER).round() as u16;
            let spacer = if i < spacer_vars.len() {
                (spacer_vars[i].value() / FLOAT_PRECISION_MULTIPLIER).round() as u16
            } else {
                0
            };

            let rect = match self.direction {
                Direction::Horizontal => Rect::new(
                    inner.x.saturating_add(offset as u16 + spacer),
                    inner.y,
                    size,
                    inner.height,
                ),
                Direction::Vertical => Rect::new(
                    inner.x,
                    inner.y.saturating_add(offset as u16 + spacer),
                    inner.width,
                    size,
                ),
            };
            rects.push(rect);
            offset += spacer as f64 + size as f64;
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_vertical_split_length() {
        let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 5);
        assert_eq!(rects[1].height, 5);
        assert_eq!(rects[0].width, 10);
        assert_eq!(rects[1].width, 10);
    }

    #[test]
    fn layout_horizontal_split_length() {
        let layout = Layout::horizontal([Constraint::Length(5), Constraint::Length(5)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 5);
        assert_eq!(rects[1].width, 5);
        assert_eq!(rects[0].height, 10);
        assert_eq!(rects[1].height, 10);
    }

    #[test]
    fn layout_split_with_margin() {
        let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)])
            .margin(1);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        // Margin reduces available height from 10 to 8
        assert_eq!(rects[0].y, 1); // top margin
        assert_eq!(rects[0].width, 8); // horizontal margin removed
    }

    #[test]
    fn layout_split_min_constraint() {
        let layout = Layout::vertical([Constraint::Min(3), Constraint::Min(3)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert!(rects[0].height >= 3);
        assert!(rects[1].height >= 3);
    }

    #[test]
    fn layout_split_max_constraint() {
        let layout = Layout::vertical([Constraint::Max(3), Constraint::Max(3)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert!(rects[0].height <= 3);
        assert!(rects[1].height <= 3);
    }

    #[test]
    fn layout_split_percentage() {
        let layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        // Allow 1 unit of rounding error
        assert!((rects[0].height as i16 - 5).abs() <= 1);
        assert!((rects[1].height as i16 - 5).abs() <= 1);
    }

    #[test]
    fn layout_split_ratio() {
        let layout = Layout::vertical([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 2); // 10 * 1/4 = 2.5 → rounds to 2 or 3
        assert_eq!(rects[0].height + rects[1].height, 10);
    }

    #[test]
    fn layout_split_empty_area() {
        let layout = Layout::vertical([Constraint::Length(5)]);
        let rects = layout.split(Rect::ZERO);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], Rect::ZERO);
    }

    #[test]
    fn layout_builder_api() {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10)])
            .margin(2)
            .flex(Flex::Center)
            .spacing(1);
        assert_eq!(layout.direction, Direction::Horizontal);
        assert_eq!(layout.constraints, vec![Constraint::Length(10)]);
        assert_eq!(layout.margin, Margin::new(2, 2));
        assert_eq!(layout.flex, Flex::Center);
        assert_eq!(layout.spacing, Spacing::Space(1));
    }

    #[test]
    fn layout_areas_const_generic() {
        let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
        let areas: [Rect; 2] = layout.areas(Rect::new(0, 0, 10, 10));
        assert_eq!(areas[0].height, 5);
        assert_eq!(areas[1].height, 5);
    }
}
```

**Note:** The `areas` method uses `split` internally and converts `Vec<Rect>` to `[Rect; N]` via `try_into().expect(...)`. Add it after `split`:

```rust
    /// Split the area and return exactly `N` rects.
    ///
    /// # Panics
    /// Panics if the number of constraints does not equal `N`.
    pub fn areas<const N: usize>(&self, area: Rect) -> [Rect; N] {
        let rects = self.split(area);
        rects.try_into().expect("constraint count must match N")
    }
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::layout`
Expected: PASS (all new unit tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/layout.rs
git commit -m "feat(layout): add Layout struct with Cassowary split"
```

---

### Task 5: Wire up `layout.rs` into `layout/mod.rs`

**Files:**
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Add re-export and module declaration**

Add `pub mod layout;` and `pub use layout::Layout;` to `src/layout/mod.rs`:

```rust
pub mod constraint;
pub mod direction;
pub mod flex;
pub mod layout;
pub mod margin;
pub mod offset;
pub mod position;
pub mod rect;
pub mod size;
pub mod spacing;
pub mod strengths;

pub use constraint::Constraint;
pub use direction::Direction;
pub use flex::Flex;
pub use layout::Layout;
pub use margin::Margin;
pub use offset::Offset;
pub use position::Position;
pub use rect::{Columns, Positions, Rect, Rows};
pub use size::Size;
pub use spacing::Spacing;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/layout/mod.rs
git commit -m "feat(layout): re-export Layout from layout module"
```

---

### Task 6: Add `Container` component

**Files:**
- Create: `src/components/container.rs`

- [ ] **Step 1: Write the file**

```rust
use crate::layout::{Layout, Rect};
use crate::{Component, InputResult, RenderError, Rendered, events::Event};

/// A component that lays out its children using a [`Layout`].
///
/// Each child is rendered into the rect assigned by the layout via
/// [`render_rect`](Component::render_rect).
pub struct Container {
    layout: Layout,
    children: Vec<Box<dyn Component>>,
}

impl Container {
    /// Create a new container with the given layout.
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            children: Vec::new(),
        }
    }

    /// Add a child component.
    pub fn push(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    /// Builder-style add.
    pub fn with_child(mut self, child: Box<dyn Component>) -> Self {
        self.children.push(child);
        self
    }
}

impl Component for Container {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        // Container needs a height hint; default to stacking vertically
        let rect = Rect::new(0, 0, width, self.children.len() as u16 * 3);
        self.render_rect(rect)
    }

    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError> {
        let mut screen = Rendered::empty();
        let areas = self.layout.split(rect);

        for (child, area) in self.children.iter().zip(areas.iter()) {
            if let Ok(rendered) = child.render_rect(*area) {
                rendered.blit_into_rect(&mut screen, *area);
            }
        }

        Ok(screen)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        for child in &mut self.children {
            let result = child.handle_input(event);
            if result != InputResult::Ignored {
                return result;
            }
        }
        InputResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Text;
    use crate::layout::Constraint;

    #[test]
    fn container_renders_children() {
        let mut container = Container::new(
            Layout::horizontal([Constraint::Length(10), Constraint::Length(10)])
        );
        container.push(Box::new(Text::new("left", 0, 0)));
        container.push(Box::new(Text::new("right", 0, 0)));

        let rendered = container.render_rect(Rect::new(0, 0, 20, 1)).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert!(rendered.lines[0].contains("left"));
        assert!(rendered.lines[0].contains("right"));
    }

    #[test]
    fn container_vertical_split() {
        let mut container = Container::new(
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
        );
        container.push(Box::new(Text::new("top", 0, 0)));
        container.push(Box::new(Text::new("bottom", 0, 0)));

        let rendered = container.render_rect(Rect::new(0, 0, 10, 2)).unwrap();
        assert_eq!(rendered.lines.len(), 2);
        assert!(rendered.lines[0].contains("top"));
        assert!(rendered.lines[1].contains("bottom"));
    }
}
```

- [ ] **Step 2: Register in `src/components/mod.rs`**

Find the existing component exports in `src/components/mod.rs` and add:

```rust
pub mod container;
```

and in the `pub use` section:

```rust
pub use container::Container;
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib components::container`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/components/container.rs src/components/mod.rs
git commit -m "feat(components): add Container for nested layouts"
```

---

### Task 7: Update `TUI` with optional layout

**Files:**
- Modify: `src/tui.rs`

- [ ] **Step 1: Add `layout` field to `TUI`**

Add `layout: Option<Layout>` to the `TUI` struct:

```rust
pub struct TUI {
    terminal: Box<dyn Terminal>,
    children: Vec<Box<dyn Component>>,
    overlays: Vec<Overlay>,
    focused_index: Option<usize>,
    renderer: Renderer,
    size: (u16, u16),
    previous_image_ids: std::collections::HashSet<u32>,
    clear_on_shrink: bool,
    hardware_cursor: bool,
    layout: Option<Layout>,
}
```

- [ ] **Step 2: Initialize layout field in `TUI::new`**

```rust
    pub fn new(terminal: Box<dyn Terminal>) -> Self {
        Self {
            terminal,
            children: Vec::new(),
            overlays: Vec::new(),
            focused_index: None,
            renderer: Renderer::new(),
            size: (80, 24),
            previous_image_ids: std::collections::HashSet::new(),
            clear_on_shrink: std::env::var("PHOTON_UI_CLEAR_ON_SHRINK").is_ok(),
            hardware_cursor: std::env::var("PHOTON_UI_HARDWARE_CURSOR").is_ok(),
            layout: None,
        }
    }
```

- [ ] **Step 3: Add setter method**

```rust
    /// Set a layout for splitting the terminal area among children.
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = Some(layout);
    }

    /// Clear the layout, reverting to vertical stacking.
    pub fn clear_layout(&mut self) {
        self.layout = None;
    }
```

- [ ] **Step 4: Update `render_frame` to use layout**

Replace the child rendering loop in `render_frame` (lines 242-255) with:

```rust
        // Render children using layout if set, otherwise stack vertically.
        let mut screen = Rendered::empty();
        let term_rect = Rect::new(0, 0, width, height);

        if let Some(layout) = &self.layout {
            let areas = layout.split(term_rect);
            for (child, area) in self.children.iter().zip(areas.iter()) {
                if let Ok(rendered) = child.render_rect(*area) {
                    rendered.blit_into_rect(&mut screen, *area);
                }
            }
        } else {
            // Original vertical stacking behavior
            let mut row = 0usize;
            for child in &self.children {
                if let Ok(rendered) = child.render(width) {
                    for line in &rendered.lines {
                        screen.lines.push(line.clone());
                    }
                    if let Some((r, c)) = rendered.cursor {
                        screen.cursor = Some((row + r, c));
                    }
                    screen.images.extend(rendered.images);
                    row += rendered.lines.len();
                }
            }
        }
```

- [ ] **Step 5: Update `compose_screen` similarly**

Replace `compose_screen` with a layout-aware version:

```rust
    #[cfg(test)]
    fn compose_screen(&self, width: u16, height: u16) -> crate::renderer::Rendered {
        let mut screen = crate::renderer::Rendered::empty();
        let term_rect = Rect::new(0, 0, width, height);

        if let Some(layout) = &self.layout {
            let areas = layout.split(term_rect);
            for (child, area) in self.children.iter().zip(areas.iter()) {
                if let Ok(rendered) = child.render_rect(*area) {
                    rendered.blit_into_rect(&mut screen, *area);
                }
            }
        } else {
            let mut row = 0usize;
            for child in &self.children {
                if let Ok(rendered) = child.render(width) {
                    for line in &rendered.lines {
                        screen.lines.push(line.clone());
                    }
                    if let Some((r, c)) = rendered.cursor {
                        screen.cursor = Some((row + r, c));
                    }
                    row += rendered.lines.len();
                }
            }
        }
        screen
    }
```

- [ ] **Step 6: Run tests**

Run: `cargo test --lib tui`
Expected: PASS (all existing tui tests pass)

- [ ] **Step 7: Commit**

```bash
git add src/tui.rs
git commit -m "feat(tui): add optional Layout for splitting terminal area"
```

---

### Task 8: Integration tests for layout engine

**Files:**
- Create: `tests/layout_engine_tests.rs`

- [ ] **Step 1: Write the test file**

```rust
use photon_ui::layout::{Constraint, Direction, Flex, Layout, Margin, Rect};

#[test]
fn layout_split_two_equal_vertical() {
    let layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[0].width, 10);
    assert!((rects[0].height as i16 - 5).abs() <= 1);
    assert!((rects[1].height as i16 - 5).abs() <= 1);
}

#[test]
fn layout_split_three_horizontal() {
    let layout = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ]);
    let rects = layout.split(Rect::new(0, 0, 30, 5));
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].width, 10);
    assert_eq!(rects[1].width, 10);
    assert_eq!(rects[2].width, 10);
}

#[test]
fn layout_split_with_margin_reduces_size() {
    let layout = Layout::vertical([Constraint::Length(5)])
        .margin(2);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects[0].width, 6); // 10 - 2*2
}

#[test]
fn layout_split_min_max_combined() {
    let layout = Layout::vertical([Constraint::Min(2), Constraint::Max(8)]);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert!(rects[0].height >= 2);
    assert!(rects[1].height <= 8);
}

#[test]
fn layout_split_ratio_one_to_three() {
    let layout = Layout::vertical([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)]);
    let rects = layout.split(Rect::new(0, 0, 10, 20));
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].height + rects[1].height, 20);
}

#[test]
fn layout_builder_chaining() {
    let layout = Layout::horizontal([Constraint::Length(5)])
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .margin(1)
        .flex(Flex::Center)
        .spacing(1);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects.len(), 2);
}

#[test]
fn layout_areas_const_generic() {
    let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
    let areas: [Rect; 2] = layout.areas(Rect::new(0, 0, 10, 10));
    assert_eq!(areas[0].height, 5);
    assert_eq!(areas[1].height, 5);
}

#[test]
fn layout_split_empty_returns_zeros() {
    let layout = Layout::vertical([Constraint::Length(5)]);
    let rects = layout.split(Rect::ZERO);
    assert_eq!(rects[0], Rect::ZERO);
}

#[test]
fn layout_flex_start_leaves_trailing_space() {
    let layout = Layout::vertical([Constraint::Length(3)])
        .flex(Flex::Start);
    let rects = layout.split(Rect::new(0, 0, 10, 10));
    assert_eq!(rects[0].height, 3);
}

#[test]
fn layout_flex_space_between_no_edge_space() {
    let layout = Layout::horizontal([Constraint::Length(3), Constraint::Length(3)])
        .flex(Flex::SpaceBetween);
    let rects = layout.split(Rect::new(0, 0, 10, 1));
    assert_eq!(rects.len(), 2);
    // Edge spacers should be 0, so first rect starts at x=0
    assert_eq!(rects[0].x, 0);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test layout_engine_tests`
Expected: PASS (all tests)

- [ ] **Step 3: Commit**

```bash
git add tests/layout_engine_tests.rs
git commit -m "test(layout): add Layout integration tests"
```

---

### Task 9: Property-based tests for layout invariants

**Files:**
- Create: `tests/layout_engine_proptests.rs`

- [ ] **Step 1: Write the test file**

```rust
use photon_ui::layout::{Constraint, Layout, Rect};
use proptest::prelude::*;

proptest! {
    #[test]
    fn layout_split_produces_correct_count(
        constraints in proptest::collection::vec(constraint_strategy(), 1..10),
        area in rect_strategy()
    ) {
        let layout = Layout::vertical(constraints.clone());
        let rects = layout.split(area);
        prop_assert_eq!(rects.len(), constraints.len());
    }

    #[test]
    fn layout_split_rects_fit_inside_area(
        constraints in proptest::collection::vec(constraint_strategy(), 1..5),
        area in rect_strategy()
    ) {
        let layout = Layout::vertical(constraints);
        let rects = layout.split(area);
        for rect in &rects {
            prop_assert!(rect.x >= area.x);
            prop_assert!(rect.y >= area.y);
            prop_assert!(rect.right() <= area.right());
            prop_assert!(rect.bottom() <= area.bottom());
        }
    }

    #[test]
    fn layout_split_rects_are_non_overlapping(
        constraints in proptest::collection::vec(constraint_strategy(), 2..5),
        area in rect_strategy()
    ) {
        let layout = Layout::vertical(constraints);
        let rects = layout.split(area);
        for window in rects.windows(2) {
            prop_assert!(
                window[0].bottom() <= window[1].y,
                "rects should not overlap vertically: {:?} vs {:?}",
                window[0], window[1]
            );
        }
    }

    #[test]
    fn layout_split_areas_sum_to_parent_height(
        constraints in proptest::collection::vec(constraint_strategy(), 1..5),
        area in non_empty_rect_strategy()
    ) {
        let layout = Layout::vertical(constraints);
        let rects = layout.split(area);
        let total_height: u16 = rects.iter().map(|r| r.height).sum();
        // Allow rounding error of 1 per rect
        let max_error = rects.len() as u16;
        prop_assert!(
            total_height <= area.height + max_error && total_height + max_error >= area.height,
            "heights {} should approximate area height {}",
            total_height, area.height
        );
    }
}

fn constraint_strategy() -> impl Strategy<Value = Constraint> {
    prop_oneof![
        (1u16..50).prop_map(Constraint::Length),
        (1u16..50).prop_map(Constraint::Min),
        (1u16..50).prop_map(Constraint::Max),
        (1u16..100).prop_map(Constraint::Percentage),
    ]
}

fn rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..100, 0u16..100, 1u16..100, 1u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn non_empty_rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..50, 0u16..50, 5u16..100, 5u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test layout_engine_proptests`
Expected: PASS (all proptest cases pass)

- [ ] **Step 3: Commit**

```bash
git add tests/layout_engine_proptests.rs
git commit -m "test(layout): add property-based tests for layout invariants"
```

---

### Task 10: Add layout demo page to example app

**Files:**
- Modify: `examples/demo.rs`

- [ ] **Step 1: Add layout imports and new page**

Add to imports:
```rust
use photon_ui::layout::{Constraint, Layout};
use photon_ui::components::Container;
```

Update header to show 6 pages:
```rust
let header = format!(
    " Photon UI Demo  |  Page {}/6  |  1-6=pages  Tab=focus  q=quit ",
    self.page
);
```

Add page 6 to match:
```rust
            6 => self.load_page_layout_engine(),
```

Add key handler for '6':
```rust
                KeyCode::Char('6') => {
                    self.page = 6;
                    self.load_page();
                    return true;
                }
```

Add the page loader:
```rust
    fn load_page_layout_engine(&mut self) {
        self.tui.mount(std::boxed::Box::new(Text::new(
            "Layout Engine — Cassowary constraint solver:",
            0,
            0,
        )));

        let mut container = Container::new(
            Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ])
        );
        container.push(Box::new(Text::new("Top panel (Length 3)", 0, 0)));
        container.push(Box::new(Text::new("Middle panel (Min 2) — expands to fill", 0, 0)));
        container.push(Box::new(Text::new("Bottom panel (Length 3)", 0, 0)));
        self.tui.mount(std::boxed::Box::new(container));
    }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check --example demo`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add examples/demo.rs
git commit -m "feat(demo): add page 6 demonstrating Layout::split"
```

---

### Task 11: Full suite verification

**Files:**
- All files

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: All tests pass (existing + new)

- [ ] **Step 2: Run example check**

Run: `cargo check --example demo`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(layout): verify full suite passes with layout engine"
```

---

## Self-Review Checklist

**1. Spec coverage:**
- `kasuari` dependency → Task 1
- `render_rect` backward-compatible API → Task 2
- Solver strength constants → Task 3
- `Layout` struct with builder API → Task 4
- `Layout::split` using Cassowary → Task 4
- `Layout::areas` const generic → Task 4
- `Container` component → Task 6
- `TUI` optional layout → Task 7
- Integration tests → Task 8
- Property-based tests → Task 9
- Demo page → Task 10

**2. Placeholder scan:** No TBD, TODO, or vague steps. Every step has exact code, exact commands, expected output.

**3. Type consistency:**
- `Layout::new`/`vertical`/`horizontal` use `IntoIterator<Item: Into<Constraint>>` — matches ratatui API
- `Layout::split` returns `Vec<Rect>` — consistent with Phase 1 `Rect`
- `Container::render_rect` uses `blit_into_rect` — consistent with Phase 13
- `render_rect` default delegates to `render(width)` — backward compatible
- Strength constants use `kasuari` expression syntax (`| EQ(REQUIRED) |`)

**4. Edge cases handled:**
- Empty area → returns zero rects
- Margin larger than area → returns zero rects
- Single constraint → works
- Fill constraints with no other constraints → handled by flex/grow
- Rounding errors in solver → accounted for with `FLOAT_PRECISION_MULTIPLIER`
