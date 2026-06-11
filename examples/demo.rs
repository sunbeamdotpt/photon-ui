//! Photon UI — Comprehensive Multi-Page Demo
//!
//! Run with:
//! ```bash
//! cargo run --example demo
//! ```
//!
//! Showcases **every** Photon UI component across six pages:
//!
//! | Page | Components |
//! |------|-----------|
//! | 1    | Text, TruncatedText, Box, Markdown, Spacer + Layout primitives |
//! | 2    | Input, Editor, SelectList, SettingsList |
//! | 3    | Loader, CancellableLoader, Overlay |
//! | 4    | Layout engine, Theme + Button, Panel |
//! | 5    | Full dashboard — all components |
//! | 6    | Layer compositor + shadows |
//!
//! # Keybindings
//!
//! | Key | Action |
//! |-----|--------|
//! | `1`–`6` | Switch demo page |
//! | `Tab` / `Shift+Tab` | Cycle focus |
//! | `t` | Toggle light/dark theme (on page 4) |
//! | `q` | Quit |
//! | `Ctrl+C` | Quit |
//!
//! Page-specific bindings are shown on each page.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::Duration,
};

use crossterm::event::{
    KeyCode,
    KeyModifiers,
};
use photon_ui::{
    Anchor,
    Event,
    InputResult,
    Layer,
    Overlay,
    OverlayConstraints,
    OverlayPosition,
    RenderError,
    Rendered,
    Shadow,
    TUI,
    Terminal,
    components::{
        Box as BoxComponent,
        Breadcrumbs,
        Button,
        CancellableLoader,
        Div,
        Divider,
        Editor,
        Header,
        ImageWidget,
        Input,
        Loader,
        Markdown,
        Modal,
        Panel,
        ProgressBar,
        Segment,
        SelectList,
        SettingsList,
        Sidebar,
        SidebarItem,
        Spacer,
        StatusBar,
        Table,
        Tabs,
        Text,
        TreeNode,
        TreeView,
        TruncatedText,
        table::{
            Column,
            Row,
        },
    },
    layout::{
        Border,
        Constraint,
        Direction,
        Flex,
        Layout,
        Margin,
        Offset,
        Position,
        Rect,
        Size,
        Spacing,
    },
    terminal::ProcessTerminal,
    theme::Theme,
};

const DEMO_MARKDOWN: &str = r#"# Photon UI

A **Rust** TUI library with:

- *Bold*, *italic*, and `inline code`
- Headings and paragraphs
- CommonMark support via pulldown-cmark
"#;

/// Wrapper that lets us tick a shared Loader while it's mounted in the TUI.
struct SharedLoader(Rc<RefCell<Loader>>);

impl photon_ui::Component for SharedLoader {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        self.0.borrow().render(width)
    }
}

/// Wrapper that lets us tick a shared CancellableLoader while it's mounted.
struct SharedCancellableLoader(Rc<RefCell<CancellableLoader>>);

impl photon_ui::Component for SharedCancellableLoader {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        self.0.borrow().render(width)
    }

    fn handle_input(&mut self, event: &Event) -> InputResult {
        self.0.borrow_mut().handle_input(event)
    }
}

/// A panel that fills its assigned rect with a solid background color.
struct ColoredPanel {
    label: String,
    bg: u8, // ANSI background color code (40-47)
}

impl ColoredPanel {
    fn new(label: &str, bg: u8) -> Self {
        Self {
            label: label.to_string(),
            bg,
        }
    }
}

impl photon_ui::Component for ColoredPanel {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let rect = photon_ui::layout::Rect::new(0, 0, width, 1);
        self.render_rect(rect)
    }

    fn render_rect(&self, rect: photon_ui::layout::Rect) -> Result<Rendered, RenderError> {
        let mut lines = Vec::new();
        let label = format!(" {} ", self.label);
        let w = rect.width as usize;
        let first = format!("\x1b[{}m{:width$}\x1b[0m", self.bg, label, width = w);
        lines.push(first);
        for _ in 1..rect.height {
            lines.push(format!("\x1b[{}m{:width$}\x1b[0m", self.bg, "", width = w));
        }
        Ok(Rendered {
            lines,
            cursor: None,
            images: Vec::new(),
        })
    }
}

/// Positions a child component at an absolute `(x, y)` offset within the
/// terminal. Leading empty rows and columns are transparent so lower layers
/// remain visible around the positioned content.
struct Positioned {
    child: Box<dyn photon_ui::Component>,
    x: u16,
    y: u16,
}

