//! Targeted tests that close coverage gaps revealed by cargo-llvm-cov.
//! Each test is labeled with the function and branch it specifically exercises.

use breadchunks::{chunk, default_length_counter, ChunkOptions};

// ── tokens::default_length_counter ────────────────────────────────────────

#[test]
fn counter_newline_collapsed() {
    // newline between words → single space → same as "a b"
    assert_eq!(default_length_counter("a\nb"), 3);
}

#[test]
fn counter_tab_collapsed() {
    assert_eq!(default_length_counter("a\tb"), 3);
}

#[test]
fn counter_multibyte_chars_counted_not_bytes() {
    // "→" is 3 bytes but 1 char
    assert_eq!(default_length_counter("a→b"), 3);
}

// ── utils::set_length — empty vs non-empty breadcrumb ─────────────────────

#[test]
fn set_length_empty_breadcrumb() {
    // Level-0 chunks (preface, no headers) have empty breadcrumb
    let chunks = chunk("Plain text, no headers.", None);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].breadcrumb, "");
    // length must equal char count of text (no breadcrumb prefix)
    assert_eq!(chunks[0].length, default_length_counter(&chunks[0].text));
}

#[test]
fn set_length_non_empty_breadcrumb() {
    let chunks = chunk("# H1\n\nsome content", None);
    assert_eq!(chunks.len(), 1);
    // length includes "H1\n\nsome content" char count
    let expected = default_length_counter("H1\n\nsome content");
    assert_eq!(chunks[0].length, expected);
}

// ── utils::restore_code_placeholders — zero / one / multiple ──────────────

#[test]
fn restore_zero_blocks() {
    // No code blocks → no placeholders, text unchanged
    let chunks = chunk("# H\n\nNo code here.", None);
    assert_eq!(chunks[0].text, "No code here.");
}

#[test]
fn restore_one_block() {
    let text = "# H\n\n```\ncode\n```";
    let chunks = chunk(text, None);
    let combined: String = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join("");
    assert!(combined.contains("```\ncode\n```"));
}

#[test]
fn restore_multiple_blocks() {
    let text = "# H\n\n```\nblock1\n```\n\nSome text.\n\n```\nblock2\n```";
    let chunks = chunk(text, None);
    let combined: String = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(combined.contains("block1"));
    assert!(combined.contains("block2"));
}

// ── utils::header_is_superset_of — all four match arms ────────────────────

#[test]
fn superset_different_length_returns_false() {
    // header_is_superset_of is exercised indirectly via merge_phase3
    // We force a cross-hierarchy situation where the check must return false
    // (two h2s that are siblings, so child detection fails)
    let text = "# Root\n\nRoot text.\n\n## Sibling A\n\nSib A text.\n\n## Sibling B\n\nSib B text.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(10000),
            max_length: Some(100000),
            ..Default::default()
        }),
    );
    // Sibling B must NOT be absorbed into Sibling A
    let combined: String = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(combined.contains("Sib A text."));
    assert!(combined.contains("Sib B text."));
}

#[test]
fn superset_parent_none_returns_true() {
    // A deeply nested child IS a child of a shallower parent → phase3 absorbs it
    let text = "# Root\n\nRoot.\n\n## Child\n\nChild.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            min_length: Some(10000),
            max_length: Some(100000),
            ..Default::default()
        }),
    );
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("## Child"));
}

#[test]
fn superset_parent_some_child_none_returns_false() {
    // h2 "A" followed by h3 "B" then h2 "A" again — the second h2 should NOT
    // be treated as a child of the first h2, even though they share "Root".
    let text = "# Root\n\nR.\n\n## A\n\nA text.\n\n### Sub\n\nSub text.\n\n## B\n\nB text.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    // After phase1, B should be a separate chunk, not merged into A
    assert!(chunks.iter().any(|c| c.header == Some("B".to_string())));
}

// ── merge::should_merge — all four quadrants ──────────────────────────────

