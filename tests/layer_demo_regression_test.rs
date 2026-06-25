use photon_ui::{
    Component,
    Layer,
    RenderError,
    Rendered,
    Shadow,
    compositor::Compositor,
};

struct Positioned {
    child: Box<dyn Component>,
    x: u16,
    y: u16,
}

impl Positioned {
    fn new(child: Box<dyn Component>, x: u16, y: u16) -> Self {
        Self { child, x, y }
    }
}

impl Component for Positioned {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let empty_line = " ".repeat(width as usize);
        for _ in 0..self.y {
            lines.push(empty_line.clone());
        }

        let inner_width = width.saturating_sub(self.x);
        let child_rendered = self.child.render(inner_width)?;
        let left_pad = " ".repeat(self.x as usize);

        for line in child_rendered.lines {
            let mut final_line = left_pad.clone();
            final_line.push_str(&line);
            let vw = photon_ui::utils::visible_width(&final_line);
            if vw < width as usize {
                final_line.push_str(&" ".repeat(width as usize - vw));
            }
            lines.push(final_line);
        }

        Ok(Rendered {
            lines,
            cursor: None,
            images: child_rendered.images,
        })
    }
}

struct Card {
    title: String,
    lines: Vec<String>,
    width: u16,
}

impl Card {
    fn new(title: &str, lines: &[&str], width: u16) -> Self {
        Self {
            title: title.into(),
            lines: lines.iter().map(|s| s.to_string()).collect(),
            width,
        }
    }
}

impl Component for Card {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let w = self.width.max(4) as usize;
        let inner = w - 2;
        let text_w = inner.saturating_sub(2);
        let mut lines = Vec::new();

        lines.push(format!("┌{:─<inner$}┐", "", inner = inner));
        lines.push(format!(
            "│ {: <text_w$} │",
            self.title.chars().take(text_w).collect::<String>(),
            text_w = text_w
        ));
        for line in &self.lines {
            lines.push(format!(
                "│ {: <text_w$} │",
                line.chars().take(text_w).collect::<String>(),
                text_w = text_w
            ));
        }
        lines.push(format!("└{:─<inner$}┘", "", inner = inner));

        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }
}

struct PatternBox {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl PatternBox {
    fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl Component for PatternBox {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let inner = self.width.saturating_sub(2) as usize;
        let body_h = self.height.saturating_sub(2) as usize;
        let left = " ".repeat(self.x as usize);
        let row = format!("{:~<inner$}", "", inner = inner);

        for _ in 0..self.y {
            lines.push(String::new());
        }

        lines.push(format!("{}┌{:─<inner$}┐", left, "", inner = inner));
        for _ in 0..body_h {
            lines.push(format!("{}│{}│", left, row));
        }
        lines.push(format!("{}└{:─<inner$}┘", left, "", inner = inner));

        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                | Some('[') => {
                    chars.next();
                    while let Some(c) = chars.peek() {
                        let c2 = *c;
                        chars.next();
                        if c2.is_alphabetic() {
                            break;
                        }
                    }
                },
                | Some(']') => {
                    chars.next();
                    while let Some(c) = chars.peek() {
                        let c2 = *c;
                        chars.next();
                        if c2 == '\x07' {
                            break;
                        }
                        if c2 == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                },
                | _ => {},
            }
            continue;
        }
        out.push(ch);
    }
    out
}

#[test]
fn dim_card_renders_as_a_contiguous_box() {
    let mut comp = Compositor::new(80, 24);

    let pattern = PatternBox::new(2, 4, 70, 16);

    let dim_card = Card::new(
        "Dim Shadow",
        &[
            "This layer dims the",
            "background that is",
            "hidden behind it.",
        ],
        24,
    );
    let mut dim_layer = Layer::with_component(Box::new(Positioned::new(Box::new(dim_card), 8, 10)));
    dim_layer.shadow = Shadow::Dim {
        style: "\x1b[2m".into(),
    };

    // Composite front-to-back (TUI::render_frame iterates layers in reverse).
    comp.add_layer(&dim_layer.render(80, 24, None), &dim_layer.shadow);
    comp.add_layer(&pattern.render(80).unwrap(), &Shadow::None);

    let out = comp.finalize();
    let expected = [
        "┌──────────────────────┐",
        "│ Dim Shadow           │",
        "│ This layer dims the  │",
        "│ background that is   │",
        "│ hidden behind it.    │",
        "└──────────────────────┘",
    ];

    for (offset, line) in expected.iter().enumerate() {
        let row = 10 + offset;
        let plain = strip_ansi(&out.lines[row]);
        assert!(
            plain.contains(line),
            "row {} should contain {:?}; got {:?}",
            row,
            line,
            plain
        );
    }
}

#[test]
fn overlapping_drop_card_is_visible() {
    let mut comp = Compositor::new(80, 24);

    let pattern = PatternBox::new(2, 4, 70, 16);

    let dim_card = Card::new(
        "Dim Shadow",
        &[
            "This layer dims the",
            "background that is",
            "hidden behind it.",
        ],
        24,
    );
    let mut dim_layer = Layer::with_component(Box::new(Positioned::new(Box::new(dim_card), 8, 10)));
    dim_layer.shadow = Shadow::Dim {
        style: "\x1b[2m".into(),
    };

    let drop_card = Card::new(
        "Drop Shadow",
        &[
            "This layer casts a",
            "shadow offset from",
            "its bounding box.",
        ],
        24,
    );
    let mut drop_layer =
        Layer::with_component(Box::new(Positioned::new(Box::new(drop_card), 28, 13)));
    drop_layer.shadow = Shadow::Drop {
        style: "\x1b[48;5;240m".into(),
        offset_x: 2,
        offset_y: 1,
    };

    // Front-to-back compositing order.
    comp.add_layer(&drop_layer.render(80, 24, None), &drop_layer.shadow);
    comp.add_layer(&dim_layer.render(80, 24, None), &dim_layer.shadow);
    comp.add_layer(&pattern.render(80).unwrap(), &Shadow::None);

    let out = comp.finalize();
    assert!(out.lines.iter().any(|l| l.contains("Dim Shadow")));
    assert!(out.lines.iter().any(|l| l.contains("Drop Shadow")));
}