impl Positioned {
    fn new(child: Box<dyn photon_ui::Component>, x: u16, y: u16) -> Self {
        Self { child, x, y }
    }
}

impl photon_ui::Component for Positioned {
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

/// A component that renders a visual demonstration of layout primitives.
struct LayoutDemo;

impl photon_ui::Component for LayoutDemo {
    fn render(&self, width: u16) -> Result<Rendered, RenderError> {
        let mut screen = Rendered::empty();

        // Fill screen with blank lines up to our demo height
        let demo_height = 18u16;
        for _ in 0..demo_height {
            screen.lines.push("".to_string());
        }

        // ── Panel 1: Rect grid using blit_into_rect ──────────────────────
        let grid = Rect::new(0, 0, width / 2, 8);
        let top_left = Rect::new(grid.x, grid.y, grid.width / 2, grid.height / 2);
        let top_right = Rect::new(
            grid.x + grid.width / 2,
            grid.y,
            grid.width / 2,
            grid.height / 2,
        );
        let bot_left = Rect::new(
            grid.x,
            grid.y + grid.height / 2,
            grid.width / 2,
            grid.height / 2,
        );
        let bot_right = Rect::new(
            grid.x + grid.width / 2,
            grid.y + grid.height / 2,
            grid.width / 2,
            grid.height / 2,
        );

        let tl = Rendered {
            lines: vec![" TopLeft ".into(), format!(" {:?} ", top_left)],
            cursor: None,
            images: vec![],
        };
        tl.blit_into_rect(&mut screen, top_left);

        let tr = Rendered {
            lines: vec![" TopRight ".into(), format!(" {:?} ", top_right)],
            cursor: None,
            images: vec![],
        };
        tr.blit_into_rect(&mut screen, top_right);

        let bl = Rendered {
            lines: vec![
                " BotLeft ".into(),
                format!(" w:{} h:{} ", bot_left.width, bot_left.height),
            ],
            cursor: None,
            images: vec![],
        };
        bl.blit_into_rect(&mut screen, bot_left);

        let br = Rendered {
            lines: vec![" BotRight ".into(), format!(" area:{} ", bot_right.area())],
            cursor: None,
            images: vec![],
        };
        br.blit_into_rect(&mut screen, bot_right);

        // ── Panel 2: Margin inner/outer demo ─────────────────────────────
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

        // ── Panel 3: Position + Offset ───────────────────────────────────
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

        // ── Panel 4: Size + Constraint types ─────────────────────────────
        let size_y = 9u16;
        let size_rect = Rect::new(
            width / 2 + 1,
            size_y,
            width.saturating_sub(width / 2 + 1),
            4,
        );
        let s = Size::new(width / 3, 3);
        let constraints = [
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

        // ── Panel 5: Direction, Flex, Spacing ────────────────────────────
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

struct DemoApp {
    tui: TUI,
    page: usize,

    // Page 2: Input / Editor state
    input_text: String,
    editor_text: String,
    input_vim: bool,
    editor_vim: bool,

    // Page 3: List state
    list_selected: usize,
    settings_selected: usize,
    settings_values: Vec<bool>,

    // Page 3: Loader / Overlay state
    loader: Rc<RefCell<Loader>>,
    cancellable: Rc<RefCell<CancellableLoader>>,
    show_overlay: bool,

    // Page 4: Design system sub-page (false = buttons/panels, true = layout engine)
    design_subpage: bool,

    // Page 6: Layer compositor demo state
    layer_show_dim: bool,
    layer_show_drop: bool,
    layer_shadow_idx: usize,
}

impl DemoApp {
    fn new(tui: TUI) -> Self {
        let mut app = Self {
            tui,
            page: 1,
            input_text: String::new(),
            editor_text: "Hello, Photon UI!\nThis is the multi-line editor.".into(),
            input_vim: false,
            editor_vim: false,
            list_selected: 0,
            settings_selected: 0,
            settings_values: vec![true, false, true],
            loader: Rc::new(RefCell::new(Loader::new(
                "Loading...",
                Some("\x1b[36m".into()),
                None,
            ))),
            cancellable: Rc::new(RefCell::new(CancellableLoader::new(
                "Background task running...",
                Some("\x1b[33m".into()),
                Some("\x1b[90m".into()),
            ))),
            show_overlay: false,
            design_subpage: false,
            layer_show_dim: true,
            layer_show_drop: true,
            layer_shadow_idx: 0,
        };
        app.load_page();
        app
    }

    fn load_page(&mut self) {
        self.tui.reset();

        // Header with page indicator (skip on pages 4-5 where layout owns all children)
        if self.page != 4 && self.page != 5 {
            let header = format!(
                " Photon UI Demo  |  Page {}/6  |  1-6=pages  Tab=focus  q=quit ",
                self.page
            );
            self.tui
                .mount(std::boxed::Box::new(Text::new(&header, 0, 0)));
            self.tui.mount(std::boxed::Box::new(Spacer::new(1)));
        }

        match self.page {
            | 1 => self.load_page_basics(),
            | 2 => self.load_page_input_and_lists(),
            | 3 => self.load_page_dynamic(),
            | 4 => self.load_page_design_system(),
            | 5 => self.load_page_dashboard(),
            | 6 => self.load_page_layers(),
            | _ => {},
        }
    }

    fn load_page_basics(&mut self) {
        // ── Text components ──
        self.tui.mount(std::boxed::Box::new(Text::new(
            "Text component with pad_x=2, pad_y=1:",
            0,
            0,
        )));
        self.tui.mount(std::boxed::Box::new(Text::new(
            "  Indented content here",
            2,
            1,
        )));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        self.tui.mount(std::boxed::Box::new(Text::new(
            "TruncatedText (narrow terminal will ellipsis):",
            0,
            0,
        )));
        self.tui.mount(std::boxed::Box::new(TruncatedText::new(
            "This is a very long line that will be truncated with an ellipsis if the terminal is not wide enough to display it all",
            2,
            0,
        )));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        self.tui.mount(std::boxed::Box::new(Text::new(
            "Box with blue background:",
            0,
            0,
        )));
        self.tui.mount(std::boxed::Box::new(
            BoxComponent::new(2).with_background(|line, _w| format!("\x1b[44m{}\x1b[0m", line)),
        ));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        self.tui
            .mount(std::boxed::Box::new(Text::new("Markdown rendering:", 0, 0)));
        self.tui
            .mount(std::boxed::Box::new(Markdown::new(DEMO_MARKDOWN)));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        self.tui.mount(std::boxed::Box::new(Text::new(
            "Layout Primitives — blit_into_rect demo:",
            0,
            0,
        )));
        self.tui.mount(std::boxed::Box::new(LayoutDemo));
    }

    fn load_page_input_and_lists(&mut self) {
        // ── Input ──
        let help = if self.input_vim {
            "Input: vim mode (i=insert, Esc=normal, h/l=move, x=delete) | v=toggle mode"
        } else {
            "Input: Emacs mode (Ctrl+A=start, Ctrl+E=end, Ctrl+K=kill, Ctrl+Y=yank) | v=toggle mode"
        };
        self.tui.mount(std::boxed::Box::new(Text::new(help, 0, 0)));

        let mut input = Input::new();
        input.set_text(&self.input_text);
        if self.input_vim {
            input.set_vim_mode_enabled(true);
        }
        self.tui.mount(std::boxed::Box::new(input));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        let editor_help = if self.editor_vim {
            "Editor: vim mode (i=insert, Esc=normal, dd=delete line, yy=yank line) | V=toggle mode"
        } else {
            "Editor: Emacs mode (Ctrl+A=start, Ctrl+E=end, Ctrl+K=kill line) | V=toggle mode"
        };
        self.tui
            .mount(std::boxed::Box::new(Text::new(editor_help, 0, 0)));

        let mut editor = Editor::new();
        editor.set_text(&self.editor_text);
        if self.editor_vim {
            editor.set_vim_mode_enabled(true);
        }
        self.tui.mount(std::boxed::Box::new(editor));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        // ── Lists ──
        self.tui.mount(std::boxed::Box::new(Text::new(
            "SelectList — j/k or arrows to navigate, Enter to select:",
            0,
            0,
        )));

        let mut list = SelectList::new(
            vec![
                "Rust programming language".into(),
                "Python scripting".into(),
                "TypeScript web dev".into(),
                "Go systems programming".into(),
                "Zig low-level".into(),
                "C++ game engines".into(),
                "Haskell functional".into(),
            ],
            4,
        );
        list.set_selected(self.list_selected);
        self.tui.mount(std::boxed::Box::new(list));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        self.tui.mount(std::boxed::Box::new(Text::new(
            "SettingsList — j/k or arrows, Enter/Space to toggle:",
            0,
            0,
        )));

        let mut settings = SettingsList::new(vec![
            ("Enable dark mode".into(), self.settings_values[0]),
            ("Show line numbers".into(), self.settings_values[1]),
            ("Auto-save on exit".into(), self.settings_values[2]),
        ]);
        settings.set_selected(self.settings_selected);
        self.tui.mount(std::boxed::Box::new(settings));
    }

    fn load_page_dynamic(&mut self) {
        self.tui.mount(std::boxed::Box::new(Text::new(
            "Loader (auto-ticking spinner):",
            0,
            0,
        )));
        self.tui
            .mount(std::boxed::Box::new(SharedLoader(self.loader.clone())));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        self.tui.mount(std::boxed::Box::new(Text::new(
            "CancellableLoader — press Ctrl+C to cancel:",
            0,
            0,
        )));
        self.tui.mount(std::boxed::Box::new(SharedCancellableLoader(
            self.cancellable.clone(),
        )));
        self.tui.mount(std::boxed::Box::new(Spacer::new(1)));

        let overlay_hint = if self.show_overlay {
            "Overlay: ACTIVE  |  Press 'o' to close"
        } else {
            "Overlay: hidden  |  Press 'o' to open a centered popup"
        };
        self.tui
            .mount(std::boxed::Box::new(Text::new(overlay_hint, 0, 0)));

        if self.show_overlay {
            self.tui.add_overlay(Overlay {
                content: std::boxed::Box::new(Text::new(
                    "  Overlay Popup! Press 'o' to close  ",
                    0,
                    0,
                )),
                position: OverlayPosition::Anchor(Anchor::Center),
                constraints: OverlayConstraints {
                    min_width: 30,
                    max_height: 3,
                    margin: 2,
                    offset_x: 0,
                    offset_y: 0,
                    visible: None,
                },
            });
        }
    }

    fn load_page_design_system(&mut self) {
        if self.design_subpage {
            // ── Layout engine demo ──
            self.tui.set_layout(Layout::vertical([
                Constraint::Length(4),
                Constraint::Min(3),
                Constraint::Length(4),
            ]));
            self.tui.mount(Box::new(ColoredPanel::new(
                "Photon UI Demo  |  Page 4/4  |  l=toggle  Tab=focus  q=quit",
                44,
            )));
            self.tui.mount(Box::new(ColoredPanel::new(
                "Middle (Min 3) — fills remaining",
                42,
            )));
            self.tui
                .mount(Box::new(ColoredPanel::new("Bottom panel (Length 4)", 41)));
        } else {
            // ── Buttons ──
            let theme_name = match Theme::current() {
                | Theme::Light => "Light",
                | Theme::Dark => "Dark",
            };
            self.tui.mount(Box::new(Text::new(
                format!(
                    "Beam Design System  |  Theme: {}  |  Press 't' to toggle, 'l' for layout demo",
                    theme_name
                ),
                0,
                0,
            )));
            self.tui.mount(Box::new(Spacer::new(1)));

            self.tui.mount(Box::new(Text::new("Primary:", 0, 0)));
            self.tui.mount(Box::new(Button::primary("Primary button")));
            self.tui.mount(Box::new(Spacer::new(1)));

            self.tui.mount(Box::new(Text::new("Dark:", 0, 0)));
            self.tui.mount(Box::new(Button::dark("Dark button")));
            self.tui.mount(Box::new(Spacer::new(1)));

            self.tui.mount(Box::new(Text::new("Cream:", 0, 0)));
            self.tui.mount(Box::new(Button::cream("Cream button")));
            self.tui.mount(Box::new(Spacer::new(1)));

            self.tui.mount(Box::new(Text::new("Ghost:", 0, 0)));
            self.tui.mount(Box::new(Button::ghost("Ghost button")));
            self.tui.mount(Box::new(Spacer::new(1)));

            self.tui.mount(Box::new(Text::new("Text:", 0, 0)));
            self.tui.mount(Box::new(Button::text("Text button")));
            self.tui.mount(Box::new(Spacer::new(1)));

            // ── Panels ──
            self.tui.mount(Box::new(
                Panel::new()
                    .title("Rounded")
                    .lines(vec!["Default rounded corners".into(), "╭─╮ │ │ ╰─╯".into()]),
            ));
            self.tui.mount(Box::new(Spacer::new(1)));

            self.tui.mount(Box::new(
                Panel::new()
                    .border(photon_ui::layout::Border::THIN)
                    .title("Thin")
                    .lines(vec![
                        "Sharp corners with thin lines".into(),
                        "┌─┐ │ │ └─┘".into(),
                    ]),
            ));
        }
    }

    fn load_page_dashboard(&mut self) {
        self.tui.set_layout(Layout::vertical([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Breadcrumbs
            Constraint::Min(10),   // DashboardBody
            Constraint::Length(1), // StatusBar
        ]));

        self.tui
            .mount(Box::new(Header::new("Dashboard Demo").action("q:quit")));
        self.tui.mount(Box::new(Breadcrumbs::new(vec![
            "Home",
            "Dashboard",
            "Overview",
        ])));

        // ── DashboardBody: Sidebar + MainContent ──
        let mut dashboard_body = Div::new(Layout::horizontal([
            Constraint::Length(16), // Sidebar
            Constraint::Min(10),    // MainContent
        ]));

        let mut sidebar_column = Div::new(Layout::vertical([
            Constraint::Length(4), // Nav items
            Constraint::Min(1),    // Collapsible tags panel
        ]));
        let sidebar = Sidebar::new(vec![
            SidebarItem::new("Overview").icon("📊"),
            SidebarItem::new("Files").icon("📁"),
            SidebarItem::new("Settings").icon("⚙️"),
            SidebarItem::new("Profile").icon("👤"),
        ]);
        sidebar_column.push(Box::new(sidebar));

        let tags_panel = Div::new(Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]))
        .border(Border::ROUNDED)
        .title("Tags")
        .collapsible(true)
        .child(Box::new(Text::new("rust", 0, 0)))
        .child(Box::new(Text::new("tui", 0, 0)))
        .child(Box::new(Text::new("cli", 0, 0)));
        sidebar_column.push(Box::new(tags_panel));
        dashboard_body.push(Box::new(sidebar_column));

        // ── MainContent (vertical) with Dividers ──
        // Total = 24 lines. Each component gets exactly the height it needs
        // so nothing is clipped.
        let mut main_content = Div::new(Layout::vertical([
            Constraint::Length(1), // Tabs
            Constraint::Length(1), // Divider
            Constraint::Length(1), // ProgressBar row
            Constraint::Length(1), // Divider
            Constraint::Length(4), // DataRow  (Table 2 rows + header + sep = 4)
            Constraint::Length(1), // Divider
            Constraint::Length(2), // FormsRow
            Constraint::Length(1), // Divider
            Constraint::Length(3), // ListsRow (SelectList 3 visible)
            Constraint::Length(1), // Divider
            Constraint::Length(1), // ButtonsRow
            Constraint::Length(1), // Divider
            Constraint::Length(5), // ContentRow (Panel top + 3 lines + bottom = 5)
            Constraint::Length(1), // Divider
            Constraint::Length(1), // SystemRow
        ]));

        main_content.push(Box::new(Tabs::new(vec!["Overview", "Resources", "Logs"])));
        main_content.push(Box::new(Divider::horizontal()));

        // ProgressBar row
        let mut progress_row = Div::new(Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]));
        progress_row.push(Box::new(ProgressBar::new("CPU", 0.45).width(15)));
        progress_row.push(Box::new(ProgressBar::new("RAM", 1.0).width(15)));
        main_content.push(Box::new(progress_row));
        main_content.push(Box::new(Divider::horizontal().labeled("Data")));

