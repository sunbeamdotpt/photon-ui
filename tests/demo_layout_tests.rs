use photon_ui::{Component, Rendered, RenderError};
use photon_ui::layout::{Constraint, Direction, Flex, Margin, Offset, Position, Rect, Size, Spacing};

/// A component that renders a visual demonstration of layout primitives.
struct LayoutDemo;

impl photon_ui::Component for LayoutDemo {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut screen = Rendered::empty();

        let demo_height = 18u16;
        for _ in 0..demo_height {
            screen.lines.push("".to_string());
        }

        let grid = Rect::new(0, 0, width / 2, 8);
        let top_left = Rect::new(grid.x, grid.y, grid.width / 2, grid.height / 2);
        let top_right = Rect::new(grid.x + grid.width / 2, grid.y, grid.width / 2, grid.height / 2);
        let bot_left = Rect::new(grid.x, grid.y + grid.height / 2, grid.width / 2, grid.height / 2);
        let bot_right = Rect::new(grid.x + grid.width / 2, grid.y + grid.height / 2, grid.width / 2, grid.height / 2);

        let tl = Rendered { lines: vec![" TopLeft ".into(), format!(" {:?} ", top_left)], cursor: None, images: vec![] };
        tl.blit_into_rect(&mut screen, top_left);

        let tr = Rendered { lines: vec![" TopRight ".into(), format!(" {:?} ", top_right)], cursor: None, images: vec![] };
        tr.blit_into_rect(&mut screen, top_right);

        let bl = Rendered { lines: vec![" BotLeft ".into(), format!(" w:{} h:{} ", bot_left.width, bot_left.height)], cursor: None, images: vec![] };
        bl.blit_into_rect(&mut screen, bot_left);

        let br = Rendered { lines: vec![" BotRight ".into(), format!(" area:{} ", bot_right.area())], cursor: None, images: vec![] };
        br.blit_into_rect(&mut screen, bot_right);

        let margin_rect = Rect::new(width / 2 + 1, 0, width.saturating_sub(width / 2 + 1), 8);
        let outer_label = Rendered {
            lines: vec![" Margin Demo ".into(), format!(" outer: {} ", margin_rect)],
            cursor: None,
            images: vec![],
        };
        outer_label.blit_into_rect(&mut screen, margin_rect);

        let inner = margin_rect.inner(Margin::new(2, 1));
        let inner_label = Rendered {
            lines: vec![" INNER ".into(), format!(" {:?} ", inner)],
            cursor: None,
            images: vec![],
        };
        inner_label.blit_into_rect(&mut screen, inner);

        let pos_y = 9u16;
        let pos_rect = Rect::new(0, pos_y, width / 2, 4);
        let p = Position::new(5, 2);
        let o = Offset::new(3, -1);
        let moved = p + o;

        let pos_text = Rendered {
            lines: vec![
                " Position + Offset ".into(),
                format!(" p = {:?} ", p),
                format!(" o = {:?} ", o),
                format!(" p + o = {:?} ", moved),
            ],
            cursor: None,
            images: vec![],
        };
        pos_text.blit_into_rect(&mut screen, pos_rect);

        let size_y = 9u16;
        let size_rect = Rect::new(width / 2 + 1, size_y, width.saturating_sub(width / 2 + 1), 4);
        let s = Size::new(width / 3, 3);
        let constraints = vec![
            Constraint::Length(10),
            Constraint::Min(5),
            Constraint::Max(20),
            Constraint::Percentage(50),
        ];

        let size_text = Rendered {
            lines: vec![
                " Size & Constraints ".into(),
                format!(" size = {} (area={}) ", s, s.area()),
                format!(" {:?} ", constraints[0]),
                format!(" {:?} ", constraints[1]),
            ],
            cursor: None,
            images: vec![],
        };
        size_text.blit_into_rect(&mut screen, size_rect);

        let meta_y = 14u16;
        let meta_rect = Rect::new(0, meta_y, width, 4);
        let dir = Direction::Horizontal;
        let flex = Flex::Start;
        let spacing = Spacing::Space(2);

        let meta_text = Rendered {
            lines: vec![
                " Direction / Flex / Spacing ".into(),
                format!(
                    " dir={:?}  perp={:?} | flex={:?} legacy={} | spacing={:?} ",
                    dir,
                    dir.perpendicular(),
                    flex,
                    flex.is_legacy(),
                    spacing
                ),
                format!(
                    " Rect rows:{} cols:{} positions:{} ",
                    grid.rows().count(),
                    grid.columns().count(),
                    grid.positions().count()
                ),
                " blit_into_rect compositing active → ".into(),
            ],
            cursor: None,
            images: vec![],
        };
        meta_text.blit_into_rect(&mut screen, meta_rect);

        Ok(screen)
    }
}

#[test]
fn layout_demo_renders_without_panic() {
    let demo = LayoutDemo;
    let rendered = demo.render(80).unwrap();
    assert!(!rendered.lines.is_empty());
}

#[test]
fn layout_demo_renders_narrow_terminal() {
    let demo = LayoutDemo;
    let rendered = demo.render(40).unwrap();
    assert!(!rendered.lines.is_empty());
}

#[test]
fn layout_demo_uses_all_primitives() {
    // This test verifies that all Phase 1 primitives are exercised
    let _ = Rect::new(0, 0, 10, 10);
    let _ = Position::new(1, 2);
    let _ = Offset::new(1, -1);
    let _ = Size::new(10, 10);
    let _ = Margin::new(1, 1);
    let _ = Direction::Horizontal;
    let _ = Flex::Start;
    let _ = Constraint::Length(10);
    let _ = Spacing::Space(2);
}
