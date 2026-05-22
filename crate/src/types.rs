use std::sync::Arc;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Chunk {
    /// Heading depth: 0 = preface (before first header), 1–6 = H1–H6.
    pub level: u32,
    /// Text of the nearest enclosing heading (not including the `#` markers).
    pub header: Option<String>,
    /// Six-slot heading stack (indices 0–5 = H1–H6). Each slot is `Some` if
    /// that heading level is currently active, `None` otherwise.
    pub headers: Arc<Vec<Option<String>>>,
    /// Human-readable breadcrumb path, e.g. `"Introduction > Background"`.
    pub breadcrumb: Arc<String>,
    /// Paragraph body of the chunk. Does **not** include the heading line or
    /// the breadcrumb — those are in `header`/`headers`/`breadcrumb`.
    /// To produce the string an embedding model sees, prepend
    /// `breadcrumb + "\n\n"` when `breadcrumb` is non-empty.
    pub text: String,
    /// Character count of `breadcrumb + "\n\n" + text` after whitespace
    /// collapse — the full string an embedding model sees.
    /// `text` alone is shorter; callers must prepend `breadcrumb` to
    /// reproduce this measurement.
    pub length: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ChunkOptions {
    /// Target minimum chunk size in characters (default: 512).
    pub min_length: Option<u32>,
    /// Hard maximum chunk size in characters (default: 3072).
    pub max_length: Option<u32>,
    /// Stop after this phase: 1 = split only, 2 = +merge, 3 = +absorb
    /// (default: 3).
    pub phase: Option<u32>,
    /// Fallback document title used only when the document has no ATX `#`
    /// header. If the document already starts with an H1, that H1 wins and
    /// `title` is ignored. Useful for documents whose title lives outside the
    /// body (filename, metadata, etc.).
    pub title: Option<String>,
}
