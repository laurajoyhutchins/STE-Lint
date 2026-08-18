use crate::analysis::source::{SourceDocument, SourceList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CountUnit {
    pub start: usize,
    pub end: usize,
}

pub(crate) fn word_limit_units(text: &str) -> Vec<CountUnit> {
    let source = SourceDocument::new(text);
    let mut roots = source
        .lists()
        .iter()
        .filter(|list| list.depth == 0)
        .copied()
        .collect::<Vec<_>>();
    roots.sort_by_key(|list| (list.span.start, list.span.end));

    let mut units = Vec::new();
    let mut cursor = 0usize;
    for list in roots {
        if list.span.start < cursor {
            continue;
        }
        if let Some(colon) = terminal_colon_before(text, cursor, list.span.start) {
            push_regular_units(text, cursor, colon + 1, true, &mut units);
        } else {
            push_regular_units(text, cursor, list.span.start, false, &mut units);
        }
        push_list_units(text, &source, list, &mut units);
        cursor = list.span.end;
    }
    if cursor < text.len() {
        push_regular_units(text, cursor, text.len(), false, &mut units);
    }
    units.sort_by_key(|unit| (unit.start, unit.end));
    units
}

pub(crate) fn paragraph_ranges(text: &str) -> Vec<(usize, usize)> {
    let source = SourceDocument::new(text);
    source
        .paragraph_ranges()
        .iter()
        .filter(|paragraph| {
            !source
                .list_items()
                .iter()
                .any(|item| paragraph.start >= item.span.start && paragraph.end <= item.span.end)
        })
        .map(|span| (span.start, span.end))
        .collect()
}

pub(crate) fn paragraph_prose_sentence_count(paragraph: &str) -> usize {
    let source = SourceDocument::new(paragraph);
    let mut bytes = paragraph.as_bytes().to_vec();
    for item in source.list_items() {
        blank_range_preserving_lines(&mut bytes, item.span.start, item.span.end);
    }
    let projection = String::from_utf8(bytes)
        .expect("list projection replaces complete source bytes with ASCII whitespace");
    sentence_spans(&projection, 0, projection.len(), false).len()
}

fn push_list_units(
    text: &str,
    source: &SourceDocument,
    list: SourceList,
    units: &mut Vec<CountUnit>,
) {
    let mut items = source
        .list_items()
        .iter()
        .filter(|item| item.list_id == list.id)
        .copied()
        .collect::<Vec<_>>();
    items.sort_by_key(|item| (item.span.start, item.span.end));

    for item in items {
        let mut children = source
            .lists()
            .iter()
            .filter(|child| child.depth == item.depth + 1)
            .filter(|child| {
                child.span.start >= item.content_start && child.span.end <= item.span.end
            })
            .copied()
            .collect::<Vec<_>>();
        children.sort_by_key(|child| (child.span.start, child.span.end));

        let mut cursor = item.content_start;
        for child in children {
            if let Some(colon) = terminal_colon_before(text, cursor, child.span.start) {
                push_regular_units(text, cursor, colon + 1, true, units);
            } else {
                push_regular_units(text, cursor, child.span.start, false, units);
            }
            push_list_units(text, source, child, units);
            cursor = child.span.end;
        }
        if cursor < item.content_end {
            push_regular_units(text, cursor, item.content_end, false, units);
        }
    }
}

fn terminal_colon_before(text: &str, start: usize, boundary: usize) -> Option<usize> {
    if start >= boundary || boundary > text.len() {
        return None;
    }
    let range = &text[start..boundary];
    let trimmed = range.trim_end();
    trimmed.ends_with(':').then(|| start + trimmed.len() - 1)
}

fn blank_range_preserving_lines(bytes: &mut [u8], start: usize, end: usize) {
    for byte in bytes.iter_mut().take(end).skip(start) {
        if !matches!(*byte, b'\r' | b'\n') {
            *byte = b' ';
        }
    }
}

fn push_regular_units(
    text: &str,
    start: usize,
    end: usize,
    colon_terminal: bool,
    units: &mut Vec<CountUnit>,
) {
    for (sentence_start, sentence_end) in sentence_spans(text, start, end, colon_terminal) {
        push_span_and_parentheticals(text, sentence_start, sentence_end, units);
    }
}

fn push_span_and_parentheticals(text: &str, start: usize, end: usize, units: &mut Vec<CountUnit>) {
    if text[start..end]
        .chars()
        .any(|character| character.is_alphanumeric())
    {
        units.push(CountUnit { start, end });
    }
    for (inner_start, inner_end) in top_level_parenthetical_spans(text, start, end) {
        for (sentence_start, sentence_end) in sentence_spans(text, inner_start, inner_end, false) {
            push_span_and_parentheticals(text, sentence_start, sentence_end, units);
        }
    }
}

fn sentence_spans(
    text: &str,
    start: usize,
    end: usize,
    colon_terminal: bool,
) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut sentence_start = start;
    let mut paren_depth = 0usize;
    let mut quote_end: Option<char> = None;
    let source = SourceDocument::new(text);
    let protected_spans = source
        .protected_ranges()
        .iter()
        .copied()
        .filter(|span| span.intersects(start, end))
        .collect::<Vec<_>>();
    let mut protected_index = 0usize;

    for (relative, character) in text[start..end].char_indices() {
        let absolute = start + relative;
        if let Some(expected) = quote_end {
            if character == expected {
                quote_end = None;
            }
            continue;
        }
        while protected_index < protected_spans.len()
            && absolute >= protected_spans[protected_index].end
        {
            protected_index += 1;
        }
        if protected_spans
            .get(protected_index)
            .is_some_and(|span| absolute >= span.start && absolute < span.end)
        {
            continue;
        }
        match character {
            '"' => {
                quote_end = Some('"');
                continue;
            }
            '“' => {
                quote_end = Some('”');
                continue;
            }
            '(' => {
                paren_depth += 1;
                continue;
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                continue;
            }
            _ => {}
        }
        if paren_depth > 0 {
            continue;
        }
        let boundary = match character {
            '?' | '!' => true,
            '.' => is_sentence_period(text, absolute, end),
            ':' if colon_terminal && absolute + 1 == end => true,
            _ => false,
        };
        if boundary {
            let sentence_end = absolute + character.len_utf8();
            if !text[sentence_start..sentence_end].trim().is_empty() {
                spans.push((trim_start(text, sentence_start, sentence_end), sentence_end));
            }
            sentence_start = sentence_end;
        }
    }
    if sentence_start < end && !text[sentence_start..end].trim().is_empty() {
        spans.push((trim_start(text, sentence_start, end), end));
    }
    spans
}

