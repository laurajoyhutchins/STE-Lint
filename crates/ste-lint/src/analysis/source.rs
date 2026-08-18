use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug, Clone)]
pub(crate) struct SourceDocument {
    projection: String,
}

impl SourceDocument {
    pub(crate) fn new(text: &str) -> Self {
        let mut visible = Vec::new();
        let mut protected = Vec::new();
        let mut code_block_depth = 0usize;

        for (event, range) in Parser::new(text).into_offset_iter() {
            match event {
                Event::Start(Tag::CodeBlock(_)) => {
                    code_block_depth += 1;
                    protected.push(range);
                }
                Event::End(TagEnd::CodeBlock) => {
                    protected.push(range);
                    code_block_depth = code_block_depth.saturating_sub(1);
                }
                Event::Text(_) if code_block_depth == 0 => visible.push(range),
                Event::Text(_) => protected.push(range),
                Event::Code(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::Rule
                | Event::TaskListMarker(_) => protected.push(range),
                Event::SoftBreak | Event::HardBreak => visible.push(range),
                Event::Start(_) | Event::End(_) => {}
            }
        }

        let visible = merged_ranges(visible);
        let protected = merged_ranges(protected);
        let projection = project_text(text, &visible, &protected);
        Self { projection }
    }

    pub(crate) fn linguistic_projection(&self) -> &str {
        &self.projection
    }
}

pub(crate) fn char_to_byte_offsets(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(text.len()))
        .collect()
}

fn merged_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|range| range.start < range.end);
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut merged: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
            continue;
        }
        merged.push(range);
    }
    merged
}

fn project_text(text: &str, visible: &[Range<usize>], protected: &[Range<usize>]) -> String {
    let mut projection = String::with_capacity(text.len());
    for (byte, character) in text.char_indices() {
        let is_protected = contains_byte(protected, byte);
        let is_visible = contains_byte(visible, byte);
        if !is_protected && (is_visible || matches!(character, '\n' | '\r')) {
            projection.push(character);
        } else {
            projection.push(' ');
        }
    }
    projection
}

fn contains_byte(ranges: &[Range<usize>], byte: usize) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= byte && byte < range.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiline_inline_code_is_protected_without_changing_character_coordinates() {
        let text = "USE `alpha.\nbeta` here. USE this.";
        let source = SourceDocument::new(text);
        assert_eq!(
            source.linguistic_projection().chars().count(),
            text.chars().count()
        );
        assert!(!source.linguistic_projection().contains("alpha"));
        assert!(source.linguistic_projection().contains("USE"));
    }

    #[test]
    fn markdown_syntax_and_code_are_not_linguistic_text() {
        let source = SourceDocument::new("# TITLE\n\nUSE **this** and `that`.\n");
        let projection = source.linguistic_projection();
        assert!(!projection.contains('#'));
        assert!(!projection.contains("that"));
        assert!(projection.contains("TITLE"));
        assert!(projection.contains("USE"));
        assert!(projection.contains("this"));
    }
}
