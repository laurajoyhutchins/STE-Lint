use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, LexiconEntry};
use ste_glossary::{TechnicalTerm, TermRole, TermStatus};

use super::semantic::dictionary_evidence;
use crate::AnalysisDocument;

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut index = 0;

    while index < analysis.tokens().len() {
        let token = &analysis.tokens()[index];
        if token_is_machine_like_source(analysis.text(), token.start, token.end) {
            index += 1;
            continue;
        }

        let glossary_match = analysis.longest_glossary_match_at(index);
        let dictionary_match = analysis.longest_dictionary_match_at(index);
        let glossary_wins = glossary_match.as_ref().is_some_and(|glossary| {
            dictionary_match
                .as_ref()
                .is_none_or(|dictionary| glossary.token_width >= dictionary.token_width)
        });

        if glossary_wins {
            let matched = glossary_match.unwrap();
            if let Some(diagnostic) =
                glossary_diagnostic(&matched.text, matched.start, matched.end, matched.term)
            {
                diagnostics.push(diagnostic);
            }
            index += matched.token_width;
            continue;
        }

        if let Some(matched) = dictionary_match {
            if let Some(diagnostic) = dictionary_diagnostic(
                &matched.text,
                matched.start,
                matched.end,
                &matched.candidates,
            ) {
                diagnostics.push(diagnostic);
            }
            index += matched.token_width;
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
                "required_classification": ["noun", "verb", "not_a_term"]
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

    let rules = if term.has_role(TermRole::Noun) {
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
            "term_id": term.id,
            "canonical": term.canonical,
            "roles": term.roles,
            "domain": term.domain,
            "status": term.status,
            "replacement": term.replacement,
        })),
        autofix: None,
    })
}

fn token_is_machine_like_source(text: &str, token_start: usize, token_end: usize) -> bool {
    let start = text[..token_start]
        .rfind(char::is_whitespace)
        .map_or(0, |index| index + 1);
    let end = text[token_end..]
        .find(char::is_whitespace)
        .map_or(text.len(), |index| token_end + index);
    is_machine_like(&text[start..end])
}

fn is_machine_marker(character: char) -> bool {
    matches!(character, '_' | '/' | '\\' | '-' | '.')
}

fn is_machine_like(token: &str) -> bool {
    if token.chars().any(|character| character.is_ascii_digit()) {
        return true;
    }

    let characters = token.chars().collect::<Vec<_>>();
    characters.windows(3).any(|window| {
        window[0].is_alphanumeric() && is_machine_marker(window[1]) && window[2].is_alphanumeric()
    }) || token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.starts_with('\\')
}
