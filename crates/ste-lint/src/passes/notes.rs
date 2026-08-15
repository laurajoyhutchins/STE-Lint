use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, RuntimeLexicon, VerbClassification};

use crate::LintMode;
use crate::document_structure::note_blocks;
use crate::structure::word_limit_units;

pub(crate) fn check(text: &str, lexicon: &RuntimeLexicon, mode: LintMode) -> Vec<Diagnostic> {
    if mode != LintMode::Procedural {
        return Vec::new();
    }

    let max_base_words = lexicon
        .entries()
        .iter()
        .filter(|entry| entry.status == ApprovalStatus::Approved)
        .filter_map(|entry| entry.verb_paradigm.as_ref())
        .filter(|paradigm| paradigm.classification != VerbClassification::DefectiveModal)
        .map(|paradigm| paradigm.base_form.split_whitespace().count())
        .max()
        .unwrap_or(1);

    let mut diagnostics = Vec::new();
    for note in note_blocks(text) {
        if note.content_start >= note.end {
            continue;
        }
        let content = &text[note.content_start..note.end];
        for unit in word_limit_units(content) {
            let sentence_start = note.content_start + unit.start;
            let sentence_end = note.content_start + unit.end;
            let Some((verb_end, ambiguous)) =
                imperative_prefix(&text[sentence_start..sentence_end], lexicon, max_base_words)
            else {
                continue;
            };
            let verb = &text[sentence_start..sentence_start + verb_end];
            let (code, severity, message) = if ambiguous {
                (
                    "STE-NOTE-002",
                    Severity::Blocked,
                    format!(
                        "The note sentence starts with '{verb}', which can be an approved imperative verb but has another approved dictionary identity; resolve the grammatical use."
                    ),
                )
            } else {
                (
                    "STE-NOTE-001",
                    Severity::Error,
                    format!("Do not use the imperative form '{verb}' in a note."),
                )
            };
            diagnostics.push(Diagnostic {
                code: code.into(),
                severity,
                message,
                span: Span {
                    start: sentence_start,
                    end: sentence_start + verb_end,
                },
                rules: vec!["5.5".into()],
                evidence: Some(json!({
                    "coverage": "note_initial_approved_base_form_v1",
                    "note_start": note.start,
                    "approved_base_form": verb,
                    "ambiguous_dictionary_identity": ambiguous,
                    "limitations": [
                        "only sentence-initial source-backed approved base forms are classified as imperative candidates",
                        "defective modal auxiliaries are excluded because their base spelling is not an imperative command"
                    ]
                })),
                autofix: None,
            });
        }
    }
    diagnostics
}

fn imperative_prefix(
    sentence: &str,
    lexicon: &RuntimeLexicon,
    max_words: usize,
) -> Option<(usize, bool)> {
    let words = word_spans(sentence);
    if words.is_empty() || words[0].0 != 0 {
        return None;
    }
    let max_end = max_words.min(words.len());
    for end_index in (0..max_end).rev() {
        if !(0..end_index).all(|index| {
            sentence[words[index].1..words[index + 1].0]
                .chars()
                .all(char::is_whitespace)
        }) {
            continue;
        }
        let end = words[end_index].1;
        let phrase = &sentence[..end];
        let candidates = lexicon.lookup_form_candidates(phrase);
        let imperative_candidate = candidates.iter().any(|entry| {
            entry.status == ApprovalStatus::Approved
                && entry.verb_paradigm.as_ref().is_some_and(|paradigm| {
                    paradigm.classification != VerbClassification::DefectiveModal
                        && paradigm.base_form.eq_ignore_ascii_case(phrase)
                })
        });
        if !imperative_candidate {
            continue;
        }
        let ambiguous = candidates.iter().any(|entry| {
            entry.status == ApprovalStatus::Approved
                && !entry.verb_paradigm.as_ref().is_some_and(|paradigm| {
                    paradigm.classification != VerbClassification::DefectiveModal
                        && paradigm.base_form.eq_ignore_ascii_case(phrase)
                })
        });
        return Some((end, ambiguous));
    }
    None
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
