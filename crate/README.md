# breadchunks

Heading-aware, token-budgeted semantic chunker for Markdown.

See the [repository README](../README.md) for full documentation.

## Quick start

```toml
[dependencies]
breadchunks = "0.1"
```

```rust
use breadchunks::{chunk, ChunkOptions};

let chunks = chunk("# H1\n\nHello.\n\n## H2\n\nWorld.", None);
```

## License

MIT
