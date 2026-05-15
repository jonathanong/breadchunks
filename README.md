# breadchunks

[![CI](https://github.com/jonathanong/breadchunks/actions/workflows/ci.yml/badge.svg)](https://github.com/jonathanong/breadchunks/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/breadchunks)](https://crates.io/crates/breadchunks)
[![npm](https://img.shields.io/npm/v/breadchunks)](https://www.npmjs.com/package/breadchunks)
[![codecov](https://codecov.io/gh/jonathanong/breadchunks/branch/main/graph/badge.svg)](https://codecov.io/gh/jonathanong/breadchunks)

Heading-aware, token-budgeted semantic chunker for Markdown.

Given a Markdown document, `breadchunks` splits it by heading hierarchy and merges/splits chunks to stay within a character budget. Designed for RAG pipelines and embedding workflows where section context matters.

## How it works

Three-phase pipeline:

1. **Phase 1 — Split**: Split at header boundaries. Every paragraph becomes its own chunk, tagged with its full heading breadcrumb (`H1 > H2 > H3`). Code blocks are protected — `# comment` inside a fenced block is never treated as a Markdown heading.
2. **Phase 2 — Merge same-breadcrumb**: Merge adjacent chunks that share the same breadcrumb and are below `minLength`.
3. **Phase 3 — Parent absorption** (bottom-up, h6→h1): Absorb small child sections into their parent header when the combined size stays under `maxLength`.

Chunk `length` is a character count after collapsing all whitespace runs to a single space. The same logic applies when computing the length of `breadcrumb + "\n\n" + text` (the full string an embedding model sees).

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
| `headers` | `Vec<Option<String>>` | Full 6-slot heading stack (h1–h6) |
| `breadcrumb` | `String` | Human-readable path: `"H1 > H2 > H3"` |
| `text` | `String` | Chunk body (without the heading line) |
| `length` | `u32` | `default_length_counter(breadcrumb + "\n\n" + text)` |

### `default_length_counter`

Collapses all whitespace runs to a single space, trims, then counts Unicode characters (not bytes). This is what populates `chunk.length`. Export it for consistent counts elsewhere:

```rust
use breadchunks::default_length_counter;
let n = default_length_counter("hello  world"); // 11
```

## Node (N-API)

```bash
npm install breadchunks
```

```js
import { chunk, chunkSync } from 'breadchunks'

// Preferred: async batch with Buffers (runs on libuv threadpool)
const [chunks] = await chunk([Buffer.from(markdown)], { minLength: 400, maxLength: 2000 })
for (const c of chunks) {
  console.log(`[${c.breadcrumb}]`, c.text.slice(0, 80))
}

// Process multiple documents in one call
const results = await chunk([docA, docB, docC])
// results[0] → Chunk[] for docA, results[1] → Chunk[] for docB, …

// Sync escape hatch (blocks the event loop)
const [syncChunks] = chunkSync([markdown])
```

TypeScript types are included (`index.d.ts`). Options and return shape mirror the Rust API.

Both `chunk` and `chunkSync` accept a batch of `Buffer | string` inputs. `Buffer` is preferred — it avoids a round-trip UTF-8 re-encode from the JS string heap. Pass `string` when you already have one.

### Node `ChunkOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `minLength` | `number?` | `512` | Target minimum chunk size (chars) |
| `maxLength` | `number?` | `3072` | Hard maximum chunk size (chars) |
| `phase` | `number?` | `3` | Stop after this phase (1, 2, or 3) |
| `title` | `string?` | `undefined` | Document title — prepended to every breadcrumb |

## Development

```bash
# Rust crate tests + coverage report
cd crate
cargo test
cargo llvm-cov --html   # opens target/llvm-cov/html/index.html

# Node package (requires Rust toolchain + Node 18+)
cd package
npm install
npm run build:debug
npm test
```

## Releasing

Releases are managed via the [Release workflow](.github/workflows/release.yml). Trigger it manually from GitHub Actions with a `patch`, `minor`, or `major` bump. The workflow:

1. Bumps `crate/Cargo.toml` and `package/package.json` versions, commits, and tags.
2. Cross-compiles the native module for all supported platforms.
3. Publishes the Rust crate to [crates.io](https://crates.io/crates/breadchunks) and the Node package to [npm](https://www.npmjs.com/package/breadchunks).

**Required secrets:** `RELEASE_TOKEN` (PAT with push access), `CARGO_REGISTRY_TOKEN`, `NPM_TOKEN`, `CODECOV_TOKEN`.

## License

MIT
