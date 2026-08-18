use crate::analysis::source::{SourceDocument, SourceListItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NoteBlock {
    pub start: usize,
    pub end: usize,
    pub content_start: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ListItem {
    pub line_start: usize,
    pub line_end: usize,
    pub content_start: usize,
    pub content_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimpleListBlock {
    pub introduced_by_colon: bool,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyLabel {
    Warning,
    Caution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SafetyBlock {
    pub start: usize,
    pub end: usize,
    pub content_start: usize,
    pub content_end: usize,
    pub label: SafetyLabel,
}

#[derive(Debug, Clone, Copy)]
struct LineSpan {
    start: usize,
    end: usize,
}

pub(crate) fn note_blocks(text: &str) -> Vec<NoteBlock> {
    let lines = line_spans(text);
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let raw = line_text(text, line);
        let trimmed = raw.trim_start();
        let indent = raw.len() - trimmed.len();
        if trimmed.len() < 5 || !trimmed[..5].eq_ignore_ascii_case("NOTE:") {
            index += 1;
            continue;
        }

        let label_start = line.start + indent;
        let after_label = label_start + 5;
        let content_ws = text[after_label..line.start + raw.len()].len()
            - text[after_label..line.start + raw.len()].trim_start().len();
        let mut end = line.end;
        let mut continuation = index + 1;
        while continuation < lines.len() {
            let next_line = lines[continuation];
            let next_raw = line_text(text, next_line);
            if next_raw.trim().is_empty() {
                break;
            }
            let next_indent = next_raw.len() - next_raw.trim_start().len();
            if next_indent <= indent {
                break;
            }
            end = next_line.end;
            continuation += 1;
        }

        blocks.push(NoteBlock {
            start: line.start,
            end,
            content_start: after_label + content_ws,
        });
        index = continuation.max(index + 1);
    }
    blocks
}

pub(crate) fn safety_blocks(text: &str) -> Vec<SafetyBlock> {
    let mut blocks = Vec::new();
    for line in line_spans(text) {
        let raw = line_text(text, line);
        let trimmed = raw.trim_start();
        let leading = raw.len() - trimmed.len();
        let Some((label_len, label)) = safety_label(trimmed) else {
            continue;
        };
        let after_label = &trimmed[label_len..];
        let content_leading = after_label.len() - after_label.trim_start().len();
        let content_start = line.start + leading + label_len + content_leading;
        let content_end = line.start + raw.len();
        blocks.push(SafetyBlock {
            start: line.start + leading,
            end: content_end,
            content_start,
            content_end,
            label,
        });
    }
    blocks
}

pub(crate) fn starts_condition(text: &str) -> bool {
    let first = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character: char| character.is_ascii_punctuation());
    matches!(
        first.to_ascii_lowercase().as_str(),
        "after" | "before" | "if" | "when" | "while"
    )
}

pub(crate) fn simple_list_blocks(text: &str) -> Vec<SimpleListBlock> {
    let source = SourceDocument::new(text);
    let mut semantic = Vec::<(usize, usize, usize, SimpleListBlock)>::new();

    for list in source.lists() {
        let mut items = source
            .list_items()
            .iter()
            .filter(|item| item.list_id == list.id)
            .map(|item| projected_list_item(text, &source, *item))
            .collect::<Vec<_>>();
        items.sort_by_key(|item| (item.line_start, item.line_end));
        if items.is_empty() {
            continue;
        }

        let introduced_by_colon = terminal_colon_before(text, list.span.start).is_some();
        if let Some((depth, _, previous_end, previous)) = semantic.last_mut()
            && *depth == list.depth
            && adjacent_list_gap(&text[*previous_end..list.span.start])
        {
            previous.items.extend(items);
            *previous_end = list.span.end;
            continue;
        }

        semantic.push((
            list.depth,
            list.span.start,
            list.span.end,
            SimpleListBlock {
                introduced_by_colon,
                items,
            },
        ));
    }

    semantic
        .into_iter()
        .map(|(_, _, _, block)| block)
        .collect()
}

pub(crate) fn overlaps_note(start: usize, end: usize, notes: &[NoteBlock]) -> bool {
    notes
        .iter()
        .any(|note| start < note.end && note.start < end)
}

fn projected_list_item(
    text: &str,
    source: &SourceDocument,
    item: SourceListItem,
) -> ListItem {
    let child_start = source
        .lists()
        .iter()
        .filter(|list| list.depth == item.depth + 1)
        .filter(|list| list.span.start >= item.content_start && list.span.end <= item.span.end)
        .map(|list| list.span.start)
        .min();
    let raw_end = child_start.unwrap_or(item.content_end);
    let content_end = trim_trailing_whitespace(text, item.content_start, raw_end);
    ListItem {
        line_start: item.span.start,
        line_end: item.span.end,
        content_start: item.content_start,
        content_end: content_end.max(item.content_start),
    }
}

fn terminal_colon_before(text: &str, boundary: usize) -> Option<usize> {
    let prefix = &text[..boundary];
    let trimmed = prefix.trim_end();
    trimmed.ends_with(':').then(|| trimmed.len() - 1)
}

fn adjacent_list_gap(gap: &str) -> bool {
    gap.trim().is_empty() && !gap.contains("\n\n") && !gap.contains("\r\n\r\n")
}

fn trim_trailing_whitespace(text: &str, start: usize, end: usize) -> usize {
    let mut end = end.min(text.len());
    while end > start {
        let Some(character) = text[start..end].chars().next_back() else {
            break;
        };
        if !character.is_whitespace() {
            break;
        }
        end -= character.len_utf8();
    }
    end
}

fn safety_label(text: &str) -> Option<(usize, SafetyLabel)> {
    for (name, label) in [
        ("WARNING", SafetyLabel::Warning),
        ("CAUTION", SafetyLabel::Caution),
    ] {
        if text
            .get(..name.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
            && text[name.len()..].trim_start().starts_with(':')
        {
            let colon = text[name.len()..].find(':')? + name.len();
            return Some((colon + 1, label));
        }
    }
    None
}

fn line_text(text: &str, line: LineSpan) -> &str {
    text[line.start..line.end].trim_end_matches(['\r', '\n'])
}

fn line_spans(text: &str) -> Vec<LineSpan> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if character == '\n' {
            lines.push(LineSpan {
                start,
                end: index + 1,
            });
            start = index + 1;
        }
    }
    if start < text.len() || text.is_empty() {
        lines.push(LineSpan {
            start,
            end: text.len(),
        });
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_block_excludes_label_from_content() {
        let text = "REMOVE THAT.\nNOTE: REMOVE THIS.\n  CONTINUATION.\nREMOVE THAT.";
        let notes = note_blocks(text);
        assert_eq!(notes.len(), 1);
        assert_eq!(
            &text[notes[0].content_start..notes[0].end],
            "REMOVE THIS.\n  CONTINUATION.\n"
        );
    }

    #[test]
    fn commonmark_list_items_use_parser_offsets() {
        let text = "DO THIS:\n- Remove this.\n2. Remove that.";
        let blocks = simple_list_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].introduced_by_colon);
        assert_eq!(blocks[0].items.len(), 2);
        assert_eq!(
            &text[blocks[0].items[0].content_start..blocks[0].items[0].content_end],
            "Remove this."
        );
    }

    #[test]
    fn parenthesized_work_step_labels_are_recognized_as_list_items() {
        let blocks = simple_list_blocks("DO THIS:\n(a) Remove this.\n(b) Remove that.");
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].introduced_by_colon);
        assert_eq!(blocks[0].items.len(), 2);
    }

    #[test]
    fn wrapped_item_content_is_not_cut_at_the_first_source_line() {
        let text = "DO THIS:\n- Remove the unit and\n  put it on the bench.";
        let blocks = simple_list_blocks(text);
        assert_eq!(blocks.len(), 1);
        let item = blocks[0].items[0];
        assert!(text[item.content_start..item.content_end].contains("bench."));
    }

    #[test]
    fn nested_list_is_a_separate_semantic_block() {
        let blocks = simple_list_blocks("DO THIS:\n- Remove this:\n  - Remove that.\n- Continue.");
        assert_eq!(blocks.len(), 2);
        assert!(blocks.iter().all(|block| block.introduced_by_colon));
    }
}
