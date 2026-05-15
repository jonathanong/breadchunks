use regex::Regex;
use std::sync::LazyLock;

static WHITESPACE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("BUG: invalid WHITESPACE_REGEX"));

/// Count characters in `text` after collapsing all runs of whitespace to a
/// single space and trimming leading/trailing whitespace.
pub fn default_length_counter(text: &str) -> usize {
    let normalized = WHITESPACE_REGEX.replace_all(text.trim(), " ");
    normalized.chars().count()
}
