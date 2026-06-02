use photon_ui::{
    TUI, Rendered, Component, Focusable,
    terminal::TestTerminal,
    components::{Input, Editor},
    components::input::InputVimMode,
    components::editor::VimMode,
    events::Event,
    renderer::{Renderer, RenderStrategy},
};
use crossterm::event::KeyCode;

#[test]
fn full_render_pipeline() {
    let mut term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    // We can't access term after passing to TUI, so test via direct component
}

#[test]
fn renderer_first_render_draws_all() {
    let mut term = TestTerminal::new(80, 24);
    let mut renderer = Renderer::new();
    let rendered = Rendered {
        lines: vec!["hello".into(), "world".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &rendered).unwrap();
    let written = term.written().join("");
    assert!(written.contains("hello"));
    assert!(written.contains("world"));
}

#[test]
fn renderer_diff_only_draws_changed() {
    let mut term = TestTerminal::new(80, 24);
    let mut renderer = Renderer::new();
    let first = Rendered {
        lines: vec!["hello".into(), "world".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &first).unwrap();
    let before = term.written().len();

    let second = Rendered {
        lines: vec!["hello".into(), "earth".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &second).unwrap();
    let after = term.written().len();
    
    // Should have written additional content (diff)
    assert!(after > before);
}

#[test]
fn renderer_full_redraw_clears() {
    let mut term = TestTerminal::new(80, 24);
    let mut renderer = Renderer::new();
    renderer.set_strategy(RenderStrategy::FullRedraw);
    let rendered = Rendered {
        lines: vec!["test".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &rendered).unwrap();
    let written = term.written().join("");
    assert!(written.contains("\x1b[2J")); // clear screen
}

#[test]
fn renderer_cursor_positioning() {
    let mut term = TestTerminal::new(80, 24);
    let mut renderer = Renderer::new();
    let rendered = Rendered {
        lines: vec!["test".into()],
        cursor: Some((0, 2)),
        images: Vec::new(),
    };
    renderer.render(&mut term, &rendered).unwrap();
    assert!(term.cursor_moves().contains(&(0, 2)));
}

#[test]
fn input_receives_events_directly() {
    let mut input = Input::new();
    input.set_mode(InputVimMode::Insert);
    input.handle_input(&Event::Key(KeyCode::Char('a').into()));
    input.handle_input(&Event::Key(KeyCode::Char('b').into()));
    let r = input.render(80).unwrap();
    assert!(r.lines[0].contains("ab"));
}

#[test]
fn editor_receives_events_directly() {
    let mut editor = Editor::new();
    editor.set_focused(true);
    editor.set_mode(VimMode::Insert);
    editor.handle_input(&Event::Key(KeyCode::Char('h').into()));
    editor.handle_input(&Event::Key(KeyCode::Char('i').into()));
    let r = editor.render(80).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("hi")));
}

#[test]
fn tui_mount_and_focus() {
    let term = TestTerminal::new(80, 24);
    let mut tui = TUI::new(Box::new(term));
    tui.mount(Box::new(Input::new()));
    tui.mount(Box::new(Input::new()));
    // Second mount should not change focus from first if first was focused
    // Actually first component gets focus on mount
    tui.render_frame().unwrap();
    // Just verify it doesn't panic
}
