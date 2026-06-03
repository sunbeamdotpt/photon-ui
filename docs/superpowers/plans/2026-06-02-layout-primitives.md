# Layout Primitives Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce all geometric and constraint types (`Rect`, `Position`, `Size`, `Margin`, `Offset`, `Direction`, `Flex`, `Constraint`, `Spacing`) into a new `src/layout/` module with >90% test coverage.

**Architecture:** Port ratatui-compatible layout primitives as small, focused files in `src/layout/`. Each type file contains its own exhaustive `#[cfg(test)]` module. Integration tests in `tests/` cover cross-type interactions and property-based invariants. The existing `Rect` in `src/tui.rs` is migrated to use `layout::Rect`. No external dependencies.

**Tech Stack:** Rust 2024 edition, standard library only, `proptest` for property-based tests.

---

## File Structure

**New files:**
- `src/layout/mod.rs` — module re-exports
- `src/layout/offset.rs` — `Offset` (relative movement)
- `src/layout/position.rs` — `Position` (point in terminal)
- `src/layout/size.rs` — `Size` (dimensions)
- `src/layout/margin.rs` — `Margin` (spacing around rects)
- `src/layout/direction.rs` — `Direction` (horizontal/vertical)
- `src/layout/flex.rs` — `Flex` (space distribution modes)
- `src/layout/constraint.rs` — `Constraint` (size constraints)
- `src/layout/spacing.rs` — `Spacing` (gap/overlap between segments)
- `src/layout/rect.rs` — `Rect` (rectangle + geometric ops + iterators)
- `tests/layout_rect_tests.rs` — `Rect` integration tests
- `tests/layout_position_tests.rs` — `Position` + `Offset` integration tests
- `tests/layout_constraint_tests.rs` — `Constraint` integration tests
- `tests/layout_type_integration_tests.rs` — cross-type property tests

**Modified files:**
- `src/lib.rs` — add `pub mod layout;`
- `src/tui.rs` — migrate inline `Rect` to `crate::layout::Rect`
- `src/renderer.rs` — add `Rendered::blit_into_rect`

---

### Task 1: `Offset`

**Files:**
- Create: `src/layout/offset.rs`

- [ ] **Step 1: Write the file with tests**

```rust
/// Relative movement in the terminal coordinate system.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset {
    pub x: i16,
    pub y: i16,
}

impl Offset {
    pub const MAX: Self = Self::new(i16::MAX, i16::MAX);
    pub const MIN: Self = Self::new(i16::MIN, i16::MIN);

    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Offset {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x.saturating_add(other.x), y: self.y.saturating_add(other.y) }
    }
}

impl std::ops::Sub for Offset {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self { x: self.x.saturating_sub(other.x), y: self.y.saturating_sub(other.y) }
    }
}

impl std::ops::Neg for Offset {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { x: self.x.saturating_neg(), y: self.y.saturating_neg() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_new() {
        let o = Offset::new(5, -3);
        assert_eq!(o.x, 5);
        assert_eq!(o.y, -3);
    }

    #[test]
    fn offset_add() {
        let a = Offset::new(10, 20);
        let b = Offset::new(5, -8);
        assert_eq!(a + b, Offset::new(15, 12));
    }

    #[test]
    fn offset_sub() {
        let a = Offset::new(10, 20);
        let b = Offset::new(5, 8);
        assert_eq!(a - b, Offset::new(5, 12));
    }

    #[test]
    fn offset_neg() {
        let o = Offset::new(5, -3);
        assert_eq!(-o, Offset::new(-5, 3));
    }

    #[test]
    fn offset_add_saturates() {
        let a = Offset::new(i16::MAX, i16::MAX);
        let b = Offset::new(1, 1);
        assert_eq!(a + b, Offset::new(i16::MAX, i16::MAX));
    }

    #[test]
    fn offset_sub_saturates() {
        let a = Offset::new(i16::MIN, i16::MIN);
        let b = Offset::new(1, 1);
        assert_eq!(a - b, Offset::new(i16::MIN, i16::MIN));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::offset`
Expected: PASS (6 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/offset.rs
git commit -m "feat(layout): add Offset primitive"
```

---

### Task 2: `Size`

**Files:**
- Create: `src/layout/size.rs`

- [ ] **Step 1: Write the file with tests**

```rust
use std::fmt;

/// Dimensions in the terminal (width × height).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const ZERO: Self = Self::new(0, 0);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self::new(u16::MAX, u16::MAX);

    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub const fn area(self) -> u32 {
        self.width as u32 * self.height as u32
    }
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Self { width, height }
    }
}

