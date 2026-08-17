use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, LexiconEntry, RuntimeLexicon};
use ste_glossary::{Glossary, TechnicalTerm, TechnicalTermKind, TermStatus};

use super::semantic::dictionary_evidence;

pub(crate) fn check(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
) -> Vec<Diagnostic> {
    let tokens = tokens(text);
    let max_dictionary_words = lexicon
        .entries()
        .iter()
        .flat_map(|entry| &entry.forms)
        .map(|form| form.split_whitespace().count())
        .max()
        .unwrap_or(1);
    let max_glossary_words = glossary
        .map(|glossary| {
            glossary
                .terms
                .iter()
                .flat_map(|term| std::iter::once(&term.term).chain(term.aliases.iter()))
                .map(|term| term.split_whitespace().count())
                .max()
                .unwrap_or(1)
        })
        .unwrap_or(1);
    let max_phrase_words = max_dictionary_words.max(max_glossary_words);

    let mut diagnostics = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let token = &tokens[index];
        if is_machine_like(token.text) {
            index += 1;
            continue;
        }

        let max_window = max_phrase_words.min(tokens.len() - index);
        let mut matched_phrase = false;

        for width in (2..=max_window).rev() {
            let window = &tokens[index..index + width];
            if window.iter().any(|token| is_machine_like(token.text))
                || !window.windows(2).all(|pair| {
                    text[pair[0].end..pair[1].start]
                        .chars()
                        .all(char::is_whitespace)
                })
            {
                continue;
            }

            let phrase = window
                .iter()
                .map(|token| token.text)
                .collect::<Vec<_>>()
                .join(" ");
            let glossary_term = glossary.and_then(|glossary| glossary.lookup_term(&phrase));
            let candidates = lexicon.lookup_form_candidates(&phrase);

            if glossary_term.is_none() && candidates.is_empty() {
                continue;
            }

            let start = window[0].start;
            let end = window[width - 1].end;
            if let Some(term) = glossary_term {
                if let Some(diagnostic) = glossary_diagnostic(&phrase, start, end, term) {
                    diagnostics.push(diagnostic);
                }
            } else if let Some(diagnostic) = dictionary_diagnostic(&phrase, start, end, &candidates)
            {
                diagnostics.push(diagnostic);
            }

            index += width;
            matched_phrase = true;
            break;
        }

        if matched_phrase {
            continue;
        }

        if let Some(term) = glossary.and_then(|glossary| glossary.lookup_term(token.text)) {
            if let Some(diagnostic) = glossary_diagnostic(token.text, token.start, token.end, term)
            {
                diagnostics.push(diagnostic);
            }
            index += 1;
            continue;
        }

        let candidates = lexicon.lookup_form_candidates(token.text);
        if !candidates.is_empty() {
            if let Some(diagnostic) =
                dictionary_diagnostic(token.text, token.start, token.end, &candidates)
            {
                diagnostics.push(diagnostic);
            }
            index += 1;
            continue;
        }

        diagnostics.push(Diagnostic {
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
                "required_classification": [
                    "technical_noun",
                    "technical_verb",
                    "technical_noun_and_verb",
                    "not_a_term"
                ]
            })),
            autofix: None,
        });
        index += 1;
    }

    diagnostics
}

fn dictionary_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    candidates: &[&LexiconEntry],
) -> Option<Diagnostic> {
    let has_approved = candidates
        .iter()
        .any(|entry| entry.status == ApprovalStatus::Approved);
    let has_unapproved = candidates
        .iter()
        .any(|entry| entry.status == ApprovalStatus::Unapproved);

    if has_approved && has_unapproved {
        let mut evidence = dictionary_evidence(candidates, true);
        evidence["required_resolution"] = json!(["part_of_speech", "approved_sense"]);
        return Some(Diagnostic {
            code: "STE-LEX-002".into(),
            severity: Severity::Blocked,
            message: format!(
                "'{matched_text}' has both approved and unapproved runtime dictionary records; grammatical or sense disambiguation is required."
            ),
            span: Span { start, end },
            rules: vec!["1.1".into()],
            evidence: Some(evidence),
            autofix: None,
        });
    }

    if has_unapproved {
        return Some(Diagnostic {
            code: "STE-LEX-001".into(),
            severity: Severity::Error,
            message: format!("'{matched_text}' is not approved in the runtime STE lexicon."),
            span: Span { start, end },
            rules: vec!["1.1".into()],
            evidence: Some(dictionary_evidence(candidates, candidates.len() > 1)),
            autofix: None,
        });
    }

    None
}

fn glossary_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    term: &TechnicalTerm,
) -> Option<Diagnostic> {
    if term.status != TermStatus::Deprecated {
        return None;
    }

    let rules = if term.kind == TechnicalTermKind::TechnicalNoun {
        vec!["1.8".into()]
    } else {
        Vec::new()
    };

    Some(Diagnostic {
        code: "STE-TERM-002".into(),
        severity: Severity::Error,
        message: format!("'{matched_text}' is deprecated in the project technical glossary."),
        span: Span { start, end },
        rules,
        evidence: Some(json!({
            "canonical_term": term.term,
            "kind": term.kind,
            "domain": term.domain,
            "preferred": term.preferred,
            "status": term.status,
        })),
        autofix: None,
    })
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
