use crate::analysis::source::{SourceDocument, SourceList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CountUnit {
    pub start: usize,
    pub end: usize,
    pub word_count: usize,
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
    let word_count = count_issue9_words(&text[start..end]);
    if word_count > 0 {
        units.push(CountUnit {
            start,
            end,
            word_count,
        });
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

fn count_issue9_words(text: &str) -> usize {
    let collapsed = collapse_groups(text);
    let tokens = collapsed
        .split_whitespace()
        .map(clean_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let mut count = 0;
    let mut index = 0;

    while index < tokens.len() {
        let token = tokens[index];
        if token.eq_ignore_ascii_case("no")
            && tokens
                .get(index + 1)
                .is_some_and(|next| is_identifier(next))
        {
            count += 1;
            index += 2;
            continue;
        }

        if is_numeric(token)
            && let Some(next) = tokens.get(index + 1)
        {
            if is_clock_abbreviation(next) || is_unit(next) {
                count += 1;
                index += 2;
                continue;
            }
            if is_degree_word(next)
                && tokens
                    .get(index + 2)
                    .is_some_and(|third| is_temperature_scale(third))
            {
                count += 1;
                index += 3;
                continue;
            }
        }

        count += 1;
        index += 1;
    }
    count
}

fn collapse_groups(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '(' {
            output.push_str(" __STE_GROUP__ ");
            let mut depth = 1usize;
            for next in chars.by_ref() {
                match next {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }

        if character == '"' || character == '“' {
            output.push_str(" __STE_GROUP__ ");
            let closing = if character == '“' { '”' } else { '"' };
            for next in chars.by_ref() {
                if next == closing {
                    break;
                }
            }
            continue;
        }

        output.push(character);
    }
    output
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

fn clean_token(token: &str) -> &str {
    token
        .trim_matches(|character: char| {
            character.is_ascii_punctuation() && !matches!(character, '-' | '.')
        })
        .trim_end_matches(['.', ',', ':', ';', '?', '!'])
}

fn is_numeric(token: &str) -> bool {
    token
        .chars()
        .all(|character| character.is_ascii_digit() || matches!(character, '.' | ','))
        && token.chars().any(|character| character.is_ascii_digit())
}

fn is_identifier(token: &str) -> bool {
    token.chars().any(|character| character.is_ascii_digit())
}

fn is_clock_abbreviation(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "a.m" | "p.m" | "am" | "pm"
    )
}

fn is_degree_word(token: &str) -> bool {
    matches!(token.to_ascii_lowercase().as_str(), "degree" | "degrees")
}

fn is_temperature_scale(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "celsius" | "fahrenheit"
    )
}

fn is_unit(token: &str) -> bool {
    if matches!(
        token,
        "A" | "mA"
            | "V"
            | "mV"
            | "kV"
            | "W"
            | "kW"
            | "N"
            | "Nm"
            | "Hz"
            | "kHz"
            | "MHz"
            | "°C"
            | "°F"
            | "Ω"
            | "Ω"
    ) {
        return true;
    }

    const LOWER_UNITS: &[&str] = &[
        "mm",
        "cm",
        "m",
        "km",
        "in",
        "ft",
        "g",
        "kg",
        "mg",
        "l",
        "ml",
        "pa",
        "kpa",
        "mpa",
        "psi",
        "bar",
        "s",
        "ms",
        "min",
        "h",
        "hr",
        "ohm",
        "ohms",
        "kilogram",
        "kilograms",
        "gram",
        "grams",
        "meter",
        "meters",
        "metre",
        "metres",
        "millimeter",
        "millimeters",
        "millimetre",
        "millimetres",
        "centimeter",
        "centimeters",
        "centimetre",
        "centimetres",
        "inch",
        "inches",
        "foot",
        "feet",
        "second",
        "seconds",
        "minute",
        "minutes",
        "hour",
        "hours",
        "volt",
        "volts",
        "ampere",
        "amperes",
        "watt",
        "watts",
        "newton",
        "newtons",
    ];
    let lower = token.to_ascii_lowercase();
    LOWER_UNITS.contains(&lower.as_str())
}

fn trim_start(text: &str, start: usize, end: usize) -> usize {
    let leading = text[start..end].len() - text[start..end].trim_start().len();
    start + leading
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_and_unit_count_as_one_word() {
        assert_eq!(count_issue9_words("USE 10 kg"), 2);
        assert_eq!(count_issue9_words("USE 10 degrees Celsius"), 2);
        assert_eq!(count_issue9_words("USE 10 °C"), 2);
    }

    #[test]
    fn quoted_text_and_identifier_count_as_one_word() {
        assert_eq!(count_issue9_words("USE the \"Service Overview\" page"), 4);
        assert_eq!(count_issue9_words("EXAMINE No. 1 bearing"), 3);
    }

    #[test]
    fn time_abbreviation_counts_with_number() {
        assert_eq!(count_issue9_words("TEST at 10 a.m."), 3);
    }

    #[test]
    fn commonmark_list_item_paragraphs_are_not_prose_paragraphs() {
        let text = "INTRODUCTION.\n\nDO THIS:\n- Remove this.\n- Remove that.";
        let ranges = paragraph_ranges(text);
        assert_eq!(ranges.len(), 2);
        assert_eq!(&text[ranges[0].0..ranges[0].1], "INTRODUCTION.");
        assert_eq!(&text[ranges[1].0..ranges[1].1], "DO THIS:");
    }

    #[test]
    fn wrapped_list_item_is_one_count_unit() {
        let text = "USE:\n- ONE TWO THREE\n  FOUR FIVE.";
        let counts = word_limit_units(text)
            .into_iter()
            .map(|unit| unit.word_count)
            .collect::<Vec<_>>();
        assert_eq!(counts, vec![1, 5]);
    }

    #[test]
    fn nested_colon_list_boundaries_create_independent_units() {
        let text = "USE:\n- ONE TWO:\n  - THREE FOUR.\n- FIVE SIX.";
        let counts = word_limit_units(text)
            .into_iter()
            .map(|unit| unit.word_count)
            .collect::<Vec<_>>();
        assert_eq!(counts, vec![1, 2, 2, 2]);
    }

    #[test]
    fn legacy_ste_list_markers_use_the_same_counting_path() {
        let text = "USE:\n(a) ONE TWO.\n(b) THREE FOUR.";
        let counts = word_limit_units(text)
            .into_iter()
            .map(|unit| unit.word_count)
            .collect::<Vec<_>>();
        assert_eq!(counts, vec![1, 2, 2]);
    }
}
