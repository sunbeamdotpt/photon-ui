use photon_ui::{
    Component,
    RenderError,
    Rendered,
    components::ImageWidget,
};

#[test]
fn image_widget_default_placeholder() {
    let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None);
    let rendered = widget.render(80).unwrap();
    assert_eq!(rendered.lines, vec!["[image]"]);
}

#[test]
fn image_widget_custom_placeholder() {
    let widget = ImageWidget::new(vec![], "image/png", Some("Loading...".into()));
    let rendered = widget.render(80).unwrap();
    assert_eq!(rendered.lines, vec!["Loading..."]);
}
