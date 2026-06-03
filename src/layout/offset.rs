/// Relative movement in the terminal coordinate system.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset {
    /// Horizontal offset (positive = right, negative = left).
    pub x: i16,
    /// Vertical offset (positive = down, negative = up).
    pub y: i16,
}

impl Offset {
    /// The largest possible offset.
    pub const MAX: Self = Self::new(i16::MAX, i16::MAX);
    /// The smallest possible offset.
    pub const MIN: Self = Self::new(i16::MIN, i16::MIN);

    /// Create a new offset with the given x and y values.
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Offset {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
        }
    }
}

impl std::ops::Sub for Offset {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }
}

impl std::ops::Neg for Offset {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: self.x.saturating_neg(),
            y: self.y.saturating_neg(),
        }
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
