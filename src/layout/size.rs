use std::fmt;

/// Dimensions in the terminal (width × height).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Size {
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl Size {
    /// The largest possible size.
    pub const MAX: Self = Self::new(u16::MAX, u16::MAX);
    /// The smallest possible size (same as [`ZERO`](Size::ZERO)).
    pub const MIN: Self = Self::ZERO;
    /// Zero width and height.
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new size with the given width and height.
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Total number of cells (`width * height`).
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