        // DataRow: Table + TreeView
        let mut data_row = Div::new(Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]));
        let table = Table::new(
            vec![
                Column::new("name", "Name").width(15),
                Column::new("status", "Status").width(10),
                Column::new("size", "Size").width(8),
            ],
            vec![
                Row::new(HashMap::from([
                    ("name".to_string(), "src/main.rs".to_string()),
                    ("status".to_string(), "active".to_string()),
                    ("size".to_string(), "12.4KB".to_string()),
                ])),
                Row::new(HashMap::from([
                    ("name".to_string(), "Cargo.toml".to_string()),
                    ("status".to_string(), "active".to_string()),
                    ("size".to_string(), "1.2KB".to_string()),
                ])),
            ],
        );
        data_row.push(Box::new(table));
        let tree = TreeView::new(vec![
            TreeNode::new("src")
                .child(TreeNode::new("main.rs"))
                .child(TreeNode::new("lib.rs")),
            TreeNode::new("tests").child(TreeNode::new("integration.rs")),
        ]);
        data_row.push(Box::new(tree));
        main_content.push(Box::new(data_row));
        main_content.push(Box::new(Divider::horizontal().labeled("Forms")));

        // FormsRow: Input + Editor
        let mut forms_row = Div::new(Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]));
        let mut input = Input::new();
        input.set_text("Search components...");
        forms_row.push(Box::new(input));
        let mut editor = Editor::new();
        editor.set_text("fn main() {\n    println!(\"Hello\");\n}");
        forms_row.push(Box::new(editor));
        main_content.push(Box::new(forms_row));
        main_content.push(Box::new(Divider::horizontal().labeled("Lists")));

        // ListsRow: SelectList + SettingsList
        let mut lists_row = Div::new(Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]));
        let mut select_list = SelectList::new(
            vec![
                "Rust".into(),
                "Python".into(),
                "TypeScript".into(),
                "Go".into(),
                "Zig".into(),
            ],
            3,
        );
        select_list.set_selected(1);
        lists_row.push(Box::new(select_list));
        let mut settings = SettingsList::new(vec![
            ("Dark mode".into(), true),
            ("Auto-save".into(), false),
            ("Notifications".into(), true),
        ]);
        settings.set_selected(0);
        lists_row.push(Box::new(settings));
        main_content.push(Box::new(lists_row));
        main_content.push(Box::new(Divider::horizontal()));

        // ButtonsRow
        let mut buttons_row = Div::new(Layout::horizontal([
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ]));
        buttons_row.push(Box::new(Button::primary("Primary")));
        buttons_row.push(Box::new(Button::dark("Dark")));
        buttons_row.push(Box::new(Button::ghost("Ghost")));
        buttons_row.push(Box::new(Button::text("Text")));
        main_content.push(Box::new(buttons_row));
        main_content.push(Box::new(Divider::horizontal().labeled("Content")));

        // ContentRow: Markdown + Panel
        let mut content_row = Div::new(Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]));
        content_row.push(Box::new(Markdown::new(DEMO_MARKDOWN)));
        content_row.push(Box::new(Panel::new().title("Info").lines(vec![
            "\x1b[1m\x1b[97mPhoton UI v0.1.0\x1b[0m".into(),
            "A Rust TUI library".into(),
            "\x1b[31mBuilt with ♥\x1b[0m".into(),
        ])));
        main_content.push(Box::new(content_row));
        main_content.push(Box::new(Divider::horizontal().labeled("System")));

        // SystemRow: Loader + CancellableLoader + ImageWidget + Status + TruncatedText
        let mut system_row = Div::new(Layout::horizontal([
            Constraint::Length(14),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Min(5),
        ]));
        system_row.push(Box::new(SharedLoader(self.loader.clone())));
        system_row.push(Box::new(SharedCancellableLoader(self.cancellable.clone())));
        system_row.push(Box::new(ImageWidget::new(
            vec![],
            "image/png",
            Some("[image]".to_string()),
        )));
        system_row.push(Box::new(Text::new("\x1b[32m●\x1b[0m online", 0, 0)));
        system_row.push(Box::new(TruncatedText::new(
            "This is a very long line that will be truncated with an ellipsis if the terminal is not wide enough to display it all",
            0,
            0,
        )));
        main_content.push(Box::new(system_row));

        dashboard_body.push(Box::new(main_content));
        self.tui.mount(Box::new(dashboard_body));

        self.tui.mount(Box::new(
            StatusBar::new()
                .left(Segment::new("MODE: normal"))
                .right(Segment::new("1-6:pages  q:quit")),
        ));
    }

    fn load_page_layers(&mut self) {
        // ── Base layer: patterned background so transparency is visible ──
        self.tui.mount(Box::new(Text::new(
            "Layer 0 (base): every cell below the cards is preserved and visible through gaps.",
            0,
            0,
        )));
        self.tui.mount(Box::new(Text::new(
            "Transparent padding around the floating cards lets this text peek through.",
            0,
            0,
        )));
        self.tui.mount(Box::new(Text::new(
            "Try: d=toggle dim layer  D=toggle drop layer  s=cycle drop shadow",
            0,
            0,
        )));

        // ── Layer 1: a card with a dim shadow over everything behind it ──
        let dim_card = Panel::new().title("Dim Shadow").lines(vec![
            "This layer dims the".into(),
            "background that is".into(),
            "hidden behind it.".into(),
        ]);
        let mut dim_layer =
            Layer::with_component(Box::new(Positioned::new(Box::new(dim_card), 4, 6)));
        dim_layer.shadow = Shadow::Dim {
            style: "\x1b[2m".into(),
        };
        dim_layer.visible = self.layer_show_dim;
        self.tui.add_layer(dim_layer);

        // ── Layer 2: a card with a drop shadow ──
        let drop_shadow = match self.layer_shadow_idx % 4 {
            | 0 => Shadow::Drop {
                style: "\x1b[2m".into(),
                offset_x: 2,
                offset_y: 1,
            },
            | 1 => Shadow::Drop {
                style: "\x1b[90m".into(),
                offset_x: 3,
                offset_y: 2,
            },
            | 2 => Shadow::Drop {
                style: "\x1b[34m".into(),
                offset_x: 1,
                offset_y: 1,
            },
            | _ => Shadow::Drop {
                style: "\x1b[7m".into(),
                offset_x: 2,
                offset_y: 2,
            },
        };
        let drop_card = Panel::new().title("Drop Shadow").lines(vec![
            "This layer casts a".into(),
            "shadow offset from".into(),
            "its bounding box.".into(),
        ]);
        let mut drop_layer =
            Layer::with_component(Box::new(Positioned::new(Box::new(drop_card), 36, 10)));
        drop_layer.shadow = drop_shadow;
        drop_layer.visible = self.layer_show_drop;
        self.tui.add_layer(drop_layer);
    }

    /// Advance animation frames. Call periodically from the event loop.
    fn tick(&mut self) {
        if self.page == 3 || self.page == 5 {
            self.loader.borrow_mut().tick();
            self.cancellable.borrow_mut().tick();
        }
    }

    /// Handle an input event. Returns `false` when the app should quit.
    fn handle_input(&mut self, event: &Event) -> bool {
        // Global navigation
        if let Event::Key(key) = event {
            match key.code {
                | KeyCode::Char('1') => {
                    self.page = 1;
                    self.load_page();
                    return true;
                },
                | KeyCode::Char('2') => {
                    self.page = 2;
                    self.load_page();
                    return true;
                },
                | KeyCode::Char('3') => {
                    self.page = 3;
                    self.load_page();
                    return true;
                },
                | KeyCode::Char('4') => {
                    self.page = 4;
                    self.load_page();
                    return true;
                },
                | KeyCode::Char('5') => {
                    self.page = 5;
                    self.load_page();
                    return true;
                },
                | KeyCode::Char('6') => {
                    self.page = 6;
                    self.load_page();
                    return true;
                },
                | KeyCode::Char('q') if key.modifiers.is_empty() => return false,
                | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return false;
                },
                | _ => {},
            }
        }

        // Page-specific global keys
        if self.page == 2 &&
            let Event::Key(key) = event
        {
            if key.code == KeyCode::Char('v') && key.modifiers.is_empty() {
                self.input_vim = !self.input_vim;
                self.load_page();
                return true;
            }
            if key.code == KeyCode::Char('V') && key.modifiers.contains(KeyModifiers::SHIFT) {
                self.editor_vim = !self.editor_vim;
                self.load_page();
                return true;
            }
        }

        if self.page == 3 &&
            let Event::Key(key) = event &&
            key.code == KeyCode::Char('o') &&
            key.modifiers.is_empty()
        {
            self.show_overlay = !self.show_overlay;
            self.load_page();
            return true;
        }

        if self.page == 4 &&
            let Event::Key(key) = event
        {
            if key.code == KeyCode::Char('t') && key.modifiers.is_empty() {
                let next = match Theme::current() {
                    | Theme::Light => Theme::Dark,
                    | Theme::Dark => Theme::Light,
                };
                Theme::set(next);
                self.load_page();
                return true;
            }
            if key.code == KeyCode::Char('l') && key.modifiers.is_empty() {
                self.design_subpage = !self.design_subpage;
                self.load_page();
                return true;
            }
        }

        if self.page == 5 &&
            let Event::Key(key) = event &&
            key.code == KeyCode::Char('m') &&
            key.modifiers.is_empty()
        {
            if self.tui.modal_active() {
                self.tui.dismiss_modal();
            } else {
                let modal_content = Div::new(Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]))
                .child(Box::new(Text::new("This is a modal dialog.", 0, 0)))
                .child(Box::new(Text::new("Press Esc to dismiss.", 0, 0)));
                self.tui
                    .show_modal(Box::new(Modal::new(Box::new(modal_content)).title("Modal")));
            }
            return true;
        }

        if self.page == 6 &&
            let Event::Key(key) = event
        {
            if key.code == KeyCode::Char('d') && key.modifiers.is_empty() {
                self.layer_show_dim = !self.layer_show_dim;
                self.load_page();
                return true;
            }
            if key.code == KeyCode::Char('D') && key.modifiers.contains(KeyModifiers::SHIFT) {
                self.layer_show_drop = !self.layer_show_drop;
                self.load_page();
                return true;
            }
            if key.code == KeyCode::Char('s') && key.modifiers.is_empty() {
                self.layer_shadow_idx += 1;
                self.load_page();
                return true;
            }
        }

        self.tui.handle_input(event);
        true
    }

    fn render(&mut self) -> std::io::Result<()> {
        self.tui.render_frame()
    }
}

