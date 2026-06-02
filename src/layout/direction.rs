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
