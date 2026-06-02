use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::Offset;
use crate::tui::Rect;

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
        Self::new(rect.x, rect.y)
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
    fn position_add_clamps_at_max() {
        let p = Position::MAX;
        let o = Offset::new(1, 1);
        assert_eq!(p + o, Position::MAX);
    }

    #[test]
    fn position_sub_clamps_at_min() {
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
        let rect = Rect { x: 7, y: 8, width: 10, height: 20 };
        let p: Position = rect.into();
        assert_eq!(p, Position::new(7, 8));
    }

    #[test]
    fn position_display() {
        assert_eq!(Position::new(1, 2).to_string(), "(1, 2)");
    }
}
