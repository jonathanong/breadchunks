# breadchunks

[![CI](https://github.com/jonathanong/breadchunks/actions/workflows/ci.yml/badge.svg)](https://github.com/jonathanong/breadchunks/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/breadchunks)](https://crates.io/crates/breadchunks)
[![codecov](https://codecov.io/gh/jonathanong/breadchunks/branch/main/graph/badge.svg)](https://codecov.io/gh/jonathanong/breadchunks)

Heading-aware, token-budgeted semantic chunker for Markdown — for RAG and embedding pipelines.

## Quick start

```toml
[dependencies]
breadchunks = "0.1"
```

```rust
use breadchunks::{chunk, ChunkOptions};

let markdown = "# Introduction\n\nHello world.\n\n## Details\n\nMore info.";
let chunks = chunk(markdown, Some(ChunkOptions {
    min_length: Some(400),
    max_length: Some(2000),
    ..Default::default()
}));

for c in &chunks {
    println!("[{}] {}", c.breadcrumb, &c.text[..c.text.len().min(80)]);
}
```

## How it works

Three-phase pipeline:

1. **Phase 1 — Split**: Split at header boundaries. Each section becomes a chunk tagged with its full heading breadcrumb (`H1 > H2 > H3`). Code blocks are protected — `# comment` inside fenced code is never treated as a heading.
2. **Phase 2 — Merge same-breadcrumb**: Merge adjacent chunks that share a breadcrumb and are below `min_length`.
3. **Phase 3 — Parent absorption** (bottom-up, h6→h1): Absorb small child sections into their parent when the combined size stays under `max_length`.

## API

### `chunk(text, options) -> Vec<Chunk>`

| Option | Type | Default | Description |
|---|---|---|---|
| `min_length` | `Option<u32>` | `512` | Target minimum chunk size (chars) |
| `max_length` | `Option<u32>` | `3072` | Hard maximum chunk size (chars) |
| `phase` | `Option<u32>` | `3` | Stop after this phase (1, 2, or 3) |
| `title` | `Option<String>` | `None` | Document title prepended to every breadcrumb |

### `Chunk`

| Field | Type | Description |
|---|---|---|
| `level` | `u32` | Heading depth (0 = preface, 1–6 = h1–h6) |
| `header` | `Option<String>` | Text of the nearest heading |
| `headers` | `Vec<Option<String>>` | Full 6-slot heading stack (h1–h6) |
| `breadcrumb` | `String` | Human-readable path: `"H1 > H2 > H3"` |
| `text` | `String` | Chunk body (without the heading line) |
| `length` | `u32` | Character count of `breadcrumb + "\n\n" + text` |

### `default_length_counter(text) -> u32`

Collapses whitespace runs to a single space, trims, returns Unicode character count. Use it for consistent length measurements:

```rust
use breadchunks::default_length_counter;
assert_eq!(default_length_counter("hello  world"), 11);
```

## License

MIT
