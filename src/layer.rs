use crate::{
    Component,
    layout::{
        Layout,
        Rect,
    },
    renderer::Rendered,
};

/// Visual effect applied to lower layers to make stacking obvious.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Shadow {
    /// No shadow effect.
    #[default]
    None,
    /// Dim the entire screen behind this layer except for the layer's own
    /// opaque content. The `style` string is an ANSI prefix such as
    /// `"\x1b[2m"` for faint text.
    Dim {
        /// ANSI style prefix applied to lower-layer cells visible behind this
        /// layer.
        style: String,
    },
    /// Render a drop-shadow offset from this layer's opaque bounding box.
    Drop {
        /// ANSI style prefix applied to shadow cells.
        style: String,
        /// Horizontal offset in columns.
        offset_x: i16,
        /// Vertical offset in rows.
        offset_y: i16,
    },
}

/// Classification of a layer for backward-compatible TUI APIs.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LayerKind {
    /// The bottom layer that receives mounted components.
    #[default]
    Base,
    /// A floating layer created by `TUI::add_overlay`.
    Overlay,
    /// A capture layer created by `TUI::show_modal`.
    Modal,
}

/// A full-terminal-size surface that holds its own components and layout.
///
/// Layers stack from index `0` (bottom/floor) to `N` (top/front). The TUI
/// renders each layer independently and then composites them front-to-back,
/// stripping cells that are hidden by higher layers.
#[derive(Default)]
pub struct Layer {
    /// Components owned by this layer.
    pub components: Vec<Box<dyn Component>>,
    /// Optional layout for splitting the layer among its components.
    pub layout: Option<Layout>,
    /// Visual effect applied to lower layers behind this layer.
    pub shadow: Shadow,
    /// Whether this layer participates in rendering.
    pub visible: bool,
    /// TUI classification of this layer.
    pub kind: LayerKind,
}

