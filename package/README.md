# breadchunks

[![CI](https://github.com/jonathanong/breadchunks/actions/workflows/ci.yml/badge.svg)](https://github.com/jonathanong/breadchunks/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/breadchunks)](https://www.npmjs.com/package/breadchunks)

Heading-aware, token-budgeted semantic chunker for Markdown — for RAG and embedding pipelines.

Native Node.js addon (N-API). Prebuilt binaries for macOS arm64/x64, Linux glibc arm64/x64, and Windows x64. Alpine/musl and Windows arm64 must build from source via `napi build --release`.

## Install

```bash
npm install breadchunks
```

## Usage

```js
import { chunk } from 'breadchunks'

// Chunk a single document
const [chunks] = await chunk([Buffer.from(markdown)], { minLength: 400, maxLength: 2000 })
for (const c of chunks) {
  console.log(`[${c.breadcrumb}]`, c.text.slice(0, 80))
}

// Chunk multiple documents in one call
const results = await chunk([docA, docB, docC])
// results[0] → Chunk[] for docA, results[1] → Chunk[] for docB, …
```

TypeScript types are included (`index.d.ts`).

`chunk` accepts a batch of `Buffer | string` inputs and returns a `Promise<Chunk[][]>` — one `Chunk[]` per input. `Buffer` is preferred; it avoids a round-trip UTF-8 re-encode from the JS heap.

## How it works

Three-phase pipeline:

1. **Phase 1 — Split**: Split at header boundaries. Each section becomes its own chunk tagged with a full heading breadcrumb (`H1 > H2 > H3`). Code blocks are protected — `# comment` inside a fenced block is never treated as a heading.
2. **Phase 2 — Merge same-breadcrumb**: Merge adjacent chunks that share the same breadcrumb and are below `minLength`.
3. **Phase 3 — Parent absorption** (bottom-up, h6→h1): Absorb small child sections into their parent when the combined size stays under `maxLength`.

**Supported Markdown:** ATX headers only (`# H1` through `###### H6`). Setext headers, tilde fences (`~~~`), and 4-space-indented code are not recognized as structural boundaries.

## API

### `chunk(inputs, options?)`

```ts
function chunk(
  inputs: Array<Buffer | string>,
  options?: ChunkOptions,
): Promise<Array<Array<Chunk>>>
```

### `ChunkOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `minLength` | `number?` | `512` | Target minimum chunk size (chars) |
| `maxLength` | `number?` | `3072` | Hard maximum chunk size (chars) |
| `phase` | `number?` | `3` | Stop after this phase (1, 2, or 3) |
| `title` | `string?` | `undefined` | Document title — prepended to every breadcrumb |

### `Chunk`

| Field | Type | Description |
|---|---|---|
| `level` | `number` | Heading depth (0 = preface, 1–6 = h1–h6) |
| `header` | `string?` | Text of the nearest heading |
| `headers` | `(string \| null)[]` | Full 6-slot heading stack (h1–h6) |
| `breadcrumb` | `string` | Human-readable path: `"H1 > H2 > H3"` |
| `text` | `string` | Chunk body (without the heading line or breadcrumb). Prepend `breadcrumb + "\n\n"` for the full string an embedding model sees. |
| `length` | `number` | Character count of `breadcrumb + "\n\n" + text` after whitespace collapse. `text` alone is shorter. |

> **Note on `length`**: it counts `breadcrumb + "\n\n" + text` after collapsing all whitespace runs to a single space — _not_ `text.length`. Callers migrating from a `text.length` baseline will see systematically larger numbers.

## License

MIT
