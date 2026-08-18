use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, PartOfSpeech};
use ste_glossary::TermRole;

use crate::analysis::linguistic::{GenericPos, GenericVerbForm};
use crate::{AnalysisDocument, AuxiliaryKind, ParticipleRole, Resolution, VerbFormRole};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = perfect_diagnostics(analysis);
    diagnostics.extend(passive_diagnostics(analysis));
    diagnostics.extend(ing_diagnostics(analysis));
    diagnostics
}

fn perfect_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let words = analysis.tokens();
    let max_participle_words = analysis
        .lexicon()
        .entries()
        .iter()
        .filter(|entry| entry.status == ApprovalStatus::Approved)
        .filter_map(|entry| entry.verb_paradigm.as_ref()?.past_participle.as_ref())
        .map(|form| source_form_token_count(form))
        .max()
        .unwrap_or(1);
    let mut diagnostics = Vec::new();

    for (index, word) in words.iter().enumerate() {
        let auxiliary_resolution = auxiliary_identity(analysis, index, AuxiliaryKind::Have);
        if matches!(auxiliary_resolution, Resolution::Resolved(false) | Resolution::Unknown) {
            continue;
        }

        let Some((participle_start, participle_end, participle_ambiguous)) =
            find_participle(analysis, index + 1, max_participle_words)
        else {
            continue;
        };
        if !text[word.end..participle_start]
            .chars()
            .all(char::is_whitespace)
        {
            continue;
        }

        let participle = &text[participle_start..participle_end];
        let auxiliary_ambiguous = matches!(auxiliary_resolution, Resolution::Ambiguous(_));
        let ambiguous = auxiliary_ambiguous || participle_ambiguous;
        let (code, severity, message, rules) = if ambiguous {
            (
                "STE-VERB-002",
                Severity::Blocked,
                format!(
                    "'{} {}' has competing authoritative identities around a possible prohibited perfect construction; resolve its grammatical use.",
                    word.text, participle
                ),
                vec!["3.2".into(), "3.4".into()],
            )
        } else {
            (
                "STE-VERB-001",
                Severity::Error,
                format!(
                    "Do not use '{} {}' to make a perfect-tense construction.",
                    word.text, participle
                ),
                vec!["3.2".into(), "3.3".into(), "3.4".into()],
            )
        };
        diagnostics.push(Diagnostic {
            code: code.into(),
            severity,
            message,
            span: Span {
                start: word.start,
                end: participle_end,
            },
            rules,
            evidence: Some(json!({
                "grammar_projection": "source_backed_have_plus_past_participle",
                "auxiliary": word.text,
                "participle": participle,
                "ambiguous_auxiliary_identity": auxiliary_ambiguous,
                "ambiguous_participle_identity": participle_ambiguous,
                "autofix": "none because safe conversion to an allowed tense requires sentence-level meaning"
            })),
            autofix: None,
        });
    }

    diagnostics
}

fn passive_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for token_index in 0..analysis.tokens().len() {
        match analysis.participle_use_at(token_index) {
            Resolution::Resolved(participle)
                if participle.role == ParticipleRole::PassiveVerb =>
            {
                diagnostics.push(Diagnostic {
                    code: "STE-VERB-003".into(),
                    severity: Severity::Error,
                    message: "Past participle is resolved as a passive verb construction, not as an allowed adjective use."
                        .into(),
                    span: Span {
                        start: participle.span.start,
                        end: participle.span.end,
                    },
                    rules: vec!["3.3".into(), "3.4".into()],
                    evidence: Some(json!({
                        "grammar_projection": "resolved_be_plus_past_participle_passive",
                        "participle_role": "passive_verb",
                    })),
                    autofix: None,
                });
            }
            Resolution::Ambiguous(candidates)
                if candidates
                    .iter()
                    .any(|candidate| candidate.role == ParticipleRole::PassiveVerb)
                    && candidates
                        .iter()
                        .any(|candidate| candidate.role == ParticipleRole::Adjectival) =>
            {
                let span = candidates[0].span;
                diagnostics.push(Diagnostic {
                    code: "STE-VERB-004".into(),
                    severity: Severity::Blocked,
                    message: "Past participle has both passive-verb and adjectival interpretations; Rules 3.3 and 3.4 cannot be decided safely for this occurrence."
                        .into(),
                    span: Span {
                        start: span.start,
                        end: span.end,
                    },
                    rules: vec!["3.3".into(), "3.4".into()],
                    evidence: Some(json!({
                        "grammar_projection": "ambiguous_be_plus_past_participle",
                        "candidate_roles": ["passive_verb", "adjectival"],
                        "requires_disambiguation": true,
                    })),
                    autofix: None,
                });
            }
            _ => {}
        }
    }

    diagnostics
}

