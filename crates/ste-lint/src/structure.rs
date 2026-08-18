#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CountUnit {
    pub start: usize,
    pub end: usize,
    pub word_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct LineSpan {
    start: usize,
    end: usize,
}

use crate::analysis::source::SourceDocument;

pub(crate) fn word_limit_units(text: &str) -> Vec<CountUnit> {
    let lines = line_spans(text);
    let source = SourceDocument::new(text);
    let mut units = Vec::new();
    let mut segment_start = 0;
    let mut index = 0;

    while index + 1 < lines.len() {
        let line = lines[index];
        let line_text = text[line.start..line.end].trim_end_matches(['\r', '\n']);
        let next_line = lines[index + 1];

        if line_text.trim_end().ends_with(':')
            && list_item_content_span(text, &source, next_line).is_some()
        {
            let colon = line.start + line_text.rfind(':').unwrap();
            push_regular_units(text, segment_start, colon + 1, true, &mut units);

            index += 1;
            while index < lines.len() {
                let item_line = lines[index];
                let Some((content_start, content_end)) =
                    list_item_content_span(text, &source, item_line)
                else {
                    break;
                };
                if content_start < content_end {
                    push_span_and_parentheticals(text, content_start, content_end, &mut units);
                }
                segment_start = item_line.end;
                index += 1;
            }
            continue;
        }

        index += 1;
    }

    if segment_start < text.len() {
        push_regular_units(text, segment_start, text.len(), false, &mut units);
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
    let lines = line_spans(paragraph);
    let source = SourceDocument::new(paragraph);
    let mut prose = String::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let raw = paragraph[line.start..line.end].trim_end_matches(['\r', '\n']);
        if list_item_content_span(paragraph, &source, line).is_some() {
            index += 1;
            continue;
        }

        let introduces_list = raw.trim_end().ends_with(':')
            && lines
                .get(index + 1)
                .is_some_and(|next| list_item_content_span(paragraph, &source, *next).is_some());
        if introduces_list {
            prose.push_str(raw.trim_end_matches(':'));
            prose.push('.');
        } else {
            prose.push_str(raw);
        }
        prose.push('\n');
        index += 1;
    }

    sentence_spans(&prose, 0, prose.len(), false).len()
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

fn list_item_content_span(
    text: &str,
    source: &SourceDocument,
    line: LineSpan,
) -> Option<(usize, usize)> {
    let line_text_end = line.start
        + text[line.start..line.end]
            .trim_end_matches(['\r', '\n'])
            .len();
    if let Some(item) = source
        .list_items()
        .iter()
        .find(|item| item.span.start >= line.start && item.span.start < line.end)
    {
        return Some((item.content_start, item.content_end.min(line_text_end)));
    }

    let raw = &text[line.start..line_text_end];
    legacy_ste_list_item_content_offset(raw).map(|offset| (line.start + offset, line_text_end))
}

fn legacy_ste_list_item_content_offset(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();

    if trimmed.starts_with("• ") {
        return Some(leading + "• ".len());
    }

    let bytes = trimmed.as_bytes();
    if bytes.first() == Some(&b'(') {
        let mut index = 1;
        let label_start = index;
        while index < bytes.len() && bytes[index].is_ascii_alphanumeric() {
            index += 1;
        }
        if index > label_start
            && index + 1 < bytes.len()
            && bytes[index] == b')'
            && bytes[index + 1].is_ascii_whitespace()
        {
            return Some(leading + index + 2);
        }
        return None;
    }

    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
        index += 1;
    }
    let label = &trimmed[..index];
    if label.len() == 1
        && index + 1 < bytes.len()
        && matches!(bytes[index], b'.' | b')')
        && bytes[index + 1].is_ascii_whitespace()
    {
        return Some(leading + index + 2);
    }
    None
}

fn trim_start(text: &str, start: usize, end: usize) -> usize {
    let leading = text[start..end].len() - text[start..end].trim_start().len();
    start + leading
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
    fn prose_word_followed_by_period_is_not_a_list_label() {
        assert_eq!(legacy_ste_list_item_content_offset("USE. USE THIS."), None);
        assert_eq!(legacy_ste_list_item_content_offset("A) USE THIS."), Some(3));
        assert_eq!(
            legacy_ste_list_item_content_offset("(a) USE THIS."),
            Some(4)
        );
        assert_eq!(legacy_ste_list_item_content_offset("• USE THIS."), Some(4));
    }
}
