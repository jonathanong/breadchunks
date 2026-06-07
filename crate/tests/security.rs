use breadchunks::chunk;

#[test]
fn test_unbounded_code_block_restoration() {
    let mut malicious_markdown = String::from("```\n");
    malicious_markdown.push_str(&"A".repeat(10_000));
    malicious_markdown.push_str("\n```\n");

    for _ in 0..10_000 {
        malicious_markdown.push_str("\n\u{E000}CODE_BLOCK_0\u{E000}\n");
    }

    let chunks = chunk(&malicious_markdown, None);

    let total_len: usize = chunks.iter().map(|c| c.text.len()).sum();
    println!("Total text len: {}", total_len);
    assert!(
        total_len < 100_000_000,
        "Vulnerable to unbounded code block expansion. Total length: {}",
        total_len
    );
}
