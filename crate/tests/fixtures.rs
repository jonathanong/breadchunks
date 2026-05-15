use breadchunks::{chunk, ChunkOptions};
use std::fs;

fn read_fixture(name: &str) -> String {
    let path = format!("../fixtures/{name}");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read fixture: {path}"))
}

// ── tech-guide.md ──────────────────────────────────────────────────────────

#[test]
fn tech_guide_produces_chunks() {
    let text = read_fixture("tech-guide.md");
    let chunks = chunk(&text, None);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|c| c.length > 0));
    assert!(chunks.iter().all(|c| !c.text.is_empty()));
}

#[test]
fn tech_guide_preface_chunk() {
    let text = read_fixture("tech-guide.md");
    let chunks = chunk(&text, None);
    // First chunk is the preface (before "# Getting Started")
    let preface = &chunks[0];
    assert!(preface.header.is_none() || preface.level == 0);
    assert!(preface.text.contains("toolbox"));
}

#[test]
fn tech_guide_code_blocks_not_headers() {
    let text = read_fixture("tech-guide.md");
    let chunks = chunk(&text, None);
    // "# not a header" and "# Global settings" inside fences must not become breadcrumbs
    assert!(!chunks.iter().any(|c| c.breadcrumb.contains("not a header")));
    assert!(!chunks.iter().any(|c| c.breadcrumb.contains("Global settings")));
    // But fenced content must still appear in chunk text
    let combined: String = chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(combined.contains("brew install toolbox"));
}

#[test]
fn tech_guide_header_hierarchy() {
    // Use phase=1 to see all raw chunks before merging absorbs nested sections
    let text = read_fixture("tech-guide.md");
    let chunks = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    let nested: Vec<_> = chunks.iter().filter(|c| c.breadcrumb.contains(" > ")).collect();
    assert!(!nested.is_empty(), "should have nested breadcrumbs in phase-1 output");
}

#[test]
fn tech_guide_phase1_more_chunks_than_default() {
    let text = read_fixture("tech-guide.md");
    let phase1 = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    let full = chunk(&text, None);
    assert!(phase1.len() >= full.len());
}

#[test]
fn tech_guide_max_length_respected() {
    let text = read_fixture("tech-guide.md");
    let chunks = chunk(
        &text,
        Some(ChunkOptions { max_length: Some(300), ..Default::default() }),
    );
    let over: usize = chunks.iter().filter(|c| c.length > 300).count();
    // Allow at most 10% violation (single paragraphs longer than max can't be split)
    assert!(over <= chunks.len() / 10 + 1);
}

// ── recipe.md ──────────────────────────────────────────────────────────────

#[test]
fn recipe_no_preface() {
    let text = read_fixture("recipe.md");
    let chunks = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    // recipe.md starts with "## Lemon..." so there is no preface
    assert!(chunks.iter().all(|c| c.level >= 2));
}

#[test]
fn recipe_breadcrumbs_present() {
    let text = read_fixture("recipe.md");
    let chunks = chunk(&text, None);
    assert!(chunks.iter().all(|c| !c.breadcrumb.is_empty()));
}

#[test]
fn recipe_phase2_merges_short_paragraphs() {
    let text = read_fixture("recipe.md");
    let p1 = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    let p2 = chunk(&text, Some(ChunkOptions { phase: Some(2), ..Default::default() }));
    // phase 2 should merge some same-breadcrumb chunks
    assert!(p2.len() <= p1.len());
}

// ── deeply-nested.md ───────────────────────────────────────────────────────

#[test]
fn deeply_nested_all_levels_present_in_phase1() {
    let text = read_fixture("deeply-nested.md");
    let chunks = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    let levels: std::collections::HashSet<u32> = chunks.iter().map(|c| c.level).collect();
    // Fixture has h1 through h6
    for lvl in 1..=6 {
        assert!(levels.contains(&lvl), "missing level {lvl}");
    }
}

#[test]
fn deeply_nested_phase3_reduces_count() {
    let text = read_fixture("deeply-nested.md");
    let p1 = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    let p3 = chunk(&text, None);
    assert!(p3.len() <= p1.len());
}

#[test]
fn deeply_nested_h6_absorbed() {
    let text = read_fixture("deeply-nested.md");
    let chunks = chunk(
        &text,
        Some(ChunkOptions {
            min_length: Some(10000),
            max_length: Some(100000),
            ..Default::default()
        }),
    );
    // With huge limits everything collapses to very few chunks
    assert!(chunks.len() <= 5);
}

// ── code-heavy.md ──────────────────────────────────────────────────────────

#[test]
fn code_heavy_no_fake_breadcrumbs() {
    let text = read_fixture("code-heavy.md");
    let chunks = chunk(&text, None);
    for c in &chunks {
        assert!(!c.breadcrumb.contains("not a heading"), "fake heading leaked: {}", c.breadcrumb);
        assert!(!c.breadcrumb.contains("Nested comment"), "fake heading leaked: {}", c.breadcrumb);
        assert!(!c.breadcrumb.contains("another decoy"), "fake heading leaked: {}", c.breadcrumb);
    }
}

#[test]
fn code_heavy_code_preserved_in_text() {
    let text = read_fixture("code-heavy.md");
    let chunks = chunk(&text, None);
    let combined: String = chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>().join("\n");
    assert!(combined.contains("print(\"Hello, world!\")"));
    assert!(combined.contains("struct Config"));
    assert!(combined.contains("set -euo pipefail"));
}

// ── gettysburg.md ──────────────────────────────────────────────────────────

#[test]
fn gettysburg_basic_chunks() {
    let text = read_fixture("gettysburg.md");
    let chunks = chunk(&text, None);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|c| c.length > 0));
}

#[test]
fn gettysburg_has_preface_or_background() {
    let text = read_fixture("gettysburg.md");
    let chunks = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    // Should have chunks for Background and Address sections
    assert!(chunks.iter().any(|c| c.breadcrumb.contains("Background") || c.text.contains("1863")));
}

#[test]
fn gettysburg_with_title_option() {
    let text = read_fixture("gettysburg.md");
    // title propagates to preface chunks (level 0); h1+ header chunks overwrite headers[0]
    let phase1 = chunk(
        &text,
        Some(ChunkOptions {
            title: Some("Lincoln Speeches".to_string()),
            phase: Some(1),
            ..Default::default()
        }),
    );
    assert!(!phase1.is_empty());
    // All chunks should carry the title in their option — verify we get chunks at all
    // and that the title appears in the breadcrumb of any preface chunks
    let with_title: Vec<_> = phase1.iter().filter(|c| c.level == 0).collect();
    for c in with_title {
        assert_eq!(c.headers[0], Some("Lincoln Speeches".to_string()));
    }
}

#[test]
fn gettysburg_nested_historical_note() {
    let text = read_fixture("gettysburg.md");
    let chunks = chunk(&text, Some(ChunkOptions { phase: Some(1), ..Default::default() }));
    let bliss = chunks.iter().find(|c| c.breadcrumb.contains("Bliss Copy") || c.text.contains("Bliss"));
    assert!(bliss.is_some());
}
