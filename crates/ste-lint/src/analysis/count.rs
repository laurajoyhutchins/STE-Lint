use std::sync::OnceLock;

use serde::Deserialize;
use ste_glossary::AliasKind;

use crate::{AnalysisDocument, CountGroupKind, NamedEntityClass, TextAuthorityKind};

use super::source::SourceDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountGroup {
    pub kind: CountGroupKind,
    pub start: usize,
    pub end: usize,
    pub source: String,
}

impl CountGroup {
    fn contains(&self, start: usize, end: usize) -> bool {
        self.start <= start && end <= self.end
    }
}

#[derive(Debug, Clone)]
pub struct CountGroupProjection<'a> {
    text: &'a str,
    groups: Vec<CountGroup>,
}

impl<'a> CountGroupProjection<'a> {
    pub fn from_analysis(analysis: &'a AnalysisDocument<'a>) -> Self {
        let text = analysis.text();
        let source_document = SourceDocument::with_context(text, analysis.context());
        let mut groups = Vec::new();

        for heading in source_document.heading_ranges() {
            groups.push(CountGroup {
                kind: CountGroupKind::Heading,
                start: heading.start,
                end: heading.end,
                source: "document-native heading structure".into(),
            });
        }

        if let Some(context) = analysis.context() {
            for occurrence in &context.occurrences {
                if let Some(kind) = occurrence.count_group {
                    groups.push(CountGroup {
                        kind,
                        start: occurrence.start,
                        end: occurrence.end,
                        source: occurrence.source.clone(),
                    });
                }
                if let Some(authority) = occurrence.text_authority {
                    let kind = match authority {
                        TextAuthorityKind::QuotedExternalText | TextAuthorityKind::Formula => {
                            Some(CountGroupKind::QuotedText)
                        }
                        TextAuthorityKind::Title => Some(CountGroupKind::Title),
                        TextAuthorityKind::Placard => Some(CountGroupKind::Placard),
                        TextAuthorityKind::Label => Some(CountGroupKind::Label),
                        TextAuthorityKind::DocumentNumbering => {
                            Some(CountGroupKind::DocumentNumberingExcluded)
                        }
                        TextAuthorityKind::ProtectedText | TextAuthorityKind::CodeOrVerbatim => {
                            None
                        }
                    };
                    if let Some(kind) = kind {
                        groups.push(CountGroup {
                            kind,
                            start: occurrence.start,
                            end: occurrence.end,
                            source: occurrence.source.clone(),
                        });
                    }
                }
            }
        }

        for mention in analysis.entity_mentions() {
            if let Some(class) = mention.named_entity_class {
                groups.push(CountGroup {
                    kind: proper_noun_kind(class),
                    start: mention.span.start,
                    end: mention.span.end,
                    source: mention.provenance.join("; "),
                });
            }
            if matches!(
                mention.alias_kind,
                Some(AliasKind::Abbreviation | AliasKind::Acronym)
            ) {
                groups.push(CountGroup {
                    kind: CountGroupKind::Abbreviation,
                    start: mention.span.start,
                    end: mention.span.end,
                    source: mention.provenance.join("; "),
                });
            }
        }

        groups.extend(quoted_groups(text));
        groups.extend(parenthetical_groups(text));
        groups.extend(hyphenated_groups(text));
        groups.extend(mechanical_groups(text, analysis));

        Self {
            text,
            groups: canonicalize(groups),
        }
    }

    pub fn groups(&self) -> &[CountGroup] {
        &self.groups
    }