fn ing_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for token_index in 0..analysis.tokens().len() {
        let Some(evidence) = analysis.linguistic_token(token_index) else {
            continue;
        };
        if !evidence.verb_forms.contains(&GenericVerbForm::Progressive) {
            continue;
        }
        if !source_links_to_approved_verb(analysis, token_index) {
            continue;
        }

        let token = &analysis.tokens()[token_index];
        let governed_noun = governed_technical_noun_member(analysis, token_index);
        let approved_adjective = exact_approved_adjective_identity(analysis, token_index);
        let previous_be = previous_auxiliary_identity(analysis, token_index, AuxiliaryKind::Be);

        match previous_be {
            Resolution::Resolved(true) => {
                if governed_noun || approved_adjective {
                    diagnostics.push(Diagnostic {
                        code: "STE-VERB-006".into(),
                        severity: Severity::Blocked,
                        message: format!(
                            "'{}' has both prohibited progressive-verb and allowed noun/adjective interpretations; the occurrence cannot be decided safely.",
                            token.text
                        ),
                        span: Span {
                            start: token.start,
                            end: token.end,
                        },
                        rules: vec!["3.2".into(), "3.4".into(), "3.5".into()],
                        evidence: Some(json!({
                            "grammar_projection": "ambiguous_be_plus_ing",
                            "governed_technical_noun_identity": governed_noun,
                            "approved_adjective_identity": approved_adjective,
                            "requires_disambiguation": true,
                        })),
                        autofix: None,
                    });
                } else {
                    diagnostics.push(Diagnostic {
                        code: "STE-VERB-005".into(),
                        severity: Severity::Error,
                        message: format!(
                            "Do not use the progressive verb construction ending in '{}'.",
                            token.text
                        ),
                        span: Span {
                            start: token.start,
                            end: token.end,
                        },
                        rules: vec!["3.2".into(), "3.4".into(), "3.5".into()],
                        evidence: Some(json!({
                            "grammar_projection": "source_linked_be_plus_progressive",
                            "generic_morphology": "progressive",
                            "ste_authority": "approved_runtime_lemma",
                        })),
                        autofix: None,
                    });
                }
            }
            Resolution::Ambiguous(_) => {
                diagnostics.push(Diagnostic {
                    code: "STE-VERB-006".into(),
                    severity: Severity::Blocked,
                    message: format!(
                        "The auxiliary identity before '{}' is ambiguous, so the possible progressive construction cannot be decided safely.",
                        token.text
                    ),
                    span: Span {
                        start: token.start,
                        end: token.end,
                    },
                    rules: vec!["3.2".into(), "3.4".into(), "3.5".into()],
                    evidence: Some(json!({
                        "grammar_projection": "ambiguous_auxiliary_plus_ing",
                        "requires_disambiguation": true,
                    })),
                    autofix: None,
                });
            }
            Resolution::Resolved(false) | Resolution::Unknown => {
                if !governed_noun && evidence.occurrence_pos == Some(GenericPos::Verb) {
                    diagnostics.push(Diagnostic {
                        code: "STE-VERB-007".into(),
                        severity: Severity::Error,
                        message: format!(
                            "Source-linked -ing form '{}' is resolved as a verb, not as an allowed governed technical noun or modifier.",
                            token.text
                        ),
                        span: Span {
                            start: token.start,
                            end: token.end,
                        },
                        rules: vec!["3.5".into()],
                        evidence: Some(json!({
                            "grammar_projection": "source_linked_ing_verbal_occurrence",
                            "generic_occurrence_pos": "verb",
                            "ste_authority": "approved_runtime_lemma",
                            "governed_technical_noun_identity": false,
                        })),
                        autofix: None,
                    });
                }
            }
        }
    }

    diagnostics
}