/// Attempt to detect terminal background color via OSC 11 query.
/// Returns `true` for dark, `false` for light, `None` if detection fails.
fn detect_dark_background() -> Option<bool> {
    use std::{
        io::{
            self,
            Read,
            Write,
        },
        time::{
            Duration,
            Instant,
        },
    };

    // Send OSC 11 query
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b]11;?\x07").ok()?;
    stdout.flush().ok()?;

    // Give the terminal a moment to respond
    std::thread::sleep(Duration::from_millis(50));

    let mut response = Vec::new();
    let mut buf = [0u8; 128];
    let start = Instant::now();

    while start.elapsed() < Duration::from_millis(200) {
        match io::stdin().read(&mut buf) {
            | Ok(0) => break,
            | Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                if response.contains(&0x07) {
                    break;
                }
            },
            | Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            },
            | Err(_) => break,
        }
    }

    if response.is_empty() {
        return None;
    }

    let s = String::from_utf8_lossy(&response);

    // Parse \x1b]11;rgb:RRRR/GGGG/BBBB\x07 or \x1b]11;rgb:RR/GG/BB\x07
    let rgb_idx = s.find("rgb:")?;
    let rgb_part = &s[rgb_idx + 4..];
    let end_idx = rgb_part.find(['\x07', '\x1b']).unwrap_or(rgb_part.len());
    let rgb = &rgb_part[..end_idx];

    let channels: Vec<&str> = rgb.split('/').collect();
    if channels.len() != 3 {
        return None;
    }

    let hex2 = |s: &str| u8::from_str_radix(&s[..2.min(s.len())], 16).ok();
    let r = hex2(channels[0])? as f32 / 255.0;
    let g = hex2(channels[1])? as f32 / 255.0;
    let b = hex2(channels[2])? as f32 / 255.0;

    // Relative luminance (sRGB)
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    Some(luminance < 0.5)
}

