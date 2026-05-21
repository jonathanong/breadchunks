use breadchunks::{chunk, ChunkOptions};

#[test]
fn test_empty_text() {
    let chunks = chunk("", None);
    assert!(chunks.is_empty());
}

#[test]
fn test_no_headers() {
    let text = "This is a simple paragraph without any headers.";
    let chunks = chunk(text, None);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].level, 0);
    assert_eq!(chunks[0].text, text);
}

#[test]
fn test_single_header() {
    let text = "# Header 1\n\nThis is content under header 1.";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].level, 1);
    assert_eq!(chunks[0].header, Some("Header 1".to_string()));
}

#[test]
fn test_multiple_headers() {
    let text = "# Header 1\n\nContent 1.\n\n## Header 2\n\nContent 2.";
    let options = ChunkOptions {
        phase: Some(1),
        ..Default::default()
    };
    let chunks = chunk(text, Some(options));
    assert!(chunks.len() >= 2);
    assert!(chunks
        .iter()
        .any(|c| c.header == Some("Header 1".to_string())));
    assert!(chunks
        .iter()
        .any(|c| c.header == Some("Header 2".to_string())));
}

#[test]
fn test_breadcrumb_building() {
    let text = "# H1\n\nC1\n\n## H2\n\nC2\n\n### H3\n\nC3";
    let chunks = chunk(text, None);

    let h1_chunk = chunks.iter().find(|c| c.header == Some("H1".to_string()));
    let h2_chunk = chunks.iter().find(|c| c.header == Some("H2".to_string()));
    let h3_chunk = chunks.iter().find(|c| c.header == Some("H3".to_string()));

    if let Some(c) = h1_chunk {
        assert_eq!(c.breadcrumb.as_str(), "H1");
    }
    if let Some(c) = h2_chunk {
        assert_eq!(c.breadcrumb.as_str(), "H1 > H2");
    }
    if let Some(c) = h3_chunk {
        assert_eq!(c.breadcrumb.as_str(), "H1 > H2 > H3");
    }
}

#[test]
fn test_code_block_preservation() {
    let text = "# Header\n\n```rust\nfn main() {}\n```\n\nMore text.";
    let chunks = chunk(text, None);
    let combined: String = chunks.iter().map(|c| c.text.clone()).collect();
    assert!(combined.contains("```rust"));
    assert!(combined.contains("fn main() {}"));
}

#[test]
fn test_phase_1_only() {
    let text = "# H1\n\na\n\n# H1\n\nb";
    let options = ChunkOptions {
        phase: Some(1),
        ..Default::default()
    };
    let chunks = chunk(text, Some(options));
    assert!(chunks.len() >= 2);
}

#[test]
fn test_phase_2_merge_same_breadcrumb() {
    let text = "# H1\n\na\n\n# H1\n\nb";
    let options = ChunkOptions {
        phase: Some(2),
        min_length: Some(128),
        max_length: Some(1000),
        ..Default::default()
    };
    let chunks = chunk(text, Some(options));
    let phase1_chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    assert!(chunks.len() < phase1_chunks.len());
}

#[test]
fn test_title_option() {
    let text = "Content before headers.";
    let options = ChunkOptions {
        title: Some("My Title".to_string()),
        ..Default::default()
    };
    let chunks = chunk(text, Some(options));
    assert_eq!(chunks[0].headers[0], Some("My Title".to_string()));
}

#[test]
fn test_min_max_length() {
    let text = "# H1\n\na\n\n# H2\n\nb";
    let options = ChunkOptions {
        min_length: Some(1),
        max_length: Some(10),
        ..Default::default()
    };
    let chunks = chunk(text, Some(options));
    for c in &chunks {
        assert!(c.length <= 10);
    }
}

#[test]
fn test_simple_h1_h2_h3() {
    let text = "\n# H1\n\nParagraph 1\n\n## H2\n\nParagraph 2\n\n## H3\n\nParagraph 3\n";
    let chunks = chunk(text, None);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].breadcrumb.as_str(), "H1");
    assert!(chunks[0].length > 0);
}

#[test]
fn test_with_hashtags() {
    let text = "\n# H1\n\n[#123](https://example.com)\n";
    let chunks = chunk(text, None);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].breadcrumb.as_str(), "H1");
    assert_eq!(chunks[0].text, "[#123](https://example.com)");
}

