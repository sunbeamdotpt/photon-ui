use criterion::{black_box, criterion_group, criterion_main, Criterion};
use photon_ui::utils::wrap_text_with_ansi;

fn bench_wrap_plain_text(c: &mut Criterion) {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    
    c.bench_function("wrap_plain_text_80", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(text), black_box(80)))
    });
    
    c.bench_function("wrap_plain_text_40", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(text), black_box(40)))
    });
}

fn bench_wrap_ansi_text(c: &mut Criterion) {
    let text = "\x1b[1m\x1b[31mLorem ipsum\x1b[0m dolor \x1b[32msit amet\x1b[0m, consectetur \x1b[1madipiscing\x1b[22m elit. \x1b[33mSed do eiusmod\x1b[0m tempor incididunt ut labore et dolore magna aliqua. \x1b[34mUt enim ad minim\x1b[0m veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.";
    
    c.bench_function("wrap_ansi_text_80", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(text), black_box(80)))
    });
}

fn bench_wrap_hyperlink_text(c: &mut Criterion) {
    let text = "\x1b]8;;https://example.com\x1b\\Click here\x1b]8;;\x1b\\ for more info about \x1b]8;;https://rust-lang.org\x1b\\Rust\x1b]8;;\x1b\\ and \x1b]8;;https://github.com\x1b\\GitHub\x1b]8;;\x1b\\ which are great tools for development.";
    
    c.bench_function("wrap_hyperlink_text_80", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(text), black_box(80)))
    });
}

fn bench_wrap_large_text(c: &mut Criterion) {
    let paragraph = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ";
    let text = paragraph.repeat(100); // ~6.5k chars
    
    c.bench_function("wrap_large_text_80", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(&text), black_box(80)))
    });
    
    c.bench_function("wrap_large_text_40", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(&text), black_box(40)))
    });
}

fn bench_wrap_cjk_text(c: &mut Criterion) {
    let text = "这是一个中文文本示例，用于测试终端文本换行功能。中文字符占用两个显示宽度，因此需要特殊的处理逻辑来确保正确换行。\n这是第二行文本，继续测试换行功能。";
    
    c.bench_function("wrap_cjk_text_40", |b| {
        b.iter(|| wrap_text_with_ansi(black_box(text), black_box(40)))
    });
}

criterion_group!(
    wrap_benches,
    bench_wrap_plain_text,
    bench_wrap_ansi_text,
    bench_wrap_hyperlink_text,
    bench_wrap_large_text,
    bench_wrap_cjk_text
);
criterion_main!(wrap_benches);
