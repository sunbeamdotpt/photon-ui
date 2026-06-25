use std::io;

macro_rules! try_io {
    ($expr:expr) => {
        match $expr {
            | Ok(v) => v,
            | Err(e) => return Err(e),
        }
    };
}

#[cfg(test)]
use crate::renderer::Rendered;
use crate::{
    Component,
    compositor::Compositor,
    image::delete_kitty_image,
    layer::Layer,
    layout::Layout,
    renderer::{
        RenderStrategy,
        Renderer,
    },
    terminal::Terminal,
};

/// Anchor point for positioning an overlay on the terminal screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// Center of the screen.
    Center,
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
    /// Top edge, centered horizontally.
    TopCenter,
    /// Bottom edge, centered horizontally.
    BottomCenter,
    /// Left edge, centered vertically.
    LeftCenter,
    /// Right edge, centered vertically.
    RightCenter,
}

/// How an overlay's position is expressed.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPosition {
    /// Position relative to an anchor point.
    Anchor(Anchor),
    /// Absolute coordinates `(row, col)`.
    At(u16, u16),
    /// Percentage coordinates as strings, e.g. `"50%"`.
    Percent(String, String),
}

/// Constraints applied when computing an overlay's final position.
#[derive(Debug, Clone)]
pub struct OverlayConstraints {
    /// Minimum width in columns.
    pub min_width: u16,
    /// Maximum height in rows.
    pub max_height: u16,
    /// Margin from screen edges when using an anchor.
    pub margin: u16,
    /// Horizontal offset applied after computing the anchor position.
    pub offset_x: i16,
    /// Vertical offset applied after computing the anchor position.
    pub offset_y: i16,
    /// Optional visibility predicate: `(cols, rows) -> bool`.
    pub visible: Option<fn(u16, u16) -> bool>,
}

pub use crate::layout::Rect;

/// A floating component rendered on top of the main UI.
pub struct Overlay {
    /// The component to render.
    pub content: Box<dyn Component>,
    /// How the overlay's position is determined.
    pub position: OverlayPosition,
    /// Sizing and visibility constraints.
    pub constraints: OverlayConstraints,
}

impl Overlay {
    /// Compute the screen rectangle for this overlay given the terminal size
    /// and the content's natural dimensions.
    ///
    /// Returns `None` if the overlay's visibility predicate returns `false`.
    pub fn compute_position(
        &self,
        term_w: u16,
        term_h: u16,
        content_w: u16,
        content_h: u16,
    ) -> Option<Rect> {
        let w = content_w.max(self.constraints.min_width);
        let h = content_h.min(self.constraints.max_height).max(1);

        if let Some(vis) = self.constraints.visible &&
            !vis(term_w, term_h)
        {
            return None;
        }

        let (row, col) = match &self.position {
            | OverlayPosition::Anchor(anchor) => {
                let r = match anchor {
                    | Anchor::Center | Anchor::LeftCenter | Anchor::RightCenter => {
                        (term_h.saturating_sub(h)) / 2
                    },
                    | Anchor::TopLeft | Anchor::TopRight | Anchor::TopCenter => {
                        self.constraints.margin
                    },
                    | Anchor::BottomLeft | Anchor::BottomRight | Anchor::BottomCenter => {
                        term_h.saturating_sub(h + self.constraints.margin)
                    },
                };
                let c = match anchor {
                    | Anchor::Center | Anchor::TopCenter | Anchor::BottomCenter => {
                        (term_w.saturating_sub(w)) / 2
                    },
                    | Anchor::TopLeft | Anchor::BottomLeft | Anchor::LeftCenter => {
                        self.constraints.margin
                    },
                    | Anchor::TopRight | Anchor::BottomRight | Anchor::RightCenter => {
                        term_w.saturating_sub(w + self.constraints.margin)
                    },
                };
                (r, c)
            },
            | OverlayPosition::At(r, c) => (*r, *c),
            | OverlayPosition::Percent(px, py) => {
                let parse_pct = |s: &str| -> u16 {
                    s.trim_end_matches('%').parse::<f64>().unwrap_or(0.0) as u16
                };
                let pct_x = parse_pct(px);
                let pct_y = parse_pct(py);
                let r = (term_h as f64 * pct_y as f64 / 100.0) as u16;
                let c = (term_w as f64 * pct_x as f64 / 100.0) as u16;
                (r, c)
            },
        };

        Some(Rect {
            y: (row as i16 + self.constraints.offset_y).max(0) as u16,
            x: (col as i16 + self.constraints.offset_x).max(0) as u16,
            width: w.min(term_w.saturating_sub(col)),
            height: h.min(term_h.saturating_sub(row)),
        })
    }
}