#[test]
fn test_no_header_with_title() {
    let text = "\nthis is a test article without a header\n\nthis is a test article without a header\n\nthis is a test article without a header\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            title: Some("test title".to_string()),
            ..Default::default()
        }),
    );
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].breadcrumb.as_str(), "test title");
    assert_eq!(chunks[0].header, Some("test title".to_string()));
}

#[test]
fn test_nested_headers_h2_reset() {
    let text = "\n# H1\n\nParagraph 1\n\n## H2\n\nParagraph 2\n\n### H3\n\nParagraph 3\n\n## H2-2\n\nParagraph 4\n";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    let h2_chunk = chunks
        .iter()
        .find(|c| c.breadcrumb.contains("H2-2") || c.text.contains("Paragraph 4"));
    assert!(h2_chunk.is_some());
    assert!(h2_chunk.unwrap().breadcrumb.contains("H1"));
}

#[test]
fn test_whitespace_only() {
    let chunks = chunk("   \n\n  ", None);
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_custom_min_max() {
    let text = "\n# H1\n\nShort paragraph.\n\n## H2\n\nAnother short paragraph.\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(40),
            max_length: Some(400),
            ..Default::default()
        }),
    );
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|c| c.length <= 400));
}

#[test]
fn test_multiple_paragraphs_same_header() {
    let text = "\n# H1\n\nFirst paragraph.\n\nSecond paragraph.\n\nThird paragraph.\n";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    let h1_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.breadcrumb.as_str() == "H1")
        .collect();
    assert!(!h1_chunks.is_empty());
}

#[test]
fn test_code_blocks_no_fake_headers() {
    let text = "\n# Code Block Test\n\nParagraph before code.\n\n```\n# not a heading\nconst value = 42\n```\n\nParagraph after code.\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(10),
            max_length: Some(500),
            ..Default::default()
        }),
    );

    let code_chunk = chunks.iter().find(|c| c.text.contains("```"));
    assert!(code_chunk.is_some());
    assert!(code_chunk.unwrap().text.contains("# not a heading"));
    assert!(!chunks
        .iter()
        .any(|c| c.breadcrumb.contains("not a heading")));
}

#[test]
fn test_fenced_code_with_language() {
    let text = "\n# JavaScript Example\n\n```javascript\nfunction test() {\n  console.log(\"# Not a header\")\n}\n```\n";
    let chunks = chunk(text, None);
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("```javascript"));
    assert!(chunks[0].text.contains("function test()"));
}

#[test]
fn test_inline_code_hashtags() {
    let text = "\n# H1\n\nUse `git commit -m \"#123 fix\"` for commits.\n";
    let chunks = chunk(text, None);
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("`git commit"));
}

#[test]
fn test_phase1_returns_early() {
    let text = "\n# H1\n\nParagraph 1\n\n## H2\n\nParagraph 2\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    assert!(!chunks.is_empty());
    assert!(chunks.iter().any(|c| c.breadcrumb.as_str() == "H1"));
}

#[test]
fn test_phase2_after_first_merge() {
    let text = "\n# H1\n\nParagraph 1\n\n## H2\n\nParagraph 2\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(2),
            ..Default::default()
        }),
    );
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|c| c.length > 0));
}

#[test]
fn test_paragraphs_merge_under_min_length() {
    let text = "\n# Merge Example\n\nShort one.\n\nShort two.\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(1000),
            max_length: Some(10000),
            ..Default::default()
        }),
    );
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("Short one.\n\nShort two."));
}

#[test]
fn test_parent_absorbs_child() {
    let text = "\n# Parent Header\n\nParent paragraph.\n\n## Child Header\n\nChild paragraph.\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(1000),
            max_length: Some(10000),
            ..Default::default()
        }),
    );
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("## Child Header"));
    assert!(chunks[0].text.contains("Child paragraph."));
}

#[test]
fn test_h6_merges_into_parent() {
    let text = "\n# Root\n\nRoot paragraph.\n\n###### Deep Child\n\nDeep child paragraph.\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(1000),
            max_length: Some(10000),
            ..Default::default()
        }),
    );
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("###### Deep Child"));
    assert!(chunks[0].text.contains("Deep child paragraph"));
}