    pub fn count_range(&self, start: usize, end: usize) -> usize {
        if start >= end || end > self.text.len() {
            return 0;
        }

        if self.groups.iter().any(|group| {
            group.contains(start, end)
                && matches!(
                    group.kind,
                    CountGroupKind::QuotedText
                        | CountGroupKind::Title
                        | CountGroupKind::Heading
                        | CountGroupKind::Placard
                        | CountGroupKind::Label
                        | CountGroupKind::ProperNoun
                        | CountGroupKind::ProperNounPerson
                        | CountGroupKind::ProperNounGroup
                        | CountGroupKind::ProperNounOrganization
                        | CountGroupKind::ProperNounGeopoliticalEntity
                )
        }) {
            return 1;
        }

        let mut selected = self
            .groups
            .iter()
            .filter(|group| group.start >= start && group.end <= end)
            .collect::<Vec<_>>();
        selected.sort_by(|left, right| {
            left.start
                .cmp(&right.start)
                .then_with(|| right.end.cmp(&left.end))
                .then_with(|| count_group_rank(left.kind).cmp(&count_group_rank(right.kind)))
        });

        let mut count = 0usize;
        let mut cursor = start;
        for group in selected {
            if group.start < cursor {
                continue;
            }
            count += plain_word_count(&self.text[cursor..group.start]);
            count += group_value(group.kind);
            cursor = group.end;
        }
        count + plain_word_count(&self.text[cursor..end])
    }
}

fn proper_noun_kind(class: NamedEntityClass) -> CountGroupKind {
    match class {
        NamedEntityClass::Person => CountGroupKind::ProperNounPerson,
        NamedEntityClass::Group => CountGroupKind::ProperNounGroup,
        NamedEntityClass::Organization => CountGroupKind::ProperNounOrganization,
        NamedEntityClass::GeopoliticalEntity => CountGroupKind::ProperNounGeopoliticalEntity,
    }
}

fn group_value(kind: CountGroupKind) -> usize {
    usize::from(kind != CountGroupKind::DocumentNumberingExcluded)
}

fn canonicalize(mut groups: Vec<CountGroup>) -> Vec<CountGroup> {
    groups.retain(|group| group.start < group.end);
    groups.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
            .then_with(|| count_group_rank(left.kind).cmp(&count_group_rank(right.kind)))
    });
    groups.dedup_by(|right, left| right.start == left.start && right.end == left.end);
    groups
}

fn count_group_rank(kind: CountGroupKind) -> u8 {
    match kind {
        CountGroupKind::DocumentNumberingExcluded => 0,
        CountGroupKind::Heading
        | CountGroupKind::Title
        | CountGroupKind::Placard
        | CountGroupKind::Label => 1,
        CountGroupKind::QuotedText | CountGroupKind::Parenthetical => 2,
        CountGroupKind::ProperNoun
        | CountGroupKind::ProperNounPerson
        | CountGroupKind::ProperNounGroup
        | CountGroupKind::ProperNounOrganization
        | CountGroupKind::ProperNounGeopoliticalEntity => 3,
        CountGroupKind::NumberWithUnit | CountGroupKind::Abbreviation => 4,
        CountGroupKind::AlphanumericIdentifier | CountGroupKind::Number => 5,
        CountGroupKind::HyphenatedWord => 6,
    }
}

fn quoted_groups(text: &str) -> Vec<CountGroup> {
    let mut groups = Vec::new();
    let mut quote: Option<(usize, char)> = None;

    for (start, character) in text.char_indices() {
        match quote {
            Some((open, expected)) if character == expected => {
                groups.push(CountGroup {
                    kind: CountGroupKind::QuotedText,
                    start: open,
                    end: start + character.len_utf8(),
                    source: "Issue 9 quotation-mark structure".into(),
                });
                quote = None;
            }
            Some(_) => {}
            None if character == '"' => quote = Some((start, '"')),
            None if character == '“' => quote = Some((start, '”')),
            None => {}
        }
    }
    groups
}

fn parenthetical_groups(text: &str) -> Vec<CountGroup> {
    let mut groups = Vec::new();
    let mut stack = Vec::new();
    for (start, character) in text.char_indices() {
        if character == '(' {
            stack.push(start);
        } else if character == ')'
            && let Some(open) = stack.pop()
        {
            groups.push(CountGroup {
                kind: CountGroupKind::Parenthetical,
                start: open,
                end: start + 1,
                source: "Issue 9 Rule 8.5 parenthetical grouping".into(),
            });
        }
    }
    groups
}

