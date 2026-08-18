use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, LexiconEntry, PartOfSpeech};

use super::semantic::dictionary_evidence;
use crate::analysis::linguistic::GenericPos;
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

        if matched
            .candidates
            .iter()
            .all(|entry| entry.status == ApprovalStatus::Approved)
        {
            match observed_part_of_speech(
                analysis,
                &matched.candidates,
                matched.token_start,
                matched.token_width,
            ) {
                Some((observed, basis))
                    if !part_has_compatible_candidate(observed, &matched.candidates) =>
                {
                    diagnostics.push(role_diagnostic(
                        &matched.text,
                        matched.start,
                        matched.end,
                        observed,
                        basis,
                        &matched.candidates,
                    ));
                }
                Some(_) => {}
                None => diagnostics.push(unresolved_role_diagnostic(
                    &matched.text,
                    matched.start,
                    matched.end,
                    &matched.candidates,
                )),
            }
        }

        index += matched.token_width;
    }

    diagnostics
}

fn observed_part_of_speech(
    analysis: &AnalysisDocument<'_>,
    candidates: &[&LexiconEntry],
    token_start: usize,
    token_width: usize,
) -> Option<(PartOfSpeech, &'static str)> {
    let generic = (token_width == 1)
        .then(|| analysis.linguistic_token(token_start))
        .flatten()
        .and_then(|evidence| evidence.occurrence_pos)
        .and_then(generic_pos_to_ste);

    if let Some(bounded) = analysis.dictionary_role_at(token_start, token_width) {
        match bounded.role {
            ObservedRole::Verbal => {
                // The bounded procedural frame is the STE grammatical projection.
                // A generic tagger result can corroborate it, but cannot override it.
                let basis = if generic == Some(PartOfSpeech::Verb) {
                    "harper_brill_pos_tag"
                } else {
                    bounded.basis
                };
                return Some((PartOfSpeech::Verb, basis));
            }
            ObservedRole::Nominal => {
                if let Some(role @ (PartOfSpeech::Noun | PartOfSpeech::Pronoun)) = generic {
                    return Some((role, "harper_brill_pos_tag"));
                }

                let mut nominal_roles = candidates
                    .iter()
                    .filter_map(|entry| entry.part_of_speech)
                    .filter(|role| matches!(role, PartOfSpeech::Noun | PartOfSpeech::Pronoun))
                    .collect::<Vec<_>>();
                nominal_roles.sort_by_key(part_order);
                nominal_roles.dedup();
                return (nominal_roles.len() == 1).then(|| (nominal_roles[0], bounded.basis));
            }
        }
    }

    generic.map(|role| (role, "harper_brill_pos_tag"))
}

fn generic_pos_to_ste(pos: GenericPos) -> Option<PartOfSpeech> {
    match pos {
        GenericPos::Adjective => Some(PartOfSpeech::Adjective),
        GenericPos::Adposition => Some(PartOfSpeech::Preposition),
        GenericPos::Adverb => Some(PartOfSpeech::Adverb),
        GenericPos::Auxiliary | GenericPos::Verb => Some(PartOfSpeech::Verb),
        GenericPos::Conjunction => Some(PartOfSpeech::Conjunction),
        // Universal POS DET includes demonstratives and other determiner uses that
        // do not map one-to-one to the STE dictionary's Article category. Treat it
        // as unresolved evidence instead of manufacturing an STE part of speech.
        GenericPos::Determiner => None,
        GenericPos::Noun | GenericPos::ProperNoun => Some(PartOfSpeech::Noun),
        GenericPos::Pronoun => Some(PartOfSpeech::Pronoun),
        GenericPos::Interjection
        | GenericPos::Numeral
        | GenericPos::Particle
        | GenericPos::Symbol => None,
    }
}

fn role_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    observed_role: PartOfSpeech,
    role_basis: &str,
    candidates: &[&LexiconEntry],
) -> Diagnostic {
    let role_name = role_name(observed_role);
    let mut rules = vec!["1.2".into()];
    if observed_role == PartOfSpeech::Verb {
        rules.push("3.7".into());
    }
    let mut evidence = dictionary_evidence(candidates, false);
    evidence["observed_role"] = json!(role_name);
    evidence["role_basis"] = json!(role_basis);

    Diagnostic {
        code: "STE-GRAM-001".into(),
        severity: Severity::Error,
        message: format!(
            "Approved dictionary word '{matched_text}' is used as {role_name}, which is incompatible with its approved part of speech."
        ),
        span: Span { start, end },
        rules,
        evidence: Some(evidence),
        autofix: None,
    }
}

fn unresolved_role_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    candidates: &[&LexiconEntry],
) -> Diagnostic {
    let mut evidence = dictionary_evidence(candidates, false);
    evidence["role_basis"] = json!("syntactic_role_unresolved");
    Diagnostic {
        code: "STE-GRAM-004".into(),
        severity: Severity::Blocked,
        message: format!(
            "Cannot resolve the grammatical role of approved dictionary word '{matched_text}' without guessing; Rule 1.2 compliance is unresolved."
        ),
        span: Span { start, end },
        rules: vec!["1.2".into()],
        evidence: Some(evidence),
        autofix: None,
    }
}

fn part_has_compatible_candidate(role: PartOfSpeech, candidates: &[&LexiconEntry]) -> bool {
    candidates
        .iter()
        .any(|entry| entry.part_of_speech == Some(role))
}

fn part_order(part: &PartOfSpeech) -> u8 {
    match part {
        PartOfSpeech::Noun => 0,
        PartOfSpeech::Verb => 1,
        PartOfSpeech::Adjective => 2,
        PartOfSpeech::Adverb => 3,
        PartOfSpeech::Pronoun => 4,
        PartOfSpeech::Article => 5,
        PartOfSpeech::Preposition => 6,
        PartOfSpeech::Conjunction => 7,
    }
}

fn role_name(role: PartOfSpeech) -> &'static str {
    match role {
        PartOfSpeech::Noun => "a noun",
        PartOfSpeech::Verb => "a verb",
        PartOfSpeech::Adjective => "an adjective",
        PartOfSpeech::Adverb => "an adverb",
        PartOfSpeech::Pronoun => "a pronoun",
        PartOfSpeech::Article => "an article",
        PartOfSpeech::Preposition => "a preposition",
        PartOfSpeech::Conjunction => "a conjunction",
    }
}
