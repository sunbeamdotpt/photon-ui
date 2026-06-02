//! Cassowary solver strength constants for layout constraints.
//!
//! Higher strength = higher priority. The solver tries to satisfy stronger
//! constraints before weaker ones. Strengths are expressed as floating-point
//! values for kasuari.

use kasuari::Strength;

/// REQUIRED strength — must be satisfied absolutely.
pub const REQUIRED: f64 = Strength::REQUIRED.value();

/// Strong strength — area bounds and variable ordering.
pub const STRONG: f64 = Strength::STRONG.value();

/// Medium strength — size equality constraints (Length, Percentage, Ratio).
pub const MEDIUM: f64 = Strength::MEDIUM.value();

/// Weak strength — grow constraints (Fill, flex distribution).
pub const WEAK: f64 = Strength::WEAK.value();

/// Spacer size equality (highest non-required).
pub const SPACER_SIZE_EQ: f64 = REQUIRED * 0.999;

/// Minimum size ≥ constraint.
pub const MIN_SIZE_GE: f64 = STRONG * 1.5;

/// Maximum size ≤ constraint.
pub const MAX_SIZE_LE: f64 = STRONG * 1.4;

/// Exact length.
pub const LENGTH_SIZE_EQ: f64 = MEDIUM * 1.5;

/// Percentage of parent area.
pub const PERCENTAGE_SIZE_EQ: f64 = MEDIUM * 1.4;

/// Ratio of parent area.
pub const RATIO_SIZE_EQ: f64 = MEDIUM * 1.3;

/// Maximum size equality (for Max constraint).
pub const MAX_SIZE_EQ: f64 = MEDIUM * 1.2;

/// Minimum size equality (for Min constraint).
pub const MIN_SIZE_EQ: f64 = MEDIUM * 1.1;

/// Fill grow — lowest constraint priority.
pub const FILL_GROW: f64 = WEAK * 1.5;

/// Generic grow.
pub const GROW: f64 = WEAK * 1.4;

/// Space grow — flex distribution.
pub const SPACE_GROW: f64 = WEAK * 1.3;

/// All segments grow equally.
pub const ALL_SEGMENT_GROW: f64 = WEAK * 1.0;