fn hyphenated_groups(text: &str) -> Vec<CountGroup> {
    lexemes(text)
        .into_iter()
        .filter(|lexeme| {
            lexeme.text.contains(['-', '‐', '‑']) && lexeme.text.chars().any(char::is_alphabetic)
        })
        .map(|lexeme| CountGroup {
            kind: CountGroupKind::HyphenatedWord,
            start: lexeme.start,
            end: lexeme.end,
            source: "Issue 9 Rule 8.7 hyphenated grouping".into(),
        })
        .collect()
}

fn mechanical_groups(text: &str, analysis: &AnalysisDocument<'_>) -> Vec<CountGroup> {
    let tokens = lexemes(text);
    let mut groups = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let token = &tokens[index];

        if token.text.eq_ignore_ascii_case("no.")
            && let Some(next) = tokens.get(index + 1)
            && is_number_expression(next.text)
        {
            groups.push(CountGroup {
                kind: CountGroupKind::AlphanumericIdentifier,
                start: token.start,
                end: next.end,
                source: "Issue 9 alphanumeric identifier syntax".into(),
            });
            index += 2;
            continue;
        }

        if is_number_expression(token.text) {
            if let Some(unit_end_index) = unit_expression_end(&tokens, index + 1, analysis) {
                groups.push(CountGroup {
                    kind: CountGroupKind::NumberWithUnit,
                    start: token.start,
                    end: tokens[unit_end_index].end,
                    source: "governed measurement-unit authority".into(),
                });
                index = unit_end_index + 1;
                continue;
            }
            if let Some(next) = tokens.get(index + 1)
                && is_clock_abbreviation(next.text)
            {
                groups.push(CountGroup {
                    kind: CountGroupKind::Abbreviation,
                    start: token.start,
                    end: next.end,
                    source: "Issue 9 abbreviation counting example class".into(),
                });
                index += 2;
                continue;
            }
            groups.push(CountGroup {
                kind: CountGroupKind::Number,
                start: token.start,
                end: token.end,
                source: "Issue 9 numeric syntax".into(),
            });
            index += 1;
            continue;
        }

        if is_alphanumeric_identifier(token.text) {
            groups.push(CountGroup {
                kind: CountGroupKind::AlphanumericIdentifier,
                start: token.start,
                end: token.end,
                source: "Issue 9 alphanumeric identifier syntax".into(),
            });
        } else if is_clock_abbreviation(token.text) {
            groups.push(CountGroup {
                kind: CountGroupKind::Abbreviation,
                start: token.start,
                end: token.end,
                source: "Issue 9 abbreviation counting example class".into(),
            });
        }
        index += 1;
    }

    groups
}

fn unit_expression_end(
    tokens: &[Lexeme<'_>],
    start: usize,
    analysis: &AnalysisDocument<'_>,
) -> Option<usize> {
    let first = tokens.get(start)?;
    let max_end = (start + 5).min(tokens.len());

    for end in (start..max_end).rev() {
        let phrase = tokens[start..=end]
            .iter()
            .map(|token| token.text)
            .collect::<Vec<_>>()
            .join(" ");
        if is_governed_unit_phrase(&phrase, analysis) {
            return Some(end);
        }
    }

    if !is_unit_expression_token(first.text) {
        return None;
    }

    let mut end = start;
    while let Some(next) = tokens.get(end + 1) {
        if end - start >= 3 || !is_unit_expression_token(next.text) {
            break;
        }
        end += 1;
    }
    Some(end)
}

fn is_governed_unit_phrase(phrase: &str, analysis: &AnalysisDocument<'_>) -> bool {
    if builtin_unit_phrase(phrase) {
        return true;
    }
    analysis.context().is_some_and(|context| {
        context.measurement_units.iter().any(|unit| {
            std::iter::once(&unit.canonical)
                .chain(&unit.forms)
                .any(|surface| surface == phrase)
        })
    })
}

fn builtin_unit_phrase(phrase: &str) -> bool {
    let registry = unit_registry();
    registry
        .multiword_names
        .iter()
        .any(|name| name.eq_ignore_ascii_case(phrase))
        || registry
            .names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(phrase))
        || registry.symbols.iter().any(|symbol| symbol == phrase)
        || is_prefixed_unit(phrase, registry)
        || is_compound_unit(phrase)
        || is_spaced_compound_unit(phrase)
}

