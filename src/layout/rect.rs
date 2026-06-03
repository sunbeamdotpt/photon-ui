use std::{
    cmp::{
        max,
        min,
    },
    fmt,
};

use super::{
    Margin,
    Offset,
    Position,
    Size,
};

/// A rectangular area in the terminal.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Rect {
    /// Column of the top-left corner.
    pub x: u16,
    /// Row of the top-left corner.
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl Rect {
    /// A rect that covers the entire addressable terminal area.
    pub const MAX: Self = Self::new(0, 0, u16::MAX, u16::MAX);
    /// The smallest possible rect (same as [`ZERO`](Rect::ZERO)).
    pub const MIN: Self = Self::ZERO;
    /// A zero-width, zero-height rect at the origin.
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    /// Create a new rect, clamping width/height so the rect does not overflow
    /// `u16`.
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        let width = x.saturating_add(width).saturating_sub(x);
        let height = y.saturating_add(height).saturating_sub(y);
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Total number of cells inside this rect.
    pub const fn area(self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Returns `true` if the rect has zero width or height.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// The leftmost column (same as `x`).
    pub const fn left(self) -> u16 {
        self.x
    }

    /// The first column after the right edge (`x + width`).
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// The topmost row (same as `y`).
    pub const fn top(self) -> u16 {
        self.y
    }

    /// The first row after the bottom edge (`y + height`).
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// The row of the top edge (same as `y`).
    pub const fn row(self) -> u16 {
        self.y
    }

    /// The column of the left edge (same as `x`).
    pub const fn col(self) -> u16 {
        self.x
    }

    /// Shrink this rect by the given margin on all sides.
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

    /// Expand this rect by the given margin on all sides.
    pub const fn outer(self, margin: Margin) -> Self {
        let x = self.x.saturating_sub(margin.horizontal);
        let y = self.y.saturating_sub(margin.vertical);
        let width = self
            .right()
            .saturating_add(margin.horizontal)
            .saturating_sub(x);
        let height = self
            .bottom()
            .saturating_add(margin.vertical)
            .saturating_sub(y);
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Move this rect by the given offset, clamping to the valid `u16` range.
    pub fn offset(self, offset: Offset) -> Self {
        self + offset
    }

    /// Resize this rect to the given dimensions, keeping the top-left corner.
    pub const fn resize(self, size: Size) -> Self {
        Self {
            width: self.x.saturating_add(size.width).saturating_sub(self.x),
            height: self.y.saturating_add(size.height).saturating_sub(self.y),
            ..self
        }
    }

    /// The smallest rect that contains both `self` and `other`.
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

    /// The overlap between `self` and `other`.
    ///
    /// Returns a zero-area rect if they do not intersect.
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

    /// Returns `true` if `self` and `other` overlap.
    pub const fn intersects(self, other: Self) -> bool {
        self.x < other.right() &&
            self.right() > other.x &&
            self.y < other.bottom() &&
            self.bottom() > other.y
    }

    /// Returns `true` if the given position lies inside this rect.
    pub const fn contains(self, position: Position) -> bool {
        position.x >= self.x &&
            position.x < self.right() &&
            position.y >= self.y &&
            position.y < self.bottom()
    }

    /// Clamp this rect so it fits entirely inside `other`.
    pub fn clamp(self, other: Self) -> Self {
        let width = self.width.min(other.width);
        let height = self.height.min(other.height);
        let x = self.x.clamp(other.x, other.right().saturating_sub(width));
        let y = self.y.clamp(other.y, other.bottom().saturating_sub(height));
        Self::new(x, y, width, height)
    }

    /// Iterate over each row in this rect as a 1-cell-high [`Rect`].
    pub const fn rows(self) -> Rows {
        Rows::new(self)
    }

    /// Iterate over each column in this rect as a 1-cell-wide [`Rect`].
    pub const fn columns(self) -> Columns {
        Columns::new(self)
    }

    /// Iterate over every cell position in this rect.
    pub const fn positions(self) -> Positions {
        Positions::new(self)
    }

    /// Return the top-left corner as a [`Position`].
    pub const fn as_position(self) -> Position {
        Position {
            x: self.x,
            y: self.y,
        }
    }

    /// Return the dimensions as a [`Size`].
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
        let x = i32::from(self.x)
            .saturating_add(i32::from(offset.x))
            .clamp(0, max) as u16;
        let y = i32::from(self.y)
            .saturating_add(i32::from(offset.y))
            .clamp(0, max) as u16;
        Self { x, y, ..self }
    }
}

impl std::ops::Sub<Offset> for Rect {
    type Output = Self;

    fn sub(self, offset: Offset) -> Self::Output {
        let max = i32::from(u16::MAX);
        let x = i32::from(self.x)
            .saturating_sub(i32::from(offset.x))
            .clamp(0, max) as u16;
        let y = i32::from(self.y)
            .saturating_sub(i32::from(offset.y))
            .clamp(0, max) as u16;
        Self { x, y, ..self }
    }
}

/// Iterator over rows within a Rect.
#[derive(Debug, Clone)]
/// Iterator over rows within a Rect.
pub struct Rows {
    rect: Rect,
    current: u16,
}

impl Rows {
    /// Create a new row iterator for the given rect.
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
/// Iterator over columns within a Rect.
pub struct Columns {
    rect: Rect,
    current: u16,
}

impl Columns {
    /// Create a new column iterator for the given rect.
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
/// Iterator over all positions within a Rect.
pub struct Positions {
    rect: Rect,
    current: u16,
}

impl Positions {
    /// Create a new position iterator for the given rect.
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
        assert_eq!(r.width, 5);
        assert_eq!(r.height, 3);
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
        assert_eq!(r.width, 2);
        assert_eq!(r.height, 1);
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
