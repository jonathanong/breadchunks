# breadchunks

Heading-aware, token-budgeted semantic chunker for Markdown.

Given a Markdown document, `breadchunks` splits it by heading hierarchy and merges/splits chunks to stay within a character budget. Designed for RAG pipelines and embedding workflows where section context matters.

## Algorithm

Three-phase pipeline:

1. **Phase 1 — Split**: Split at header boundaries. Every paragraph becomes its own chunk, tagged with its full heading breadcrumb (`H1 > H2 > H3`).
2. **Phase 2 — Merge same-breadcrumb**: Merge adjacent chunks that share the same breadcrumb and are below `minLength`.
3. **Phase 3 — Parent absorption** (bottom-up, h6→h1): Absorb small child sections into their parent header when the combined size stays under `maxLength`.

Code blocks are protected throughout — `# comment` inside a fenced block is never treated as a Markdown heading.

## Rust

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

### `ChunkOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `min_length` | `Option<u32>` | `512` | Target minimum chunk size (chars) |
| `max_length` | `Option<u32>` | `3072` | Hard maximum chunk size (chars) |
| `phase` | `Option<u32>` | `3` | Stop after this phase (1, 2, or 3) |
| `title` | `Option<String>` | `None` | Document title — prepended to every breadcrumb |

### `Chunk`

| Field | Type | Description |
|---|---|---|
| `level` | `u32` | Heading depth (0 = preface, 1–6 = h1–h6) |
| `header` | `Option<String>` | Text of the nearest heading |
| `headers` | `Vec<Option<String>>` | Full 6-slot heading stack |
| `breadcrumb` | `String` | Human-readable path: `"H1 > H2 > H3"` |
| `text` | `String` | Chunk body (without the heading line) |
| `length` | `u32` | `default_length_counter(breadcrumb + "\n\n" + text)` |

## Node (N-API)

```bash
npm install breadchunks
```

```js
import { chunk } from 'breadchunks'

const chunks = chunk(markdown, { minLength: 400, maxLength: 2000 })
```

TypeScript types are included (`index.d.ts`). Options and return shape mirror the Rust API with camelCase names.

## Length counter

`default_length_counter(text)` collapses all whitespace runs to a single space, trims, then counts Unicode characters (not bytes). This is what populates `chunk.length`. Export it if you need consistent counts elsewhere:

```rust
use breadchunks::default_length_counter;
let n = default_length_counter("hello  world"); // 11
```

## Development

```bash
# Rust crate tests + 100% line coverage
cd crate
cargo test
cargo llvm-cov --fail-under-lines 100

# Node package (requires Rust toolchain + napi-rs CLI)
cd package
npm install
npm run build:debug
npm test
```

## License

MIT
