use super::types::Chunk;
use super::utils::{restore_code_placeholders, set_length};
use regex::Regex;
use std::sync::LazyLock;

static CODE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"```[\s\S]*?```|`[^`]+`").expect("BUG: invalid code block regex"));

static HEADER_REGEX: LazyLock<Regex> =
    // Match markdown headers that are at the start of the document or after a newline.
    // We intentionally keep only the full match and strip a leading '\n' in code.
    LazyLock::new(|| {
        Regex::new(r"(?:^|\n)(?:#{1,6}\s+.+)").expect("BUG: invalid header regex")
    });

static PARAGRAPH_SPLIT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\s*\n").expect("BUG: invalid paragraph split regex"));

/// Phase 1: Split markdown into one chunk per paragraph, grouped under its nearest header.
pub fn split_by_headers(text: &str, title: Option<&str>) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let title_owned = title.map(std::string::ToString::to_string);

    let (text_without_code, code_blocks) = extract_code_blocks(text);
    let mut headers: Vec<Option<String>> = vec![title_owned.clone(), None, None, None, None, None];

    let first_header = HEADER_REGEX.find(&text_without_code);

    if let Some(first_match) = first_header {
        let preface_content = &text_without_code[..first_match.start()];
        split_paragraphs(SplitParagraphsArgs {
            content: preface_content,
            code_blocks: &code_blocks,
            level: 0,
            header: title,
            headers: &headers,
            chunks: &mut chunks,
        });
    }

    // Collect only the byte positions and header text slice we actually need.
    // Using `find_iter` avoids the overhead of `Captures` objects.
    let header_matches: Vec<(usize, usize, &str)> = HEADER_REGEX
        .find_iter(&text_without_code)
        .map(|m| {
            let raw = m.as_str();
            let header_text = raw.strip_prefix('\n').unwrap_or(raw);
            (m.start(), m.end(), header_text)
        })
        .collect();

    for (i, &(_, full_end, header_text)) in header_matches.iter().enumerate() {
        let header_text = header_text.trim_end_matches('\r');
        let level = header_text.bytes().take_while(|&b| b == b'#').count() as u32;
        let header_content_raw = header_text.trim_start_matches('#').trim();
        let header_content =
            restore_code_placeholders(header_content_raw, &code_blocks).into_owned();

        headers[(level - 1) as usize] = Some(header_content.clone());
        headers[level as usize..].fill(None);

        let content_end = if i + 1 < header_matches.len() {
            header_matches[i + 1].0
        } else {
            text_without_code.len()
        };

        let section_content = &text_without_code[full_end..content_end];
        split_paragraphs(SplitParagraphsArgs {
            content: section_content,
            code_blocks: &code_blocks,
            level,
            header: Some(header_content.as_str()),
            headers: &headers,
            chunks: &mut chunks,
        });
    }

    if header_matches.is_empty() {
        split_paragraphs(SplitParagraphsArgs {
            content: &text_without_code,
            code_blocks: &code_blocks,
            level: 0,
            header: title,
            headers: &headers,
            chunks: &mut chunks,
        });
    }

    chunks
}

/// Single-pass extraction: replaces each code block with a PUA-bracketed
/// placeholder (`U+E000 CODE_BLOCK_N U+E000`) that cannot appear in ordinary
/// Markdown, then returns the substituted text and the extracted blocks.
fn extract_code_blocks(text: &str) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for (i, m) in CODE_BLOCK_REGEX.find_iter(text).enumerate() {
        out.push_str(&text[cursor..m.start()]);
        use std::fmt::Write as _;
        let _ = write!(out, "\u{E000}CODE_BLOCK_{i}\u{E000}");
        blocks.push(m.as_str().to_string());
        cursor = m.end();
    }
    out.push_str(&text[cursor..]);
    (out, blocks)
}

fn build_breadcrumb(headers: &[Option<String>]) -> String {
    let mut iter = headers.iter().filter_map(|h| h.as_deref());
    let Some(first) = iter.next() else {
        return String::new();
    };

    // Pre-allocate a reasonable size to avoid most reallocations.
    // 64 bytes is often enough for a typical breadcrumb.
    let mut s = String::with_capacity(first.len() + 64);
    s.push_str(first);

    for h in iter {
        s.push_str(" > ");
        s.push_str(h);
    }

    s
}

struct SplitParagraphsArgs<'a, 'b> {
    content: &'a str,
    code_blocks: &'a [String],
    level: u32,
    header: Option<&'a str>,
    headers: &'a [Option<String>],
    chunks: &'b mut Vec<Chunk>,
}

