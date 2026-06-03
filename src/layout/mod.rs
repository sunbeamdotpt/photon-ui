//! Constraint-based layout engine.
//!
//! Uses the Cassowary constraint solver (`kasuari`) to split a `Rect`
//! into sub-rects according to a set of `Constraint` values.

/// Border drawing primitives.
pub mod border;
/// Size constraints for layout segments.
pub mod constraint;
/// Horizontal or vertical layout direction.
pub mod direction;
/// How excess space is distributed among segments.
pub mod flex;
/// Layout configuration and splitting.
pub mod layout;
/// Margin around a rectangular area.
pub mod margin;
/// Relative offset in terminal coordinates.
pub mod offset;
/// Absolute position in terminal coordinates.
pub mod position;
/// Terminal rectangle primitives.
pub mod rect;
/// Width/height dimensions.
pub mod size;
/// Gap or overlap between layout segments.
pub mod spacing;
/// Cassowary constraint strengths.
pub mod strengths;

pub use border::{
    Border,
    draw_border,
};
pub use constraint::Constraint;
pub use direction::Direction;
pub use flex::Flex;
pub use margin::Margin;
pub use offset::Offset;
pub use position::Position;
pub use rect::{
    Columns,
    Positions,
    Rect,
    Rows,
};
pub use size::Size;
pub use spacing::Spacing;
