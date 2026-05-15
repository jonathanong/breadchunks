use super::types::Chunk;
use super::utils::{header_is_superset_of, set_length};

fn should_merge(a_length: u32, b_length: u32, min_length: u32, max_length: u32) -> bool {
    if a_length >= min_length && b_length >= min_length {
        return false;
    }
    a_length + b_length <= max_length
}

/// Phase 2: Linear pass merging consecutive chunks that share the same breadcrumb.
pub fn merge_phase2(chunks: Vec<Chunk>, min_length: u32, max_length: u32) -> Vec<Chunk> {
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
                    prev.text = format!("{}\n\n{}", prev.text, chunk.text);
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
pub fn merge_phase3(chunks: Vec<Chunk>, min_length: u32, max_length: u32) -> Vec<Chunk> {
    let mut result = chunks;

    for level in (1..=6).rev() {
        let mut merged = Vec::with_capacity(result.len());
        let mut i = 0;

        while i < result.len() {
            if result[i].level == level && result[i].length < max_length {
                let mut current = result[i].clone();
                let parent_headers = current.headers.clone();
                i += 1;

                while i < result.len() {
                    let is_child = result[i].level > level
                        && header_is_superset_of(&parent_headers, &result[i].headers);

                    if !is_child {
                        break;
                    }

                    if should_merge(current.length, result[i].length, min_length, max_length) {
                        let child = &result[i];
                        let header_prefix = "#".repeat(child.level as usize);
                        let child_header = child.header.as_deref().unwrap_or("");

                        current.text = format!(
                            "{}\n\n{} {}\n\n{}",
                            current.text, header_prefix, child_header, child.text
                        );
                        set_length(&mut current);
                        i += 1;
                    } else {
                        break;
                    }
                }

                merged.push(current);
            } else {
                merged.push(result[i].clone());
                i += 1;
            }
        }

        result = merged;
    }

    result
}