fn source_links_to_approved_verb(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    if let Some(matched) = analysis.dictionary_match_at(token_index, 1)
        && matched.candidates.iter().any(|entry| {
            entry.status == ApprovalStatus::Approved
                && entry.part_of_speech == Some(PartOfSpeech::Verb)
        })
    {
        return true;
    }

    let Some(lemma) = analysis
        .linguistic_token(token_index)
        .and_then(|evidence| evidence.lemma.as_deref())
    else {
        return false;
    };
    analysis.lexicon().lookup_lemma(lemma).iter().any(|entry| {
        entry.status == ApprovalStatus::Approved && entry.part_of_speech == Some(PartOfSpeech::Verb)
    })
}

fn governed_technical_noun_member(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    (0..=token_index).any(|start| {
        analysis.longest_glossary_match_at(start).is_some_and(|matched| {
            token_index >= matched.token_start
                && token_index < matched.token_start + matched.token_width
                && matched.roles.contains(&TermRole::Noun)
        })
    })
}

fn exact_approved_adjective_identity(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    analysis
        .dictionary_match_at(token_index, 1)
        .is_some_and(|matched| {
            matched.candidates.iter().any(|entry| {
                entry.status == ApprovalStatus::Approved
                    && entry.part_of_speech == Some(PartOfSpeech::Adjective)
            })
        })
}

fn previous_auxiliary_identity(
    analysis: &AnalysisDocument<'_>,
    token_index: usize,
    kind: AuxiliaryKind,
) -> Resolution<bool> {
    if token_index == 0
        || analysis.tokens()[token_index - 1].sentence_id
            != analysis.tokens()[token_index].sentence_id
        || !analysis.text()[analysis.tokens()[token_index - 1].end..analysis.tokens()[token_index].start]
            .chars()
            .all(char::is_whitespace)
    {
        return Resolution::Resolved(false);
    }
    auxiliary_identity(analysis, token_index - 1, kind)
}

fn auxiliary_identity(
    analysis: &AnalysisDocument<'_>,
    token_index: usize,
    kind: AuxiliaryKind,
) -> Resolution<bool> {
    let Some(matched) = analysis.dictionary_match_at(token_index, 1) else {
        return Resolution::Unknown;
    };
    let verdicts = matched
        .candidates
        .iter()
        .map(|entry| {
            entry.status == ApprovalStatus::Approved
                && match kind {
                    AuxiliaryKind::Be => entry.lemma.eq_ignore_ascii_case("be"),
                    AuxiliaryKind::Have => entry.lemma.eq_ignore_ascii_case("have"),
                    AuxiliaryKind::Modal => entry.verb_paradigm.as_ref().is_some_and(|paradigm| {
                        paradigm.classification == ste_data::VerbClassification::DefectiveModal
                    }),
                }
        })
        .collect::<Vec<_>>();
    let has_true = verdicts.contains(&true);
    let has_false = verdicts.contains(&false);
    match (has_true, has_false) {
        (true, false) => Resolution::Resolved(true),
        (false, true) => Resolution::Resolved(false),
        (true, true) => Resolution::Ambiguous(vec![true, false]),
        (false, false) => Resolution::Unknown,
    }
}

fn find_participle(
    analysis: &AnalysisDocument<'_>,
    start_index: usize,
    max_words: usize,
) -> Option<(usize, usize, bool)> {
    if start_index >= analysis.tokens().len() {
        return None;
    }
    let max_width = max_words.min(analysis.tokens().len() - start_index);

    for width in (1..=max_width).rev() {
        let Some(matched) = analysis.source_dictionary_match_at(start_index, width) else {
            continue;
        };
        let matching = matched.verb_forms.iter().any(|candidate| {
            candidate.entry.status == ApprovalStatus::Approved
                && candidate.role == VerbFormRole::PastParticiple
        });
        if !matching {
            continue;
        }
        let ambiguous = matched.candidates.iter().any(|entry| {
            entry.status == ApprovalStatus::Approved
                && (entry.part_of_speech != Some(PartOfSpeech::Verb)
                    || entry
                        .verb_paradigm
                        .as_ref()
                        .and_then(|paradigm| paradigm.past_participle.as_deref())
                        .is_none_or(|form| !form.eq_ignore_ascii_case(&matched.text)))
        });
        return Some((matched.start, matched.end, ambiguous));
    }
    None
}

