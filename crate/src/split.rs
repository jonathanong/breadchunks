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

    let header_matches: Vec<_> = HEADER_REGEX.find_iter(&text_without_code).collect();

    for (i, header_match) in header_matches.iter().enumerate() {
        let header_text = header_match.as_str().trim().to_string();
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

        let content_start = header_match.end();
        let content_end = if i + 1 < header_matches.len() {
            header_matches[i + 1].start()
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

        let should_create = text.is_empty() || !restored_content.trim().is_empty();

        if should_create {
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

fn extract_code_blocks(text: &str) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let mut result = text.to_string();

    for (i, capture) in CODE_BLOCK_REGEX.find_iter(text).enumerate() {
        blocks.push(capture.as_str().to_string());
        let placeholder = format!("___CODE_BLOCK_{i}___");
        result = result.replacen(capture.as_str(), &placeholder, 1);
    }

    (result, blocks)
}

fn build_breadcrumb(headers: &[Option<String>]) -> String {
    headers
        .iter()
        .filter_map(|h| h.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join(" > ")
}
