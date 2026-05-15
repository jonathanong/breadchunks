use super::types::Chunk;
use super::utils::{restore_code_placeholders, set_length};
use regex::Regex;
use std::sync::LazyLock;

static CODE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"```[\s\S]*?```|`[^`]+`").expect("BUG: invalid code block regex"));

static HEADER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\n)(#{1,6}\s+.+)").expect("BUG: invalid header regex"));

static PARAGRAPH_SPLIT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\s*\n").expect("BUG: invalid paragraph split regex"));

/// Phase 1: Split markdown into one chunk per paragraph, grouped under its nearest header.
pub fn split_by_headers(text: &str, title: Option<&str>) -> Vec<Chunk> {
    let mut chunks = Vec::new();

    let (text_without_code, code_blocks) = extract_code_blocks(text);

    let first_header = HEADER_REGEX.find(&text_without_code);

    if let Some(first_match) = first_header {
        let preface_content = &text_without_code[..first_match.start()];

        let paragraphs: Vec<&str> = PARAGRAPH_SPLIT_REGEX
            .split(preface_content)
            .filter(|p| !p.trim().is_empty())
            .collect();

        for paragraph in paragraphs {
            let restored_content = restore_code_placeholders(paragraph.trim(), &code_blocks);

            let mut chunk = Chunk {
                level: 0,
                header: title.map(std::string::ToString::to_string),
                headers: vec![
                    title.map(std::string::ToString::to_string),
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
                breadcrumb: title.unwrap_or("").to_string(),
                text: restored_content,
                length: 0,
            };

            set_length(&mut chunk);
            chunks.push(chunk);
        }
    }

    let mut headers: Vec<Option<String>> = vec![
        title.map(std::string::ToString::to_string),
        None,
        None,
        None,
        None,
        None,
    ];

    let header_matches: Vec<_> = HEADER_REGEX.captures_iter(&text_without_code).collect();

    for (i, cap) in header_matches.iter().enumerate() {
        let full_match = cap.get(0).unwrap();
        let header_text = cap.get(1).unwrap().as_str();
        let level = header_text.chars().take_while(|&c| c == '#').count() as u32;
        let header_content = header_text.trim_start_matches('#').trim();

        headers[(level - 1) as usize] = Some(header_content.to_string());

        for header in headers
            .iter_mut()
            .skip(level as usize)
            .take(6 - level as usize)
        {
            *header = None;
        }

        let breadcrumb = build_breadcrumb(&headers);

        let content_start = full_match.end();
        let content_end = if i + 1 < header_matches.len() {
            header_matches[i + 1].get(0).unwrap().start()
        } else {
            text_without_code.len()
        };

        let section_content = &text_without_code[content_start..content_end];

        let paragraphs: Vec<&str> = PARAGRAPH_SPLIT_REGEX
            .split(section_content)
            .filter(|p| !p.trim().is_empty())
            .collect();

        for paragraph in paragraphs {
            let restored_content = restore_code_placeholders(paragraph.trim(), &code_blocks);

            let mut chunk = Chunk {
                level,
                header: Some(header_content.to_string()),
                headers: headers.clone(),
                breadcrumb: breadcrumb.clone(),
                text: restored_content,
                length: 0,
            };

            set_length(&mut chunk);
            chunks.push(chunk);
        }
    }

    if header_matches.is_empty() {
        let restored_content = restore_code_placeholders(text_without_code.trim(), &code_blocks);

        if !restored_content.trim().is_empty() {
            let mut chunk = Chunk {
                level: 0,
                header: title.map(std::string::ToString::to_string),
                headers: vec![
                    title.map(std::string::ToString::to_string),
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
                breadcrumb: title.unwrap_or("").to_string(),
                text: restored_content,
                length: 0,
            };
            set_length(&mut chunk);
            chunks.push(chunk);
        }
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
        out.push_str(&format!("\u{E000}CODE_BLOCK_{i}\u{E000}"));
        blocks.push(m.as_str().to_string());
        cursor = m.end();
    }
    out.push_str(&text[cursor..]);
    (out, blocks)
}

fn build_breadcrumb(headers: &[Option<String>]) -> String {
    headers
        .iter()
        .filter_map(|h| h.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join(" > ")
}
