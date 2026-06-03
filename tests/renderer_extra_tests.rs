use photon_ui::{
    renderer::{
        RenderStrategy,
        Rendered,
        Renderer,
    },
    terminal::TestTerminal,
};

#[test]
fn renderer_diff_no_change() {
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
        lines: vec!["hello".into(), "world".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &second).unwrap();
    let after = term.written().len();
    // No diff output should have been written
    assert_eq!(after, before);
}

#[test]
fn renderer_diff_shorter_lines() {
    let mut term = TestTerminal::new(80, 24);
    let mut renderer = Renderer::new();
    let first = Rendered {
        lines: vec!["hello".into(), "world".into(), "extra".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &first).unwrap();

    let second = Rendered {
        lines: vec!["hello".into(), "world".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &second).unwrap();
    let written = term.written().join("");
    // Diff should clear the deleted extra line with \x1b[2K
    assert!(written.contains("\x1b[2K"), "should clear deleted lines");
}

#[test]
fn renderer_diff_from_none() {
    let mut term = TestTerminal::new(80, 24);
    let mut renderer = Renderer::new();
    renderer.set_strategy(RenderStrategy::Diff);
    let rendered = Rendered {
        lines: vec!["test".into()],
        cursor: None,
        images: Vec::new(),
    };
    renderer.render(&mut term, &rendered).unwrap();
    let written = term.written().join("");
    assert!(written.contains("test"));
}

#[test]
fn renderer_previous_accessor() {
    let mut renderer = Renderer::new();
    assert!(renderer.previous().is_none());
}
