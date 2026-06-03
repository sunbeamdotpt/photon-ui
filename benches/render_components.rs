use criterion::{
    Criterion,
    black_box,
    criterion_group,
    criterion_main,
};
use crossterm::event::KeyCode;
use photon_ui::{
    Component,
    Event,
    Focusable,
    components::{
        Box as BoxComp,
        Editor,
        Input,
        Markdown,
        SelectList,
        SettingsList,
        TruncatedText,
    },
};

fn type_string(editor: &mut Editor, s: &str) {
    for ch in s.chars() {
        if ch == '\n' {
            editor.handle_input(&Event::Key(KeyCode::Enter.into()));
        } else {
            editor.handle_input(&Event::Key(KeyCode::Char(ch).into()));
        }
    }
}

fn bench_render_editor(c: &mut Criterion) {
    let mut editor = Editor::new();
    editor.set_focused(true);
    // Simulate a medium-sized document
    let text = "Line one\nLine two\nLine three\nLine four\nLine five\n".repeat(20);
    type_string(&mut editor, &text);

    c.bench_function("render_editor_100_lines", |b| {
        b.iter(|| editor.render(black_box(80)).unwrap())
    });

    c.bench_function("render_editor_100_lines_40w", |b| {
        b.iter(|| editor.render(black_box(40)).unwrap())
    });
}

fn bench_render_markdown(c: &mut Criterion) {
    let md = Markdown::new("# Heading\n\nThis is a paragraph with **bold** and *italic* text.\n\n- Item one\n- Item two\n- Item three\n\n`code block` and more text.\n\n".repeat(10));

    c.bench_function("render_markdown", |b| {
        b.iter(|| md.render(black_box(80)).unwrap())
    });
}

fn bench_render_input(c: &mut Criterion) {
    let mut input = Input::new();
    input.set_focused(true);
    for _ in 0..50 {
        input.handle_input(&Event::Key(KeyCode::Char('x').into()));
    }

    c.bench_function("render_input_50_chars", |b| {
        b.iter(|| input.render(black_box(40)).unwrap())
    });
}

fn bench_render_settings_list(c: &mut Criterion) {
    let list = SettingsList::new(
        (0..20)
            .map(|i| (format!("Option {}", i), i % 2 == 0))
            .collect(),
    );

    c.bench_function("render_settings_list_20", |b| {
        b.iter(|| list.render(black_box(80)).unwrap())
    });
}

fn bench_render_select_list(c: &mut Criterion) {
    let list = SelectList::new((0..50).map(|i| format!("Item {}", i)).collect(), 10);

    c.bench_function("render_select_list_50", |b| {
        b.iter(|| list.render(black_box(80)).unwrap())
    });
}

fn bench_render_box(c: &mut Criterion) {
    let bx = BoxComp::new(2);

    c.bench_function("render_box", |b| {
        b.iter(|| bx.render(black_box(80)).unwrap())
    });
}

fn bench_render_truncated_text(c: &mut Criterion) {
    let tt = TruncatedText::new("Short text", 2, 1);

    c.bench_function("render_truncated_text", |b| {
        b.iter(|| tt.render(black_box(40)).unwrap())
    });
}

criterion_group!(
    render_benches,
    bench_render_editor,
    bench_render_markdown,
    bench_render_input,
    bench_render_settings_list,
    bench_render_select_list,
    bench_render_box,
    bench_render_truncated_text,
);
criterion_main!(render_benches);