fn source_form_token_count(value: &str) -> usize {
    value
        .split(|character: char| character.is_whitespace() || character == '-')
        .filter(|part| !part.is_empty())
        .count()
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ste_data::RuntimeLexicon;

    fn lexicon() -> RuntimeLexicon {
        RuntimeLexicon::from_json(
            r#"{
              "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_verb_constructions"},
              "entries": [
                {"lemma":"HAVE","status":"approved","part_of_speech":"verb","forms":["HAVE","HAS","HAD"],"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"REMOVE","status":"approved","part_of_speech":"verb","forms":["REMOVE","REMOVES","REMOVED"],"verb_paradigm":{"classification":"lexical","source_sequence":["REMOVE","REMOVES","REMOVED","REMOVED"],"base_form":"REMOVE","simple_present_variants":["REMOVES"],"simple_past_variants":["REMOVED"],"past_participle":"REMOVED"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"TURN OFF","status":"approved","part_of_speech":"verb","forms":["TURN OFF","TURNS OFF","TURNED OFF"],"verb_paradigm":{"classification":"lexical","source_sequence":["TURN OFF","TURNS OFF","TURNED OFF","TURNED OFF"],"base_form":"TURN OFF","simple_present_variants":["TURNS OFF"],"simple_past_variants":["TURNED OFF"],"past_participle":"TURNED OFF"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"POWER-UP","status":"approved","part_of_speech":"verb","forms":["POWER-UP","POWERS-UP","POWERED-UP"],"verb_paradigm":{"classification":"lexical","source_sequence":["POWER-UP","POWERS-UP","POWERED-UP","POWERED-UP"],"base_form":"POWER-UP","simple_present_variants":["POWERS-UP"],"simple_past_variants":["POWERED-UP"],"past_participle":"POWERED-UP"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"COMPLETE","status":"approved","part_of_speech":"verb","forms":["COMPLETE","COMPLETES","COMPLETED"],"verb_paradigm":{"classification":"lexical","source_sequence":["COMPLETE","COMPLETES","COMPLETED","COMPLETED"],"base_form":"COMPLETE","simple_present_variants":["COMPLETES"],"simple_past_variants":["COMPLETED"],"past_participle":"COMPLETED"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"COMPLETED","status":"approved","part_of_speech":"adjective","forms":["COMPLETED"],"senses":[],"alternatives":[],"restrictions":[]}
              ]
            }"#,
        )
        .unwrap()
    }

    fn diagnostics(text: &str, lexicon: &RuntimeLexicon) -> Vec<Diagnostic> {
        let analysis =
            AnalysisDocument::new(text, lexicon, None, None, crate::LintMode::Descriptive);
        check(&analysis)
    }

    #[test]
    fn reports_unambiguous_direct_perfect_tense() {
        let lexicon = lexicon();
        let diagnostics = diagnostics("THE UNIT HAS REMOVED THE PART.", &lexicon);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "STE-VERB-001")
            .unwrap();
        assert_eq!(diagnostic.severity, Severity::Error);
        assert!(diagnostic.autofix.is_none());
    }

    #[test]
    fn recognizes_multiword_past_participle() {
        let lexicon = lexicon();
        let diagnostics = diagnostics("THE UNIT HAS TURNED OFF.", &lexicon);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "STE-VERB-001")
        );
    }

    #[test]
    fn recognizes_hyphenated_source_backed_past_participle_without_parallel_tokens() {
        let lexicon = lexicon();
        let diagnostics = diagnostics("THE UNIT HAS POWERED-UP.", &lexicon);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "STE-VERB-001")
        );
    }

    #[test]
    fn blocks_when_participle_has_competing_approved_identity() {
        let lexicon = lexicon();
        let diagnostics = diagnostics("THE UNIT HAS COMPLETED WORK.", &lexicon);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "STE-VERB-002")
            .unwrap();
        assert_eq!(diagnostic.severity, Severity::Blocked);
    }

    #[test]
    fn simple_past_without_have_auxiliary_is_not_reported() {
        let lexicon = lexicon();
        assert!(diagnostics("THE UNIT REMOVED THE PART.", &lexicon).is_empty());
    }

    #[test]
    fn punctuation_between_auxiliary_and_participle_breaks_direct_pattern() {
        let lexicon = lexicon();
        assert!(diagnostics("THE UNIT HAS, REMOVED THE PART.", &lexicon).is_empty());
    }
}
