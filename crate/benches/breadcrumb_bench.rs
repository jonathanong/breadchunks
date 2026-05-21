use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn build_breadcrumb_old(headers: &[Option<String>]) -> String {
    headers
        .iter()
        .filter_map(|h| h.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join(" > ")
}

fn build_breadcrumb_new(headers: &[Option<String>]) -> String {
    headers
        .iter()
        .filter_map(|h| h.as_deref())
        .collect::<Vec<_>>()
        .join(" > ")
}

fn criterion_benchmark(c: &mut Criterion) {
    let headers = vec![
        Some("Chapter 1".to_string()),
        None,
        Some("Section 1.1".to_string()),
        Some("Subsection 1.1.1".to_string()),
        None,
        Some("Details".to_string()),
    ];

    c.bench_function("build_breadcrumb_old", |b| {
        b.iter(|| build_breadcrumb_old(black_box(&headers)))
    });
    c.bench_function("build_breadcrumb_new", |b| {
        b.iter(|| build_breadcrumb_new(black_box(&headers)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
