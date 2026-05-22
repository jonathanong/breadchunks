use breadchunks::default_length_counter;

#[test]
fn test_empty_string() {
    assert_eq!(default_length_counter(""), 0);
    assert_eq!(default_length_counter("   "), 0);
}

#[test]
fn test_single_word() {
    assert_eq!(default_length_counter("hello"), 5);
}

#[test]
fn test_whitespace_normalization() {
    assert_eq!(default_length_counter("hello   world"), 11);
    assert_eq!(default_length_counter("hello world"), 11);
}

#[test]
fn test_two_words() {
    assert_eq!(default_length_counter("hello world"), 11);
}

#[test]
fn test_minimum_one_char() {
    assert_eq!(default_length_counter("a"), 1);
}

#[test]
fn test_multibyte_utf8() {
    // "日本語" is 3 characters, 9 bytes — must count chars not bytes
    assert_eq!(default_length_counter("日本語"), 3);
}

#[test]
fn test_mixed_whitespace() {
    // tabs and newlines collapse to single space
    assert_eq!(default_length_counter("a\t\nb"), 3);
}

#[test]
fn test_trailing_whitespace() {
    assert_eq!(default_length_counter("hello "), 5);
    assert_eq!(default_length_counter("hello  "), 5);
    assert_eq!(default_length_counter("hello \t\n "), 5);
    assert_eq!(default_length_counter("   "), 0);
    assert_eq!(default_length_counter(""), 0);
}
