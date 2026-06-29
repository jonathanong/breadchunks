use super::tokens::default_length_counter;
use super::types::Chunk;

/// Compute character length for a chunk. Includes the breadcrumb in the count
/// because embeddings will see "breadcrumb\n\ntext" as the full input.
///
/// Counts breadcrumb and text independently. The `\n\n` separator (which
/// collapses to a single space under whitespace-normalization, contributing 1
/// to the count) is included only when both sides normalize to a non-zero
/// length. Zero allocations.
pub fn set_length(chunk: &mut Chunk) {
    let b = default_length_counter(chunk.breadcrumb.as_str());
    let t = default_length_counter(&chunk.text);
    chunk.length = if b == 0 || t == 0 { b + t } else { b + 1 + t };
}

pub fn derive_t(len: usize, b: usize) -> usize {
    if len <= b {
        0
    } else if b == 0 {
        len
    } else {
        len - b - 1
    }
}

pub fn merge_text_len(t_a: usize, t_b: usize) -> usize {
    if t_a == 0 {
        t_b
    } else if t_b == 0 {
        t_a
    } else {
        t_a + 1 + t_b
    }
}

pub fn update_length_after_merge(
    current_len: usize,
    current_breadcrumb_len: usize,
    chunk_len: usize,
    chunk_breadcrumb_len: usize,
) -> usize {
    let t_current = derive_t(current_len, current_breadcrumb_len);
    let t_chunk = derive_t(chunk_len, chunk_breadcrumb_len);
    let t_new = merge_text_len(t_current, t_chunk);
    if current_breadcrumb_len == 0 || t_new == 0 {
        current_breadcrumb_len + t_new
    } else {
        current_breadcrumb_len + 1 + t_new
    }
}

pub struct UpdateLengthAfterAbsorbArgs<'a> {
    pub current_len: usize,
    pub current_breadcrumb_len: usize,
    pub child_len: usize,
    pub child_breadcrumb_len: usize,
    pub header_prefix: &'a str,
    pub child_header: &'a str,
}

pub fn update_length_after_absorb(args: UpdateLengthAfterAbsorbArgs<'_>) -> usize {
    let t_current = derive_t(args.current_len, args.current_breadcrumb_len);
    let t_child = derive_t(args.child_len, args.child_breadcrumb_len);
    let t_header = merge_text_len(
        default_length_counter(args.header_prefix),
        default_length_counter(args.child_header),
    );
    let t_appended = merge_text_len(t_header, t_child);
    let t_new = merge_text_len(t_current, t_appended);
    if args.current_breadcrumb_len == 0 || t_new == 0 {
        args.current_breadcrumb_len + t_new
    } else {
        args.current_breadcrumb_len + 1 + t_new
    }
}

