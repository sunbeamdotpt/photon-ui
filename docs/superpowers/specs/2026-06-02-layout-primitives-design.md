# Layout Primitives — Phase 1 Design

> **Scope:** Introduce all geometric and constraint types required for a Cassowary-based layout engine. No solver. No `Layout::split`. Just solid, well-tested primitives.

## Goal

Build the foundation for a ratatui-compatible layout engine inside photon-ui. Every type lives in `src/layout/` and has exhaustive unit tests plus property-based integration tests. Target: >90% test coverage.

## Architecture

### Module Structure

```
src/
  layout/
    mod.rs        — Re-exports
    rect.rs       — Rect + geometric ops + iterators
    position.rs   — Position
    size.rs       — Size
    margin.rs     — Margin
    offset.rs     — Offset
    direction.rs  — Direction
    flex.rs       — Flex
    constraint.rs — Constraint
    spacing.rs    — Spacing
```

### No New Dependencies

Phase 1 uses only the standard library. `kasuari` (Cassowary solver) is added in Phase 2.

### Existing `Rect` Migration

Photon-ui's current `Rect` in `src/tui.rs` has `row`/`col`/`width`/`height`. Phase 1 replaces it with `layout::Rect` (`x`/`y`/`width`/`height`) and adds `row()`/`col()` accessors for backward compatibility. All existing call sites in `tui.rs`, `renderer.rs`, and tests are updated.

---

## Type Specifications

### `Rect` (`src/layout/rect.rs`)

```rust
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
```

**Constants:**
- `Rect::ZERO` — zero-sized at origin
- `Rect::MAX` — `new(0, 0, u16::MAX, u16::MAX)`

**Construction:**
- `new(x, y, width, height) -> Self` — clamps width/height so `right()` and `bottom()` stay within `u16::MAX`
- `From<(Position, Size)>`, `From<Size>`

**Geometry:**
- `area(self) -> u32` — `width * height` as `u32`
- `is_empty(self) -> bool` — `width == 0 || height == 0`
- `left(self) -> u16` — `x`
- `right(self) -> u16` — `x.saturating_add(width)`
- `top(self) -> u16` — `y`
- `bottom(self) -> u16` — `y.saturating_add(height)`

**Spatial ops:**
- `inner(self, margin: Margin) -> Self` — shrink by margin on all sides; returns `ZERO` if margin exceeds dimensions
- `outer(self, margin: Margin) -> Self` — expand by margin on all sides; clamp to `u16::MAX`
- `offset(self, offset: Offset) -> Self` — move by offset; clamp to `u16` bounds
- `resize(self, size: Size) -> Self` — change size, clamp so right/bottom stay in bounds
- `union(self, other: Self) -> Self` — bounding box of both
- `intersection(self, other: Self) -> Self` — overlap; `ZERO` if no overlap
- `intersects(self, other: Self) -> bool`
- `contains(self, position: Position) -> bool` — inclusive of borders
- `clamp(self, other: Self) -> Self` — move and resize to fit entirely inside `other`

**Compatibility accessors (photon-ui convention):**
- `row(self) -> u16` — returns `y`
- `col(self) -> u16` — returns `x`

**Iteration:**
- `rows(self) -> Rows` — iterator yielding `Rect { height: 1, ... }` for each row
- `columns(self) -> Columns` — iterator yielding `Rect { width: 1, ... }` for each column
- `positions(self) -> Positions` — iterator yielding `Position` for each cell in row-major order

**Conversion:**
- `as_position(self) -> Position`
- `as_size(self) -> Size`

---

### `Position` (`src/layout/position.rs`)

```rust
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}
```

**Constants:** `ORIGIN = new(0, 0)`, `MIN = ORIGIN`, `MAX = new(u16::MAX, u16::MAX)`

**Ops:**
- `new(x, y)`
- `offset(self, offset: Offset) -> Self`
- `Add<Offset>`, `Sub<Offset>` — clamp to `u16` bounds
- `AddAssign<Offset>`, `SubAssign<Offset>`
- `From<(u16, u16)>`, `Into<(u16, u16)>`, `From<Rect>`
- `Display` → `"(x, y)"`

---

### `Size` (`src/layout/size.rs`)

```rust
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}
```

**Constants:** `ZERO = new(0, 0)`, `MIN = ZERO`, `MAX = new(u16::MAX, u16::MAX)`

**Ops:**
- `new(width, height)`
- `area(self) -> u32`
- `From<(u16, u16)>`, `Into<(u16, u16)>`, `From<Rect>`
- `Display` → `"widthxheight"`

---

### `Margin` (`src/layout/margin.rs`)

```rust
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Margin {
    pub horizontal: u16,
    pub vertical: u16,
}
```

**Ops:**
- `new(horizontal, vertical)`
- `From<u16>` — uniform margin
- `Display` → `"horizontalxvertical"`

---