fn is_unit_expression_token(token: &str) -> bool {
    builtin_unit_phrase(token) || is_compound_unit(token)
}

fn is_compound_unit(value: &str) -> bool {
    if !value.contains(['/', '·', '⋅', '*']) {
        return false;
    }
    let atoms = value
        .split(['/', '·', '⋅', '*'])
        .filter(|atom| !atom.is_empty())
        .collect::<Vec<_>>();
    atoms.len() >= 2
        && atoms
            .iter()
            .all(|atom| builtin_unit_atom(strip_unit_power(atom)))
}

fn is_spaced_compound_unit(value: &str) -> bool {
    let atoms = value.split_whitespace().collect::<Vec<_>>();
    atoms.len() >= 2
        && atoms
            .iter()
            .all(|atom| builtin_unit_atom(strip_unit_power(atom)))
}

fn builtin_unit_atom(value: &str) -> bool {
    let registry = unit_registry();
    registry.symbols.iter().any(|symbol| symbol == value)
        || registry
            .names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(value))
        || is_prefixed_unit(value, registry)
}

fn is_prefixed_unit(value: &str, registry: &UnitRegistry) -> bool {
    for prefix in &registry.prefix_symbols {
        if let Some(remainder) = value.strip_prefix(prefix.as_str())
            && !remainder.is_empty()
            && registry
                .prefixable_symbols
                .iter()
                .any(|symbol| symbol == remainder)
        {
            return true;
        }
    }

    let lower = value.to_lowercase();
    for prefix in &registry.prefix_names {
        if let Some(remainder) = lower.strip_prefix(&prefix.to_lowercase())
            && !remainder.is_empty()
            && registry
                .prefixable_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(remainder))
        {
            return true;
        }
    }
    false
}

fn strip_unit_power(value: &str) -> &str {
    value
        .trim_end_matches(['²', '³'])
        .split_once('^')
        .map_or(value.trim_end_matches(['²', '³']), |(base, _)| base)
}

fn is_number_expression(value: &str) -> bool {
    let value = value.trim();
    let value = value.strip_prefix(['+', '-', '−']).unwrap_or(value);
    if value.is_empty() {
        return false;
    }

    for separator in ['-', '–', '—'] {
        if let Some((left, right)) = value.split_once(separator)
            && !left.is_empty()
            && !right.is_empty()
        {
            return is_number_atom(left) && is_number_atom(right);
        }
    }
    if let Some((left, right)) = value.split_once('/') {
        return is_number_atom(left) && is_number_atom(right);
    }

    let ordinal = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|suffix| value.strip_suffix(suffix));
    ordinal.map_or_else(|| is_number_atom(value), is_number_atom)
}

fn is_number_atom(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let mut digits = 0usize;
    let mut decimal_points = 0usize;
    for character in value.chars() {
        match character {
            '0'..='9' => digits += 1,
            '.' => decimal_points += 1,
            ',' | '\u{202f}' | '\u{00a0}' => {}
            _ => return false,
        }
    }
    digits > 0 && decimal_points <= 1
}

