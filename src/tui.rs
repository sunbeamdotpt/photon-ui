use crate::renderer::{Renderer, Rendered, RenderStrategy};
use crate::terminal::Terminal;
use crate::Component;
use std::io;

/// Anchor point for positioning an overlay on the terminal screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    LeftCenter,
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

/// A rectangular region on the terminal screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Row (y) coordinate.
    pub row: u16,
    /// Column (x) coordinate.
    pub col: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

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
    pub fn compute_position(&self, term_w: u16, term_h: u16, content_w: u16, content_h: u16) -> Option<Rect> {
        let w = content_w.max(self.constraints.min_width);
        let h = content_h.min(self.constraints.max_height).max(1);

        if let Some(vis) = self.constraints.visible {
            if !vis(term_w, term_h) {
                return None;
            }
        }

        let (row, col) = match &self.position {
            OverlayPosition::Anchor(anchor) => {
                let r = match anchor {
                    Anchor::Center | Anchor::LeftCenter | Anchor::RightCenter => {
                        (term_h.saturating_sub(h)) / 2
                    }
                    Anchor::TopLeft | Anchor::TopRight | Anchor::TopCenter => self.constraints.margin,
                    Anchor::BottomLeft | Anchor::BottomRight | Anchor::BottomCenter => {
                        term_h.saturating_sub(h + self.constraints.margin)
                    }
                };
                let c = match anchor {
                    Anchor::Center | Anchor::TopCenter | Anchor::BottomCenter => {
                        (term_w.saturating_sub(w)) / 2
                    }
                    Anchor::TopLeft | Anchor::BottomLeft | Anchor::LeftCenter => self.constraints.margin,
                    Anchor::TopRight | Anchor::BottomRight | Anchor::RightCenter => {
                        term_w.saturating_sub(w + self.constraints.margin)
                    }
                };
                (r, c)
            }
            OverlayPosition::At(r, c) => (*r, *c),
            OverlayPosition::Percent(px, py) => {
                let parse_pct = |s: &str| -> u16 {
                    s.trim_end_matches('%').parse::<f64>().unwrap_or(0.0) as u16
                };
                let pct_x = parse_pct(px);
                let pct_y = parse_pct(py);
                let r = (term_h as f64 * pct_y as f64 / 100.0) as u16;
                let c = (term_w as f64 * pct_x as f64 / 100.0) as u16;
                (r, c)
            }
        };

        Some(Rect {
            row: (row as i16 + self.constraints.offset_y).max(0) as u16,
            col: (col as i16 + self.constraints.offset_x).max(0) as u16,
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
/// use photon_ui::{TUI, TestTerminal};
/// use photon_ui::components::Text;
///
/// let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
/// tui.mount(Box::new(Text::new("Hello", 0, 0)));
/// tui.render_frame().unwrap();
/// ```
pub struct TUI {
    terminal: Box<dyn Terminal>,
    children: Vec<Box<dyn Component>>,
    overlays: Vec<Overlay>,
    focused_index: Option<usize>,
    renderer: Renderer,
    size: (u16, u16),
    previous_image_ids: std::collections::HashSet<u32>,
    clear_on_shrink: bool,
    hardware_cursor: bool,
}

impl TUI {
    /// Create a new TUI backed by the given terminal.
    pub fn new(terminal: Box<dyn Terminal>) -> Self {
        Self {
            terminal,
            children: Vec::new(),
            overlays: Vec::new(),
            focused_index: None,
            renderer: Renderer::new(),
            size: (80, 24),
            previous_image_ids: std::collections::HashSet::new(),
            clear_on_shrink: std::env::var("PHOTON_UI_CLEAR_ON_SHRINK").is_ok(),
            hardware_cursor: std::env::var("PHOTON_UI_HARDWARE_CURSOR").is_ok(),
        }
    }

    /// Borrow the underlying terminal.
    pub fn terminal(&self) -> &dyn Terminal {
        &*self.terminal
    }

    /// Add a component to the TUI.
    ///
    /// The component is appended to the children list. If no component
    /// currently has focus, the new component receives focus automatically.
    pub fn mount(&mut self, component: Box<dyn Component>) {
        let idx = self.children.len();
        self.children.push(component);
        if self.focused_index.is_none() {
            self.set_focus(idx);
        }
    }

    /// Move focus to the component at `index`.
    ///
    /// The previously focused component, if any, is unfocused first.
    pub fn set_focus(&mut self, index: usize) {
        if let Some(old) = self.focused_index {
            if old < self.children.len() {
                if let Some(f) = self.children[old].as_focusable_mut() {
                    f.set_focused(false);
                }
            }
        }
        self.focused_index = Some(index);
        if index < self.children.len() {
            if let Some(f) = self.children[index].as_focusable_mut() {
                f.set_focused(true);
            }
        }
    }

    /// Remove all children and reset focus.
    pub fn clear_children(&mut self) {
        self.children.clear();
        self.focused_index = None;
    }

    /// Add an overlay on top of the main UI.
    pub fn add_overlay(&mut self, overlay: Overlay) {
        self.overlays.push(overlay);
    }

    /// Remove all overlays.
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
    }

    /// Restore the terminal (leave alternate screen, disable raw mode, show cursor).
    pub fn stop(&mut self) -> io::Result<()> {
        self.terminal.stop()
    }

    /// Render one frame to the terminal.
    ///
    /// 1. Queries terminal size.
    /// 2. Decides [`RenderStrategy`] (first render, full redraw on resize, or diff).
    /// 3. Renders all children and overlays into a composite screen buffer.
    /// 4. Deletes stale terminal images.
    /// 5. Writes the result through the [`Renderer`].
    /// 6. Positions the hardware cursor.
    pub fn render_frame(&mut self) -> io::Result<()> {
        let (width, height) = self.terminal.size()?;
        let size_changed = self.size != (width, height);
        self.size = (width, height);

        if self.renderer.previous().is_none() {
            self.renderer.set_strategy(RenderStrategy::FirstRender);
        } else if size_changed {
            self.renderer.set_strategy(RenderStrategy::FullRedraw);
        } else {
            self.renderer.set_strategy(RenderStrategy::Diff);
        }

        // Concatenate child lines vertically, matching the original TUI Container behavior.
        let mut screen = Rendered::empty();
        let mut row = 0usize;
        for child in &self.children {
            if let Ok(rendered) = child.render(width) {
                for line in &rendered.lines {
                    screen.lines.push(line.clone());
                }
                if let Some((r, c)) = rendered.cursor {
                    screen.cursor = Some((row + r, c));
                }
                screen.images.extend(rendered.images);
                row += rendered.lines.len();
            }
        }

        // Pad to terminal height so overlays can be placed at absolute rows.
        if !self.overlays.is_empty() {
            while screen.lines.len() < height as usize {
                screen.lines.push("".to_string());
            }
        }

        for overlay in &self.overlays {
            if let Ok(rendered) = overlay.content.render(width) {
                if let Some(rect) = overlay.compute_position(width, height, rendered.lines.len() as u16, 1) {
                    rendered.blit_onto(&mut screen, rect.row, rect.col);
                }
            }
        }

        let current_ids: std::collections::HashSet<u32> = screen.images.iter().map(|i| i.id).collect();
        for id in &self.previous_image_ids {
            if !current_ids.contains(id) {
                self.terminal.write(&format!("\x1b_Ga=d,d=I,i={}\x1b\\", id))?;
            }
        }
        self.previous_image_ids = current_ids;

        self.renderer.render(&mut *self.terminal, &screen)?;

        if let Some((row, col)) = screen.cursor {
            self.terminal.move_cursor(row as u16, col as u16)?;
            if self.hardware_cursor {
                self.terminal.show_cursor()?;
            } else {
                self.terminal.hide_cursor()?;
            }
        }

        Ok(())
    }

    /// Compute the composite screen buffer without writing to the terminal.
    /// Test-only helper to inspect layout.
    #[cfg(test)]
    fn compose_screen(&self, width: u16, _height: u16) -> crate::renderer::Rendered {
        let mut screen = crate::renderer::Rendered::empty();
        let mut row = 0usize;
        for child in &self.children {
            if let Ok(rendered) = child.render(width) {
                for line in &rendered.lines {
                    screen.lines.push(line.clone());
                }
                if let Some((r, c)) = rendered.cursor {
                    screen.cursor = Some((row + r, c));
                }
                row += rendered.lines.len();
            }
        }
        screen
    }

    /// Dispatch an event to the focused component, falling back to other
    /// children if the focused one returns [`InputResult::Ignored`].
    ///
    /// Also handles `Tab` to cycle focus between focusable children.
    pub fn handle_input(&mut self, event: &crate::events::Event) {
        // Handle Tab to cycle focus
        if let crate::events::Event::Key(key) = event {
            if key.code == crossterm::event::KeyCode::Tab {
                self.cycle_focus(1);
                return;
            }
            if key.code == crossterm::event::KeyCode::BackTab {
                self.cycle_focus(-1);
                return;
            }
        }

        // Try focused child first
        if let Some(idx) = self.focused_index {
            if idx < self.children.len() {
                let result = self.children[idx].handle_input(event);
                if !matches!(result, crate::InputResult::Ignored) {
                    return;
                }
            }
        }

        // Fall through to other children
        for (i, child) in self.children.iter_mut().enumerate() {
            if Some(i) == self.focused_index {
                continue;
            }
            let result = child.handle_input(event);
            if !matches!(result, crate::InputResult::Ignored) {
                return;
            }
        }
    }

    /// Move focus to the next (or previous) focusable component.
    fn cycle_focus(&mut self, delta: isize) {
        let focusable: Vec<usize> = self.children.iter().enumerate()
            .filter(|(_, c)| c.as_focusable().is_some())
            .map(|(i, _)| i)
            .collect();
        if focusable.is_empty() { return; }

        let current = match self.focused_index.and_then(|idx| {
            focusable.iter().position(|&i| i == idx)
        }) {
            Some(pos) => pos,
            None => {
                self.set_focus(focusable[0]);
                return;
            }
        };

        let new_pos = if delta >= 0 {
            (current + delta as usize) % focusable.len()
        } else {
            let d = (-delta) as usize % focusable.len();
            (current + focusable.len() - d) % focusable.len()
        };
        self.set_focus(focusable[new_pos]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Text;
    use crate::TestTerminal;

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
                images: vec![crate::renderer::ImageCommand { id: 1, data: "data".into() }],
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
        tui.children.clear();
        tui.children.push(Box::new(Text::new("text", 0, 0)));
        tui.render_frame().unwrap();
        // Just verify no panic
    }

    #[test]
    fn tui_hardware_cursor() {
        unsafe {
            std::env::set_var("PHOTON_UI_HARDWARE_CURSOR", "1");
        }
        let term = TestTerminal::new(80, 24);
        let mut tui = TUI::new(Box::new(term));
        tui.mount(Box::new(Text::new("hello", 0, 0)));
        tui.render_frame().unwrap();
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
        assert_eq!(tui.focused_index, Some(0));

        // Tab moves to first focusable (SelectList at index 1)
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.focused_index, Some(1));

        // Tab moves to next focusable (Input at index 2)
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.focused_index, Some(2));

        // Tab wraps back to first focusable
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.focused_index, Some(1));
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
        assert_eq!(tui.focused_index, Some(0));

        // BackTab moves to previous focusable (wraps to Input)
        tui.handle_input(&crate::events::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::BackTab,
            crossterm::event::KeyModifiers::empty(),
        )));
        assert_eq!(tui.focused_index, Some(1));
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
        assert_eq!(tui.focused_index, Some(0));
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
        assert_eq!(rect.row, 5);
        assert_eq!(rect.col, 10);
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
        assert_eq!(rect.row, 10);
        assert_eq!(rect.col, 50);
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
        assert_eq!(rect.row, 0);
        assert_eq!(rect.col, 0);
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
        assert_eq!(rect.row, 7);
        assert_eq!(rect.col, 15);
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
        assert_eq!(rect.row, 0);
        assert_eq!(rect.col, 0);
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
        // height: h = 10.min(3).max(1) = 3, then min(3, 24.saturating_sub(70)) = min(3, 0) = 0
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
        let list = crate::components::SelectList::new(vec![
            "Option 1: Hello world".into(),
            "Option 2: Foo bar baz".into(),
            "Option 3: Lorem ipsum".into(),
            "Option 4: Vim bindings".into(),
            "Option 5: Blazing fast".into(),
        ], 3);
        tui.mount(Box::new(list));
        let input = crate::components::Input::new();
        tui.mount(Box::new(input));
        tui.set_focus(2);

        let screen = tui.compose_screen(80, 24);

        // Expected layout (8 content lines):
        // 0: blank (Text1 pad_y)
        // 1: Photon UI Demo
        // 2: blank (Text1 pad_y)
        // 3: keybindings text
        // 4: first list item (selected)
        // 5: second list item
        // 6: third list item
        // 7: input line
        assert_eq!(screen.lines.len(), 8, "expected 8 content lines, got {}", screen.lines.len());
        assert_eq!(screen.lines[0].trim_end(), "", "row 0 should be blank from Text1 pad_y");
        assert!(screen.lines[1].contains("Photon UI Demo"), "row 1 should contain header: got {:?}", screen.lines[1]);
        assert_eq!(screen.lines[2].trim_end(), "", "row 2 should be blank from Text1 pad_y");
        assert!(screen.lines[3].contains("j/k = navigate"), "row 3 should contain keybindings: got {:?}", screen.lines[3]);
        assert!(screen.lines[4].starts_with("> Option 1"), "row 4 should be selected list item: got {:?}", screen.lines[4]);
        assert!(screen.lines[5].starts_with("  Option 2"), "row 5 should be unselected list item: got {:?}", screen.lines[5]);
        assert!(screen.lines[6].starts_with("  Option 3"), "row 6 should be unselected list item: got {:?}", screen.lines[6]);
        assert_eq!(screen.lines[7].trim_end(), "", "row 7 should be empty input line");
    }

}