#[test]
fn should_merge_both_big_no_merge() {
    // a=600, b=600, min=400, max=2000 → both >= min → false
    let text = format!(
        "# H1\n\n{}\n\n# H1\n\n{}\n",
        "word ".repeat(120), // ~600 chars
        "word ".repeat(120),
    );
    let p1 = chunk(
        &text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    let p2 = chunk(
        &text,
        Some(ChunkOptions {
            phase: Some(2),
            min_length: Some(400),
            max_length: Some(2000),
            ..Default::default()
        }),
    );
    // Both chunks are big → phase2 should NOT merge them
    assert_eq!(p1.len(), p2.len());
}

#[test]
fn should_merge_one_small_within_max_merges() {
    // a=small, b=small, sum <= max → merge
    let text = "# H\n\nShort.\n\n# H\n\nAlso short.";
    let p1 = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    let p2 = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(2),
            min_length: Some(1000),
            max_length: Some(10000),
            ..Default::default()
        }),
    );
    assert!(p2.len() < p1.len());
}

#[test]
fn should_merge_sum_exceeds_max_no_merge() {
    // a=small (< min), b=huge, sum > max → false
    let text = format!(
        "# H\n\nSmall.\n\n# H\n\n{}\n",
        "word ".repeat(500), // far over any max we set
    );
    let p2 = chunk(
        &text,
        Some(ChunkOptions {
            phase: Some(2),
            min_length: Some(1000),
            max_length: Some(100), // tiny max
            ..Default::default()
        }),
    );
    // "Small." is well under max=100, but the large chunk is over → no merge
    assert!(p2.len() >= 2);
}

// ── merge::merge_phase2 — all branches ────────────────────────────────────

#[test]
fn phase2_empty_input() {
    // Empty markdown → phase 1 returns nothing (whitespace-only → 0 chunks)
    // BUT chunk("   ") returns 0 chunks, so phase2 gets an empty vec
    let chunks = chunk(
        "   \n\n   ",
        Some(ChunkOptions {
            phase: Some(2),
            ..Default::default()
        }),
    );
    assert_eq!(chunks.len(), 0);
}

#[test]
fn phase2_different_breadcrumb_not_merged() {
    let text = "# A\n\nText A.\n\n## B\n\nText B.";
    let p1 = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    let p2 = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(2),
            min_length: Some(10000),
            max_length: Some(100000),
            ..Default::default()
        }),
    );
    // Different breadcrumbs ("A" vs "A > B") → no merge even with huge min
    assert_eq!(p1.len(), p2.len());
}

#[test]
fn phase2_trailing_accumulator_flushed() {
    // Single chunk: accumulator is flushed in the `result.extend(current)` line
    let chunks = chunk("# Only\n\nOne chunk.", None);
    assert_eq!(chunks.len(), 1);
}

// ── merge::merge_phase3 — chunk not at level / at max_length ──────────────

#[test]
fn phase3_chunk_at_max_length_stays() {
    // A chunk whose length >= max_length should NOT be absorbed by phase3
    let long = "word ".repeat(1000); // ~5000 chars
    let text = format!("# Parent\n\n{}\n\n## Child\n\nChild text.\n", long);
    let chunks = chunk(
        &text,
        Some(ChunkOptions {
            min_length: Some(10000),
            max_length: Some(100), // parent chunk is far above this
            ..Default::default()
        }),
    );
    // Parent is already at/above max → child stays separate
    assert!(chunks.len() >= 2);
}

#[test]
fn phase3_child_too_large_stays_separate() {
    // Large child (> max when added to parent) must NOT be absorbed
    let big_child = "word ".repeat(1000);
    let text = format!("# Parent\n\nSmall parent.\n\n## Child\n\n{}\n", big_child);
    let chunks = chunk(
        &text,
        Some(ChunkOptions {
            min_length: Some(10000),
            max_length: Some(200),
            ..Default::default()
        }),
    );
    assert!(chunks.len() >= 2);
}

// ── split::split_by_headers — various edge cases ──────────────────────────

#[test]
fn split_no_headers_empty_text() {
    // empty input → 0 chunks (symmetric with whitespace-only input)
    let chunks = chunk("", None);
    assert!(chunks.is_empty());
}

#[test]
fn split_no_headers_nonempty_text() {
    // plain text, no headers → single level-0 chunk
    let chunks = chunk("Hello, world!", None);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].level, 0);
    assert_eq!(chunks[0].breadcrumb, "");
}

