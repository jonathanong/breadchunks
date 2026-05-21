use super::types::Chunk;
use super::utils::{header_is_superset_of, set_length};

const HASHES: &str = "######";

fn should_merge(a_length: usize, b_length: usize, min_length: usize, max_length: usize) -> bool {
    if a_length >= min_length && b_length >= min_length {
        return false;
    }
    a_length + b_length <= max_length
}

/// Phase 2: Linear pass merging consecutive chunks that share the same breadcrumb.
pub fn merge_phase2(chunks: Vec<Chunk>, min_length: usize, max_length: usize) -> Vec<Chunk> {
    if chunks.is_empty() {
        return chunks;
    }

    let mut result = Vec::new();
    let mut current: Option<Chunk> = None;

    for chunk in chunks {
        match current.take() {
            None => {
                current = Some(chunk);
            }
            Some(mut prev) => {
                if prev.breadcrumb == chunk.breadcrumb
                    && should_merge(prev.length, chunk.length, min_length, max_length)
                {
                    prev.text.reserve(chunk.text.len() + 2);
                    prev.text.push_str("\n\n");
                    prev.text.push_str(&chunk.text);
                    set_length(&mut prev);
                    current = Some(prev);
                    continue;
                }

                result.push(prev);
                current = Some(chunk);
            }
        }
    }

    result.extend(current);
    result
}

/// Phase 3: Hierarchical merge — absorb child sections into parent headers (bottom-up).
///
/// For each heading level from h6 down to h1, any chunk whose length is below
/// `max_length` will absorb immediately-following child chunks (those at a
/// deeper level whose heading path is a suffix of the parent's) as long as the
/// combined length stays within `max_length`.
///
/// **Note:** absorbed children's heading lines are rendered into `chunk.text`
/// (e.g. `## Child Section\n\n...`) but `chunk.headers` continues to reflect
/// only the parent's heading path. To enumerate all headings inside a merged
/// chunk, scan `chunk.text` for ATX headers.
pub fn merge_phase3(chunks: Vec<Chunk>, min_length: usize, max_length: usize) -> Vec<Chunk> {
    let mut result = chunks;

    for level in (1..=6).rev() {
        let mut merged = Vec::with_capacity(result.len());
        let mut iter = std::mem::take(&mut result).into_iter().peekable();

        while let Some(mut current) = iter.next() {
            if current.level == level && current.length < max_length {
                while let Some(next) = iter.peek() {
                    let is_child = next.level > level
                        && header_is_superset_of(&current.headers, &next.headers);

                    if !is_child {
                        break;
                    }

                    if !should_merge(current.length, next.length, min_length, max_length) {
                        break;
                    }

                    let Some(child) = iter.next() else {
                        break;
                    };
                    let header_prefix = &HASHES[..child.level.min(6) as usize];
                    let child_header = child.header.as_deref().unwrap_or("");
                    current.text.reserve(
                        2 + header_prefix.len() + 1 + child_header.len() + 2 + child.text.len(),
                    );
                    current.text.push_str("\n\n");
                    current.text.push_str(header_prefix);
                    current.text.push(' ');
                    current.text.push_str(child_header);
                    current.text.push_str("\n\n");
                    current.text.push_str(&child.text);
                    set_length(&mut current);
                }

                merged.push(current);
            } else {
                merged.push(current);
            }
        }

        result = merged;
    }

    result
}
