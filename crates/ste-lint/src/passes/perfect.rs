use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, PartOfSpeech};

use crate::{AnalysisDocument, VerbFormRole};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
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
        let auxiliary = word.text;
        if !matches!(
            auxiliary.to_ascii_lowercase().as_str(),
            "have" | "has" | "had"
        ) {
            continue;
        }

        let Some((participle_start, participle_end, ambiguous)) =
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
        let (code, severity, message, rules) = if ambiguous {
            (
                "STE-VERB-002",
                Severity::Blocked,
                format!(
                    "'{auxiliary} {participle}' can be a prohibited perfect-tense construction, but '{participle}' has another approved dictionary identity; resolve its grammatical use."
                ),
                vec!["3.2".into(), "3.4".into()],
            )
        } else {
            (
                "STE-VERB-001",
                Severity::Error,
                format!(
                    "Do not use '{auxiliary} {participle}' to make a perfect-tense construction."
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
                "coverage": "direct_have_plus_approved_participle_v1",
                "auxiliary": auxiliary,
                "participle": participle,
                "ambiguous_dictionary_identity": ambiguous,
                "autofix": "none because safe conversion to an allowed tense requires sentence-level meaning"
            })),
            autofix: None,
        });
    }

    diagnostics
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
              "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_perfect"},
              "entries": [
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
