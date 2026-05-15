use super::tokens::default_length_counter;
use super::types::Chunk;

/// Compute character length for a chunk. Includes the breadcrumb in the count
/// because embeddings will see "breadcrumb\n\ntext" as the full input.
pub fn set_length(chunk: &mut Chunk) {
    if chunk.breadcrumb.is_empty() {
        chunk.length = default_length_counter(&chunk.text);
    } else {
        let text = format!("{}\n\n{}", chunk.breadcrumb, chunk.text);
        chunk.length = default_length_counter(&text);
    }
}

/// Replace `___CODE_BLOCK_N___` placeholders back with the original code content.
pub fn restore_code_placeholders(text: &str, blocks: &[String]) -> String {
    let mut result = text.to_string();
    for (i, block) in blocks.iter().enumerate() {
        let placeholder = format!("___CODE_BLOCK_{i}___");
        result = result.replace(&placeholder, block);
    }
    result
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
        let r = restore_code_placeholders("A ___CODE_BLOCK_0___ B", &["X".to_string()]);
        assert_eq!(r, "A X B");
    }
    #[test]
    fn restore_many() {
        let r = restore_code_placeholders(
            "___CODE_BLOCK_0___ ___CODE_BLOCK_1___",
            &["A".to_string(), "B".to_string()],
        );
        assert_eq!(r, "A B");
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
