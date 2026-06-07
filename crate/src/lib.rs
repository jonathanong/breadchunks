#![deny(clippy::all)]
//! Heading-aware, token-budgeted semantic chunker for Markdown.
//!
//! Splits a Markdown document by heading hierarchy and merges/splits chunks
//! to stay within a character budget. Designed for RAG pipelines and embedding
//! workflows where section context matters.
//!
//! ## Supported Markdown
//!
//! - **Headers:** ATX only (`# H1` through `###### H6`). Setext headers
//!   (`=====`/`-----` underlines) and headings with 7+ `#` are not recognized.
//! - **Code blocks:** backtick-fenced (` ``` `) and inline (`` ` `` … `` ` ``).
//!   Tilde fences (`~~~`) and 4-space-indented code are **not** protected —
//!   `#` lines inside them are parsed as headers. Switch to backtick fences
//!   to avoid mis-splits.
//!
//! ## Algorithm
//!
//! Three-phase pipeline:
//! - **Phase 1**: Split at header boundaries, one chunk per paragraph.
//! - **Phase 2**: Merge adjacent chunks with identical breadcrumbs that are
//!   below `min_length` (linear pass).
//! - **Phase 3**: Absorb child sections into parent headers (bottom-up, h6→h1).
//!
//! ## Example
//!
//! ```
//! use breadchunks::{chunk, ChunkOptions};
//!
//! let markdown = "# Introduction\n\nHello world.\n\n## Details\n\nMore info.";
//! let options = ChunkOptions {
//!     min_length: Some(400),
//!     max_length: Some(2000),
//!     ..Default::default()
//! };
//! let chunks = chunk(markdown, Some(options));
//! assert!(!chunks.is_empty());
//! ```

mod merge;
mod split;
mod tokens;
mod types;
mod utils;

pub use tokens::default_length_counter;
pub use types::{Chunk, ChunkOptions};

/// Chunk markdown text into semantically meaningful pieces.
///
/// Runs up to three phases depending on `options.phase` (default: 3).
pub fn chunk(text: &str, options: Option<ChunkOptions>) -> Vec<Chunk> {
    let opts = options.unwrap_or_default();

    let min_length = opts.min_length.unwrap_or(512) as usize;
    let max_length = opts.max_length.unwrap_or(3072) as usize;
    let phase = opts.phase.unwrap_or(3);
    let title = opts.title.as_deref();

    let mut chunks = split::split_by_headers(text, title);

    if phase >= 2 {
        chunks = merge::merge_phase2(chunks, min_length, max_length);
    }

    if phase >= 3 {
        chunks = merge::merge_phase3(chunks, min_length, max_length);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_default_options() {
        let text = "# Title\n\nParagraph 1.\n\n## Subtitle\n\nParagraph 2.";
        let chunks = chunk(text, None);
        // Uses default options (phase = 3, min_length = 512, max_length = 3072)
        // Everything should be merged into a single chunk because total length < 512
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].breadcrumb.as_str(), "Title");
    }

    #[test]
    fn test_chunk_phase_1() {
        let text = "# Title\n\nParagraph 1.\n\n## Subtitle\n\nParagraph 2.";
        let options = ChunkOptions {
            phase: Some(1),
            ..Default::default()
        };
        let chunks = chunk(text, Some(options));
        // Phase 1 only: should not merge, resulting in 2 chunks
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].breadcrumb.as_str(), "Title");
        assert_eq!(chunks[1].breadcrumb.as_str(), "Title > Subtitle");
    }

    #[test]
    fn test_chunk_custom_lengths() {
        let text = "# Title\n\nShort paragraph 1.\n\nShort paragraph 2.";
        let options = ChunkOptions {
            min_length: Some(10),
            max_length: Some(20),
            ..Default::default()
        };
        let chunks = chunk(text, Some(options));
        // Since length limit is low, they might not be merged if they exceed max_length
        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_chunk_with_title() {
        let text = "Paragraph without header.";
        let options = ChunkOptions {
            title: Some("My Custom Title".to_string()),
            ..Default::default()
        };
        let chunks = chunk(text, Some(options));
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].breadcrumb.as_str(), "My Custom Title");
        assert_eq!(chunks[0].header.as_deref(), Some("My Custom Title"));
    }

    #[test]
    fn test_chunk_empty_text() {
        let chunks = chunk("", None);
        assert_eq!(chunks.len(), 0);
    }
}
