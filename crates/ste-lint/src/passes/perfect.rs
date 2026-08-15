use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, PartOfSpeech, RuntimeLexicon};

pub(crate) fn check(text: &str, lexicon: &RuntimeLexicon) -> Vec<Diagnostic> {
    let words = word_spans(text);
    let max_participle_words = lexicon
        .entries()
        .iter()
        .filter(|entry| entry.status == ApprovalStatus::Approved)
        .filter_map(|entry| entry.verb_paradigm.as_ref()?.past_participle.as_ref())
        .map(|form| form.split_whitespace().count())
        .max()
        .unwrap_or(1);
    let mut diagnostics = Vec::new();

    for (index, &(aux_start, aux_end)) in words.iter().enumerate() {
        let auxiliary = &text[aux_start..aux_end];
        if !matches!(
            auxiliary.to_ascii_lowercase().as_str(),
            "have" | "has" | "had"
        ) {
            continue;
        }

        let Some((participle_start, participle_end, ambiguous)) =
            find_participle(text, &words, index + 1, max_participle_words, lexicon)
        else {
            continue;
        };
        let participle = &text[participle_start..participle_end];
        let (code, severity, message) = if ambiguous {
            (
                "STE-VERB-002",
                Severity::Blocked,
                format!(
                    "'{auxiliary} {participle}' can be a prohibited perfect-tense construction, but '{participle}' has another approved dictionary identity; resolve its grammatical use."
                ),
            )
        } else {
            (
                "STE-VERB-001",
                Severity::Error,
                format!(
                    "Do not use '{auxiliary} {participle}' to make a perfect-tense construction."
                ),
            )
        };
        diagnostics.push(Diagnostic {
            code: code.into(),
            severity,
            message,
            span: Span {
                start: aux_start,
                end: participle_end,
            },
            rules: vec!["3.2".into(), "3.4".into()],
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
    text: &str,
    words: &[(usize, usize)],
    start_index: usize,
    max_words: usize,
    lexicon: &RuntimeLexicon,
) -> Option<(usize, usize, bool)> {
    if start_index >= words.len() {
        return None;
    }
    let max_end = (start_index + max_words).min(words.len());

    for end_index in (start_index..max_end).rev() {
        if !is_whitespace_joined(text, words, start_index, end_index) {
            continue;
        }
        let start = words[start_index].0;
        let end = words[end_index].1;
        let phrase = &text[start..end];
        let candidates = lexicon.lookup_form_candidates(phrase);
        let matching = candidates.iter().any(|entry| {
            entry.status == ApprovalStatus::Approved
                && entry
                    .verb_paradigm
                    .as_ref()
                    .and_then(|paradigm| paradigm.past_participle.as_deref())
                    .is_some_and(|form| form.eq_ignore_ascii_case(phrase))
        });
        if !matching {
            continue;
        }
        let ambiguous = candidates.iter().any(|entry| {
            entry.status == ApprovalStatus::Approved
                && (entry.part_of_speech != Some(PartOfSpeech::Verb)
                    || entry
                        .verb_paradigm
                        .as_ref()
                        .and_then(|paradigm| paradigm.past_participle.as_deref())
                        .is_none_or(|form| !form.eq_ignore_ascii_case(phrase)))
        });
        return Some((start, end, ambiguous));
    }
    None
}

fn is_whitespace_joined(
    text: &str,
    words: &[(usize, usize)],
    start_index: usize,
    end_index: usize,
) -> bool {
    (start_index..end_index).all(|index| {
        text[words[index].1..words[index + 1].0]
            .chars()
            .all(char::is_whitespace)
    })
}

fn word_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (index, character) in text.char_indices() {
        let is_word = character.is_alphabetic() || character == '-';
        match (start, is_word) {
            (None, true) => start = Some(index),
            (Some(word_start), false) => {
                spans.push((word_start, index));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(word_start) = start {
        spans.push((word_start, text.len()));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lexicon() -> RuntimeLexicon {
        RuntimeLexicon::from_json(
            r#"{
              "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_perfect"},
              "entries": [
                {"lemma":"REMOVE","status":"approved","part_of_speech":"verb","forms":["REMOVE","REMOVES","REMOVED"],"verb_paradigm":{"classification":"lexical","source_sequence":["REMOVE","REMOVES","REMOVED","REMOVED"],"base_form":"REMOVE","simple_present_variants":["REMOVES"],"simple_past_variants":["REMOVED"],"past_participle":"REMOVED"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"TURN OFF","status":"approved","part_of_speech":"verb","forms":["TURN OFF","TURNS OFF","TURNED OFF"],"verb_paradigm":{"classification":"lexical","source_sequence":["TURN OFF","TURNS OFF","TURNED OFF","TURNED OFF"],"base_form":"TURN OFF","simple_present_variants":["TURNS OFF"],"simple_past_variants":["TURNED OFF"],"past_participle":"TURNED OFF"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"COMPLETE","status":"approved","part_of_speech":"verb","forms":["COMPLETE","COMPLETES","COMPLETED"],"verb_paradigm":{"classification":"lexical","source_sequence":["COMPLETE","COMPLETES","COMPLETED","COMPLETED"],"base_form":"COMPLETE","simple_present_variants":["COMPLETES"],"simple_past_variants":["COMPLETED"],"past_participle":"COMPLETED"},"senses":[],"alternatives":[],"restrictions":[]},
                {"lemma":"COMPLETED","status":"approved","part_of_speech":"adjective","forms":["COMPLETED"],"senses":[],"alternatives":[],"restrictions":[]}
              ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn reports_unambiguous_direct_perfect_tense() {
        let diagnostics = check("THE UNIT HAS REMOVED THE PART.", &lexicon());
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "STE-VERB-001")
            .unwrap();
        assert_eq!(diagnostic.severity, Severity::Error);
        assert!(diagnostic.autofix.is_none());
    }

    #[test]
    fn recognizes_multiword_past_participle() {
        let diagnostics = check("THE UNIT HAS TURNED OFF.", &lexicon());
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "STE-VERB-001")
        );
    }

    #[test]
    fn blocks_when_participle_has_competing_approved_identity() {
        let diagnostics = check("THE UNIT HAS COMPLETED WORK.", &lexicon());
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "STE-VERB-002")
            .unwrap();
        assert_eq!(diagnostic.severity, Severity::Blocked);
    }

    #[test]
    fn simple_past_without_have_auxiliary_is_not_reported() {
        assert!(check("THE UNIT REMOVED THE PART.", &lexicon()).is_empty());
    }

    #[test]
    fn punctuation_between_auxiliary_and_participle_breaks_direct_pattern() {
        assert!(check("THE UNIT HAS, REMOVED THE PART.", &lexicon()).is_empty());
    }
}
