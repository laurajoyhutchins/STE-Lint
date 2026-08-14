use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, RuntimeLexicon};
use ste_glossary::Glossary;

pub(crate) fn check(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
) -> Vec<Diagnostic> {
    tokens(text)
        .into_iter()
        .filter_map(|token| {
            if is_machine_like(token.text) {
                return None;
            }

            if let Some(entry) = lexicon.lookup_form(token.text) {
                return match entry.status {
                    ApprovalStatus::Approved => None,
                    ApprovalStatus::Unapproved => Some(Diagnostic {
                        code: "STE-LEX-001".into(),
                        severity: Severity::Error,
                        message: format!(
                            "'{}' is not approved in the runtime STE lexicon.",
                            token.text
                        ),
                        span: Span {
                            start: token.start,
                            end: token.end,
                        },
                        rules: vec!["1.1".into(), "9.2".into()],
                        evidence: Some(json!({
                            "lemma": entry.lemma,
                            "part_of_speech": entry.part_of_speech,
                            "alternatives": entry.alternatives,
                        })),
                        autofix: None,
                    }),
                };
            }

            if glossary.is_some_and(|glossary| glossary.contains_term(token.text)) {
                return None;
            }

            Some(Diagnostic {
                code: "STE-TERM-001".into(),
                severity: Severity::Blocked,
                message: format!(
                    "'{}' is not in the runtime lexicon or project technical glossary.",
                    token.text
                ),
                span: Span {
                    start: token.start,
                    end: token.end,
                },
                rules: vec!["1.1".into()],
                evidence: Some(json!({
                    "term": token.text,
                    "required_classification": ["technical_noun", "technical_verb", "not_a_term"]
                })),
                autofix: None,
            })
        })
        .collect()
}

struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

fn tokens(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    for raw in text.split_whitespace() {
        let relative = text[cursor..].find(raw).unwrap_or(0);
        let raw_start = cursor + relative;
        let raw_end = raw_start + raw.len();
        cursor = raw_end;

        let leading = raw
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_punctuation() && !is_machine_marker(*c))
            .map(|(_, c)| c.len_utf8())
            .sum::<usize>();
        let trailing = raw
            .char_indices()
            .rev()
            .take_while(|(_, c)| c.is_ascii_punctuation() && !is_machine_marker(*c))
            .map(|(_, c)| c.len_utf8())
            .sum::<usize>();

        if leading + trailing >= raw.len() {
            continue;
        }

        let start = raw_start + leading;
        let end = raw_end - trailing;
        let cleaned = &text[start..end];

        if cleaned.chars().all(char::is_alphabetic) || is_machine_like(cleaned) {
            tokens.push(Token {
                text: cleaned,
                start,
                end,
            });
        }
    }

    tokens
}

fn is_machine_marker(character: char) -> bool {
    matches!(character, '_' | '/' | '\\' | '-' | '.')
}

fn is_machine_like(token: &str) -> bool {
    token.chars().any(is_machine_marker) || token.chars().any(|c| c.is_ascii_digit())
}
