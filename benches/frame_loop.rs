use criterion::{black_box, criterion_group, criterion_main, Criterion};
use photon_ui::{Event, Focusable, TUI, TestTerminal};
use photon_ui::components::{Editor, Input, Text};

fn bench_frame_render(c: &mut Criterion) {
    let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
    let mut editor = Editor::new();
    editor.set_focused(true);
    let input = Input::new();
    let text = Text::new("Status bar", 0, 0);
    
    tui.mount(Box::new(editor));
    tui.mount(Box::new(input));
    tui.mount(Box::new(text));
    tui.set_focus(0);
    
    c.bench_function("frame_render_3_components", |b| {
        b.iter(|| tui.render_frame().unwrap())
    });
}

fn bench_frame_resize(c: &mut Criterion) {
    let mut tui = TUI::new(Box::new(TestTerminal::new(80, 24)));
    let editor = Editor::new();
    tui.mount(Box::new(editor));
    tui.set_focus(0);
    
    c.bench_function("frame_resize_event", |b| {
        b.iter(|| {
            tui.handle_input(black_box(&Event::Resize(80, 24)));
        })
    });
}

criterion_group!(frame_benches, bench_frame_render, bench_frame_resize);
criterion_main!(frame_benches);
