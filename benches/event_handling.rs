use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use photon_ui::{Component, Event, Focusable};
use photon_ui::components::Editor;
use crossterm::event::KeyCode;

fn bench_editor_typing(c: &mut Criterion) {
    c.bench_function("editor_type_100_chars", |b| {
        b.iter_batched(
            || {
                let mut editor = Editor::new();
                editor.set_focused(true);
                editor
            },
            |mut editor| {
                for ch in "abcdefghijklmnopqrstuvwxyz".chars().cycle().take(100) {
                    let _ = editor.handle_input(black_box(&Event::Key(KeyCode::Char(ch).into())));
                }
                editor
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_editor_navigate(c: &mut Criterion) {
    c.bench_function("editor_navigate_100_keys", |b| {
        b.iter_batched(
            || {
                let mut editor = Editor::new();
                editor.set_focused(true);
                for ch in "hello world this is a test".chars() {
                    editor.handle_input(&Event::Key(KeyCode::Char(ch).into()));
                }
                editor
            },
            |mut editor| {
                for _ in 0..50 {
                    let _ = editor.handle_input(black_box(&Event::Key(KeyCode::Left.into())));
                    let _ = editor.handle_input(black_box(&Event::Key(KeyCode::Right.into())));
                }
                editor
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_editor_backspace(c: &mut Criterion) {
    c.bench_function("editor_backspace_100", |b| {
        b.iter_batched(
            || {
                let mut editor = Editor::new();
                editor.set_focused(true);
                for ch in "abcdefghijklmnopqrstuvwxyz".repeat(4).chars() {
                    editor.handle_input(&Event::Key(KeyCode::Char(ch).into()));
                }
                editor
            },
            |mut editor| {
                for _ in 0..100 {
                    let _ = editor.handle_input(black_box(&Event::Key(KeyCode::Backspace.into())));
                }
                editor
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(event_benches, bench_editor_typing, bench_editor_navigate, bench_editor_backspace);
criterion_main!(event_benches);
