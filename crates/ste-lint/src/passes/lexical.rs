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

            let candidates = lexicon.lookup_form_candidates(token.text);
            if !candidates.is_empty() {
                let has_approved = candidates
                    .iter()
                    .any(|entry| entry.status == ApprovalStatus::Approved);
                let has_unapproved = candidates
                    .iter()
                    .any(|entry| entry.status == ApprovalStatus::Unapproved);
                let evidence_candidates = candidates
                    .iter()
                    .map(|entry| {
                        json!({
                            "lemma": entry.lemma,
                            "part_of_speech": entry.part_of_speech,
                            "status": entry.status,
                            "alternatives": entry.alternatives,
                        })
                    })
                    .collect::<Vec<_>>();

                if has_approved && has_unapproved {
                    return Some(Diagnostic {
                        code: "STE-LEX-002".into(),
                        severity: Severity::Blocked,
                        message: format!(
                            "'{}' has both approved and unapproved runtime dictionary records; grammatical or sense disambiguation is required.",
                            token.text
                        ),
                        span: Span {
                            start: token.start,
                            end: token.end,
                        },
                        rules: vec!["1.1".into(), "9.2".into()],
                        evidence: Some(json!({
                            "candidates": evidence_candidates,
                            "required_resolution": ["part_of_speech", "approved_sense"]
                        })),
                        autofix: None,
                    });
                }

                if has_unapproved {
                    return Some(Diagnostic {
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
                            "candidates": evidence_candidates,
                        })),
                        autofix: None,
                    });
                }

                return None;
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
            .take_while(|(_, c)| is_boundary_punctuation(*c))
            .map(|(_, c)| c.len_utf8())
            .sum::<usize>();
        let trailing = raw
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_boundary_punctuation(*c))
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

fn is_boundary_punctuation(character: char) -> bool {
    character.is_ascii_punctuation() && !matches!(character, '_' | '/' | '\\' | '-')
}

fn is_machine_marker(character: char) -> bool {
    matches!(character, '_' | '/' | '\\' | '-' | '.')
}

fn is_machine_like(token: &str) -> bool {
    token.chars().any(is_machine_marker) || token.chars().any(|c| c.is_ascii_digit())
}
