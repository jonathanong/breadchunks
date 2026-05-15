# Code Examples

A document heavy on fenced code blocks to verify that `# Not a header` inside code is never treated as a Markdown heading.

## Python Examples

### Hello World

```python
# This comment looks like a heading but is inside a code block
print("Hello, world!")
```

Inline backtick: use `# comment` syntax in Python for single-line comments.

### Data Classes

```python
# dataclass example
from dataclasses import dataclass

@dataclass
class Point:
    # x coordinate
    x: float
    # y coordinate
    y: float
```

## Rust Examples

### Struct Definition

```rust
// # Not a heading inside a code block
struct Config {
    /// # also not a heading
    name: String,
    value: u32,
}
```

### Main Function

```rust
fn main() {
    // # still not a heading
    let cfg = Config {
        name: String::from("default"),
        value: 42,
    };
    println!("{}", cfg.name);
}
```

## Shell Scripts

### Setup Script

```bash
#!/usr/bin/env bash
# # Nested comment — definitely not a heading

set -euo pipefail

# Check dependencies
for cmd in curl git jq; do
    # # another decoy
    command -v "$cmd" >/dev/null || { echo "Missing: $cmd"; exit 1; }
done
```

### Cleanup Script

```bash
#!/usr/bin/env bash
# Remove temporary files
rm -rf /tmp/build-*
# # this comment has two hash marks
echo "Done"
```

## TOML Configuration

```toml
# Top-level comment — not a heading
[package]
# # another fake heading
name = "example"
version = "0.1.0"
```

## Inline Code in Prose

Use `#[derive(Debug)]` to auto-implement `Debug` in Rust. In shell, `#!` is a shebang. Neither `#123` nor `##heading-without-space` should appear as chunks.