/// Top-level TUI manager.
///
/// Owns the terminal, a list of mounted components, overlays, and a
/// [`Renderer`] that performs differential drawing. Only one component
/// receives focus at a time; it is the sole recipient of input events.
///
/// # Example
///
/// ```no_run
/// use photon_ui::{
///     TUI,
///     TestTerminal,
///     components::Text,
/// };
///
/// let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
/// tui.mount(Box::new(Text::new("Hello", 0, 0)));
/// tui.render_frame().unwrap();
/// ```
pub struct TUI {
    terminal: Box<dyn Terminal>,
    /// Stacking full-terminal-size layers. Index `0` is the bottom/floor layer
    /// and receives components from [`TUI::mount`]. Higher indices are rendered
    /// on top.
    layers: Vec<Layer>,
    /// Focused component index within each layer, parallel to `layers`.
    layer_focus: Vec<Option<usize>>,
    /// Index of the layer that currently receives input.
    focused_layer: usize,
    /// Previously focused layer index before a modal was shown.
    pre_modal_layer: Option<usize>,
    /// Positioned floating components rendered above the layer stack.
    overlays: Vec<Overlay>,
    /// Modal dialog rendered above everything.
    modal: Option<Box<dyn Component>>,
    renderer: Renderer,
    size: (u16, u16),
    previous_image_ids: std::collections::HashSet<u32>,
    hardware_cursor: bool,
}

impl TUI {
    /// Create a new TUI backed by the given terminal.
    pub fn new(terminal: Box<dyn Terminal>) -> Self {
        Self {
            terminal,
            layers: vec![Layer::new()],
            layer_focus: vec![None],
            focused_layer: 0,
            pre_modal_layer: None,
            overlays: Vec::new(),
            modal: None,
            renderer: Renderer::new(),
            size: (80, 24),
            previous_image_ids: std::collections::HashSet::new(),
            hardware_cursor: std::env::var("PHOTON_UI_HARDWARE_CURSOR").is_ok(),
        }
    }

    /// Borrow the underlying terminal.
    pub fn terminal(&self) -> &dyn Terminal {
        &*self.terminal
    }

    /// Add a component to the TUI.
    ///
    /// The component is appended to the bottom layer. If no component
    /// currently has focus, the new component receives focus automatically.
    pub fn mount(&mut self, component: Box<dyn Component>) {
        let idx = self.layers[0].components.len();
        self.layers[0].mount(component);
        if self.layer_focus[0].is_none() {
            self.set_focus(idx);
        }
    }

    /// Move focus to the component at `index` in the bottom layer.
    ///
    /// The previously focused component, if any, is unfocused first.
    pub fn set_focus(&mut self, index: usize) {
        self.unfocus_current();
        self.focused_layer = 0;
        self.layer_focus[0] = Some(index);
        if index < self.layers[0].components.len() &&
            let Some(f) = self.layers[0].components[index].as_focusable_mut()
        {
            f.set_focused(true);
        }
    }

    /// Remove all children and reset focus.
    pub fn clear_children(&mut self) {
        self.layers[0].components.clear();
        self.layer_focus[0] = None;
        self.focused_layer = 0;
    }

    fn unfocus_current(&mut self) {
        if let Some(layer_idx) = self.focused_layer_safe() &&
            let Some(component_idx) = self.layer_focus.get(layer_idx).copied().flatten() &&
            component_idx < self.layers[layer_idx].components.len() &&
            let Some(f) = self.layers[layer_idx].components[component_idx].as_focusable_mut()
        {
            f.set_focused(false);
        }
    }

    fn focused_layer_safe(&self) -> Option<usize> {
        if self.focused_layer < self.layers.len() {
            Some(self.focused_layer)
        } else {
            None
        }
    }

    /// Add an overlay on top of the main UI.
    pub fn add_overlay(&mut self, overlay: Overlay) {
        self.overlays.push(overlay);
    }

