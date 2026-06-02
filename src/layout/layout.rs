use std::hash::{Hash, Hasher};

use kasuari::{Solver, Variable};
use kasuari::WeightedRelation::*;

use super::{Constraint, Direction, Flex, Margin, Rect, Spacing};
use super::strengths;

const FLOAT_PRECISION_MULTIPLIER: f64 = 100.0;

/// A layout configuration that splits a [`Rect`] into sub-rects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: Margin,
    flex: Flex,
    spacing: Spacing,
}

impl Layout {
    pub fn new<I>(direction: Direction, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self {
            direction,
            constraints: constraints.into_iter().map(Into::into).collect(),
            margin: Margin::new(0, 0),
            flex: Flex::default(),
            spacing: Spacing::default(),
        }
    }

    pub fn vertical<I>(constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self::new(Direction::Vertical, constraints)
    }

    pub fn horizontal<I>(constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self::new(Direction::Horizontal, constraints)
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn constraints<I>(mut self, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.constraints = constraints.into_iter().map(Into::into).collect();
        self
    }

    pub fn margin(mut self, margin: u16) -> Self {
        self.margin = Margin::new(margin, margin);
        self
    }

    pub fn horizontal_margin(mut self, margin: u16) -> Self {
        self.margin.horizontal = margin;
        self
    }

    pub fn vertical_margin(mut self, margin: u16) -> Self {
        self.margin.vertical = margin;
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    pub fn spacing<T: Into<Spacing>>(mut self, spacing: T) -> Self {
        self.spacing = spacing.into();
        self
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::default(),
            constraints: Vec::new(),
            margin: Margin::default(),
            flex: Flex::default(),
            spacing: Spacing::default(),
        }
    }
}

impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.direction.hash(state);
        self.constraints.hash(state);
        self.margin.hash(state);
        self.flex.hash(state);
        self.spacing.hash(state);
    }
}

