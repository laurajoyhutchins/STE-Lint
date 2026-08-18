use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_glossary::{TechnicalTerm, TermRole, TermStatus};

use crate::analysis::linguistic::GenericPos;
use crate::{AnalysisDocument, ObservedRole};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    if analysis.glossary().is_none() || analysis.tokens().is_empty() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let mut index = 0;
    while index < analysis.tokens().len() {
        let Some(matched) = analysis.longest_glossary_match_at(index) else {
            index += 1;
            continue;
        };

        if matched.term.status == TermStatus::Approved {
            match observed_role(analysis, matched.token_start, matched.token_width) {
                Some((observed, basis)) => {
                    if let Some(diagnostic) = role_diagnostic(
                        matched.term,
                        &matched.text,
                        matched.start,
                        matched.end,
                        observed,
                        basis,
                    ) {
                        diagnostics.push(diagnostic);
                    }
                }
                None => {
                    if let Some(diagnostic) = unresolved_role_diagnostic(
                        matched.term,
                        &matched.text,
                        matched.start,
                        matched.end,
                    ) {
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        index += matched.token_width;
    }

    diagnostics
}

fn observed_role(
    analysis: &AnalysisDocument<'_>,
    token_start: usize,
    token_width: usize,
) -> Option<(ObservedRole, &'static str)> {
    if token_width == 1
        && let Some(pos) = analysis
            .linguistic_token(token_start)
            .and_then(|evidence| evidence.occurrence_pos)
    {
        return match pos {
            GenericPos::Verb | GenericPos::Auxiliary => {
                Some((ObservedRole::Verbal, "harper_brill_pos_tag"))
            }
            GenericPos::Noun | GenericPos::ProperNoun | GenericPos::Pronoun => {
                Some((ObservedRole::Nominal, "harper_brill_pos_tag"))
            }
            GenericPos::Adjective
            | GenericPos::Adposition
            | GenericPos::Adverb
            | GenericPos::Conjunction
            | GenericPos::Determiner
            | GenericPos::Interjection
            | GenericPos::Numeral
            | GenericPos::Particle
            | GenericPos::Symbol => None,
        };
    }

    if token_width > 1 {
        let positions = (token_start..token_start + token_width)
            .filter_map(|index| {
                analysis
                    .linguistic_token(index)
                    .and_then(|evidence| evidence.occurrence_pos)
            })
            .collect::<Vec<_>>();
        if positions.len() == token_width {
            if matches!(positions.first(), Some(GenericPos::Verb | GenericPos::Auxiliary)) {
                return Some((ObservedRole::Verbal, "harper_brill_multiword_verb_head"));
            }
            if matches!(positions.last(), Some(GenericPos::Noun | GenericPos::ProperNoun))
                && positions.iter().all(|pos| {
                    matches!(
                        pos,
                        GenericPos::Adjective
                            | GenericPos::Determiner
                            | GenericPos::Noun
                            | GenericPos::Numeral
                            | GenericPos::ProperNoun
                    )
                })
            {
                return Some((ObservedRole::Nominal, "harper_brill_multiword_noun_phrase"));
            }
        }
    }

    analysis
        .technical_role_at(token_start, token_width)
        .map(|evidence| (evidence.role, evidence.basis))
}

fn role_diagnostic(
    term: &TechnicalTerm,
    matched_text: &str,
    start: usize,
    end: usize,
    observed_role: ObservedRole,
    role_basis: &str,
) -> Option<Diagnostic> {
    let (required_role, role_name, code, rule, message) = match observed_role {
        ObservedRole::Nominal => (
            TermRole::Noun,
            "nominal",
            "STE-TERM-004",
            "1.13",
            format!("Technical verb '{matched_text}' is used in a noun position."),
        ),
        ObservedRole::Verbal => (
            TermRole::Verb,
            "verbal",
            "STE-TERM-003",
            "1.7",
            format!("Technical noun '{matched_text}' is used in a verb position."),
        ),
    };
    if term.has_role(required_role) {
        return None;
    }

    Some(Diagnostic {
        code: code.into(),
        severity: Severity::Error,
        message,
        span: Span { start, end },
        rules: vec![rule.into()],
        evidence: Some(json!({
            "term_id": &term.id,
            "canonical": &term.canonical,
            "matched_text": matched_text,
            "governed_roles": &term.roles,
            "observed_role": role_name,
            "role_basis": role_basis,
            "domain": &term.domain,
            "sources": &term.sources,
        })),
        autofix: None,
    })
}

fn unresolved_role_diagnostic(
    term: &TechnicalTerm,
    matched_text: &str,
    start: usize,
    end: usize,
) -> Option<Diagnostic> {
    let (rule, required) = match term.roles.as_slice() {
        [TermRole::Noun] => ("1.7", "non-verbal"),
        [TermRole::Verb] => ("1.13", "non-nominal"),
        _ => return None,
    };
    Some(Diagnostic {
        code: "STE-TERM-005".into(),
        severity: Severity::Blocked,
        message: format!(
            "Cannot resolve whether governed technical term '{matched_text}' is used in the required {required} role without guessing."
        ),
        span: Span { start, end },
        rules: vec![rule.into()],
        evidence: Some(json!({
            "term_id": &term.id,
            "canonical": &term.canonical,
            "matched_text": matched_text,
            "governed_roles": &term.roles,
            "role_basis": "syntactic_role_unresolved",
            "domain": &term.domain,
            "sources": &term.sources,
        })),
        autofix: None,
    })
}
