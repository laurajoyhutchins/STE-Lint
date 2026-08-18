use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug, Clone)]
pub(crate) struct SourceDocument {
    projection: String,
    paragraphs: Vec<Range<usize>>,
    list_items: Vec<Range<usize>>,
    inline_code: Vec<Range<usize>>,
    protected: Vec<Range<usize>>,
}

impl SourceDocument {
    pub(crate) fn new(text: &str) -> Self {
        let mut visible = Vec::new();
        let mut paragraphs = Vec::new();
        let mut list_items = Vec::new();
        let mut inline_code = Vec::new();
        let mut protected = Vec::new();
        let mut paragraph = None;
        let mut item = None;
        let mut code_block_depth = 0usize;

        for (event, range) in Parser::new(text).into_offset_iter() {
            match event {
                Event::Start(Tag::Paragraph) => paragraph = Some(None),
                Event::End(TagEnd::Paragraph) => {
                    if let Some(Some(range)) = paragraph.take() {
                        paragraphs.push(range);
                    }
                }
                Event::Start(Tag::Item) => item = Some(None),
                Event::End(TagEnd::Item) => {
                    if let Some(Some(range)) = item.take() {
                        list_items.push(range);
                    }
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    code_block_depth += 1;
                    protected.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::End(TagEnd::CodeBlock) => {
                    protected.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                    code_block_depth = code_block_depth.saturating_sub(1);
                }
                Event::Text(_) if code_block_depth == 0 => {
                    visible.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::Text(_) => {
                    protected.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::Code(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {
                    inline_code.push(range.clone());
                    protected.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::Html(_) | Event::InlineHtml(_) => {
                    protected.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::SoftBreak | Event::HardBreak => {
                    visible.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::FootnoteReference(_) | Event::Rule | Event::TaskListMarker(_) => {
                    protected.push(range.clone());
                    extend_active(&mut paragraph, &range);
                    extend_active(&mut item, &range);
                }
                Event::Start(_) | Event::End(_) => {}
            }
        }

        let visible = merged_ranges(visible);
        let paragraphs = merged_ranges(paragraphs);
        let list_items = merged_ranges(list_items);
        let inline_code = merged_ranges(inline_code);
        let protected = merged_ranges(protected);
        let projection = project_text(text, &visible, &protected);

        Self {
            projection,
            paragraphs,
            list_items,
            inline_code,
            protected,
        }
    }

    pub(crate) fn linguistic_projection(&self) -> &str {
        &self.projection
    }

    pub(crate) fn paragraph_ranges(&self) -> &[Range<usize>] {
        &self.paragraphs
    }

    pub(crate) fn list_item_ranges(&self) -> &[Range<usize>] {
        &self.list_items
    }

    pub(crate) fn inline_code_ranges(&self) -> &[Range<usize>] {
        &self.inline_code
    }

    pub(crate) fn protected_ranges(&self) -> &[Range<usize>] {
        &self.protected
    }
}

pub(crate) fn char_to_byte_offsets(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(text.len()))
        .collect()
}

fn extend_active(active: &mut Option<Option<Range<usize>>>, range: &Range<usize>) {
    let Some(current) = active else {
        return;
    };
    match current {
        Some(current) => {
            current.start = current.start.min(range.start);
            current.end = current.end.max(range.end);
        }
        None => *current = Some(range.clone()),
    }
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
        if !is_protected && is_visible {
            projection.push(character);
        } else if !is_protected && matches!(character, '\n' | '\r') {
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
        assert_eq!(source.inline_code_ranges().len(), 1);
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