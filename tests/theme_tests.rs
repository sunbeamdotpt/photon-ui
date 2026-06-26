use std::sync::Arc;

use photon_ui::{
    Component,
    components::ProgressBar,
    theme::{
        Color,
        Palette,
        PaletteHandle,
        Theme,
    },
};

#[derive(Debug)]
struct TestPalette;

impl Palette for TestPalette {
    fn background(&self) -> Color {
        Color::BLACK
    }

    fn surface(&self) -> Color {
        Color::BLACK
    }

    fn field(&self) -> Color {
        Color::BLACK
    }

    fn text(&self) -> Color {
        Color::BLACK
    }

    fn text_muted(&self) -> Color {
        Color::BLACK
    }

    fn text_on_accent(&self) -> Color {
        Color::BLACK
    }

    fn accent(&self) -> Color {
        Color::from_hex("#ff00ff").unwrap()
    }

    fn accent_hover(&self) -> Color {
        Color::BLACK
    }

    fn border(&self) -> Color {
        Color::from_hex("#00ff00").unwrap()
    }

    fn border_muted(&self) -> Color {
        Color::BLACK
    }

    fn focus(&self) -> Color {
        Color::BLACK
    }

    fn success(&self) -> Color {
        Color::BLACK
    }

    fn warning(&self) -> Color {
        Color::BLACK
    }

    fn error(&self) -> Color {
        Color::BLACK
    }

    fn info(&self) -> Color {
        Color::BLACK
    }
}

#[test]
fn component_uses_custom_palette_colors() {
    Theme::set(Theme::Light);
    Theme::set_palette(Arc::new(TestPalette));

    let pb = ProgressBar::new("", 0.5).width(10);
    let rendered = pb.render(80).unwrap();
    let line = &rendered.lines[0];

    // Filled portion should use custom accent (#ff00ff)
    assert!(
        line.contains("\x1b[38;2;255;0;255m"),
        "expected custom accent ANSI code in {}",
        line.replace('\x1b', "\\x1b")
    );

    // Empty portion should use custom border (#00ff00)
    assert!(
        line.contains("\x1b[38;2;0;255;0m"),
        "expected custom border ANSI code in {}",
        line.replace('\x1b', "\\x1b")
    );

    Theme::clear_palette();
}

#[test]
fn with_clears_custom_palette_for_component() {
    Theme::set(Theme::Light);
    Theme::set_palette(Arc::new(TestPalette));

    Theme::with(Theme::Dark, || {
        let pb = ProgressBar::new("", 0.5).width(10);
        let rendered = pb.render(80).unwrap();
        let line = &rendered.lines[0];

        // Should use Dark theme colors, not custom palette
        assert!(
            !line.contains("\x1b[38;2;255;0;255m"),
            "did not expect custom accent inside Theme::with"
        );
    });

    Theme::clear_palette();
}

#[test]
fn palette_handle_can_be_cloned_and_shared() {
    Theme::set(Theme::Light);
    let handle: PaletteHandle = Arc::new(TestPalette);
    Theme::set_palette(Arc::clone(&handle));

    assert!(Theme::has_palette());
    assert_eq!(Theme::palette().accent().to_hex(), "#ff00ff");

    Theme::clear_palette();
}
