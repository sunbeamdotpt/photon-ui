use std::fmt;

/// A size constraint for layout elements.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Constraint {
    /// Minimum size in terminal columns or rows.
    Min(u16),
    /// Maximum size in terminal columns or rows.
    Max(u16),
    /// Exact size in terminal columns or rows.
    Length(u16),
    /// Percentage of the total available space.
    Percentage(u16),
    /// Proportional fraction of the total space (numerator, denominator).
    Ratio(u32, u32),
    /// Grows to fill remaining space with the given priority.
    Fill(u16),
}

impl Constraint {
    /// Create a vector of [`Constraint::Length`] values.
    pub fn from_lengths<T>(lengths: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>, {
        lengths.into_iter().map(Self::Length).collect()
    }

    /// Create a vector of [`Constraint::Ratio`] values.
    pub fn from_ratios<T>(ratios: T) -> Vec<Self>
    where
        T: IntoIterator<Item = (u32, u32)>, {
        ratios.into_iter().map(|(n, d)| Self::Ratio(n, d)).collect()
    }

    /// Create a vector of [`Constraint::Percentage`] values.
    pub fn from_percentages<T>(percentages: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>, {
        percentages.into_iter().map(Self::Percentage).collect()
    }

    /// Create a vector of [`Constraint::Min`] values.
    pub fn from_mins<T>(mins: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>, {
        mins.into_iter().map(Self::Min).collect()
    }

    /// Create a vector of [`Constraint::Max`] values.
    pub fn from_maxes<T>(maxes: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>, {
        maxes.into_iter().map(Self::Max).collect()
    }

    /// Create a vector of [`Constraint::Fill`] values.
    pub fn from_fills<T>(fills: T) -> Vec<Self>
    where
        T: IntoIterator<Item = u16>, {
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
            | Self::Percentage(p) => write!(f, "Percentage({p})"),
            | Self::Ratio(n, d) => write!(f, "Ratio({n}, {d})"),
            | Self::Length(l) => write!(f, "Length({l})"),
            | Self::Fill(l) => write!(f, "Fill({l})"),
            | Self::Max(m) => write!(f, "Max({m})"),
            | Self::Min(m) => write!(f, "Min({m})"),
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
        assert_eq!(c.as_ref(), &Constraint::Length(5));
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
        assert_eq!(
            c,
            vec![
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(3)
            ]
        );
    }

    #[test]
    fn constraint_from_ratios() {
        let c = Constraint::from_ratios([(1, 4), (1, 2)]);
        assert_eq!(c, vec![Constraint::Ratio(1, 4), Constraint::Ratio(1, 2)]);
    }

    #[test]
    fn constraint_from_percentages() {
        let c = Constraint::from_percentages([25, 50]);
        assert_eq!(
            c,
            vec![Constraint::Percentage(25), Constraint::Percentage(50)]
        );
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
        assert_eq!(
            c,
            vec![
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(3)
            ]
        );
    }
}
