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
        let text = "REMOVE THAT.\nNOTE: REMOVE THIS.\n  CONTINUATION.\nREMOVE THAT.";
        let notes = note_blocks(text);
        assert_eq!(notes.len(), 1);
        assert_eq!(
            &text[notes[0].content_start..notes[0].end],
            "REMOVE THIS.\n  CONTINUATION.\n"
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
