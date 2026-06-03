use photon_ui::{
    Component,
    components::Markdown,
};

#[test]
fn markdown_renders_heading() {
    let md = Markdown::new("# Hello\n\nWorld");
    let r = md.render(80).unwrap();
    // Heading should be bold + underline, no # prefix
    let heading = r.lines.iter().find(|l| l.contains("Hello")).unwrap();
    assert!(
        !heading.contains("# Hello"),
        "heading should not have # prefix: {}",
        heading
    );
    assert!(
        heading.contains("\x1b[1m"),
        "heading should be bold: {}",
        heading
    );
    assert!(
        heading.contains("\x1b[4m"),
        "heading should be underlined: {}",
        heading
    );
    assert!(r.lines.iter().any(|l| l.contains("World")));
}

#[test]
fn markdown_renders_code() {
    let md = Markdown::new("Some `code` here");
    let r = md.render(80).unwrap();
    // Default inline code: cyan foreground, no backticks
    assert!(r.lines[0].contains("\x1b[36m"));
    assert!(r.lines[0].contains("code"));
    assert!(!r.lines[0].contains("`code`"));
}

#[test]
fn markdown_renders_list() {
    let md = Markdown::new("- Item 1\n- Item 2");
    let r = md.render(80).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("Item 1")));
    assert!(r.lines.iter().any(|l| l.contains("Item 2")));
}

#[test]
fn markdown_long_line_wraps() {
    let md = Markdown::new(
        "This is a very long paragraph that should definitely wrap when rendered with a narrow width constraint.",
    );
    let r = md.render(20).unwrap();
    assert!(r.lines.len() > 1);
}

#[test]
fn markdown_renders_block_html() {
    let md = Markdown::new("<div>Hello world</div>\n\nmore");
    let r = md.render(80).unwrap();
    assert!(!r.lines.is_empty());
}