impl From<Size> for (u16, u16) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_new() {
        let s = Size::new(10, 20);
        assert_eq!(s.width, 10);
        assert_eq!(s.height, 20);
    }

    #[test]
    fn size_area() {
        assert_eq!(Size::new(10, 20).area(), 200);
        assert_eq!(Size::ZERO.area(), 0);
    }

    #[test]
    fn size_from_tuple() {
        let s: Size = (5, 7).into();
        assert_eq!(s, Size::new(5, 7));
    }

    #[test]
    fn size_into_tuple() {
        let s = Size::new(3, 4);
        let (w, h): (u16, u16) = s.into();
        assert_eq!(w, 3);
        assert_eq!(h, 4);
    }

    #[test]
    fn size_display() {
        assert_eq!(Size::new(10, 20).to_string(), "10x20");
    }

    #[test]
    fn size_max_area() {
        assert_eq!(Size::MAX.area(), u16::MAX as u32 * u16::MAX as u32);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::size`
Expected: PASS (6 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/size.rs
git commit -m "feat(layout): add Size primitive"
```

---

### Task 3: `Margin`

**Files:**
- Create: `src/layout/margin.rs`

- [ ] **Step 1: Write the file with tests**

```rust
use std::fmt;

/// Spacing around a rectangular area.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Margin {
    pub horizontal: u16,
    pub vertical: u16,
}

impl Margin {
    pub const fn new(horizontal: u16, vertical: u16) -> Self {
        Self { horizontal, vertical }
    }
}

impl From<u16> for Margin {
    fn from(value: u16) -> Self {
        Self::new(value, value)
    }
}

impl fmt::Display for Margin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.horizontal, self.vertical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_new() {
        let m = Margin::new(2, 1);
        assert_eq!(m.horizontal, 2);
        assert_eq!(m.vertical, 1);
    }

    #[test]
    fn margin_from_u16() {
        let m: Margin = 5.into();
        assert_eq!(m, Margin::new(5, 5));
    }

    #[test]
    fn margin_display() {
        assert_eq!(Margin::new(2, 1).to_string(), "2x1");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::margin`
Expected: PASS (3 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/margin.rs
git commit -m "feat(layout): add Margin primitive"
```

---

### Task 4: `Direction`

**Files:**
- Create: `src/layout/direction.rs`

- [ ] **Step 1: Write the file with tests**

```rust
/// Layout direction.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Direction {
    Horizontal,
    #[default]
    Vertical,
}

impl Direction {
    pub const fn perpendicular(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_perpendicular() {
        assert_eq!(Direction::Horizontal.perpendicular(), Direction::Vertical);
        assert_eq!(Direction::Vertical.perpendicular(), Direction::Horizontal);
    }

    #[test]
    fn direction_default() {
        assert_eq!(Direction::default(), Direction::Vertical);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::direction`
Expected: PASS (2 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/direction.rs
git commit -m "feat(layout): add Direction primitive"
```

---

### Task 5: `Flex`

**Files:**
- Create: `src/layout/flex.rs`

- [ ] **Step 1: Write the file with tests**

```rust
/// How excess space is distributed when layout constraints are satisfied.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Flex {
    /// Excess space goes into the last element (ratatui legacy behavior).
    Legacy,
    /// Align items to the start.
    #[default]
    Start,
    /// Align items to the end.
    End,
    /// Center items.
    Center,
    /// Space between items, none at edges.
    SpaceBetween,
    /// Space around items.
    SpaceAround,
    /// Even space before, between, and after items.
    SpaceEvenly,
}

impl Flex {
    pub const fn is_legacy(self) -> bool {
        matches!(self, Self::Legacy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_default() {
        assert_eq!(Flex::default(), Flex::Start);
    }

    #[test]
    fn flex_is_legacy() {
        assert!(Flex::Legacy.is_legacy());
        assert!(!Flex::Start.is_legacy());
        assert!(!Flex::End.is_legacy());
        assert!(!Flex::Center.is_legacy());
        assert!(!Flex::SpaceBetween.is_legacy());
        assert!(!Flex::SpaceAround.is_legacy());
        assert!(!Flex::SpaceEvenly.is_legacy());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::flex`
Expected: PASS (2 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/flex.rs
git commit -m "feat(layout): add Flex primitive"
```

---

### Task 6: `Spacing`

**Files:**
- Create: `src/layout/spacing.rs`

- [ ] **Step 1: Write the file with tests**

```rust
/// Gap or overlap between layout segments.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Spacing {
    Space(u16),
    Overlap(u16),
}

impl Default for Spacing {
    fn default() -> Self {
        Self::Space(0)
    }
}

impl From<u16> for Spacing {
    fn from(value: u16) -> Self {
        Self::Space(value)
    }
}

impl From<i16> for Spacing {
    fn from(value: i16) -> Self {
        if value < 0 {
            Self::Overlap(value.unsigned_abs())
        } else {
            Self::Space(value as u16)
        }
    }
}

impl From<i32> for Spacing {
    fn from(value: i32) -> Self {
        Self::from(value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_default() {
        assert_eq!(Spacing::default(), Spacing::Space(0));
    }

    #[test]
    fn spacing_from_u16() {
        assert_eq!(Spacing::from(5_u16), Spacing::Space(5));
    }

    #[test]
    fn spacing_from_i16_positive() {
        assert_eq!(Spacing::from(3_i16), Spacing::Space(3));
    }

    #[test]
    fn spacing_from_i16_negative() {
        assert_eq!(Spacing::from(-2_i16), Spacing::Overlap(2));
    }

    #[test]
    fn spacing_from_i32_clamped() {
        assert_eq!(Spacing::from(i32::MAX), Spacing::Space(i16::MAX as u16));
        assert_eq!(Spacing::from(i32::MIN), Spacing::Overlap(i16::MAX as u16 + 1));
    }

    #[test]
    fn spacing_from_i32_zero() {
        assert_eq!(Spacing::from(0_i32), Spacing::Space(0));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::spacing`
Expected: PASS (6 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/spacing.rs
git commit -m "feat(layout): add Spacing primitive"
```


---

### Task 7: `Position`

**Files:**
- Create: `src/layout/position.rs`

- [ ] **Step 1: Write the file with tests**

```rust
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::{Offset, Rect};

/// A point in the terminal coordinate system.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);
    pub const MIN: Self = Self::ORIGIN;
    pub const MAX: Self = Self::new(u16::MAX, u16::MAX);

    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn offset(self, offset: Offset) -> Self {
        self + offset
    }
}

impl From<(u16, u16)> for Position {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
    }
}

impl From<Position> for (u16, u16) {
    fn from(position: Position) -> Self {
        (position.x, position.y)
    }
}

impl From<Rect> for Position {
    fn from(rect: Rect) -> Self {
        rect.as_position()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add<Offset> for Position {
    type Output = Self;
    fn add(self, offset: Offset) -> Self::Output {
        let max = i32::from(u16::MAX);
        let x = i32::from(self.x).saturating_add(i32::from(offset.x)).clamp(0, max) as u16;
        let y = i32::from(self.y).saturating_add(i32::from(offset.y)).clamp(0, max) as u16;
        Self { x, y }
    }
}

impl Sub<Offset> for Position {
    type Output = Self;
    fn sub(self, offset: Offset) -> Self::Output {
        let max = i32::from(u16::MAX);
        let x = i32::from(self.x).saturating_sub(i32::from(offset.x)).clamp(0, max) as u16;
        let y = i32::from(self.y).saturating_sub(i32::from(offset.y)).clamp(0, max) as u16;
        Self { x, y }
    }
}

impl AddAssign<Offset> for Position {
    fn add_assign(&mut self, offset: Offset) {
        *self = *self + offset;
    }
}

impl SubAssign<Offset> for Position {
    fn sub_assign(&mut self, offset: Offset) {
        *self = *self - offset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_new() {
        let p = Position::new(5, 10);
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 10);
    }

    #[test]
    fn position_from_tuple() {
        let p: Position = (3, 4).into();
        assert_eq!(p, Position::new(3, 4));
    }

    #[test]
    fn position_into_tuple() {
        let p = Position::new(1, 2);
        let (x, y): (u16, u16) = p.into();
        assert_eq!(x, 1);
        assert_eq!(y, 2);
    }

    #[test]
    fn position_add_offset() {
        let p = Position::new(5, 5);
        let o = Offset::new(3, -2);
        assert_eq!(p + o, Position::new(8, 3));
    }

    #[test]
    fn position_sub_offset() {
        let p = Position::new(5, 5);
        let o = Offset::new(3, 2);
        assert_eq!(p - o, Position::new(2, 3));
    }

    #[test]
    fn position_add_clamps_to_max() {
        let p = Position::MAX;
        let o = Offset::new(1, 1);
        assert_eq!(p + o, Position::MAX);
    }

    #[test]
    fn position_sub_clamps_to_min() {
        let p = Position::ORIGIN;
        let o = Offset::new(1, 1);
        assert_eq!(p - o, Position::ORIGIN);
    }

    #[test]
    fn position_add_assign() {
        let mut p = Position::new(1, 1);
        p += Offset::new(2, 3);
        assert_eq!(p, Position::new(3, 4));
    }

    #[test]
    fn position_sub_assign() {
        let mut p = Position::new(5, 5);
        p -= Offset::new(2, 3);
        assert_eq!(p, Position::new(3, 2));
    }

    #[test]
    fn position_from_rect() {
        let rect = Rect::new(7, 8, 10, 20);
        let p: Position = rect.into();
        assert_eq!(p, Position::new(7, 8));
    }

    #[test]
    fn position_display() {
        assert_eq!(Position::new(1, 2).to_string(), "(1, 2)");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::position`
Expected: PASS (12 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/position.rs
git commit -m "feat(layout): add Position primitive"
```

---

### Task 8: `Constraint`

**Files:**
- Create: `src/layout/constraint.rs`

- [ ] **Step 1: Write the file with tests**

```rust
use std::fmt;

/// A size constraint for layout elements.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Constraint {
    Min(u16),
    Max(u16),
    Length(u16),
    Percentage(u16),
    Ratio(u32, u32),
    Fill(u16),
}

impl Constraint {
    pub fn from_lengths<T>(lengths: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>,
    {
        lengths.into_iter().map(Self::Length).collect()
    }

    pub fn from_ratios<T>(ratios: T) -> Vec<Self>
    where
        T: IntoIterator<Item = (u32, u32)>,
    {
        ratios.into_iter().map(|(n, d)| Self::Ratio(n, d)).collect()
    }

    pub fn from_percentages<T>(percentages: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>,
    {
        percentages.into_iter().map(Self::Percentage).collect()
    }

    pub fn from_mins<T>(mins: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>,
    {
        mins.into_iter().map(Self::Min).collect()
    }

    pub fn from_maxes<T>(maxes: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>,
    {
        maxes.into_iter().map(Self::Max).collect()
    }

    pub fn from_fills<T>(fills: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>,
    {
        fills.into_iter().map(Self::Fill).collect()
    }
}

impl From<u16> for Constraint {
    fn from(length: u16) -> Self {
        Self::Length(length)
    }
}

impl From<&Self> for Constraint {
    fn from(constraint: &Self) -> Self {
        *constraint
    }
}

impl AsRef<Self> for Constraint {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for Constraint {
    fn default() -> Self {
        Self::Percentage(100)
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percentage(p) => write!(f, "Percentage({p})"),
            Self::Ratio(n, d) => write!(f, "Ratio({n}, {d})"),
            Self::Length(l) => write!(f, "Length({l})"),
            Self::Fill(l) => write!(f, "Fill({l})"),
            Self::Max(m) => write!(f, "Max({m})"),
            Self::Min(m) => write!(f, "Min({m})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_default() {
        assert_eq!(Constraint::default(), Constraint::Percentage(100));
    }

    #[test]
    fn constraint_from_u16() {
        let c: Constraint = 10.into();
        assert_eq!(c, Constraint::Length(10));
    }

    #[test]
    fn constraint_from_ref() {
        let c = Constraint::Length(5);
        assert_eq!(Constraint::from(&c), c);
    }

    #[test]
    fn constraint_as_ref() {
        let c = Constraint::Length(5);
        assert_eq!(c.as_ref(), &c);
    }

    #[test]
    fn constraint_display() {
        assert_eq!(Constraint::Percentage(50).to_string(), "Percentage(50)");
        assert_eq!(Constraint::Ratio(1, 2).to_string(), "Ratio(1, 2)");
        assert_eq!(Constraint::Length(10).to_string(), "Length(10)");
        assert_eq!(Constraint::Fill(1).to_string(), "Fill(1)");
        assert_eq!(Constraint::Max(20).to_string(), "Max(20)");
        assert_eq!(Constraint::Min(5).to_string(), "Min(5)");
    }

    #[test]
    fn constraint_from_lengths() {
        let c = Constraint::from_lengths([1, 2, 3]);
        assert_eq!(c, vec![Constraint::Length(1), Constraint::Length(2), Constraint::Length(3)]);
    }

    #[test]
    fn constraint_from_ratios() {
        let c = Constraint::from_ratios([(1, 4), (1, 2)]);
        assert_eq!(c, vec![Constraint::Ratio(1, 4), Constraint::Ratio(1, 2)]);
    }

    #[test]
    fn constraint_from_percentages() {
        let c = Constraint::from_percentages([25, 50]);
        assert_eq!(c, vec![Constraint::Percentage(25), Constraint::Percentage(50)]);
    }

    #[test]
    fn constraint_from_mins() {
        let c = Constraint::from_mins([1, 2]);
        assert_eq!(c, vec![Constraint::Min(1), Constraint::Min(2)]);
    }

    #[test]
    fn constraint_from_maxes() {
        let c = Constraint::from_maxes([10, 20]);
        assert_eq!(c, vec![Constraint::Max(10), Constraint::Max(20)]);
    }

    #[test]
    fn constraint_from_fills() {
        let c = Constraint::from_fills([1, 2, 3]);
        assert_eq!(c, vec![Constraint::Fill(1), Constraint::Fill(2), Constraint::Fill(3)]);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::constraint`
Expected: PASS (12 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/constraint.rs
git commit -m "feat(layout): add Constraint primitive"
```

---

### Task 9: `Rect`

**Files:**
- Create: `src/layout/rect.rs`

- [ ] **Step 1: Write the file with tests**

```rust
use std::cmp::{max, min};
use std::fmt;

use super::{Margin, Offset, Position, Size};

/// A rectangular area in the terminal.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const ZERO: Self = Self::new(0, 0, 0, 0);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self::new(0, 0, u16::MAX, u16::MAX);

    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        let width = x.saturating_add(width).saturating_sub(x);
        let height = y.saturating_add(height).saturating_sub(y);
        Self { x, y, width, height }
    }

    pub const fn area(self) -> u32 {
        self.width as u32 * self.height as u32
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub const fn left(self) -> u16 {
        self.x
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn top(self) -> u16 {
        self.y
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub const fn row(self) -> u16 {
        self.y
    }

    pub const fn col(self) -> u16 {
        self.x
    }

    pub const fn inner(self, margin: Margin) -> Self {
        let doubled_h = margin.horizontal.saturating_mul(2);
        let doubled_v = margin.vertical.saturating_mul(2);
        if self.width < doubled_h || self.height < doubled_v {
            Self::ZERO
        } else {
            Self {
                x: self.x.saturating_add(margin.horizontal),
                y: self.y.saturating_add(margin.vertical),
                width: self.width.saturating_sub(doubled_h),
                height: self.height.saturating_sub(doubled_v),
            }
        }
    }

    pub const fn outer(self, margin: Margin) -> Self {
        let x = self.x.saturating_sub(margin.horizontal);
        let y = self.y.saturating_sub(margin.vertical);
        let width = self.right().saturating_add(margin.horizontal).saturating_sub(x);
        let height = self.bottom().saturating_add(margin.vertical).saturating_sub(y);
        Self { x, y, width, height }
    }

    pub fn offset(self, offset: Offset) -> Self {
        self + offset
    }

    pub const fn resize(self, size: Size) -> Self {
        Self {
            width: self.x.saturating_add(size.width).saturating_sub(self.x),
            height: self.y.saturating_add(size.height).saturating_sub(self.y),
            ..self
        }
    }

    pub fn union(self, other: Self) -> Self {
        let x1 = min(self.x, other.x);
        let y1 = min(self.y, other.y);
        let x2 = max(self.right(), other.right());
        let y2 = max(self.bottom(), other.bottom());
        Self {
            x: x1,
            y: y1,
            width: x2.saturating_sub(x1),
            height: y2.saturating_sub(y1),
        }
    }

    pub fn intersection(self, other: Self) -> Self {
        let x1 = max(self.x, other.x);
        let y1 = max(self.y, other.y);
        let x2 = min(self.right(), other.right());
        let y2 = min(self.bottom(), other.bottom());
        Self {
            x: x1,
            y: y1,
            width: x2.saturating_sub(x1),
            height: y2.saturating_sub(y1),
        }
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    pub const fn contains(self, position: Position) -> bool {
        position.x >= self.x
            && position.x < self.right()
            && position.y >= self.y
            && position.y < self.bottom()
    }

    pub fn clamp(self, other: Self) -> Self {
        let width = self.width.min(other.width);
        let height = self.height.min(other.height);
        let x = self.x.clamp(other.x, other.right().saturating_sub(width));
        let y = self.y.clamp(other.y, other.bottom().saturating_sub(height));
        Self::new(x, y, width, height)
    }

    pub const fn rows(self) -> Rows {
        Rows::new(self)
    }

    pub const fn columns(self) -> Columns {
        Columns::new(self)
    }

    pub const fn positions(self) -> Positions {
        Positions::new(self)
    }

    pub const fn as_position(self) -> Position {
        Position { x: self.x, y: self.y }
    }

    pub const fn as_size(self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}

impl From<(Position, Size)> for Rect {
    fn from((position, size): (Position, Size)) -> Self {
        Self {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    }
}

impl From<Size> for Rect {
    fn from(size: Size) -> Self {
        Self {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        }
    }
}

impl std::ops::Add<Offset> for Rect {
    type Output = Self;
    fn add(self, offset: Offset) -> Self::Output {
        let max = i32::from(u16::MAX);
        let x = i32::from(self.x).saturating_add(i32::from(offset.x)).clamp(0, max) as u16;
        let y = i32::from(self.y).saturating_add(i32::from(offset.y)).clamp(0, max) as u16;
        Self { x, y, ..self }
    }
}

impl std::ops::Sub<Offset> for Rect {
    type Output = Self;
    fn sub(self, offset: Offset) -> Self::Output {
        let max = i32::from(u16::MAX);
        let x = i32::from(self.x).saturating_sub(i32::from(offset.x)).clamp(0, max) as u16;
        let y = i32::from(self.y).saturating_sub(i32::from(offset.y)).clamp(0, max) as u16;
        Self { x, y, ..self }
    }
}

/// Iterator over rows within a Rect.
#[derive(Debug, Clone)]
pub struct Rows {
    rect: Rect,
    current: u16,
}

impl Rows {
    pub const fn new(rect: Rect) -> Self {
        Self { rect, current: 0 }
    }
}

impl Iterator for Rows {
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.rect.height {
            return None;
        }
        let row = Rect {
            x: self.rect.x,
            y: self.rect.y + self.current,
            width: self.rect.width,
            height: 1,
        };
        self.current += 1;
        Some(row)
    }
}

/// Iterator over columns within a Rect.
#[derive(Debug, Clone)]
pub struct Columns {
    rect: Rect,
    current: u16,
}

impl Columns {
    pub const fn new(rect: Rect) -> Self {
        Self { rect, current: 0 }
    }
}

impl Iterator for Columns {
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.rect.width {
            return None;
        }
        let col = Rect {
            x: self.rect.x + self.current,
            y: self.rect.y,
            width: 1,
            height: self.rect.height,
        };
        self.current += 1;
        Some(col)
    }
}

/// Iterator over all positions within a Rect.
#[derive(Debug, Clone)]
pub struct Positions {
    rect: Rect,
    current: u16,
}

impl Positions {
    pub const fn new(rect: Rect) -> Self {
        Self { rect, current: 0 }
    }
}

impl Iterator for Positions {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        let area = self.rect.area();
        if self.current as u32 >= area {
            return None;
        }
        let x = self.rect.x + (self.current % self.rect.width);
        let y = self.rect.y + (self.current / self.rect.width);
        self.current += 1;
        Some(Position { x, y })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_new() {
        let r = Rect::new(1, 2, 3, 4);
        assert_eq!(r.x, 1);
        assert_eq!(r.y, 2);
        assert_eq!(r.width, 3);
        assert_eq!(r.height, 4);
    }

    #[test]
    fn rect_new_clamps_overflow() {
        let r = Rect::new(u16::MAX - 5, u16::MAX - 3, 100, 100);
        assert_eq!(r.width, 6);
        assert_eq!(r.height, 4);
    }

    #[test]
    fn rect_area() {
        assert_eq!(Rect::new(0, 0, 3, 4).area(), 12);
        assert_eq!(Rect::ZERO.area(), 0);
    }

    #[test]
    fn rect_is_empty() {
        assert!(Rect::new(0, 0, 0, 5).is_empty());
        assert!(Rect::new(0, 0, 5, 0).is_empty());
        assert!(!Rect::new(0, 0, 1, 1).is_empty());
    }

    #[test]
    fn rect_edges() {
        let r = Rect::new(1, 2, 3, 4);
        assert_eq!(r.left(), 1);
        assert_eq!(r.right(), 4);
        assert_eq!(r.top(), 2);
        assert_eq!(r.bottom(), 6);
    }

    #[test]
    fn rect_row_col_compat() {
        let r = Rect::new(5, 10, 1, 1);
        assert_eq!(r.row(), 10);
        assert_eq!(r.col(), 5);
    }

    #[test]
    fn rect_inner() {
        let r = Rect::new(0, 0, 10, 10).inner(Margin::new(2, 3));
        assert_eq!(r, Rect::new(2, 3, 6, 4));
    }

    #[test]
    fn rect_inner_zero_when_margin_too_large() {
        let r = Rect::new(0, 0, 3, 3).inner(Margin::new(2, 2));
        assert_eq!(r, Rect::ZERO);
    }

    #[test]
    fn rect_outer() {
        let r = Rect::new(10, 20, 5, 5).outer(Margin::new(2, 3));
        assert_eq!(r, Rect::new(8, 17, 9, 11));
    }

    #[test]
    fn rect_outer_saturates() {
        let r = Rect::new(0, 0, 5, 5).outer(Margin::new(10, 10));
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
    }

    #[test]
    fn rect_offset() {
        let r = Rect::new(5, 5, 10, 10).offset(Offset::new(3, -2));
        assert_eq!(r, Rect::new(8, 3, 10, 10));
    }

    #[test]
    fn rect_offset_clamps() {
        let r = Rect::new(0, 0, 1, 1).offset(Offset::new(-5, -5));
        assert_eq!(r, Rect::new(0, 0, 1, 1));
    }

    #[test]
    fn rect_resize() {
        let r = Rect::new(1, 1, 5, 5).resize(Size::new(3, 3));
        assert_eq!(r, Rect::new(1, 1, 3, 3));
    }

    #[test]
    fn rect_resize_clamps() {
        let r = Rect::new(u16::MAX - 2, u16::MAX - 1, 1, 1).resize(Size::new(10, 10));
        assert_eq!(r.width, 3);
        assert_eq!(r.height, 2);
    }

    #[test]
    fn rect_union() {
        let a = Rect::new(0, 0, 5, 5);
        let b = Rect::new(3, 3, 5, 5);
        assert_eq!(a.union(b), Rect::new(0, 0, 8, 8));
    }

    #[test]
    fn rect_intersection() {
        let a = Rect::new(0, 0, 5, 5);
        let b = Rect::new(3, 3, 5, 5);
        assert_eq!(a.intersection(b), Rect::new(3, 3, 2, 2));
    }

    #[test]
    fn rect_intersection_no_overlap() {
        let a = Rect::new(0, 0, 2, 2);
        let b = Rect::new(5, 5, 2, 2);
        assert_eq!(a.intersection(b), Rect::new(5, 5, 0, 0));
    }

    #[test]
    fn rect_intersects() {
        assert!(Rect::new(0, 0, 5, 5).intersects(Rect::new(3, 3, 5, 5)));
        assert!(!Rect::new(0, 0, 2, 2).intersects(Rect::new(5, 5, 2, 2)));
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(1, 1, 3, 3);
        assert!(r.contains(Position::new(1, 1)));
        assert!(r.contains(Position::new(3, 3)));
        assert!(!r.contains(Position::new(0, 1)));
        assert!(!r.contains(Position::new(4, 4)));
    }

    #[test]
    fn rect_clamp() {
        let area = Rect::new(0, 0, 100, 100);
        let r = Rect::new(80, 80, 30, 30).clamp(area);
        assert_eq!(r, Rect::new(70, 70, 30, 30));
    }

    #[test]
    fn rect_rows() {
        let rows: Vec<_> = Rect::new(0, 0, 3, 2).rows().collect();
        assert_eq!(rows, vec![Rect::new(0, 0, 3, 1), Rect::new(0, 1, 3, 1)]);
    }

    #[test]
    fn rect_columns() {
        let cols: Vec<_> = Rect::new(0, 0, 2, 3).columns().collect();
        assert_eq!(cols, vec![Rect::new(0, 0, 1, 3), Rect::new(1, 0, 1, 3)]);
    }

    #[test]
    fn rect_positions() {
        let positions: Vec<_> = Rect::new(1, 1, 2, 2).positions().collect();
        assert_eq!(
            positions,
            vec![
                Position::new(1, 1),
                Position::new(2, 1),
                Position::new(1, 2),
                Position::new(2, 2),
            ]
        );
    }

    #[test]
    fn rect_as_position() {
        assert_eq!(Rect::new(5, 10, 1, 1).as_position(), Position::new(5, 10));
    }

    #[test]
    fn rect_as_size() {
        assert_eq!(Rect::new(0, 0, 5, 7).as_size(), Size::new(5, 7));
    }

    #[test]
    fn rect_display() {
        assert_eq!(Rect::new(1, 2, 3, 4).to_string(), "3x4+1+2");
    }

    #[test]
    fn rect_from_position_and_size() {
        let p = Position::new(1, 2);
        let s = Size::new(3, 4);
        let r: Rect = (p, s).into();
        assert_eq!(r, Rect::new(1, 2, 3, 4));
    }

    #[test]
    fn rect_from_size() {
        let r: Rect = Size::new(5, 7).into();
        assert_eq!(r, Rect::new(0, 0, 5, 7));
    }

    #[test]
    fn rect_add_offset() {
        let r = Rect::new(1, 2, 3, 4) + Offset::new(5, -1);
        assert_eq!(r, Rect::new(6, 1, 3, 4));
    }

    #[test]
    fn rect_sub_offset() {
        let r = Rect::new(5, 5, 3, 4) - Offset::new(2, 3);
        assert_eq!(r, Rect::new(3, 2, 3, 4));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib layout::rect`
Expected: PASS (28 tests)

- [ ] **Step 3: Commit**

```bash
git add src/layout/rect.rs
git commit -m "feat(layout): add Rect primitive with geometric ops and iterators"
```

---

### Task 10: `layout/mod.rs`

**Files:**
- Create: `src/layout/mod.rs`

- [ ] **Step 1: Write the file**

```rust
pub mod constraint;
pub mod direction;
pub mod flex;
pub mod margin;
pub mod offset;
pub mod position;
pub mod rect;
pub mod size;
pub mod spacing;

pub use constraint::Constraint;
pub use direction::Direction;
pub use flex::Flex;
pub use margin::Margin;
pub use offset::Offset;
pub use position::Position;
pub use rect::{Columns, Positions, Rect, Rows};
pub use size::Size;
pub use spacing::Spacing;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully (no errors from the new layout module)

- [ ] **Step 3: Commit**

```bash
git add src/layout/mod.rs
git commit -m "feat(layout): add layout module re-exports"
```

---

### Task 11: Update `src/lib.rs`

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Add layout module declaration**

Insert after `pub mod utils;`:
```rust
pub mod layout;
```

The relevant section of `src/lib.rs` should read:
```rust
pub mod autocomplete;
pub mod components;
pub mod events;
pub mod fuzzy;
pub mod image;
pub mod keybindings;
pub mod kill_ring;
pub mod layout;
pub mod renderer;
pub mod terminal;
pub mod theme;
pub mod tui;
pub mod undo_stack;
pub mod utils;
pub mod word_navigation;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat(layout): expose layout module in lib.rs"
```


---

### Task 12: Migrate Existing `Rect` in `src/tui.rs`

**Files:**
- Modify: `src/tui.rs`

- [ ] **Step 1: Remove inline `Rect` and use `crate::layout::Rect`**

Remove the existing `Rect` struct definition from `src/tui.rs` (lines 49-59):
```rust
/// A rectangular region on the terminal screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Row (y) coordinate.
    pub row: u16,
    /// Column (x) coordinate.
    pub col: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}
```

Replace it with:
```rust
pub use crate::layout::Rect;
```

- [ ] **Step 2: Update `Overlay::compute_position` to use new `Rect` field names**

In `Overlay::compute_position`, change the `Some(Rect { ... })` return from:
```rust
Some(Rect {
    row: (row as i16 + self.constraints.offset_y).max(0) as u16,
    col: (col as i16 + self.constraints.offset_x).max(0) as u16,
    width: w.min(term_w.saturating_sub(col)),
    height: h.min(term_h.saturating_sub(row)),
})
```
to:
```rust
Some(Rect {
    y: (row as i16 + self.constraints.offset_y).max(0) as u16,
    x: (col as i16 + self.constraints.offset_x).max(0) as u16,
    width: w.min(term_w.saturating_sub(col)),
    height: h.min(term_h.saturating_sub(row)),
})
```

- [ ] **Step 3: Update `render_frame` overlay blitting**

Change:
```rust
rendered.blit_onto(&mut screen, rect.row, rect.col);
```
to:
```rust
rendered.blit_onto(&mut screen, rect.y, rect.x);
```

- [ ] **Step 4: Update `compose_screen` to not reference `Rect` fields**

No field references here; just verify it still compiles.

- [ ] **Step 5: Update in-file tests that construct `Rect`**

Find all `Rect { row: ..., col: ... }` in the `#[cfg(test)]` module of `src/tui.rs` and change to `Rect { y: ..., x: ... }`.

Also update assertions that check `.row` or `.col` to use `.y` and `.x`.

- [ ] **Step 6: Update `tests/tui_tests.rs`**

In `tests/tui_tests.rs`, find all assertions using `.row` or `.col` on `Rect` and change to `.y` and `.x`. For example:
```rust
assert_eq!(rect.row, 5);
assert_eq!(rect.col, 10);
```
becomes:
```rust
assert_eq!(rect.y, 5);
assert_eq!(rect.x, 10);
```

- [ ] **Step 7: Run tests**

Run: `cargo test --lib tui && cargo test --test tui_tests`
Expected: PASS (all existing tui tests pass)

- [ ] **Step 8: Commit**

```bash
git add src/tui.rs tests/tui_tests.rs
git commit -m "refactor(tui): migrate Rect to layout::Rect"
```

---

### Task 13: Add `blit_into_rect` to `src/renderer.rs`

**Files:**
- Modify: `src/renderer.rs`

- [ ] **Step 1: Add `use crate::layout::Rect;` at the top of `src/renderer.rs`**

- [ ] **Step 2: Add `blit_into_rect` method to `Rendered`**

Add after the existing `blit_onto` method (after line 78):

```rust
    /// Composite this rendered content into a target at the given rect.
    ///
    /// Lines are clipped to `rect.height`. Each line is inserted at `rect.x`
    /// and truncated to `rect.width`. The cursor and images are translated.
    pub fn blit_into_rect(&self, target: &mut Rendered, rect: crate::layout::Rect) {
        for (i, line) in self.lines.iter().enumerate().take(rect.height as usize) {
            let target_row = rect.y as usize + i;
            if target_row >= target.lines.len() {
                // Pad target with empty lines up to target_row
                while target.lines.len() <= target_row {
                    target.lines.push(String::new());
                }
            }
            let col = rect.x as usize;
            let target_line = &mut target.lines[target_row];
            // Pad target line so insertion has something to overwrite
            if target_line.len() < col {
                target_line.push_str(&" ".repeat(col - target_line.len()));
            }
            // Truncate source line to fit within rect.width
            let source = if line.len() > rect.width as usize {
                &line[..rect.width as usize]
            } else {
                line.as_str()
            };
            // Replace the segment [col..col+source.len()]
            let end = col + source.len();
            if end > target_line.len() {
                target_line.push_str(&" ".repeat(end - target_line.len()));
            }
            target_line.replace_range(col..end, source);
        }
        if let Some((r, c)) = self.cursor {
            target.cursor = Some((rect.y as usize + r, rect.x as usize + c));
        }
        target.images.extend(self.images.clone());
    }
```

- [ ] **Step 3: Add tests for `blit_into_rect`**

Add to the `#[cfg(test)]` module in `src/renderer.rs`:

```rust
    #[test]
    fn blit_into_rect_basic() {
        let mut target = Rendered {
            lines: vec!["hello world".into(), "second line".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into(), "Z".into()],
            cursor: Some((0, 1)),
            images: vec![ImageCommand { id: 1, data: "img".into() }],
        };
        source.blit_into_rect(&mut target, crate::layout::Rect::new(6, 0, 10, 2));
        assert_eq!(target.lines[0], "hello XYrld");
        assert_eq!(target.lines[1], "second Zine");
        assert_eq!(target.cursor, Some((0, 7)));
        assert_eq!(target.images.len(), 1);
    }

    #[test]
    fn blit_into_rect_clips_height() {
        let mut target = Rendered {
            lines: vec!["aaaaaaaaaa".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["1".into(), "2".into(), "3".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, crate::layout::Rect::new(0, 0, 10, 1));
        assert_eq!(target.lines[0], "1");
        assert_eq!(target.lines.len(), 1);
    }

    #[test]
    fn blit_into_rect_clips_width() {
        let mut target = Rendered {
            lines: vec!["aaaaaaaaaa".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["1234567890ABCDEF".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, crate::layout::Rect::new(0, 0, 5, 1));
        assert_eq!(target.lines[0], "12345");
    }

    #[test]
    fn blit_into_rect_pads_short_target() {
        let mut target = Rendered {
            lines: vec!["hi".into()],
            cursor: None,
            images: Vec::new(),
        };
        let source = Rendered {
            lines: vec!["XY".into()],
            cursor: None,
            images: Vec::new(),
        };
        source.blit_into_rect(&mut target, crate::layout::Rect::new(5, 0, 10, 1));
        assert_eq!(target.lines[0], "hi   XY");
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib renderer`
Expected: PASS (all existing + 4 new tests)

- [ ] **Step 5: Commit**

```bash
git add src/renderer.rs
git commit -m "feat(renderer): add blit_into_rect for rect-aware compositing"
```

---

### Task 14: Integration Tests — `Rect` Combinations

**Files:**
- Create: `tests/layout_rect_tests.rs`

- [ ] **Step 1: Write the test file**

```rust
use photon_ui::layout::{Margin, Offset, Position, Rect, Size};

#[test]
fn rect_inner_preserves_area_when_no_margin() {
    let r = Rect::new(0, 0, 10, 10);
    assert_eq!(r.inner(Margin::new(0, 0)), r);
}

#[test]
fn rect_outer_then_inner_roundtrip() {
    let r = Rect::new(10, 10, 20, 20);
    let m = Margin::new(2, 3);
    assert_eq!(r.outer(m).inner(m), r);
}

#[test]
fn rect_union_with_self() {
    let r = Rect::new(1, 2, 3, 4);
    assert_eq!(r.union(r), r);
}

#[test]
fn rect_intersection_with_self() {
    let r = Rect::new(1, 2, 3, 4);
    assert_eq!(r.intersection(r), r);
}

#[test]
fn rect_intersection_commutative() {
    let a = Rect::new(0, 0, 5, 5);
    let b = Rect::new(3, 3, 5, 5);
    assert_eq!(a.intersection(b), b.intersection(a));
}

#[test]
fn rect_contains_corners() {
    let r = Rect::new(0, 0, 3, 3);
    assert!(r.contains(Position::new(0, 0)));
    assert!(r.contains(Position::new(2, 2)));
    assert!(!r.contains(Position::new(3, 3)));
}

#[test]
fn rect_clamp_already_inside() {
    let outer = Rect::new(0, 0, 100, 100);
    let inner = Rect::new(10, 10, 5, 5);
    assert_eq!(inner.clamp(outer), inner);
}

#[test]
fn rect_rows_count_matches_height() {
    let r = Rect::new(0, 0, 5, 3);
    assert_eq!(r.rows().count(), 3);
}

#[test]
fn rect_columns_count_matches_width() {
    let r = Rect::new(0, 0, 4, 6);
    assert_eq!(r.columns().count(), 4);
}

#[test]
fn rect_positions_count_matches_area() {
    let r = Rect::new(0, 0, 3, 4);
    assert_eq!(r.positions().count(), 12);
}

#[test]
fn rect_resize_to_zero() {
    let r = Rect::new(5, 5, 10, 10).resize(Size::ZERO);
    assert_eq!(r, Rect::new(5, 5, 0, 0));
}

#[test]
fn rect_offset_by_zero_is_unchanged() {
    let r = Rect::new(1, 2, 3, 4);
    assert_eq!(r.offset(Offset::new(0, 0)), r);
}

#[test]
fn rect_intersects_edge_case_touching() {
    let a = Rect::new(0, 0, 2, 2);
    let b = Rect::new(2, 0, 2, 2);
    assert!(!a.intersects(b));
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test layout_rect_tests`
Expected: PASS (12 tests)

- [ ] **Step 3: Commit**

```bash
git add tests/layout_rect_tests.rs
git commit -m "test(layout): add Rect integration tests"
```

---

### Task 15: Integration Tests — `Position` + `Offset`

**Files:**
- Create: `tests/layout_position_tests.rs`

- [ ] **Step 1: Write the test file**

```rust
use photon_ui::layout::{Offset, Position};

#[test]
fn position_add_offset_roundtrip() {
    let p = Position::new(50, 50);
    let o = Offset::new(10, -5);
    assert_eq!((p + o) - o, p);
}

#[test]
fn position_add_offset_clamps_at_max() {
    let p = Position::MAX;
    let o = Offset::new(1, 1);
    assert_eq!(p + o, Position::MAX);
}

#[test]
fn position_sub_offset_clamps_at_min() {
    let p = Position::ORIGIN;
    let o = Offset::new(1, 1);
    assert_eq!(p - o, Position::ORIGIN);
}

#[test]
fn position_add_large_positive_offset_clamps() {
    let p = Position::new(100, 100);
    let o = Offset::new(1000, 1000);
    let result = p + o;
    assert_eq!(result, Position::MAX);
}

#[test]
fn position_sub_large_negative_offset_clamps() {
    let p = Position::new(100, 100);
    let o = Offset::new(-1000, -1000);
    let result = p - o;
    assert_eq!(result, Position::MAX);
}

#[test]
fn position_add_sub_assign() {
    let mut p = Position::new(10, 10);
    p += Offset::new(5, 3);
    assert_eq!(p, Position::new(15, 13));
    p -= Offset::new(2, 4);
    assert_eq!(p, Position::new(13, 9));
}

#[test]
fn position_origin_plus_offset() {
    assert_eq!(
        Position::ORIGIN + Offset::new(5, 7),
        Position::new(5, 7)
    );
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test layout_position_tests`
Expected: PASS (7 tests)

- [ ] **Step 3: Commit**

```bash
git add tests/layout_position_tests.rs
git commit -m "test(layout): add Position + Offset integration tests"
```

---

### Task 16: Integration Tests — `Constraint`

**Files:**
- Create: `tests/layout_constraint_tests.rs`

- [ ] **Step 1: Write the test file**

```rust
use photon_ui::layout::Constraint;

#[test]
fn constraint_from_lengths_vec() {
    let c = Constraint::from_lengths(vec![1, 2, 3]);
    assert_eq!(c, vec![Constraint::Length(1), Constraint::Length(2), Constraint::Length(3)]);
}

#[test]
fn constraint_from_ratios_vec() {
    let c = Constraint::from_ratios(vec![(1, 2), (1, 3)]);
    assert_eq!(c, vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 3)]);
}

#[test]
fn constraint_from_percentages_vec() {
    let c = Constraint::from_percentages(vec![25, 50, 25]);
    assert_eq!(c, vec![Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)]);
}

#[test]
fn constraint_all_variants_display() {
    let variants = vec![
        Constraint::Min(1),
        Constraint::Max(2),
        Constraint::Length(3),
        Constraint::Percentage(50),
        Constraint::Ratio(1, 4),
        Constraint::Fill(1),
    ];
    let displays: Vec<String> = variants.iter().map(|c| c.to_string()).collect();
    assert_eq!(displays, vec![
        "Min(1)",
        "Max(2)",
        "Length(3)",
        "Percentage(50)",
        "Ratio(1, 4)",
        "Fill(1)",
    ]);
}

#[test]
fn constraint_from_u16_is_length() {
    let c: Constraint = 42.into();
    assert_eq!(c, Constraint::Length(42));
}

#[test]
fn constraint_as_ref_returns_self() {
    let c = Constraint::Fill(3);
    assert_eq!(c.as_ref(), &Constraint::Fill(3));
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test layout_constraint_tests`
Expected: PASS (6 tests)

- [ ] **Step 3: Commit**

```bash
git add tests/layout_constraint_tests.rs
git commit -m "test(layout): add Constraint integration tests"
```

---

### Task 17: Property-Based Integration Tests

**Files:**
- Create: `tests/layout_type_integration_tests.rs`

- [ ] **Step 1: Write the test file**

```rust
use photon_ui::layout::{Margin, Offset, Position, Rect, Size};
use proptest::prelude::*;

proptest! {
    #[test]
    fn rect_intersection_commutative(a in rect_strategy(), b in rect_strategy()) {
        prop_assert_eq!(a.intersection(b), b.intersection(a));
    }

    #[test]
    fn rect_intersects_symmetric(a in rect_strategy(), b in rect_strategy()) {
        prop_assert_eq!(a.intersects(b), b.intersects(a));
    }

    #[test]
    fn rect_union_contains_both(a in rect_strategy(), b in rect_strategy()) {
        let u = a.union(b);
        prop_assert!(u.intersects(a));
        prop_assert!(u.intersects(b));
    }

    #[test]
    fn rect_clamp_fits_inside(other in rect_strategy()) {
        let fixed = Rect::new(10, 10, 50, 50);
        let clamped = fixed.clamp(other);
        prop_assert!(clamped.x >= other.x);
        prop_assert!(clamped.y >= other.y);
        prop_assert!(clamped.right() <= other.right());
        prop_assert!(clamped.bottom() <= other.bottom());
    }

    #[test]
    fn rect_inner_area_less_or_equal(r in rect_strategy(), m in margin_strategy()) {
        let inner = r.inner(m);
        prop_assert!(inner.area() <= r.area());
    }

    #[test]
    fn position_add_sub_offset_roundtrip(
        p in position_strategy(),
        o in offset_strategy()
    ) {
        let result = (p + o) - o;
        // Due to clamping, roundtrip only holds when no overflow occurred
        let no_overflow_x = i32::from(p.x) + i32::from(o.x) >= 0
            && i32::from(p.x) + i32::from(o.x) <= i32::from(u16::MAX);
        let no_overflow_y = i32::from(p.y) + i32::from(o.y) >= 0
            && i32::from(p.y) + i32::from(o.y) <= i32::from(u16::MAX);
        if no_overflow_x && no_overflow_y {
            prop_assert_eq!(result, p);
        }
    }

    #[test]
    fn rect_positions_count_equals_area(r in non_empty_rect_strategy()) {
        prop_assert_eq!(r.positions().count() as u32, r.area());
    }

    #[test]
    fn rect_rows_count_equals_height(r in non_empty_rect_strategy()) {
        prop_assert_eq!(r.rows().count() as u16, r.height);
    }

    #[test]
    fn rect_columns_count_equals_width(r in non_empty_rect_strategy()) {
        prop_assert_eq!(r.columns().count() as u16, r.width);
    }
}

fn rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..200, 0u16..200, 0u16..100, 0u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn non_empty_rect_strategy() -> impl Strategy<Value = Rect> {
    (0u16..200, 0u16..200, 1u16..100, 1u16..100)
        .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn margin_strategy() -> impl Strategy<Value = Margin> {
    (0u16..20, 0u16..20).prop_map(|(h, v)| Margin::new(h, v))
}

fn position_strategy() -> impl Strategy<Value = Position> {
    (0u16..300, 0u16..300).prop_map(|(x, y)| Position::new(x, y))
}

fn offset_strategy() -> impl Strategy<Value = Offset> {
    (-100i16..100, -100i16..100).prop_map(|(x, y)| Offset::new(x, y))
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test layout_type_integration_tests`
Expected: PASS (all proptest cases pass)

- [ ] **Step 3: Commit**

```bash
git add tests/layout_type_integration_tests.rs
git commit -m "test(layout): add cross-type property-based integration tests"
```

---

### Task 18: Full Test Suite Verification

**Files:**
- All files

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: All tests pass (existing + new)

- [ ] **Step 2: Check test coverage**

Run: `cargo llvm-cov --lib --html`
Then open `target/llvm-cov-target/html/index.html` and verify `src/layout/` coverage is >90%.

If coverage is low on any file, add targeted unit tests to that file's `#[cfg(test)]` module.

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "test(layout): verify >90% coverage on layout primitives"
```

---

## Self-Review Checklist

**1. Spec coverage:**
- `Rect` with all geometric ops + iterators → Task 9, Task 14, Task 17
- `Position` with offset arithmetic → Task 7, Task 15, Task 17
- `Size` → Task 2
- `Margin` → Task 3
- `Offset` → Task 1, Task 15, Task 17
- `Direction` → Task 4
- `Flex` → Task 5
- `Constraint` with all constructors → Task 8, Task 16
- `Spacing` → Task 6
- `layout/mod.rs` re-exports → Task 10
- `lib.rs` integration → Task 11
- `tui.rs` Rect migration → Task 12
- `renderer.rs` `blit_into_rect` → Task 13
- Property-based tests → Task 17

**2. Placeholder scan:** No TBD, TODO, "implement later", or vague steps. Every step has exact code, exact commands, expected output.

**3. Type consistency:** All method signatures and type names match the spec. `Rect::row()`/`col()` compatibility accessors are included. `blit_into_rect` takes `crate::layout::Rect`.
