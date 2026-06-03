use std::fmt;

/// Spacing around a rectangular area.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Margin {
    /// Horizontal margin (left and right) in columns.
    pub horizontal: u16,
    /// Vertical margin (top and bottom) in rows.
    pub vertical: u16,
}

impl Margin {
    /// Create a new margin with the given horizontal and vertical values.
    pub const fn new(horizontal: u16, vertical: u16) -> Self {
        Self {
            horizontal,
            vertical,
        }
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