/// Replace `\u{E000}CODE_BLOCK_N\u{E000}` placeholders back with the original code content.
/// Single-pass O(N) where N is the length of `text`.
pub fn restore_code_placeholders<'a>(
    text: &'a str,
    blocks: &[String],
) -> std::borrow::Cow<'a, str> {
    const SENTINEL: char = '\u{E000}';
    if blocks.is_empty() {
        return std::borrow::Cow::Borrowed(text);
    }

    let mut remaining = text;
    let mut start = match remaining.find(SENTINEL) {
        Some(idx) => idx,
        None => return std::borrow::Cow::Borrowed(text),
    };

    let mut out = String::with_capacity(text.len());
    loop {
        out.push_str(&remaining[..start]);
        remaining = &remaining[start + SENTINEL.len_utf8()..];

        if let Some(end) = remaining.find(SENTINEL) {
            let tag = &remaining[..end];
            remaining = &remaining[end + SENTINEL.len_utf8()..];
            if let Some(block) = tag
                .strip_prefix("CODE_BLOCK_")
                .and_then(|idx_str| idx_str.parse::<usize>().ok())
                .and_then(|idx| blocks.get(idx))
            {
                out.push_str(block);
            } else {
                // Not a valid placeholder — emit the delimiters and tag verbatim.
                out.push(SENTINEL);
                out.push_str(tag);
                out.push(SENTINEL);
            }
        } else {
            // Lone sentinel with no closing pair — emit verbatim.
            out.push(SENTINEL);
            break;
        }

        start = match remaining.find(SENTINEL) {
            Some(idx) => idx,
            None => break,
        };
    }

    out.push_str(remaining);
    std::borrow::Cow::Owned(out)
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

    for (p_opt, c_opt) in parent.iter().zip(child.iter()) {
        match (p_opt, c_opt) {
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
    use super::{
        header_is_superset_of, restore_code_placeholders, set_length, update_length_after_absorb,
        UpdateLengthAfterAbsorbArgs,
    };
    use crate::types::Chunk;
    fn s(v: &str) -> Option<String> {
        Some(v.to_string())
    }
    fn chunk(breadcrumb: &str, text: &str) -> Chunk {
        Chunk {
            level: 1,
            header: None,
            headers: std::sync::Arc::new(vec![None; 6]),
            breadcrumb: std::sync::Arc::new(breadcrumb.to_string()),
            text: text.to_string(),
            length: 0,
        }
    }
    #[test]
    fn set_length_empty_breadcrumb_nonempty_text() {
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
    fn set_length_breadcrumb_empty_text() {
        // Non-empty breadcrumb with empty text: separator must not be counted.
        let mut c = chunk("H1", "");
        set_length(&mut c);
        assert_eq!(c.length, 2); // breadcrumb only: "H1"
    }
    #[test]
    fn set_length_breadcrumb_whitespace_text() {
        // Non-empty breadcrumb with whitespace-only text: text normalizes to 0,
        // so separator must not be counted.
        let mut c = chunk("H1", "   ");
        set_length(&mut c);
        assert_eq!(c.length, 2); // breadcrumb only: "H1"
    }
    #[test]
    fn restore_zero() {
        assert_eq!(restore_code_placeholders("no code", &[]), "no code");
    }
    #[test]
    fn restore_one() {
        let placeholder = "\u{E000}CODE_BLOCK_0\u{E000}";
        let txt = format!("A {placeholder} B");
        let r = restore_code_placeholders(&txt, &["X".to_string()]);
        assert_eq!(r, "A X B");
    }
    #[test]
    fn restore_many() {
        let p0 = "\u{E000}CODE_BLOCK_0\u{E000}";
        let p1 = "\u{E000}CODE_BLOCK_1\u{E000}";
        let txt = format!("{p0} {p1}");
        let r = restore_code_placeholders(&txt, &["A".to_string(), "B".to_string()]);
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

    #[test]
    fn test_update_length_after_absorb() {
        let update = |current_len: usize,
                      current_breadcrumb_len: usize,
                      child_len: usize,
                      child_breadcrumb_len: usize,
                      header_prefix: &'static str,
                      child_header: &'static str| {
            update_length_after_absorb(UpdateLengthAfterAbsorbArgs {
                current_len,
                current_breadcrumb_len,
                child_len,
                child_breadcrumb_len,
                header_prefix,
                child_header,
            })
        };

        // All parts present:
        // current_len: 10, current_breadcrumb: 5 -> t_current = 10 - 5 - 1 = 4
        // child_len: 8, child_breadcrumb: 3 -> t_child = 8 - 3 - 1 = 4
        // header_prefix: "#", child_header: "A" -> "# A" len 3 -> t_header = 3
        // t_appended = merge(3, 4) = 3 + 1 + 4 = 8
        // t_new = merge(4, 8) = 4 + 1 + 8 = 13
        // current_breadcrumb_len != 0 -> 5 + 1 + 13 = 19
        assert_eq!(update(10, 5, 8, 3, "#", "A"), 19);

        // Current text is empty:
        // current_len: 5, current_breadcrumb: 5 -> t_current = 0
        // child_len: 8, child_breadcrumb: 3 -> t_child = 4
        // header: "# A" -> 3
        // t_appended = 8
        // t_new = merge(0, 8) = 8
        // current_breadcrumb != 0 -> 5 + 1 + 8 = 14
        assert_eq!(update(5, 5, 8, 3, "#", "A"), 14);

        // Child text is empty:
        // current_len: 10, current_breadcrumb: 5 -> t_current = 4
        // child_len: 3, child_breadcrumb: 3 -> t_child = 0
        // header: "# A" -> 3
        // t_appended = merge(3, 0) = 3
        // t_new = merge(4, 3) = 4 + 1 + 3 = 8
        // current_breadcrumb != 0 -> 5 + 1 + 8 = 14
        assert_eq!(update(10, 5, 3, 3, "#", "A"), 14);

        // Current breadcrumb is empty:
        // current_len: 4, current_breadcrumb: 0 -> t_current = 4
        // child_len: 8, child_breadcrumb: 3 -> t_child = 4
        // header: "# A" -> 3
        // t_appended = 8
        // t_new = merge(4, 8) = 4 + 1 + 8 = 13
        // current_breadcrumb == 0 -> 0 + 13 = 13
        assert_eq!(update(4, 0, 8, 3, "#", "A"), 13);

        // Empty breadcrumbs and texts:
        // current_len: 0, current_breadcrumb: 0 -> t_current = 0
        // child_len: 0, child_breadcrumb: 0 -> t_child = 0
        // header: " " -> 0
        // t_appended = 0
        // t_new = 0
        // current_breadcrumb == 0 -> 0
        assert_eq!(update(0, 0, 0, 0, "", ""), 0);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::{derive_t, merge_text_len, update_length_after_merge};

    #[test]
    fn test_derive_t() {
        assert_eq!(derive_t(10, 5), 4);
        assert_eq!(derive_t(5, 10), 0);
        assert_eq!(derive_t(5, 0), 5);
    }

    #[test]
    fn test_derive_t_len_less_than_or_equal_to_b() {
        assert_eq!(derive_t(5, 5), 0);
        assert_eq!(derive_t(4, 5), 0);
        assert_eq!(derive_t(0, 5), 0);
    }

    #[test]
    fn test_derive_t_b_is_zero() {
        assert_eq!(derive_t(5, 0), 5);
        assert_eq!(derive_t(10, 0), 10);
    }

    #[test]
    fn test_derive_t_len_greater_than_b_and_b_positive() {
        assert_eq!(derive_t(10, 4), 5); // 10 - 4 - 1 = 5
        assert_eq!(derive_t(6, 1), 4); // 6 - 1 - 1 = 4
    }

    #[test]
    fn test_merge_text_len() {
        assert_eq!(merge_text_len(0, 5), 5);
        assert_eq!(merge_text_len(5, 0), 5);
        assert_eq!(merge_text_len(5, 5), 11);
    }

    #[test]
    fn test_update_length_after_merge() {
        assert_eq!(update_length_after_merge(10, 0, 10, 0), 21);
    }
}

#[cfg(test)]
mod gaps_tests {
    use super::*;

    #[test]
    fn test_derive_t_and_merge_text_len_gaps() {
        assert_eq!(derive_t(5, 5), 0);
        assert_eq!(derive_t(4, 5), 0);
        assert_eq!(merge_text_len(0, 5), 5);
        assert_eq!(merge_text_len(5, 0), 5);
        assert_eq!(update_length_after_merge(0, 0, 0, 0), 0);
    }

    #[test]
    fn test_update_length_after_absorb_blank_child_heading_has_no_extra_separator() {
        assert_eq!(
            update_length_after_absorb(UpdateLengthAfterAbsorbArgs {
                current_len: 2,
                current_breadcrumb_len: 0,
                child_len: 0,
                child_breadcrumb_len: 0,
                header_prefix: "##",
                child_header: "",
            }),
            5,
            "blank child heading should not allocate separator after header marker"
        );
    }
}