fn is_sentence_period(text: &str, period: usize, range_end: usize) -> bool {
    let before = text[..period].chars().next_back();
    let after = text[period + 1..range_end].chars().next();
    if before.is_some_and(|c| c.is_ascii_digit()) && after.is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    if after.is_some_and(char::is_alphabetic) {
        return false;
    }
    let next_nonspace = text[period + 1..range_end]
        .chars()
        .find(|character| !character.is_whitespace());
    let token_start = text[..period]
        .rfind(|character: char| character.is_whitespace())
        .map_or(0, |index| index + 1);
    let token =
        text[token_start..period].trim_matches(|character: char| character.is_ascii_punctuation());
    if token.eq_ignore_ascii_case("no") && next_nonspace.is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    if token.len() <= 3 && next_nonspace.is_some_and(char::is_lowercase) {
        return false;
    }
    true
}

fn top_level_parenthetical_spans(text: &str, start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut depth = 0usize;
    let mut inner_start = 0usize;
    for (relative, character) in text[start..end].char_indices() {
        let absolute = start + relative;
        match character {
            '(' => {
                if depth == 0 {
                    inner_start = absolute + 1;
                }
                depth += 1;
            }
            ')' if depth > 0 => {
                depth -= 1;
                if depth == 0 && inner_start < absolute {
                    spans.push((inner_start, absolute));
                }
            }
            _ => {}
        }
    }
    spans
}

fn trim_start(text: &str, start: usize, end: usize) -> usize {
    let leading = text[start..end].len() - text[start..end].trim_start().len();
    start + leading
}