#[test]
fn split_no_headers_with_title() {
    // title propagates to header/breadcrumb in no-headers fallback
    let chunks = chunk(
        "Hello.",
        Some(ChunkOptions {
            title: Some("My Doc".to_string()),
            ..Default::default()
        }),
    );
    assert_eq!(chunks[0].breadcrumb, "My Doc");
    assert_eq!(chunks[0].header, Some("My Doc".to_string()));
    assert_eq!(chunks[0].headers[0], Some("My Doc".to_string()));
}

#[test]
fn split_preface_whitespace_paragraphs_filtered() {
    // Blank lines between preface and first header must not create empty chunks
    let text = "\n\n\n\n# H1\n\nContent.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    // No empty-text chunks
    assert!(chunks.iter().all(|c| !c.text.is_empty()));
}

#[test]
fn split_header_level_6_clears_nothing_below() {
    // h6 is the deepest; there is nothing below level 6 to clear
    let text = "###### H6\n\nDeep content.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    assert_eq!(chunks[0].level, 6);
    assert_eq!(chunks[0].breadcrumb, "H6");
}

#[test]
fn split_header_level_1_clears_h2_through_h6() {
    // After h1, h2-h6 slots must all be None
    let text = "## H2\n\nA.\n\n### H3\n\nB.\n\n# Back to H1\n\nC.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    let h1_chunk = chunks
        .iter()
        .find(|c| c.header == Some("Back to H1".to_string()))
        .unwrap();
    // All slots except h1 (index 0) must be None
    for slot in &h1_chunk.headers[1..] {
        assert!(slot.is_none(), "slot was not cleared: {slot:?}");
    }
}

#[test]
fn split_last_header_content_end_is_text_len() {
    // The last header uses text.len() as content_end (not a later header start)
    let text = "# Only Header\n\nThe only paragraph.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    assert!(chunks
        .iter()
        .any(|c| c.text.contains("The only paragraph.")));
}

#[test]
fn split_preface_with_title() {
    // title appears in preface chunk's breadcrumb when headers exist
    let text = "Intro.\n\n# H1\n\nBody.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            title: Some("Doc".to_string()),
            ..Default::default()
        }),
    );
    let preface = chunks.iter().find(|c| c.text.contains("Intro.")).unwrap();
    assert_eq!(preface.breadcrumb, "Doc");
    assert_eq!(preface.header, Some("Doc".to_string()));
}

#[test]
fn split_preface_without_title() {
    // no title → preface has empty breadcrumb
    let text = "Intro.\n\n# H1\n\nBody.";
    let chunks = chunk(
        text,
        Some(ChunkOptions {
            phase: Some(1),
            ..Default::default()
        }),
    );
    let preface = chunks.iter().find(|c| c.text.contains("Intro.")).unwrap();
    assert_eq!(preface.breadcrumb, "");
    assert!(preface.header.is_none());
}

// ── code-block extraction — placeholder correctness ───────────────────────

#[test]
fn placeholder_collision_regression() {
    // A document that happens to contain the old ___CODE_BLOCK_0___ pattern
    // alongside a real code block. With the PUA-based placeholder scheme the
    // literal text is never touched by extract/restore, so it passes through
    // unchanged while the real code block is correctly preserved.
    let text = "# H\n\n___CODE_BLOCK_0___\n\n```\nreal code\n```";
    let chunks = chunk(text, None);
    let all_text: String = chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(
        all_text.contains("___CODE_BLOCK_0___"),
        "literal placeholder text must pass through unchanged"
    );
    assert!(
        all_text.contains("```\nreal code\n```"),
        "real code block must be preserved"
    );
}

#[test]
fn identical_code_blocks_both_restored() {
    // Two identical fenced blocks in the same document must each be restored
    // independently; a content-based replacement scheme would swap one.
    let text = "# H\n\n```\nfoo\n```\n\nsome text\n\n```\nfoo\n```";
    let chunks = chunk(text, None);
    let all_text: String = chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>().join(" ");
    assert_eq!(
        all_text.matches("```\nfoo\n```").count(),
        2,
        "both identical code blocks must be preserved separately"
    );
}
