use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_glossary::{TechnicalTerm, TechnicalTermKind, TermStatus};

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

        if matched.term.status == TermStatus::Approved
            && let Some(observed) =
                analysis.technical_role_at(matched.token_start, matched.token_width)
            && let Some(diagnostic) = role_diagnostic(
                matched.term,
                &matched.text,
                matched.start,
                matched.end,
                observed.role,
                observed.basis,
            )
        {
            diagnostics.push(diagnostic);
        }

        index += matched.token_width;
    }

    diagnostics
}

fn role_diagnostic(
    term: &TechnicalTerm,
    matched_text: &str,
    start: usize,
    end: usize,
    observed_role: ObservedRole,
    role_basis: &str,
) -> Option<Diagnostic> {
    let role_name = match observed_role {
        ObservedRole::Nominal => "nominal",
        ObservedRole::Verbal => "verbal",
    };
    let (code, rule, message) = match (term.kind, observed_role) {
        (TechnicalTermKind::TechnicalNoun, ObservedRole::Verbal) => (
            "STE-TERM-003",
            "1.7",
            format!(
                "Technical noun '{matched_text}' is used in a bounded imperative verb position."
            ),
        ),
        (TechnicalTermKind::TechnicalVerb, ObservedRole::Nominal) => (
            "STE-TERM-004",
            "1.13",
            format!("Technical verb '{matched_text}' is used in a bounded noun position."),
        ),
        _ => return None,
    };

    Some(Diagnostic {
        code: code.into(),
        severity: Severity::Error,
        message,
        span: Span { start, end },
        rules: vec![rule.into()],
        evidence: Some(json!({
            "canonical_term": &term.term,
            "matched_text": matched_text,
            "governed_kind": term.kind,
            "observed_role": role_name,
            "role_basis": role_basis,
            "domain": &term.domain,
            "provenance": &term.provenance,
        })),
        autofix: None,
    })
}
