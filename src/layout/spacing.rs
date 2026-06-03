/** Gap or overlap between layout segments. */
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Spacing {
    /// Positive gap between layout segments.
    Space(u16),
    /// Negative overlap that causes segments to encroach on each other.
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
        assert_eq!(
            Spacing::from(i32::MIN),
            Spacing::Overlap(i16::MAX as u16 + 1)
        );
    }

    #[test]
    fn spacing_from_i32_zero() {
        assert_eq!(Spacing::from(0_i32), Spacing::Space(0));
    }
}
