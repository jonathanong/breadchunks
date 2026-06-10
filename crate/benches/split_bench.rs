use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

pub fn bench_split(c: &mut Criterion) {
    let markdown = "
# Intro
Some intro text here.

# H1 Header
Paragraph 1 under H1.

Paragraph 2 under H1.

## H2 Header
Paragraph 1 under H2.

Paragraph 2 under H2.

Paragraph 3 under H2.

### H3 Header
Paragraph 1 under H3.

Paragraph 2 under H3.

Paragraph 3 under H3.
"
    .repeat(100);

    let title = Some("My Benchmark Title");

    c.bench_function("split_by_headers", |b| {
        b.iter(|| {
            let options = breadchunks::ChunkOptions {
                title: title.map(|t| t.to_string()),
                phase: Some(1),
                ..Default::default()
            };
            breadchunks::chunk(black_box(&markdown), black_box(Some(&options)))
        })
    });
}

criterion_group!(benches, bench_split);
criterion_main!(benches);
