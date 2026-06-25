use photon_ui::{
    Component,
    components::ImageWidget,
    image::ImageProtocol,
    layout::Rect,
};

#[test]
fn image_widget_default_placeholder() {
    let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None);
    let rendered = widget.render(80).unwrap();
    assert!(rendered.lines[0].starts_with("[image]"));
}

#[test]
fn image_widget_custom_placeholder() {
    let widget = ImageWidget::new(vec![], "image/png", Some("Loading...".into())).with_size(10, 1);
    let rendered = widget.render(80).unwrap();
    assert_eq!(rendered.lines, vec!["Loading..."]);
}

#[test]
fn image_widget_emits_kitty_command_by_default() {
    let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_size(10, 5);
    let rendered = widget.render(80).unwrap();
    assert_eq!(rendered.images.len(), 1);
    let data = &rendered.images[0].data;
    assert!(data.contains("_G"));
    assert!(data.contains("a=p"));
    assert!(data.contains("c=10"));
    assert!(data.contains("r=5"));
}

#[test]
fn image_widget_emits_iterm2_command_when_configured() {
    let widget =
        ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_protocol(ImageProtocol::Iterm2);
    let rendered = widget.render(80).unwrap();
    assert_eq!(rendered.images.len(), 1);
    let data = &rendered.images[0].data;
    assert!(data.contains("1337"));
    assert!(data.contains("File=inline=1"));
}

#[test]
fn image_widget_skips_command_for_empty_data() {
    let widget = ImageWidget::new(vec![], "image/png", None).with_size(10, 5);
    let rendered = widget.render(80).unwrap();
    assert!(rendered.images.is_empty());
}

#[test]
fn image_widget_reserves_configured_lines() {
    let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_size(12, 7);
    let rendered = widget.render(80).unwrap();
    assert_eq!(rendered.lines.len(), 7);
    assert_eq!(photon_ui::utils::visible_width(&rendered.lines[1]), 12);
}

#[test]
fn image_widget_render_rect_clips_height() {
    let widget = ImageWidget::new(vec![0x89, 0x50], "image/png", None).with_size(8, 8);
    let rendered = widget.render_rect(Rect::new(0, 0, 80, 3)).unwrap();
    assert_eq!(rendered.lines.len(), 3);
}
