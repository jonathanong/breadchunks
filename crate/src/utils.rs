use super::tokens::default_length_counter;
use super::types::Chunk;

/// Compute character length for a chunk. Includes the breadcrumb in the count
/// because embeddings will see "breadcrumb\n\ntext" as the full input.
///
/// Counts breadcrumb and text independently then adds 1 for the `\n\n`
/// separator (which collapses to a single space under whitespace-normalization).
/// Zero allocations.
pub fn set_length(chunk: &mut Chunk) {
    let b = default_length_counter(&chunk.breadcrumb);
    let t = default_length_counter(&chunk.text);
    chunk.length = if b == 0 { t } else { b + 1 + t };
}

/// Replace `\u{E000}CODE_BLOCK_N\u{E000}` placeholders back with the original code content.
/// Single-pass O(N) where N is the length of `text`.
pub fn restore_code_placeholders(text: &str, blocks: &[String]) -> String {
    const SENTINEL: char = '\u{E000}';
    if blocks.is_empty() || !text.contains(SENTINEL) {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut remaining = text;
    while let Some(start) = remaining.find(SENTINEL) {
        out.push_str(&remaining[..start]);
        remaining = &remaining[start + SENTINEL.len_utf8()..];
        if let Some(end) = remaining.find(SENTINEL) {
            let tag = &remaining[..end];
            remaining = &remaining[end + SENTINEL.len_utf8()..];
            if let Some(idx_str) = tag.strip_prefix("CODE_BLOCK_") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(block) = blocks.get(idx) {
                        out.push_str(block);
                        continue;
                    }
                }
            }
            // Not a valid placeholder — emit the delimiters and tag verbatim.
            out.push(SENTINEL);
            out.push_str(tag);
            out.push(SENTINEL);
        } else {
            // Lone sentinel with no closing pair — emit verbatim.
            out.push(SENTINEL);
        }
    }
    out.push_str(remaining);
    out
}

/// Check if `parent`'s header path is a prefix of `child`'s header path.
///
/// Both are 6-element vecs (h1–h6). Walks left-to-right:
///   - If both have a value, they must match.
///   - If parent has a value but child doesn't → false.
///   - If parent is None, we've reached the end of the parent's path → true.
pub fn header_is_superset_of(parent: &[Option<String>], child: &[Option<String>]) -> bool {
    if parent.len() != child.len() {
        return false;
    }

    for i in 0..parent.len() {
        match (&parent[i], &child[i]) {
            (Some(p), Some(c)) => {
                if p != c {
                    return false;
                }
            }
            (Some(_), None) => return false,
            (None, _) => return true,
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{header_is_superset_of, restore_code_placeholders, set_length};
    use crate::types::Chunk;
    fn s(v: &str) -> Option<String> {
        Some(v.to_string())
    }
    fn chunk(breadcrumb: &str, text: &str) -> Chunk {
        Chunk {
            level: 1,
            header: None,
            headers: vec![None; 6],
            breadcrumb: breadcrumb.to_string(),
            text: text.to_string(),
            length: 0,
        }
    }
    #[test]
    fn set_length_empty() {
        let mut c = chunk("", "hello world");
        set_length(&mut c);
        assert_eq!(c.length, 11);
    }
    #[test]
    fn set_length_nonempty() {
        let mut c = chunk("H1", "content");
        set_length(&mut c);
        assert_eq!(c.length, 10); // "H1 content" after whitespace normalize
    }
    #[test]
    fn restore_zero() {
        assert_eq!(restore_code_placeholders("no code", &[]), "no code");
    }
    #[test]
    fn restore_one() {
        let placeholder = "\u{E000}CODE_BLOCK_0\u{E000}";
        let r = restore_code_placeholders(&format!("A {placeholder} B"), &["X".to_string()]);
        assert_eq!(r, "A X B");
    }
    #[test]
    fn restore_many() {
        let p0 = "\u{E000}CODE_BLOCK_0\u{E000}";
        let p1 = "\u{E000}CODE_BLOCK_1\u{E000}";
        let r =
            restore_code_placeholders(&format!("{p0} {p1}"), &["A".to_string(), "B".to_string()]);
        assert_eq!(r, "A B");
    }
    #[test]
    fn restore_invalid_placeholder_passes_through() {
        // \u{E000}UNKNOWN\u{E000} is not a valid CODE_BLOCK_N — emitted verbatim
        let r = restore_code_placeholders("\u{E000}UNKNOWN\u{E000}", &["X".to_string()]);
        assert_eq!(r, "\u{E000}UNKNOWN\u{E000}");
    }
    #[test]
    fn restore_lone_sentinel_passes_through() {
        // A lone \u{E000} with no closing pair is emitted verbatim
        let r = restore_code_placeholders("before\u{E000}after", &["X".to_string()]);
        assert_eq!(r, "before\u{E000}after");
    }
    #[test]
    fn super_len_mismatch() {
        assert!(!header_is_superset_of(&[s("a")], &[s("a"), s("b")]));
    }
    #[test]
    fn super_unequal() {
        let p = vec![s("a"), s("x"), None, None, None, None];
        let c = vec![s("a"), s("y"), None, None, None, None];
        assert!(!header_is_superset_of(&p, &c));
    }
    #[test]
    fn super_parent_deeper() {
        let p = vec![s("a"), s("b"), None, None, None, None];
        let c = vec![s("a"), None, None, None, None, None];
        assert!(!header_is_superset_of(&p, &c));
    }
    #[test]
    fn super_full_match() {
        let full: Vec<Option<String>> = (1..=6).map(|i| s(&i.to_string())).collect();
        assert!(header_is_superset_of(&full, &full));
    }
}