fn is_alphanumeric_identifier(value: &str) -> bool {
    let has_letter = value
        .chars()
        .any(|character| character.is_ascii_alphabetic());
    let has_digit = value.chars().any(|character| character.is_ascii_digit());
    has_letter
        && has_digit
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '/' | '.' | ':')
        })
}

fn is_clock_abbreviation(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "a.m." | "p.m." | "a.m" | "p.m" | "am" | "pm"
    )
}

fn plain_word_count(text: &str) -> usize {
    lexemes(text)
        .into_iter()
        .filter(|lexeme| {
            lexeme
                .text
                .chars()
                .any(|character| character.is_alphanumeric())
        })
        .count()
}

#[derive(Debug, Clone, Copy)]
struct Lexeme<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

fn lexemes(text: &str) -> Vec<Lexeme<'_>> {
    let mut lexemes = Vec::new();
    let mut token_start: Option<usize> = None;

    for (index, character) in text.char_indices() {
        if character.is_whitespace() {
            if let Some(start) = token_start.take()
                && let Some(lexeme) = clean_lexeme(text, start, index)
            {
                lexemes.push(lexeme);
            }
        } else if token_start.is_none() {
            token_start = Some(index);
        }
    }
    if let Some(start) = token_start
        && let Some(lexeme) = clean_lexeme(text, start, text.len())
    {
        lexemes.push(lexeme);
    }
    lexemes
}

fn clean_lexeme(text: &str, mut start: usize, mut end: usize) -> Option<Lexeme<'_>> {
    while start < end {
        let character = text[start..end].chars().next()?;
        if matches!(
            character,
            '"' | '“' | '\'' | '(' | '[' | '{' | '#' | '*' | '`'
        ) {
            start += character.len_utf8();
        } else {
            break;
        }
    }

    let raw = &text[start..end];
    let preserves_terminal_period =
        matches!(raw.to_ascii_lowercase().as_str(), "a.m." | "p.m." | "no.");
    if !preserves_terminal_period {
        while start < end {
            let character = text[start..end].chars().next_back()?;
            if matches!(
                character,
                '"' | '”' | '\'' | ')' | ']' | '}' | ',' | ';' | ':' | '?' | '!' | '.' | '`'
            ) {
                end -= character.len_utf8();
            } else {
                break;
            }
        }
    }

    (start < end).then(|| Lexeme {
        start,
        end,
        text: &text[start..end],
    })
}

#[derive(Debug, Deserialize)]
struct UnitRegistry {
    symbols: Vec<String>,
    names: Vec<String>,
    multiword_names: Vec<String>,
    prefix_symbols: Vec<String>,
    prefix_names: Vec<String>,
    prefixable_symbols: Vec<String>,
    prefixable_names: Vec<String>,
}

fn unit_registry() -> &'static UnitRegistry {
    static REGISTRY: OnceLock<UnitRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../data/rule-8-6-units.json"))
            .expect("embedded Rule 8.6 unit registry must be valid JSON")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_classifier_is_bounded() {
        for value in ["10", "-10", "1,000.5", "1/2", "10-12", "10–12", "21st"] {
            assert!(is_number_expression(value), "{value}");
        }
        for value in ["A10", "10x", "one", "1/part"] {
            assert!(!is_number_expression(value), "{value}");
        }
    }

    #[test]
    fn identifier_classifier_requires_letters_and_digits() {
        for value in ["36L7", "A-10", "STEP2", "1A/2"] {
            assert!(is_alphanumeric_identifier(value), "{value}");
        }
        for value in ["10", "ABC", "A+B"] {
            assert!(!is_alphanumeric_identifier(value), "{value}");
        }
    }

    #[test]
    fn built_in_units_cover_compound_and_prefixed_forms() {
        for value in ["kg", "kPa", "°C", "kg/m³", "m/s", "N", "N m", "Pa s"] {
            assert!(builtin_unit_phrase(value), "{value}");
        }
        assert!(builtin_unit_phrase("degrees Celsius"));
    }
}
