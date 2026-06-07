use std::fmt::Write as _;

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
    if chunks.is_empty() {
        return chunks;
    }

    let mut result = chunks;

    for level in (1..=6).rev() {
        let mut merged = Vec::with_capacity(result.len());
        let mut iter = std::mem::take(&mut result).into_iter().peekable();

        while let Some(mut current) = iter.next() {
            if current.level == level && current.length < max_length {
                while let Some(child) = iter.next_if(|next| {
                    let is_child = next.level > level
                        && header_is_superset_of(&current.headers, &next.headers);

                    is_child && should_merge(current.length, next.length, min_length, max_length)
                }) {
                    let header_prefix = &HASHES[..child.level.min(6) as usize];
                    let child_header = child.header.as_deref().unwrap_or_default();
                    current.text.reserve(
                        2 + header_prefix.len() + 1 + child_header.len() + 2 + child.text.len(),
                    );
                    write!(
                        &mut current.text,
                        "\n\n{} {}\n\n{}",
                        header_prefix, child_header, child.text
                    )
                    .expect("BUG: failed to write merged child section");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn chunk(breadcrumb: &str, text: &str) -> Chunk {
        let mut c = Chunk {
            level: 1,
            header: None,
            headers: Arc::new(vec![None; 6]),
            breadcrumb: Arc::new(breadcrumb.to_string()),
            text: text.to_string(),
            length: 0,
        };
        set_length(&mut c);
        c
    }

    fn create_test_chunk(
        level: u32,
        header: Option<&str>,
        headers: Vec<Option<&str>>,
        text: &str,
        length: usize,
    ) -> Chunk {
        let mut h_vec = vec![None; 6];
        for (i, h) in headers.into_iter().enumerate() {
            if i < 6 {
                h_vec[i] = h.map(|s| s.to_string());
            }
        }
        Chunk {
            level,
            header: header.map(|s| s.to_string()),
            headers: Arc::new(h_vec),
            breadcrumb: Arc::new("".to_string()),
            text: text.to_string(),
            length,
        }
    }

    #[test]
    fn test_merge_phase2_empty() {
        let result = merge_phase2(vec![], 100, 1000);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_phase2_no_merge_different_breadcrumb() {
        let c1 = chunk("A", "text 1");
        let c2 = chunk("B", "text 2");
        let result = merge_phase2(vec![c1, c2], 100, 1000);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_phase2_merge_same_breadcrumb() {
        let c1 = chunk("A", "text 1"); // small
        let c2 = chunk("A", "text 2"); // small
        let result = merge_phase2(vec![c1, c2], 100, 1000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "text 1\n\ntext 2");
        assert_eq!(result[0].breadcrumb.as_str(), "A");
    }

    #[test]
    fn test_merge_phase2_no_merge_both_large() {
        // Both >= 10, min_length is 10
        let c1 = chunk("A", "0123456789");
        let c2 = chunk("A", "0123456789");
        let result = merge_phase2(vec![c1.clone(), c2.clone()], 10, 1000);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_phase2_no_merge_too_big() {
        let c1 = chunk("A", "01234");
        let c2 = chunk("A", "01234");
        // Combine length > max_length (e.g. 5)
        let result = merge_phase2(vec![c1.clone(), c2.clone()], 10, 5);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_phase2_multiple_merge() {
        let c1 = chunk("A", "1");
        let c2 = chunk("A", "2");
        let c3 = chunk("A", "3");
        let c4 = chunk("B", "4"); // different breadcrumb
        let result = merge_phase2(vec![c1, c2, c3, c4], 100, 1000);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "1\n\n2\n\n3");
        assert_eq!(result[1].text, "4");
    }

    #[test]
    fn test_merge_phase3_empty() {
        let result = merge_phase3(vec![], 100, 1000);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_phase3_absorbs_child() {
        let parent = create_test_chunk(
            1,
            Some("Parent"),
            vec![Some("Parent")],
            "Parent content",
            20,
        );
        let child = create_test_chunk(
            2,
            Some("Child"),
            vec![Some("Parent"), Some("Child")],
            "Child content",
            10,
        );
        let chunks = vec![parent, child];
        let merged = merge_phase3(chunks, 100, 100);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].level, 1);
        assert!(merged[0].text.contains("Parent content"));
        assert!(merged[0].text.contains("## Child"));
        assert!(merged[0].text.contains("Child content"));
    }

    #[test]
    fn test_merge_phase3_hierarchy_constraint() {
        let parent = create_test_chunk(
            1,
            Some("Parent"),
            vec![Some("Parent")],
            "Parent content",
            20,
        );
        // Not a child according to header_is_superset_of
        let non_child = create_test_chunk(
            2,
            Some("Non-Child"),
            vec![Some("Other"), Some("Non-Child")],
            "Non-child content",
            10,
        );
        let chunks = vec![parent, non_child];
        let merged = merge_phase3(chunks, 0, 100);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_phase3_multiple_children() {
        let parent = create_test_chunk(
            1,
            Some("Parent"),
            vec![Some("Parent")],
            "Parent content",
            20,
        );
        let child1 = create_test_chunk(
            2,
            Some("Child1"),
            vec![Some("Parent"), Some("Child1")],
            "Child1 content",
            10,
        );
        let child2 = create_test_chunk(
            2,
            Some("Child2"),
            vec![Some("Parent"), Some("Child2")],
            "Child2 content",
            10,
        );

        let chunks = vec![parent, child1, child2];
        let merged = merge_phase3(chunks, 100, 100);

        assert_eq!(merged.len(), 1);
        assert!(merged[0].text.contains("Child1"));
        assert!(merged[0].text.contains("Child2"));
    }

    #[test]
    fn test_merge_phase3_max_length_respected() {
        let parent = create_test_chunk(
            1,
            Some("Parent"),
            vec![Some("Parent")],
            "Parent content",
            80,
        );
        let child = create_test_chunk(
            2,
            Some("Child"),
            vec![Some("Parent"), Some("Child")],
            "Child content",
            30,
        );
        // 80 + 30 = 110 > 100 max_length
        let chunks = vec![parent, child];
        let merged = merge_phase3(chunks, 0, 100);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_phase3_min_length_respected() {
        let parent = create_test_chunk(
            1,
            Some("Parent"),
            vec![Some("Parent")],
            "Parent content",
            60,
        );
        let child = create_test_chunk(
            2,
            Some("Child"),
            vec![Some("Parent"), Some("Child")],
            "Child content",
            60,
        );
        // Both are >= 50, so should_merge returns false
        let chunks = vec![parent, child];
        let merged = merge_phase3(chunks, 50, 1000);

        assert_eq!(merged.len(), 2);
    }
}
