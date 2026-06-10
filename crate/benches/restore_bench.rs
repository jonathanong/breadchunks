use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

// We need to access utils, but it's private.
// We can test `chunk` function with a lot of text without code blocks, which implicitly tests it.
pub fn bench_restore(c: &mut Criterion) {
    let markdown = "This is a simple paragraph without any code blocks. ".repeat(100);
    c.bench_function("restore_no_code", |b| {
        b.iter(|| {
            let options = breadchunks::ChunkOptions {
                phase: Some(1),
                ..Default::default()
            };
            breadchunks::chunk(black_box(&markdown), black_box(Some(options)))
        })
    });
}

criterion_group!(benches, bench_restore);
criterion_main!(benches);