### `Offset` (`src/layout/offset.rs`)

```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset {
    pub x: i16,
    pub y: i16,
}
```

**Constants:** `MAX = new(i16::MAX, i16::MAX)`, `MIN = new(i16::MIN, i16::MIN)`

**Ops:**
- `new(x, y)`
- `Add<Offset>`, `Sub<Offset>`, `Neg` — for `Offset` arithmetic on `Offset` itself
- `Rect` and `Position` also implement `Add<Offset>` and `Sub<Offset>`

---

### `Direction` (`src/layout/direction.rs`)

```rust
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Direction {
    Horizontal,
    #[default]
    Vertical,
}
```

**Ops:**
- `perpendicular(self) -> Self`

---

### `Flex` (`src/layout/flex.rs`)

```rust
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Flex {
    Legacy,
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
```

**Ops:**
- `is_legacy(self) -> bool`

---

### `Constraint` (`src/layout/constraint.rs`)

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Constraint {
    Min(u16),
    Max(u16),
    Length(u16),
    Percentage(u16),
    Ratio(u32, u32),
    Fill(u16),
}
```

**Ops:**
- `from_lengths(iter) -> Vec<Constraint>`
- `from_ratios(iter) -> Vec<Constraint>` — items are `(u32, u32)`
- `from_percentages(iter) -> Vec<Constraint>`
- `from_mins(iter) -> Vec<Constraint>`
- `from_maxes(iter) -> Vec<Constraint>`
- `from_fills(iter) -> Vec<Constraint>`
- `From<u16>` → `Length(u16)`
- `From<&Self>` → copy
- `AsRef<Self>`
- `Default` → `Percentage(100)`
- `Display` → `"Variant(value)"`

---

### `Spacing` (`src/layout/spacing.rs`)

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Spacing {
    Space(u16),
    Overlap(u16),
}
```

**Ops:**
- `Default` → `Space(0)`
- `From<u16>` → `Space`
- `From<i16>` → negative becomes `Overlap`, non-negative `Space`
- `From<i32>` — clamp to `i16` range then same as `From<i16>`

---

## Integration Points

### `src/tui.rs`
- Replace inline `Rect` with `crate::layout::Rect`
- Update `Overlay::compute_position` to return `crate::layout::Rect`
- Update `Rect` field accesses: `rect.row` → `rect.y`, `rect.col` → `rect.x`
- Keep `row()`/`col()` accessors on `Rect` for any external migration

### `src/lib.rs`
- Add `pub mod layout;`
- Re-export key types at crate root if useful

### `src/renderer.rs`
- Add `Rendered::blit_into_rect(&self, target: &mut Rendered, rect: Rect)`:
  1. Iterate `self.lines` up to `rect.height` lines
  2. For each line at index `i`, compute `target_row = rect.y + i`; skip if `target_row >= target.lines.len()`
  3. Pad target line with spaces if its length < `rect.x`
  4. Insert the source line into the target line starting at `rect.x`, truncating the source line so the inserted portion does not exceed `rect.width` characters
  5. Translate `self.cursor` by `(rect.y, rect.x)` if present
  6. Append `self.images` to target (image coordinate translation deferred to Phase 2 when images are rendered in rect context)

---

## Test Strategy

### Unit Tests (in-file `#[cfg(test)]`)
Every public method gets at least one test. Edge cases covered:
- Zero dimensions
- `u16::MAX` saturation
- Negative offsets clamped to zero
- Margins larger than rect
- Non-intersecting rects
- Invalid percentages / ratios (e.g. `Ratio(1, 0)` — avoid panic, treat as `1`)

### Integration Tests (`tests/`)
| File | Focus |
|---|---|
| `tests/layout_rect_tests.rs` | `Rect` combos: inner/outer, union, intersection, clamp, rows/columns/positions iterators |
| `tests/layout_position_tests.rs` | Position + Offset arithmetic, bounds clamping |
| `tests/layout_constraint_tests.rs` | Constraint constructors, `Display`, `From` conversions |
| `tests/layout_type_integration_tests.rs` | Cross-type: Rect + Margin, Position + Offset + Rect, Size + Rect |

### Property-Based Tests (`proptest`)
- `Rect::intersection(a, b) == Rect::intersection(b, a)`
- `Rect::union(a, b)` contains both `a` and `b`
- `Rect::clamp(a, b)` always fits inside `b`
- `Position + Offset - Offset` round-trips (clamping aside)
- `Rect::inner(Margin).area() <= rect.area()` (when margin fits)

---

## Deferred to Phase 2

- `Layout` struct and builder API
- `Layout::split` and Cassowary solver (`kasuari` dependency)
- `Layout::areas`, `Layout::spacers`
- Layout caching (LRU)
- `Rect::layout`, `Rect::centered_*` helpers (depend on `Layout`)
- Layout container component
- Full rect-based image coordinate translation