impl Layout {
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        self.try_split(area).unwrap_or_default()
    }

    pub fn areas<const N: usize>(&self, area: Rect) -> [Rect; N] {
        let rects = self.split(area);
        rects.try_into().expect("constraint count must match N")
    }

    fn try_split(&self, area: Rect) -> Option<Vec<Rect>> {
        let inner = area.inner(self.margin);
        if inner.is_empty() {
            return Some(vec![Rect::ZERO; self.constraints.len()]);
        }

        let mut solver = Solver::new();
        let segment_count = self.constraints.len();
        let spacer_count = segment_count.saturating_add(1);

        let segment_vars: Vec<Variable> = (0..segment_count).map(|_| Variable::new()).collect();
        let spacer_vars: Vec<Variable> = (0..spacer_count).map(|_| Variable::new()).collect();

        let total_size = match self.direction {
            Direction::Horizontal => inner.width,
            Direction::Vertical => inner.height,
        };
        let total = (total_size as f64 * FLOAT_PRECISION_MULTIPLIER) as i64;

        // All segment variables must be non-negative.
        for &var in &segment_vars {
            solver.add_constraint(var | GE(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
        }

        // Sum of all segments and spacers equals the total available space.
        let mut sum_expr = kasuari::Expression::from_constant(0.0);
        for &var in segment_vars.iter().chain(spacer_vars.iter()) {
            sum_expr += var;
        }
        solver.add_constraint(sum_expr | EQ(kasuari::Strength::new(strengths::REQUIRED)) | total as f64).ok()?;

        // Apply per-segment constraints.
        for (i, constraint) in self.constraints.iter().enumerate() {
            let var = segment_vars[i];
            match constraint {
                Constraint::Length(n) => {
                    let target = *n as f64 * FLOAT_PRECISION_MULTIPLIER;
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::LENGTH_SIZE_EQ)) | target).ok()?;
                }
                Constraint::Percentage(p) => {
                    let target = total as f64 * (*p as f64) / 100.0;
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::PERCENTAGE_SIZE_EQ)) | target).ok()?;
                }
                Constraint::Ratio(n, d) => {
                    let target = total as f64 * (*n as f64) / (*d as f64);
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::RATIO_SIZE_EQ)) | target).ok()?;
                }
                Constraint::Min(m) => {
                    let target = *m as f64 * FLOAT_PRECISION_MULTIPLIER;
                    solver.add_constraint(var | GE(kasuari::Strength::new(strengths::MIN_SIZE_GE)) | target).ok()?;
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::MIN_SIZE_EQ)) | target).ok()?;
                }
                Constraint::Max(m) => {
                    let target = *m as f64 * FLOAT_PRECISION_MULTIPLIER;
                    solver.add_constraint(var | LE(kasuari::Strength::new(strengths::MAX_SIZE_LE)) | target).ok()?;
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::MAX_SIZE_EQ)) | target).ok()?;
                }
                Constraint::Fill(_) => {
                    // Fill is handled by the flex/grow constraints below.
                }
            }
        }

        // Configure spacers based on flex and spacing.
        let spacing_value = match self.spacing {
            Spacing::Space(v) => v as f64 * FLOAT_PRECISION_MULTIPLIER,
            Spacing::Overlap(v) => -(v as f64) * FLOAT_PRECISION_MULTIPLIER,
        };

        match self.flex {
            Flex::Legacy => {
                // In legacy mode all spacers are zero; excess space stays in segments.
                for &var in &spacer_vars {
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
            }
            Flex::Start => {
                if let Some(&first) = spacer_vars.first() {
                    solver.add_constraint(first | EQ(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
                for &var in spacer_vars.iter().skip(1).take(spacer_count.saturating_sub(2)) {
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | spacing_value).ok()?;
                }
                if let Some(&last) = spacer_vars.last() {
                    solver.add_constraint(last | GE(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
            }
            Flex::End => {
                if let Some(&last) = spacer_vars.last() {
                    solver.add_constraint(last | EQ(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
                for &var in spacer_vars.iter().skip(1).take(spacer_count.saturating_sub(2)) {
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | spacing_value).ok()?;
                }
                if let Some(&first) = spacer_vars.first() {
                    solver.add_constraint(first | GE(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
            }
            Flex::Center => {
                for &var in spacer_vars.iter().skip(1).take(spacer_count.saturating_sub(2)) {
                    solver.add_constraint(var | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | spacing_value).ok()?;
                }
                if spacer_count >= 2 {
                    let first = spacer_vars[0];
                    let last = spacer_vars[spacer_count - 1];
                    solver.add_constraint((first - last) | EQ(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
            }
            Flex::SpaceBetween => {
                if let Some(&first) = spacer_vars.first() {
                    solver.add_constraint(first | EQ(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
                if let Some(&last) = spacer_vars.last() {
                    solver.add_constraint(last | EQ(kasuari::Strength::new(strengths::REQUIRED)) | 0.0).ok()?;
                }
                if spacer_count >= 3 {
                    let first_internal = spacer_vars[1];
                    for &var in spacer_vars.iter().skip(2).take(spacer_count - 3) {
                        solver.add_constraint((var - first_internal) | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | 0.0).ok()?;
                    }
                }
            }
            Flex::SpaceAround => {
                if spacer_count >= 3 {
                    let first = spacer_vars[0];
                    let last = spacer_vars[spacer_count - 1];
                    let first_internal = spacer_vars[1];
                    solver.add_constraint((first * 2.0 - first_internal) | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | 0.0).ok()?;
                    solver.add_constraint((last * 2.0 - first_internal) | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | 0.0).ok()?;
                    for &var in spacer_vars.iter().skip(2).take(spacer_count - 3) {
                        solver.add_constraint((var - first_internal) | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | 0.0).ok()?;
                    }
                }
            }
            Flex::SpaceEvenly => {
                if spacer_count >= 2 {
                    let first = spacer_vars[0];
                    for &var in spacer_vars.iter().skip(1) {
                        solver.add_constraint((var - first) | EQ(kasuari::Strength::new(strengths::SPACER_SIZE_EQ)) | 0.0).ok()?;
                    }
                }
            }
        }

        // Grow constraints for Fill segments.
        for (i, constraint) in self.constraints.iter().enumerate() {
            if let Constraint::Fill(priority) = constraint {
                let var = segment_vars[i];
                let strength = kasuari::Strength::new(strengths::FILL_GROW * (*priority as f64));
                solver.add_constraint(var | GE(strength) | 0.0).ok()?;
            }
        }

        // Grow all segments weakly so that Fill segments without explicit size can expand.
        if self.flex != Flex::Legacy {
            for &var in &segment_vars {
                solver.add_constraint(var | GE(kasuari::Strength::new(strengths::ALL_SEGMENT_GROW)) | 0.0).ok()?;
            }
        }

        // Solve and extract values.
        solver.fetch_changes();

        let mut rects = Vec::with_capacity(segment_count);
        let mut current: u16 = 0;

        for i in 0..segment_count {
            let spacer = (solver.get_value(spacer_vars[i]) / FLOAT_PRECISION_MULTIPLIER).round() as u16;
            current = current.saturating_add(spacer);

            let size = (solver.get_value(segment_vars[i]) / FLOAT_PRECISION_MULTIPLIER).round() as u16;

            let rect = match self.direction {
                Direction::Horizontal => Rect::new(
                    inner.x.saturating_add(current),
                    inner.y,
                    size,
                    inner.height,
                ),
                Direction::Vertical => Rect::new(
                    inner.x,
                    inner.y.saturating_add(current),
                    inner.width,
                    size,
                ),
            };
            rects.push(rect);

            current = current.saturating_add(size);
        }

        Some(rects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_vertical_split_length() {
        let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 5);
        assert_eq!(rects[1].height, 5);
    }

    #[test]
    fn layout_horizontal_split_length() {
        let layout = Layout::horizontal([Constraint::Length(5), Constraint::Length(5)]);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 5);
        assert_eq!(rects[1].width, 5);
    }

    #[test]
    fn layout_split_with_margin() {
        let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)])
            .margin(1);
        let rects = layout.split(Rect::new(0, 0, 10, 10));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].y, 1);
        assert_eq!(rects[0].width, 8);
    }

    #[test]
    fn layout_split_empty_area() {
        let layout = Layout::vertical([Constraint::Length(5)]);
        let rects = layout.split(Rect::ZERO);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], Rect::ZERO);
    }

    #[test]
    fn layout_builder_api() {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10)])
            .margin(2)
            .flex(Flex::Center)
            .spacing(1);
        assert_eq!(layout.direction, Direction::Horizontal);
        assert_eq!(layout.constraints, vec![Constraint::Length(10)]);
        assert_eq!(layout.margin, Margin::new(2, 2));
        assert_eq!(layout.flex, Flex::Center);
        assert_eq!(layout.spacing, Spacing::Space(1));
    }

    #[test]
    fn layout_areas_const_generic() {
        let layout = Layout::vertical([Constraint::Length(5), Constraint::Length(5)]);
        let areas: [Rect; 2] = layout.areas(Rect::new(0, 0, 10, 10));
        assert_eq!(areas[0].height, 5);
        assert_eq!(areas[1].height, 5);
    }
}