/// Detect the terminal's background color and return the matching theme.
fn detect_terminal_theme() -> Theme {
    // Try OSC 11 query first
    if let Some(is_dark) = detect_dark_background() {
        return if is_dark { Theme::Dark } else { Theme::Light };
    }

    // Fall back to COLORFGBG (xterm, rxvt, etc.)
    if let Ok(fgbg) = std::env::var("COLORFGBG") {
        let parts: Vec<&str> = fgbg.split(';').collect();
        if let Some(bg) = parts.get(1) &&
            let Ok(n) = bg.parse::<u8>()
        {
            // xterm: 0-7 are dark colors, 8-15 are light
            return if n <= 7 { Theme::Dark } else { Theme::Light };
        }
    }

    Theme::Dark
}

fn main() -> std::io::Result<()> {
    let mut term = ProcessTerminal::new();
    term.start()?;

    // Auto-detect terminal background and set theme before rendering
    let detected = detect_terminal_theme();
    Theme::set(detected);

    let tui = TUI::new(std::boxed::Box::new(term));
    let mut app = DemoApp::new(tui);

    // Initial render
    app.render()?;

    // Event loop
    loop {
        // Tick animations at ~10 fps
        app.tick();
        app.render()?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            let event = match crossterm::event::read()? {
                | crossterm::event::Event::Key(key) => Event::Key(key),
                | crossterm::event::Event::Resize(w, h) => Event::Resize(w, h),
                | crossterm::event::Event::Mouse(m) => Event::Mouse(m),
                | crossterm::event::Event::Paste(p) => Event::Paste(p),
                | crossterm::event::Event::FocusGained => Event::FocusGained,
                | crossterm::event::Event::FocusLost => Event::FocusLost,
            };

            if !app.handle_input(&event) {
                break;
            }
            app.render()?;
        }
    }

    // CRITICAL: restore terminal state (leave alternate screen, disable raw mode,
    // show cursor)
    app.tui.stop()?;

    Ok(())
}