fn split_paragraphs(args: SplitParagraphsArgs) {
    let mut paragraphs = PARAGRAPH_SPLIT_REGEX
        .split(args.content)
        .filter(|p| !p.trim().is_empty());

    if let Some(first) = paragraphs.next() {
        let breadcrumb = build_breadcrumb(args.headers);
        let prototype = Chunk {
            level: args.level,
            header: args.header.map(|s| std::sync::Arc::new(s.to_string())),
            headers: std::sync::Arc::new(args.headers.to_vec()),
            breadcrumb: std::sync::Arc::new(breadcrumb),
            text: String::new(),
            length: 0,
        };

        for paragraph in std::iter::once(first).chain(paragraphs) {
            let mut chunk = prototype.clone();
            chunk.text = restore_code_placeholders(paragraph.trim(), args.code_blocks).into_owned();
            set_length(&mut chunk);
            args.chunks.push(chunk);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_split_by_headers_empty_text() {
        let chunks = split_by_headers("", None);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_by_headers_no_headers() {
        let text = "This is a simple paragraph.\n\nAnd another one.";
        let chunks = split_by_headers(text, None);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].level, 0);
        assert_eq!(chunks[0].header, None);
        assert_eq!(chunks[0].breadcrumb.as_str(), "");
        assert_eq!(chunks[0].text, "This is a simple paragraph.");
        assert_eq!(chunks[1].level, 0);
        assert_eq!(chunks[1].header, None);
        assert_eq!(chunks[1].breadcrumb.as_str(), "");
        assert_eq!(chunks[1].text, "And another one.");
    }

    #[test]
    fn test_split_by_headers_with_title() {
        let text = "This is a simple paragraph.";
        let chunks = split_by_headers(text, Some("My Title"));
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].level, 0);
        assert_eq!(chunks[0].header, Some(Arc::new("My Title".to_string())));
        assert_eq!(chunks[0].breadcrumb.as_str(), "My Title");
        assert_eq!(chunks[0].text, "This is a simple paragraph.");
    }

    #[test]
    fn test_split_by_headers_basic() {
        let text = "# Main Header\n\nSome content.";
        let chunks = split_by_headers(text, None);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].level, 1);
        assert_eq!(chunks[0].header, Some(Arc::new("Main Header".to_string())));
        assert_eq!(chunks[0].breadcrumb.as_str(), "Main Header");
        assert_eq!(chunks[0].text, "Some content.");
    }

    #[test]
    fn test_split_by_headers_multiple_headers() {
        let text = "# H1\n\nContent 1\n\n## H2\n\nContent 2\n\n### H3\n\nContent 3";
        let chunks = split_by_headers(text, None);
        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].level, 1);
        assert_eq!(chunks[0].header, Some(Arc::new("H1".to_string())));
        assert_eq!(chunks[0].breadcrumb.as_str(), "H1");
        assert_eq!(chunks[0].text, "Content 1");

        assert_eq!(chunks[1].level, 2);
        assert_eq!(chunks[1].header, Some(Arc::new("H2".to_string())));
        assert_eq!(chunks[1].breadcrumb.as_str(), "H1 > H2");
        assert_eq!(chunks[1].text, "Content 2");

        assert_eq!(chunks[2].level, 3);
        assert_eq!(chunks[2].header, Some(Arc::new("H3".to_string())));
        assert_eq!(chunks[2].breadcrumb.as_str(), "H1 > H2 > H3");
        assert_eq!(chunks[2].text, "Content 3");
    }

    #[test]
    fn test_split_by_headers_preface() {
        let text = "Intro text.\n\n# H1\n\nContent 1";
        let chunks = split_by_headers(text, None);
        assert_eq!(chunks.len(), 2);

        assert_eq!(chunks[0].level, 0);
        assert_eq!(chunks[0].header, None);
        assert_eq!(chunks[0].breadcrumb.as_str(), "");
        assert_eq!(chunks[0].text, "Intro text.");

        assert_eq!(chunks[1].level, 1);
        assert_eq!(chunks[1].header, Some(Arc::new("H1".to_string())));
        assert_eq!(chunks[1].breadcrumb.as_str(), "H1");
        assert_eq!(chunks[1].text, "Content 1");
    }

    #[test]
    fn test_split_by_headers_code_blocks() {
        let text = "# H1\n\n```\n# Fake Header\n```\n\nMore content.";
        let chunks = split_by_headers(text, None);
        assert_eq!(chunks.len(), 2);

        assert_eq!(chunks[0].level, 1);
        assert_eq!(chunks[0].header, Some(Arc::new("H1".to_string())));
        assert_eq!(chunks[0].breadcrumb.as_str(), "H1");
        assert_eq!(chunks[0].text, "```\n# Fake Header\n```");

        assert_eq!(chunks[1].level, 1);
        assert_eq!(chunks[1].header, Some(Arc::new("H1".to_string())));
        assert_eq!(chunks[1].breadcrumb.as_str(), "H1");
        assert_eq!(chunks[1].text, "More content.");
    }
}