impl Layer {
    /// Create a new empty layer.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            layout: None,
            shadow: Shadow::None,
            visible: true,
            kind: LayerKind::Base,
        }
    }

    /// Create a layer that already contains a single component.
    pub fn with_component(component: Box<dyn Component>) -> Self {
        Self {
            components: vec![component],
            layout: None,
            shadow: Shadow::None,
            visible: true,
            kind: LayerKind::Base,
        }
    }

    /// Assign a layout to this layer.
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = Some(layout);
    }

    /// Append a component to this layer.
    pub fn mount(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    /// Set the layer kind.
    pub fn with_kind(mut self, kind: LayerKind) -> Self {
        self.kind = kind;
        self
    }

    /// Render this layer to a full-terminal-size buffer.
    ///
    /// The returned [`Rendered`] has exactly `height` lines. `focused_index`
    /// identifies which of this layer's components currently has focus so that
    /// its cursor can be translated into screen coordinates.
    pub fn render(&self, width: u16, height: u16, focused_index: Option<usize>) -> Rendered {
        if !self.visible {
            let mut rendered = Rendered::empty();
            while rendered.lines.len() < height as usize {
                rendered.lines.push(String::new());
            }
            return rendered;
        }

        let mut rendered = Rendered::empty();
        let term_rect = Rect::new(0, 0, width, height);

        if let Some(ref layout) = self.layout {
            let areas = layout.split(term_rect);
            for (i, (child, area)) in self.components.iter().zip(areas.iter()).enumerate() {
                let child_rendered = match child.render_rect(*area) {
                    | Ok(r) => r,
                    | Err(_) => continue,
                };
                child_rendered.blit_into_rect(&mut rendered, *area);
                if Some(i) == focused_index &&
                    let Some((r_local, c_local)) = child_rendered.cursor
                {
                    rendered.cursor = Some((area.y as usize + r_local, area.x as usize + c_local));
                }
            }
        } else {
            let mut row = 0usize;
            for (i, child) in self.components.iter().enumerate() {
                if row >= height as usize {
                    break;
                }
                let child_rendered = match child.render(width) {
                    | Ok(r) => r,
                    | Err(_) => continue,
                };
                let start_row = row;
                for line in &child_rendered.lines {
                    if row < height as usize {
                        rendered.lines.push(line.clone());
                        row += 1;
                    }
                }
                if Some(i) == focused_index &&
                    let Some((r_local, c_local)) = child_rendered.cursor
                {
                    rendered.cursor = Some((start_row + r_local, c_local));
                }
                rendered.images.extend(child_rendered.images.clone());
            }
        }

        while rendered.lines.len() < height as usize {
            rendered.lines.push(String::new());
        }

        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderError,
        components::Text,
        layout::{
            Constraint,
            Layout,
        },
        renderer::ImageCommand,
    };

    struct CursorComponent;

    impl Component for CursorComponent {
        fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
            Ok(Rendered {
                lines: vec!["cursor".into()],
                cursor: Some((0, 3)),
                images: Vec::new(),
            })
        }
    }

    #[test]
    fn layer_new_is_empty() {
        let layer = Layer::new();
        assert!(layer.components.is_empty());
        assert!(layer.layout.is_none());
        assert_eq!(layer.shadow, Shadow::None);
        assert!(layer.visible);
    }

    #[test]
    fn layer_render_empty_pads_to_terminal_size() {
        let layer = Layer::new();
        let rendered = layer.render(80, 24, None);
        assert_eq!(rendered.lines.len(), 24);
        assert!(rendered.cursor.is_none());
        assert!(rendered.images.is_empty());
    }

    #[test]
    fn layer_render_single_component_vertical_stack() {
        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("hello", 0, 0)));
        let rendered = layer.render(80, 24, None);
        assert!(rendered.lines[0].starts_with("hello"));
        assert_eq!(crate::utils::visible_width(&rendered.lines[0]), 80);
        assert_eq!(rendered.lines.len(), 24);
    }

    #[test]
    fn layer_render_multiple_components_vertical_stack() {
        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("first", 0, 0)));
        layer.mount(Box::new(Text::new("second", 0, 0)));
        let rendered = layer.render(80, 24, None);
        assert!(rendered.lines[0].starts_with("first"));
        assert!(rendered.lines[1].starts_with("second"));
        assert_eq!(rendered.lines.len(), 24);
    }

    #[test]
    fn layer_render_clips_component_to_size() {
        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("hello", 0, 0)));
        let rendered = layer.render(4, 2, None);
        assert!(rendered.lines[0].starts_with("hel"));
        assert_eq!(rendered.lines.len(), 2);
    }

    #[test]
    fn layer_render_focused_component_cursor() {
        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("a", 0, 0)));
        layer.mount(Box::new(CursorComponent));
        let rendered = layer.render(80, 24, Some(1));
        assert_eq!(rendered.cursor, Some((1, 3)));
    }

    #[test]
    fn layer_render_preserves_images() {
        struct ImageComponent;

        impl Component for ImageComponent {
            fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
                Ok(Rendered {
                    lines: vec!["img".into()],
                    cursor: None,
                    images: vec![ImageCommand {
                        id: 7,
                        data: "data".into(),
                    }],
                })
            }
        }

        let mut layer = Layer::new();
        layer.mount(Box::new(ImageComponent));
        let rendered = layer.render(80, 24, None);
        assert_eq!(rendered.images.len(), 1);
        assert_eq!(rendered.images[0].id, 7);
    }

    #[test]
    fn layer_shadow_variants_clone_and_equal() {
        let dim = Shadow::Dim {
            style: "\x1b[2m".into(),
        };
        let cloned = dim.clone();
        assert_eq!(dim, cloned);

        let drop_shadow = Shadow::Drop {
            style: "\x1b[2m".into(),
            offset_x: 1,
            offset_y: 1,
        };
        assert_ne!(dim, drop_shadow);
    }

    #[test]
    fn layer_visible_false_skips_render() {
        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("hello", 0, 0)));
        layer.visible = false;
        let rendered = layer.render(80, 24, None);
        assert_eq!(rendered.lines[0], "");
        assert_eq!(rendered.lines.len(), 24);
    }

    #[test]
    fn shadow_default_is_none() {
        assert_eq!(Shadow::default(), Shadow::None);
    }

    #[test]
    fn layer_kind_default_is_base() {
        assert_eq!(LayerKind::default(), LayerKind::Base);
    }

    #[test]
    fn layer_with_component() {
        let layer = Layer::with_component(Box::new(Text::new("hi", 0, 0)));
        assert_eq!(layer.components.len(), 1);
        assert!(layer.layout.is_none());
        assert_eq!(layer.shadow, Shadow::None);
        assert_eq!(layer.kind, LayerKind::Base);
        assert!(layer.visible);
    }

    #[test]
    fn layer_with_kind() {
        let layer = Layer::new().with_kind(LayerKind::Overlay);
        assert_eq!(layer.kind, LayerKind::Overlay);
    }

    #[test]
    fn layer_render_with_layout_focuses_cursor() {
        let mut layer = Layer::new();
        layer.set_layout(Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]));
        layer.mount(Box::new(Text::new("a", 0, 0)));
        layer.mount(Box::new(CursorComponent));
        let rendered = layer.render(80, 24, Some(1));
        assert!(rendered.cursor.is_some());
        let (_, col) = rendered.cursor.unwrap();
        assert!(col >= 40, "expected cursor in second half, got col {}", col);
    }

    #[test]
    fn layer_render_clips_to_height() {
        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("a", 0, 0)));
        layer.mount(Box::new(Text::new("b", 0, 0)));
        layer.mount(Box::new(Text::new("c", 0, 0)));
        layer.mount(Box::new(Text::new("d", 0, 0)));
        let rendered = layer.render(80, 2, None);
        assert_eq!(rendered.lines.len(), 2);
        assert!(rendered.lines[0].starts_with("a"));
        assert!(rendered.lines[1].starts_with("b"));
    }

    #[test]
    fn layer_render_ignores_failing_component() {
        struct Fail;

        impl Component for Fail {
            fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
                Err(RenderError::WidthOverflow {
                    line: String::new(),
                    width: 0,
                    actual: 0,
                })
            }
        }

        let mut layer = Layer::new();
        layer.mount(Box::new(Text::new("ok", 0, 0)));
        layer.mount(Box::new(Fail));
        let rendered = layer.render(80, 24, None);
        assert!(rendered.lines[0].starts_with("ok"));
    }
}
