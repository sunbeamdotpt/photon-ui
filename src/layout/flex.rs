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
