use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, LexiconEntry, PartOfSpeech};

use super::semantic::dictionary_evidence;
use crate::{AnalysisDocument, ObservedRole};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut index = 0;

    while index < analysis.tokens().len() {
        let Some(matched) = analysis.longest_dictionary_match_at(index) else {
            index += 1;
            continue;
        };

        if analysis
            .glossary()
            .and_then(|glossary| glossary.lookup_term(&matched.text))
            .is_some()
        {
            index += matched.token_width;
            continue;
        }

        if let Some(observed) = analysis.dictionary_role_at(matched.token_start, matched.token_width)
        {
            if observed.role == ObservedRole::Verbal
                && let Some(diagnostic) = unapproved_verb_form_diagnostic(
                    &matched.text,
                    matched.start,
                    matched.end,
                    observed.basis,
                    &matched.candidates,
                )
            {
                diagnostics.push(diagnostic);
            }

            if matched
                .candidates
                .iter()
                .all(|entry| entry.status == ApprovalStatus::Approved)
                && !role_has_compatible_candidate(observed.role, &matched.candidates)
            {
                diagnostics.push(role_diagnostic(
                    &matched.text,
                    matched.start,
                    matched.end,
                    observed.role,
                    observed.basis,
                    &matched.candidates,
                ));
            }
        }

        index += matched.token_width;
    }

    diagnostics
}

fn unapproved_verb_form_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    role_basis: &str,
    candidates: &[&LexiconEntry],
) -> Option<Diagnostic> {
    if candidates
        .iter()
        .any(|entry| entry.part_of_speech.is_none())
    {
        return None;
    }

    let verb_candidates = candidates
        .iter()
        .copied()
        .filter(|entry| entry.part_of_speech == Some(PartOfSpeech::Verb))
        .collect::<Vec<_>>();
    if verb_candidates.is_empty()
        || verb_candidates
            .iter()
            .any(|entry| entry.status == ApprovalStatus::Approved)
    {
        return None;
    }

    let mut evidence = dictionary_evidence(&verb_candidates, verb_candidates.len() > 1);
    evidence["observed_role"] = json!("verbal");
    evidence["role_basis"] = json!(role_basis);

    Some(Diagnostic {
        code: "STE-VERB-003".into(),
        severity: Severity::Error,
        message: format!(
            "'{matched_text}' is resolved to a bounded verbal role, but its exact runtime verb form is not approved."
        ),
        span: Span { start, end },
        rules: vec!["3.1".into()],
        evidence: Some(evidence),
        autofix: None,
    })
}

fn role_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    observed_role: ObservedRole,
    role_basis: &str,
    candidates: &[&LexiconEntry],
) -> Diagnostic {
    let role_name = role_name(observed_role);
    let mut rules = vec!["1.2".into()];
    if observed_role == ObservedRole::Verbal {
        rules.push("3.7".into());
    }
    let mut evidence = dictionary_evidence(candidates, false);
    evidence["observed_role"] = json!(role_name);
    evidence["role_basis"] = json!(role_basis);

    Diagnostic {
        code: "STE-GRAM-001".into(),
        severity: Severity::Error,
        message: format!(
            "Approved dictionary word '{matched_text}' is used in a bounded {role_name} role that is incompatible with its approved part of speech."
        ),
        span: Span { start, end },
        rules,
        evidence: Some(evidence),
        autofix: None,
    }
}

fn role_has_compatible_candidate(role: ObservedRole, candidates: &[&LexiconEntry]) -> bool {
    candidates.iter().any(|entry| {
        matches!(
            (role, entry.part_of_speech),
            (ObservedRole::Verbal, Some(PartOfSpeech::Verb))
                | (
                    ObservedRole::Nominal,
                    Some(PartOfSpeech::Noun | PartOfSpeech::Pronoun)
                )
        )
    })
}

fn role_name(role: ObservedRole) -> &'static str {
    match role {
        ObservedRole::Nominal => "nominal",
        ObservedRole::Verbal => "verbal",
    }
}
