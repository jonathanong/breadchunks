## Performance Optimizations

**Learning:** `Regex::captures_iter` has significant overhead compared to `Regex::find_iter` when you only need the matched text and can derive sub-captures via small string slicing.

In `breadchunks`, switching the header scanning loop in `crate/src/split.rs` from `captures_iter` to `find_iter` reduced matching overhead (~3.5x observed in a focused benchmark). The header regex now avoids capture groups and instead strips a leading newline and optional `\r` suffix when normalizing the header text.

**Action:** When a capture group only removes a predictable prefix/suffix (like an optional leading newline), prefer `find_iter` and explicit slicing over full capture extraction.
## Performance Optimizations

### Memory Footprint and Re-allocations during Loop Splitting
In `breadchunks`, creating chunk elements for parsing markdown structures relies on duplicating internal state over recursive splits. `split_by_headers` creates multiple paragraph splits mapping against the matching parent headers.

Initially, elements were cloned iteratively over a `.filter()` directly:
```rust
for paragraph in PARAGRAPH_SPLIT_REGEX.split(...) {
    let mut chunk = Chunk {
        // ...
        headers: headers.clone(),
        breadcrumb: build_breadcrumb(&headers),
    };
}
```
This is inherently anti-performant as `headers` and `breadcrumb` are identical over these paragraphs. To resolve the footprint:
1. `types.rs` Chunk definition migrated to utilize `Arc<Vec<Option<String>>>` and `Arc<String>`.
2. Paragraphs iteration utilizes a `prototype` instantiation of the wrapper struct that passes ownership explicitly through `prototype.clone()` where underlying allocations resolve immediately via lightweight `Arc` atomic reference counts.

### Node API Support against Arc wrappers
Since the `breadchunks-node` wrapper relies on exporting the structure against raw Javascript bindings, ensuring the conversion extracts the raw string or copies elements securely avoids boundary bugs during the promise resolutions.
```rust
// package/src/lib.rs
headers: c.headers.as_ref().clone(),
breadcrumb: c.breadcrumb.to_string(),
```

### Avoiding String Allocations for Repetitive Paragraph Headers
When chunking markdown, each paragraph under a header shares the exact same header string. In `breadchunks`, allocating a new `String` for the `header` field on every paragraph chunk was a performance bottleneck. Changing the `Chunk` struct definition from `pub header: Option<String>` to `pub header: Option<Arc<String>>` and cloning an `Arc` via a prototype chunk (rather than allocating a new string) resulted in a ~3% measurable performance improvement and significant memory reduction.

**Action:** When creating many identical objects that share string data (like text chunks under the same header), use `Arc<String>` instead of `String` to prevent redundant heap allocations.