    /// Remove all overlays.
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
    }

    /// Show a modal dialog on top of the main UI.
    ///
    /// The modal captures all input until it is dismissed. Focus is moved to
    /// the modal content automatically. When dismissed, focus returns to the
    /// previously focused layer.
    pub fn show_modal(&mut self, modal: Box<dyn Component>) {
        self.pre_modal_layer = Some(self.focused_layer);
        self.modal = Some(modal);
        if let Some(ref mut m) = self.modal &&
            let Some(f) = m.as_focusable_mut()
        {
            f.set_focused(true);
        }
    }

    /// Dismiss the currently open modal, restoring previous focus.
    pub fn dismiss_modal(&mut self) {
        if let Some(ref mut m) = self.modal &&
            let Some(f) = m.as_focusable_mut()
        {
            f.set_focused(false);
        }
        self.modal = None;
        if let Some(idx) = self.pre_modal_layer {
            let clamped = idx.min(self.layers.len().saturating_sub(1));
            self.focused_layer = clamped;
        }
        self.pre_modal_layer = None;
    }

    /// Returns `true` if a modal is currently open.
    pub fn modal_active(&self) -> bool {
        self.modal.is_some()
    }

    /// Set a layout for splitting the terminal area among children in the
    /// bottom layer.
    pub fn set_layout(&mut self, layout: Layout) {
        self.layers[0].set_layout(layout);
    }

    /// Clear the layout, reverting to vertical stacking.
    pub fn clear_layout(&mut self) {
        self.layers[0].layout = None;
    }

    /// Reset the TUI for a fresh page / screen.
    ///
    /// Clears all layers, overlays, and layout, and schedules a full screen
    /// redraw so no stale content or ANSI attributes bleed through.
    pub fn reset(&mut self) {
        self.layers.clear();
        self.layer_focus.clear();
        self.layers.push(Layer::new());
        self.layer_focus.push(None);
        self.focused_layer = 0;
        self.pre_modal_layer = None;
        self.overlays.clear();
        self.modal = None;
        self.renderer
            .set_strategy(crate::renderer::RenderStrategy::FullRedraw);
    }

    /// Add a new layer on top of the stack and return its index.
    pub fn add_layer(&mut self, layer: Layer) -> usize {
        let idx = self.layers.len();
        self.layers.push(layer);
        self.layer_focus.push(None);
        idx
    }

    /// Insert a layer at the given index.
    pub fn insert_layer(&mut self, index: usize, layer: Layer) {
        if index > self.layers.len() {
            return;
        }
        self.layers.insert(index, layer);
        self.layer_focus.insert(index, None);
        if self.focused_layer >= index {
            self.focused_layer += 1;
        }
    }

    /// Remove the layer at the given index.
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index >= self.layers.len() {
            return None;
        }
        if self.focused_layer == index {
            self.focused_layer = index
                .saturating_sub(1)
                .min(self.layers.len().saturating_sub(2));
        } else if self.focused_layer > index {
            self.focused_layer -= 1;
        }
        self.layer_focus.remove(index);
        Some(self.layers.remove(index))
    }

    /// Borrow the layer at the given index mutably.
    pub fn layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }

    /// Return the number of layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Move input focus to the given layer.
    pub fn set_focused_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.unfocus_current();
            self.focused_layer = index;
        }
    }

    /// Restore the terminal (leave alternate screen, disable raw mode, show
    /// cursor).
    pub fn stop(&mut self) -> io::Result<()> {
        self.terminal.stop()
    }

    /// Render one frame to the terminal.
    ///
    /// 1. Queries terminal size.
    /// 2. Decides [`RenderStrategy`] (first render, full redraw on resize, or
    ///    diff).
    /// 3. Renders all layers front-to-back into a composite screen buffer,
    ///    culling cells hidden by higher layers.
    /// 4. Renders overlays and modal on top of the layer stack.
    /// 5. Deletes stale terminal images.
    /// 6. Writes the result through the [`Renderer`].
    /// 7. Positions the hardware cursor.
    pub fn render_frame(&mut self) -> io::Result<()> {
        let (width, height) = try_io!(self.terminal.size());
        let size_changed = self.size != (width, height);
        self.size = (width, height);

        if self.renderer.previous().is_none() {
            self.renderer.set_strategy(RenderStrategy::FirstRender);
        } else if size_changed {
            self.renderer.set_strategy(RenderStrategy::FullRedraw);
        } else {
            self.renderer.set_strategy(RenderStrategy::Diff);
        }

        let mut compositor = Compositor::new(width, height);
        for (i, layer) in self.layers.iter().enumerate().rev() {
            let focused = self.layer_focus.get(i).copied().flatten();
            let rendered = layer.render(width, height, focused);
            compositor.add_layer(&rendered, &layer.shadow);
        }

        let mut screen = compositor.finalize();

        // Pad to terminal height so overlays can be placed at absolute rows.
        if !self.overlays.is_empty() {
            while screen.lines.len() < height as usize {
                screen.lines.push("".to_string());
            }
        }

        for overlay in &self.overlays {
            if let Ok(rendered) = overlay.content.render(width) &&
                let Some(rect) =
                    overlay.compute_position(width, height, rendered.lines.len() as u16, 1)
            {
                rendered.blit_onto(&mut screen, rect.y, rect.x);
            }
        }

        // Render modal centered on top of everything.
        if let Some(ref modal) = self.modal &&
            let Ok(rendered) = modal.render(width)
        {
            let modal_h = rendered.lines.len() as u16;
            let modal_w =
                crate::utils::visible_width(rendered.lines.first().unwrap_or(&String::new()))
                    as u16;
            let row = (height.saturating_sub(modal_h)) / 2;
            let col = (width.saturating_sub(modal_w)) / 2;
            rendered.blit_onto(&mut screen, row, col);
        }

        let current_ids: std::collections::HashSet<u32> =
            screen.images.iter().map(|i| i.id).collect();
        for id in &self.previous_image_ids {
            if !current_ids.contains(id) {
                try_io!(self.terminal.write(&delete_kitty_image(*id)));
            }
        }
        self.previous_image_ids = current_ids;

        try_io!(self.renderer.render(&mut *self.terminal, &screen));

        if let Some((row, col)) = screen.cursor {
            try_io!(self.terminal.move_cursor(row as u16, col as u16));
            if self.hardware_cursor {
                try_io!(self.terminal.show_cursor());
            } else {
                try_io!(self.terminal.hide_cursor());
            }
        }

        Ok(())
    }

    /// Compute the composite screen buffer without writing to the terminal.
    /// Test-only helper to inspect layout.
    #[cfg(test)]
    fn compose_screen(&self, width: u16, height: u16) -> crate::renderer::Rendered {
        let mut compositor = Compositor::new(width, height);
        for (i, layer) in self.layers.iter().enumerate().rev() {
            let focused = self.layer_focus.get(i).copied().flatten();
            let rendered = layer.render(width, height, focused);
            compositor.add_layer(&rendered, &layer.shadow);
        }
        compositor.finalize()
    }

    /// Dispatch an event to the focused component, falling back to other
    /// components if the focused one returns [`crate::InputResult::Ignored`].
    ///
    /// Also handles `Tab` to cycle focus between focusable components.
    pub fn handle_input(&mut self, event: &crate::events::Event) {
        // Modal capture: when a modal is open, Esc dismisses it and all other
        // input is routed to the modal content.
        if let Some(ref mut _modal) = self.modal &&
            let crate::events::Event::Key(key) = event &&
            key.code == crossterm::event::KeyCode::Esc
        {
            self.dismiss_modal();
            return;
        }
        if let Some(ref mut modal) = self.modal {
            modal.handle_input(event);
            return;
        }

        // Handle Tab to cycle focus. Try the focused component first so nested
        // containers (e.g. Div) can manage their own focus cycling.
        if let crate::events::Event::Key(key) = event {
            if key.code == crossterm::event::KeyCode::Tab {
                if self.try_handle_focused(event) {
                    return;
                }
                self.cycle_focus(1);
                return;
            }
            if key.code == crossterm::event::KeyCode::BackTab {
                if self.try_handle_focused(event) {
                    return;
                }
                self.cycle_focus(-1);
                return;
            }
        }

        if self.try_handle_focused(event) {
            return;
        }

        // Fall through to other components in all layers.
        let focused_layer = self.focused_layer;
        for (layer_idx, layer) in self.layers.iter_mut().enumerate() {
            for (component_idx, child) in layer.components.iter_mut().enumerate() {
                if layer_idx == focused_layer &&
                    Some(component_idx) == self.layer_focus.get(layer_idx).copied().flatten()
                {
                    continue;
                }
                let result = child.handle_input(event);
                if !matches!(result, crate::InputResult::Ignored) {
                    return;
                }
            }
        }
    }

    fn try_handle_focused(&mut self, event: &crate::events::Event) -> bool {
        if let Some(layer_idx) = self.focused_layer_safe() &&
            let Some(component_idx) = self.layer_focus.get(layer_idx).copied().flatten() &&
            component_idx < self.layers[layer_idx].components.len()
        {
            let result = self.layers[layer_idx].components[component_idx].handle_input(event);
            if !matches!(result, crate::InputResult::Ignored) {
                return true;
            }
        }
        false
    }

    /// Move focus to the next (or previous) focusable component.
    fn cycle_focus(&mut self, delta: isize) {
        let mut focusable: Vec<(usize, usize)> = Vec::new();
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            for (component_idx, component) in layer.components.iter().enumerate() {
                if component.as_focusable().is_some() {
                    focusable.push((layer_idx, component_idx));
                }
            }
        }
        if focusable.is_empty() {
            return;
        }

        let current_layer = self.focused_layer_safe();
        let current_component =
            current_layer.and_then(|l| self.layer_focus.get(l).copied().flatten());
        let current = match current_component.and_then(|c| {
            focusable
                .iter()
                .position(|&(l, comp)| Some(l) == current_layer && comp == c)
        }) {
            | Some(pos) => pos,
            | None => {
                self.set_focus_tuple(focusable[0]);
                return;
            },
        };

        let new_pos = if delta >= 0 {
            (current + delta as usize) % focusable.len()
        } else {
            let d = (-delta) as usize % focusable.len();
            (current + focusable.len() - d) % focusable.len()
        };
        self.set_focus_tuple(focusable[new_pos]);
    }

    fn set_focus_tuple(&mut self, (layer_idx, component_idx): (usize, usize)) {
        self.unfocus_current();
        self.focused_layer = layer_idx;
        if self.layer_focus.len() <= layer_idx {
            self.layer_focus.resize(layer_idx + 1, None);
        }
        self.layer_focus[layer_idx] = Some(component_idx);
        if layer_idx < self.layers.len() &&
            component_idx < self.layers[layer_idx].components.len() &&
            let Some(f) = self.layers[layer_idx].components[component_idx].as_focusable_mut()
        {
            f.set_focused(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        TestTerminal,
        components::Text,
        layer::{
            Layer,
            Shadow,
        },
        layout::{
            Constraint,
            Layout,
        },
    };

    #[test]
    fn tui_set_focus_invalid_index() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("a", 0, 0)));
        tui.set_focus(5); // should not panic
    }

    #[test]
    fn tui_handle_input_no_focus() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.handle_input(&crate::events::Event::Resize(10, 10)); // should not panic
    }

    #[test]
    fn tui_render_with_overlay() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("hello", 0, 0)));
        let overlay = Overlay {
            content: Box::new(Text::new("popup", 0, 0)),
            position: OverlayPosition::Anchor(Anchor::Center),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 1,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        };
        tui.overlays.push(overlay);
        tui.render_frame().unwrap();
    }

    #[test]
    fn tui_full_redraw_on_resize() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("hello", 0, 0)));
        tui.render_frame().unwrap();
        // Simulate resize by changing terminal size
        let new_term = TestTerminal::new(100, 30);
        tui.terminal = Box::new(new_term);
        tui.render_frame().unwrap();
    }

    struct ImageComponent;
    impl Component for ImageComponent {
        fn render(&self, _width: u16) -> Result<Rendered, crate::RenderError> {
            Ok(Rendered {
                lines: vec!["img".into()],
                cursor: None,
                images: vec![crate::renderer::ImageCommand {
                    id: 1,
                    data: "data".into(),
                    row: 0,
                    col: 0,
                }],
            })
        }
    }

    #[test]
    fn tui_image_cleanup() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(ImageComponent));
        tui.render_frame().unwrap();
        // Now replace with text component (no images)
        tui.layers[0].components.clear();
        tui.layers[0]
            .components
            .push(Box::new(Text::new("text", 0, 0)));
        tui.render_frame().unwrap();
        // Just verify no panic
    }

    #[test]
    fn tui_hardware_cursor() {
        // SAFETY: tests are single-threaded and no other code reads this
        // environment variable concurrently.
        unsafe {
            std::env::set_var("PHOTON_UI_HARDWARE_CURSOR", "1");
        }
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("hello", 0, 0)));
        tui.render_frame().unwrap();
        // SAFETY: tests are single-threaded.
        unsafe {
            std::env::remove_var("PHOTON_UI_HARDWARE_CURSOR");
        }
    }

    #[test]
    fn tui_tab_cycles_focus() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("a", 0, 0))); // not focusable
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.mount(Box::new(list));
        let input = crate::components::Input::new();
        tui.mount(Box::new(input));

        // First mounted component gets focus (Text at index 0)
        assert_eq!(tui.layer_focus[0], Some(0));

        // Tab moves to first focusable (SelectList at index 1)
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.layer_focus[0], Some(1));

        // Tab moves to next focusable (Input at index 2)
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.layer_focus[0], Some(2));

        // Tab wraps back to first focusable
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.layer_focus[0], Some(1));
    }

    #[test]
    fn tui_backtab_cycles_backward() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.mount(Box::new(list));
        let input = crate::components::Input::new();
        tui.mount(Box::new(input));

        // Start on SelectList (index 0)
        assert_eq!(tui.layer_focus[0], Some(0));

        // BackTab moves to previous focusable (wraps to Input)
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::BackTab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.layer_focus[0], Some(1));
    }

    #[test]
    fn tui_cycle_focus_single_focusable() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.mount(Box::new(list));

        // Tab with only one focusable stays on it
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.layer_focus[0], Some(0));
    }

    #[test]
    fn tui_no_focusables_no_panic() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("hello", 0, 0))); // not focusable
        // Tab with no focusables should not panic
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
    }

    #[test]
    fn tui_terminal_borrow() {
        let term = TestTerminal::new(80, 24);
        let tui = TUI::new(Box::new(term));
        let _ = tui.terminal();
    }

    #[test]
    fn tui_handle_input_fallthrough() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        // Add two text components (not focusable)
        tui.mount(Box::new(Text::new("a", 0, 0)));
        tui.mount(Box::new(Text::new("b", 0, 0)));
        // A non-Tab key should fall through without panic
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('x'),
            crossterm::event::KeyModifiers::empty(),
        )));
    }

    #[test]
    fn overlay_compute_position_all_anchors() {
        let constraints = OverlayConstraints {
            min_width: 5,
            max_height: 3,
            margin: 1,
            offset_x: 0,
            offset_y: 0,
            visible: None,
        };
        let anchors = vec![
            Anchor::Center,
            Anchor::TopLeft,
            Anchor::TopRight,
            Anchor::BottomLeft,
            Anchor::BottomRight,
            Anchor::TopCenter,
            Anchor::BottomCenter,
            Anchor::LeftCenter,
            Anchor::RightCenter,
        ];
        for anchor in anchors {
            let overlay = Overlay {
                content: Box::new(Text::new("test", 0, 0)),
                position: OverlayPosition::Anchor(anchor),
                constraints: constraints.clone(),
            };
            let rect = overlay.compute_position(80, 24, 10, 2);
            assert!(rect.is_some(), "anchor {:?} should produce a rect", anchor);
        }
    }

    #[test]
    fn overlay_compute_position_at() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::At(5, 10),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        };
        let rect = overlay.compute_position(80, 24, 10, 2).unwrap();
        assert_eq!(rect.y, 5);
        assert_eq!(rect.x, 10);
    }

    #[test]
    fn overlay_compute_position_percent() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::Percent("50%".into(), "25%".into()),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        };
        let rect = overlay.compute_position(100, 40, 10, 2).unwrap();
        assert_eq!(rect.y, 10);
        assert_eq!(rect.x, 50);
    }

    #[test]
    fn overlay_compute_position_percent_invalid() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::Percent("abc".into(), "xyz".into()),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        };
        let rect = overlay.compute_position(100, 40, 10, 2).unwrap();
        assert_eq!(rect.y, 0);
        assert_eq!(rect.x, 0);
    }

    #[test]
    fn overlay_compute_position_visible_false() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::Anchor(Anchor::Center),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: 0,
                offset_y: 0,
                visible: Some(|_w, _h| false),
            },
        };
        assert!(overlay.compute_position(80, 24, 10, 2).is_none());
    }

    #[test]
    fn overlay_compute_position_with_offset() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::At(10, 10),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: 5,
                offset_y: -3,
                visible: None,
            },
        };
        let rect = overlay.compute_position(80, 24, 10, 2).unwrap();
        assert_eq!(rect.y, 7);
        assert_eq!(rect.x, 15);
    }

    #[test]
    fn overlay_compute_position_negative_offset_clamped() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::At(0, 0),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: -5,
                offset_y: -5,
                visible: None,
            },
        };
        let rect = overlay.compute_position(80, 24, 10, 2).unwrap();
        assert_eq!(rect.y, 0);
        assert_eq!(rect.x, 0);
    }

    #[test]
    fn overlay_compute_position_size_clamped() {
        let overlay = Overlay {
            content: Box::new(Text::new("test", 0, 0)),
            position: OverlayPosition::At(70, 20),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 0,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        };
        let rect = overlay.compute_position(80, 24, 20, 10).unwrap();
        // width should be min(term_w - col, w) = min(80-20, 20) = 20
        assert_eq!(rect.width, 20);
        // height: h = 10.min(3).max(1) = 3, then min(3, 24.saturating_sub(70)) = min(3,
        // 0) = 0
        assert_eq!(rect.height, 0);
    }

    struct CursorComponent;
    impl Component for CursorComponent {
        fn render(&self, _width: u16) -> Result<Rendered, crate::RenderError> {
            Ok(Rendered {
                lines: vec!["cursor".into()],
                cursor: Some((0, 3)),
                images: vec![],
            })
        }
    }

    #[test]
    fn tui_render_frame_with_cursor() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(CursorComponent));
        tui.render_frame().unwrap();
    }

    #[test]
    fn tui_demo_layout_exact() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));

        tui.mount(Box::new(Text::new("Photon UI Demo", 2, 1)));
        tui.mount(Box::new(Text::new(
            "j/k = navigate list   Tab = switch focus   i = insert mode   Esc = normal mode   q = quit",
            2, 0,
        )));
        let list = crate::components::SelectList::new(
            vec![
                "Option 1: Hello world".into(),
                "Option 2: Foo bar baz".into(),
                "Option 3: Lorem ipsum".into(),
                "Option 4: Vim bindings".into(),
                "Option 5: Blazing fast".into(),
            ],
            3,
        );
        tui.mount(Box::new(list));
        let input = crate::components::Input::new();
        tui.mount(Box::new(input));
        tui.set_focus(2);

        let screen = tui.compose_screen(80, 24);

        // Expected layout (8 content lines padded to terminal height):
        // 0: blank (Text1 pad_y)
        // 1: Photon UI Demo
        // 2: blank (Text1 pad_y)
        // 3: keybindings text
        // 4: first list item (selected)
        // 5: second list item
        // 6: third list item
        // 7: input line
        assert_eq!(screen.lines.len(), 24, "expected 24 padded lines");
        assert_eq!(
            screen.lines[0].trim_end(),
            "",
            "row 0 should be blank from Text1 pad_y"
        );
        assert!(
            screen.lines[1].contains("Photon UI Demo"),
            "row 1 should contain header: got {:?}",
            screen.lines[1]
        );
        assert_eq!(
            screen.lines[2].trim_end(),
            "",
            "row 2 should be blank from Text1 pad_y"
        );
        assert!(
            screen.lines[3].contains("j/k = navigate"),
            "row 3 should contain keybindings: got {:?}",
            screen.lines[3]
        );
        assert!(
            screen.lines[4].contains("> Option 1"),
            "row 4 should be selected list item: got {:?}",
            screen.lines[4]
        );
        assert!(
            screen.lines[5].contains("  Option 2"),
            "row 5 should be unselected list item: got {:?}",
            screen.lines[5]
        );
        assert!(
            screen.lines[6].contains("  Option 3"),
            "row 6 should be unselected list item: got {:?}",
            screen.lines[6]
        );
        assert_eq!(
            screen.lines[7].trim_end(),
            "",
            "row 7 should be empty input line"
        );
    }

    /// Regression: reset() must clear children, overlays, layout, focus,
    /// and schedule a FullRedraw so stale content doesn't bleed through.
    #[test]
    fn tui_reset_clears_all_and_schedules_redraw() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));

        tui.mount(Box::new(crate::components::Text::new("hello", 0, 0)));
        tui.set_focus(0);
        tui.add_overlay(Overlay {
            content: Box::new(crate::components::Text::new("popup", 0, 0)),
            position: OverlayPosition::Anchor(Anchor::Center),
            constraints: OverlayConstraints {
                min_width: 10,
                max_height: 3,
                margin: 2,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        });
        tui.set_layout(crate::layout::Layout::vertical([
            crate::layout::Constraint::Length(1),
        ]));
        tui.render_frame().unwrap();

        // Verify preconditions: screen has content
        let screen_before = tui.compose_screen(80, 24);
        assert!(
            !screen_before.lines.is_empty(),
            "precondition: screen should have content"
        );

        tui.reset();

        // After reset, compose_screen should contain only empty padding.
        let screen = tui.compose_screen(80, 24);
        assert!(
            screen.lines.iter().all(|line| line.trim_end().is_empty()),
            "reset should clear all children"
        );

        // render_frame should not panic after reset (FullRedraw is scheduled
        // internally)
        tui.render_frame().unwrap();
    }

    #[test]
    fn tui_show_modal_captures_input() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("background", 0, 0)));
        tui.set_focus(0);

        let modal_content = Text::new("modal text", 0, 0);
        tui.show_modal(Box::new(modal_content));
        assert!(tui.modal_active());

        // Esc should dismiss the modal
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert!(!tui.modal_active());
    }

    #[test]
    fn tui_modal_restores_focus_on_dismiss() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.mount(Box::new(list));
        assert_eq!(tui.layer_focus[0], Some(0));

        tui.show_modal(Box::new(Text::new("modal", 0, 0)));
        tui.dismiss_modal();
        assert_eq!(tui.layer_focus[0], Some(0));
    }

    #[test]
    fn tui_modal_renders_without_panic() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("background", 0, 0)));

        let modal_content = crate::components::Modal::new(Box::new(Text::new("hello", 0, 0)));
        tui.show_modal(Box::new(modal_content));
        // render_frame should not panic with an active modal
        tui.render_frame().unwrap();
    }

    #[test]
    fn tui_add_layer_count_and_mut() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        assert_eq!(tui.layer_count(), 1);

        let idx = tui.add_layer(Layer::with_component(Box::new(Text::new("l1", 0, 0))));
        assert_eq!(idx, 1);
        assert_eq!(tui.layer_count(), 2);

        if let Some(layer) = tui.layer_mut(1) {
            layer.shadow = Shadow::Dim {
                style: "\x1b[2m".into(),
            };
        }
        assert!(tui.layer_mut(5).is_none());
    }

    #[test]
    fn tui_insert_layer() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.mount(Box::new(list));
        tui.set_focused_layer(0);

        tui.insert_layer(
            0,
            Layer::with_component(Box::new(Text::new("inserted", 0, 0))),
        );
        assert_eq!(tui.layer_count(), 2);
        assert_eq!(tui.focused_layer, 1);
    }

    #[test]
    fn tui_remove_layer() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.add_layer(Layer::with_component(Box::new(Text::new("top", 0, 0))));

        let removed = tui.remove_layer(1);
        assert!(removed.is_some());
        assert_eq!(tui.layer_count(), 1);
        assert!(tui.remove_layer(5).is_none());
    }

    #[test]
    fn tui_set_focused_layer() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.add_layer(Layer::with_component(Box::new(list)));

        tui.set_focused_layer(1);
        assert_eq!(tui.focused_layer, 1);

        tui.set_focused_layer(99);
        assert_eq!(tui.focused_layer, 1);
    }

    #[test]
    fn tui_clear_children() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("child", 0, 0)));
        tui.clear_children();
        assert!(tui.layers[0].components.is_empty());
        assert_eq!(tui.layer_focus[0], None);
    }

    #[test]
    fn tui_clear_overlays() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.add_overlay(Overlay {
            content: Box::new(Text::new("popup", 0, 0)),
            position: OverlayPosition::Anchor(Anchor::Center),
            constraints: OverlayConstraints {
                min_width: 5,
                max_height: 3,
                margin: 1,
                offset_x: 0,
                offset_y: 0,
                visible: None,
            },
        });
        tui.clear_overlays();
        assert!(tui.overlays.is_empty());
    }

    #[test]
    fn tui_clear_layout() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.set_layout(Layout::vertical([Constraint::Length(1)]));
        tui.clear_layout();
        assert!(tui.layers[0].layout.is_none());
    }

    #[test]
    fn tui_dismiss_focusable_modal() {
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        let list = crate::components::SelectList::new(vec!["x".into()], 1);
        tui.show_modal(Box::new(list));
        assert!(tui.modal_active());
        tui.dismiss_modal();
        assert!(!tui.modal_active());
    }
}
