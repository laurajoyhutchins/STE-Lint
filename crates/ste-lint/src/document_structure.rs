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

#[derive(Debug, Clone, Copy)]
struct LineSpan {
    start: usize,
    end: usize,
}

pub(crate) fn note_blocks(text: &str) -> Vec<NoteBlock> {
    let mut blocks = Vec::new();
    for (start, end) in paragraph_ranges(text) {
        let paragraph = &text[start..end];
        let leading = paragraph.len() - paragraph.trim_start().len();
        let label_start = start + leading;
        let remaining = &text[label_start..end];
        if remaining.len() < 5 || !remaining[..5].eq_ignore_ascii_case("NOTE:") {
            continue;
        }
        let after_label = label_start + 5;
        let content_ws = text[after_label..end].len() - text[after_label..end].trim_start().len();
        blocks.push(NoteBlock {
            start,
            end,
            content_start: after_label + content_ws,
        });
    }
    blocks
}

pub(crate) fn simple_list_blocks(text: &str) -> Vec<SimpleListBlock> {
    let lines = line_spans(text);
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let raw = line_text(text, line);
        let Some((indent, _)) = list_item_layout(raw) else {
            index += 1;
            continue;
        };

        let previous_nonblank = (0..index)
            .rev()
            .find(|candidate| !line_text(text, lines[*candidate]).trim().is_empty());
        let introduced_by_colon = previous_nonblank
            .map(|candidate| line_text(text, lines[candidate]).trim_end().ends_with(':'))
            .unwrap_or(false);

        let mut items = Vec::new();
        while index < lines.len() {
            let item_line = lines[index];
            let item_raw = line_text(text, item_line);
            let Some((item_indent, item_content_offset)) = list_item_layout(item_raw) else {
                break;
            };
            if item_indent != indent {
                break;
            }
            let content_start = item_line.start + item_content_offset;
            let content_end = item_line.start + item_raw.len();
            items.push(ListItem {
                line_start: item_line.start,
                line_end: item_line.start + item_raw.len(),
                content_start,
                content_end,
            });
            index += 1;
        }

        if !items.is_empty() {
            blocks.push(SimpleListBlock {
                introduced_by_colon,
                items,
            });
        }
    }

    blocks
}

pub(crate) fn overlaps_note(start: usize, end: usize, notes: &[NoteBlock]) -> bool {
    notes
        .iter()
        .any(|note| start < note.end && note.start < end)
}

fn paragraph_ranges(text: &str) -> Vec<(usize, usize)> {
    let lines = line_spans(text);
    let mut ranges = Vec::new();
    let mut paragraph_start = None;
    let mut paragraph_end = 0;

    for line in lines {
        let raw = line_text(text, line);
        if raw.trim().is_empty() {
            if let Some(start) = paragraph_start.take() {
                ranges.push((start, paragraph_end));
            }
            continue;
        }
        paragraph_start.get_or_insert(line.start);
        paragraph_end = line.end;
    }
    if let Some(start) = paragraph_start {
        ranges.push((start, paragraph_end));
    }
    ranges
}

fn list_item_layout(line: &str) -> Option<(usize, usize)> {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    for marker in ["- ", "* ", "• "] {
        if trimmed.starts_with(marker) {
            return Some((leading, leading + marker.len()));
        }
    }

    let bytes = trimmed.as_bytes();
    let mut index = 0;
    if bytes.first() == Some(&b'(') {
        index = 1;
        let label_start = index;
        while index < bytes.len() && bytes[index].is_ascii_alphanumeric() {
            index += 1;
        }
        if index > label_start
            && index + 1 < bytes.len()
            && bytes[index] == b')'
            && bytes[index + 1].is_ascii_whitespace()
        {
            return Some((leading, leading + index + 2));
        }
        return None;
    }

    while index < bytes.len() && bytes[index].is_ascii_alphanumeric() {
        index += 1;
    }
    let label = &trimmed[..index];
    let valid_label = label.chars().all(|character| character.is_ascii_digit())
        || (label.len() == 1 && label.as_bytes()[0].is_ascii_alphabetic());
    if valid_label
        && index > 0
        && index + 1 < bytes.len()
        && matches!(bytes[index], b'.' | b')')
        && bytes[index + 1].is_ascii_whitespace()
    {
        return Some((leading, leading + index + 2));
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
        let text = "NOTE: REMOVE THIS.\nCONTINUATION.\n\nREMOVE THAT.";
        let notes = note_blocks(text);
        assert_eq!(notes.len(), 1);
        assert_eq!(
            &text[notes[0].content_start..notes[0].end],
            "REMOVE THIS.\nCONTINUATION.\n"
        );
    }

    #[test]
    fn parenthesized_work_step_labels_are_recognized_as_list_items() {
        let blocks = simple_list_blocks("DO THIS:\n(a) Remove this.\n(b) Remove that.");
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].introduced_by_colon);
        assert_eq!(blocks[0].items.len(), 2);
    }
}
