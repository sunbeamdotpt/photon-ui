use crate::{
    Component,
    InputResult,
    RenderError,
    Rendered,
    events::Event,
    layout::{
        Rect,
        layout::Layout,
    },
};

/// A component that lays out its children using a [`Layout`].
///
/// Each child is rendered into the rect assigned by the layout via
/// [`render_rect`](Component::render_rect).
pub struct Container {
    layout: Layout,
    children: Vec<Box<dyn Component>>,
}

impl Container {
    /// Create a new container with the given layout.
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            children: Vec::new(),
        }
    }

    /// Add a child component.
    pub fn push(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    /// Builder-style add.
    pub fn with_child(mut self, child: Box<dyn Component>) -> Self {
        self.children.push(child);
        self
    }
}

impl Component for Container {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let rect = Rect::new(0, 0, width, self.children.len() as u16 * 3);
        self.render_rect(rect)
    }

    fn render_rect(&self, rect: Rect) -> Result<Rendered, RenderError> {
        let mut screen = Rendered::empty();
        let areas = self.layout.split(rect);

        for (child, area) in self.children.iter().zip(areas.iter()) {
            if let Ok(rendered) = child.render_rect(*area) {
                // Blit into the local buffer using coordinates relative to this container's origin.
                // `layout.split()` returns areas in terminal coordinates (they include
                // rect.x/y), but `screen` is a fresh local buffer whose origin is (0, 0).
                let rel_area = Rect::new(
                    area.x.saturating_sub(rect.x),
                    area.y.saturating_sub(rect.y),
                    area.width,
                    area.height,
                );
                rendered.blit_into_rect(&mut screen, rel_area);
            }
        }

        Ok(screen)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        for child in &mut self.children {
            let result = child.handle_input(event);
            if result != InputResult::Ignored {
                return result;
            }
        }
        InputResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::Text,
        layout::Constraint,
    };

    #[test]
    fn container_renders_children() {
        let mut container = Container::new(Layout::horizontal([
            Constraint::Length(10),
            Constraint::Length(10),
        ]));
        container.push(Box::new(Text::new("left", 0, 0)));
        container.push(Box::new(Text::new("right", 0, 0)));

        let rendered = container.render_rect(Rect::new(0, 0, 20, 1)).unwrap();
        assert_eq!(rendered.lines.len(), 1);
        assert!(rendered.lines[0].contains("left"));
        assert!(rendered.lines[0].contains("right"));
    }

    #[test]
    fn container_vertical_split() {
        let mut container = Container::new(Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ]));
        container.push(Box::new(Text::new("top", 0, 0)));
        container.push(Box::new(Text::new("bottom", 0, 0)));

        let rendered = container.render_rect(Rect::new(0, 0, 10, 2)).unwrap();
        assert_eq!(rendered.lines.len(), 2);
        assert!(rendered.lines[0].contains("top"));
        assert!(rendered.lines[1].contains("bottom"));
    }

    /// Regression test: nested containers with non-zero rect coordinates must
    /// not double-offset content.
    #[test]
    fn container_nonzero_rect_no_double_offset() {
        let mut outer = Container::new(Layout::horizontal([
            Constraint::Length(10),
            Constraint::Length(10),
        ]));

        let mut inner = Container::new(Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ]));
        inner.push(Box::new(Text::new("a", 0, 0)));
        inner.push(Box::new(Text::new("b", 0, 0)));

        outer.push(Box::new(Text::new("left", 0, 0)));
        outer.push(Box::new(inner));

        // Outer rect starts at (0, 2). Inner container gets y = 2 from the layout.
        let rendered = outer.render_rect(Rect::new(0, 2, 20, 2)).unwrap();
        // The inner container's content should be at local rows 0 and 1,
        // NOT shifted down by 2 due to double-offsetting.
        assert_eq!(rendered.lines.len(), 2, "expected 2 lines, got {}", rendered.lines.len());
        assert!(rendered.lines[0].contains("left"));
        assert!(rendered.lines[0].contains("a"));
        assert!(rendered.lines[1].contains("b"));
    }
}
