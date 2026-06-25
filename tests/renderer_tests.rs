use photon_ui::renderer::{
    ImageCommand,
    Rendered,
};

#[test]
fn rendered_empty() {
    let r = Rendered::empty();
    assert!(r.lines.is_empty());
    assert!(r.cursor.is_none());
    assert!(r.images.is_empty());
}

#[test]
fn rendered_blit_onto_basic() {
    let mut target = Rendered {
        lines: vec!["hello world".into(), "second line".into()],
        cursor: None,
        images: Vec::new(),
    };
    let source = Rendered {
        lines: vec!["XY".into()],
        cursor: Some((0, 1)),
        images: vec![ImageCommand {
            id: 1,
            data: "img".into(),
            row: 0,
            col: 0,
        }],
    };
    source.blit_onto(&mut target, 0, 6);
    assert_eq!(target.lines[0], "hello XYrld");
    assert_eq!(target.cursor, Some((0, 7)));
    assert_eq!(target.images.len(), 1);
}

#[test]
fn rendered_blit_onto_out_of_bounds() {
    let mut target = Rendered {
        lines: vec!["short".into()],
        cursor: None,
        images: Vec::new(),
    };
    let source = Rendered {
        lines: vec!["longer text".into()],
        cursor: Some((0, 0)),
        images: Vec::new(),
    };
    source.blit_onto(&mut target, 0, 0);
    // Line gets replaced since col=0 and source fits within target_line.len()
    // roughly
    assert_eq!(target.lines[0], "longer text");
}

#[test]
fn rendered_blit_onto_past_target_rows() {
    let mut target = Rendered {
        lines: vec!["only".into()],
        cursor: None,
        images: Vec::new(),
    };
    let source = Rendered {
        lines: vec!["x".into()],
        cursor: Some((0, 0)),
        images: Vec::new(),
    };
    source.blit_onto(&mut target, 5, 0);
    // Row 5 is past target lines, should not panic
    assert_eq!(target.lines[0], "only");
}