#[test]
fn test_empty_paragraphs() {
    let text = "\n# H1\n\n\n\n## H2\n\nContent after empty paragraphs.\n";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|c| !c.text.is_empty()));
}

#[test]
fn test_very_long_paragraph() {
    let long_text = "This is a very long paragraph. ".repeat(100);
    let text = format!("\n# H1\n\n{}\n", long_text);
    let chunks = chunk(
        &text,
        Some(ChunkOptions {
            max_length: Some(200),
            ..Default::default()
        }),
    );
    assert!(!chunks.is_empty());
    assert!(chunks.iter().any(|c| c.text.len() > 100));
}

#[test]
fn test_headers_without_content() {
    let text = "\n# H1\n\n## H2\n\n### H3\n\n## H2-2\n\nContent here.\n";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    let h2_chunk = chunks.iter().find(|c| c.text.contains("Content here"));
    assert!(h2_chunk.is_some());
    assert!(h2_chunk.unwrap().breadcrumb.contains("H1"));
}

#[test]
fn test_deep_nesting_h1_to_h6() {
    let text = "\n# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n\nContent at deepest level.\n";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    let deep_chunk = chunks.iter().find(|c| c.text.contains("deepest level"));
    assert!(deep_chunk.is_some());
    assert!(deep_chunk.unwrap().breadcrumb.contains("H1"));
}

#[test]
fn test_special_characters_in_headers() {
    let text = "\n# Header with \"quotes\" and 'apostrophes'\n\nContent here.\n\n## Sub-header with $dollar$ and %percent%\n\nMore content.\n";
    let chunks = chunk(text, None);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().any(|c| c.breadcrumb.contains("quotes")));
}

#[test]
fn test_content_before_first_header_separate_chunk() {
    let text = "\nIntro paragraph before any headings.\n\n# First Header\n\nContent under the first header.\n";
    let chunks = chunk(text, None);
    assert!(chunks.len() >= 2);
    let preface = &chunks[0];
    assert_eq!(preface.breadcrumb.as_str(), "");
    assert!(preface.header.is_none());
    assert!(preface.text.contains("Intro paragraph"));
    assert!(chunks
        .iter()
        .any(|c| c.breadcrumb.as_str() == "First Header"));
}

#[test]
fn test_header_regex_valid_only() {
    let text = "\n# Valid H1\n\nThis has a #hashtag in the middle.\n\n##Invalid header without space\n\n###### Valid H6\n\nContent continues.\n\n####### Invalid H7 (too many hashes)\n";
    let chunks = chunk(text, None);

    assert!(chunks
        .iter()
        .any(|c| c.header == Some("Valid H1".to_string())));
    assert!(!chunks
        .iter()
        .any(|c| c.header.as_ref().is_some_and(|h| h.contains("##Invalid"))));
    assert!(chunks.iter().any(|c| c.text.contains("#hashtag")));
}

#[test]
fn test_merging_respects_token_limits() {
    let text = format!(
        "\n# H1\n\n{}\n\n{}\n\n{}\n",
        "Short. ".repeat(20),
        "Short. ".repeat(20),
        "Short. ".repeat(20)
    );

    let chunks_small = chunk(
        &text,
        Some(ChunkOptions {
            max_length: Some(150),
            ..Default::default()
        }),
    );
    let chunks_large = chunk(
        &text,
        Some(ChunkOptions {
            max_length: Some(2000),
            ..Default::default()
        }),
    );

    assert!(chunks_small.len() >= chunks_large.len());
    assert!(chunks_small.iter().all(|c| c.length <= 150));
    assert!(chunks_large.iter().all(|c| c.length <= 2000));
}

#[test]
fn test_hierarchical_merging_preserves_structure() {
    let text = "\n# H1\n\nContent 1\n\n## H2\n\nContent 2\n\n### H3\n\nContent 3\n";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(40),
            max_length: Some(4000),
            ..Default::default()
        }),
    );

    assert!(chunks.len() <= 2);
    if chunks.len() == 1 {
        let main_chunk = &chunks[0];
        assert!(main_chunk.text.contains("## H2") || main_chunk.text.contains("Content 2"));
    }
}
