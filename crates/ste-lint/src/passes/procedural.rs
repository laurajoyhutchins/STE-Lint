use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, PartOfSpeech, RuntimeLexicon, VerbClassification};

use crate::{LintMode, structure::word_limit_units};

pub(crate) fn check(text: &str, lexicon: &RuntimeLexicon, mode: LintMode) -> Vec<Diagnostic> {
    let mut diagnostics = safety_openings(text, lexicon);
    if mode != LintMode::Procedural {
        return diagnostics;
    }

    diagnostics.extend(condition_commas(text));
    diagnostics.extend(imperative_forms(text, lexicon));
    diagnostics
}

fn imperative_forms(text: &str, lexicon: &RuntimeLexicon) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for unit in word_limit_units(text) {
        let start = unit.start;
        let end = unit.end;
        let sentence = text[start..end].trim_start();
        if sentence.is_empty()
            || starts_label(sentence, "NOTE")
            || starts_label(sentence, "WARNING")
            || starts_label(sentence, "CAUTION")
            || starts_condition(sentence)
        {
            continue;
        }

        let Some((word, word_start, word_end)) = first_word(text, start, end) else {
            continue;
        };
        let candidates = lexicon.lookup_form_candidates(word);
        if candidates.len() != 1 {
            continue;
        }
        let entry = candidates[0];
        if entry.status != ApprovalStatus::Approved || entry.part_of_speech != Some(PartOfSpeech::Verb)
        {
            continue;
        }
        let Some(paradigm) = &entry.verb_paradigm else {
            continue;
        };
        if paradigm.classification != VerbClassification::Lexical
            || word.eq_ignore_ascii_case(&paradigm.base_form)
        {
            continue;
        }

        diagnostics.push(Diagnostic {
            code: "STE-PROC-001".into(),
            severity: Severity::Error,
            message: format!(
                "Procedural instruction starts with non-imperative verb form '{word}'; use the source-backed base form '{}'.",
                paradigm.base_form
            ),
            span: Span {
                start: word_start,
                end: word_end,
            },
            rules: vec!["5.3".into()],
            evidence: Some(json!({
                "lemma": entry.lemma,
                "observed_form": word,
                "base_form": paradigm.base_form,
                "verb_classification": paradigm.classification,
                "source_sequence": paradigm.source_sequence,
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn condition_commas(text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for unit in word_limit_units(text) {
        let start = unit.start;
        let end = unit.end;
        let sentence = text[start..end].trim_start();
        if !starts_condition(sentence) || sentence.contains(',') {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: "STE-PROC-002".into(),
            severity: Severity::Error,
            message: "Leading procedural condition must be separated from its command by a comma."
                .into(),
            span: Span { start, end },
            rules: vec!["5.4".into()],
            evidence: Some(json!({
                "condition_position": "before_command",
                "required_separator": "comma",
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn safety_openings(text: &str, lexicon: &RuntimeLexicon) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut offset = 0usize;

    for line in text.split_inclusive('\n') {
        let without_newline = line.trim_end_matches(['\r', '\n']);
        let leading = without_newline.len() - without_newline.trim_start().len();
        let trimmed = without_newline.trim_start();
        let Some(label_len) = safety_label_len(trimmed) else {
            offset += line.len();
            continue;
        };
        let after_label = &trimmed[label_len..];
        let content_leading = after_label.len() - after_label.trim_start().len();
        let content = after_label.trim_start();
        if content.is_empty() {
            offset += line.len();
            continue;
        }
        let content_start = offset + leading + label_len + content_leading;

        if starts_condition(content) {
            offset += line.len();
            continue;
        }

        let Some((matched, candidates)) = leading_dictionary_match(content, lexicon) else {
            let end = content_start + content.split_whitespace().next().map_or(0, str::len);
            diagnostics.push(safety_error(content_start, end.max(content_start + 1)));
            offset += line.len();
            continue;
        };

        let base_verbs = candidates
            .iter()
            .filter(|entry| {
                entry.status == ApprovalStatus::Approved
                    && entry.part_of_speech == Some(PartOfSpeech::Verb)
                    && entry.verb_paradigm.as_ref().is_some_and(|paradigm| {
                        paradigm.classification == VerbClassification::Lexical
                            && matched.eq_ignore_ascii_case(&paradigm.base_form)
                    })
            })
            .count();

        if base_verbs > 0 && base_verbs == candidates.len() {
            offset += line.len();
            continue;
        }

        let span_end = content_start + matched.len();
        if base_verbs > 0 {
            diagnostics.push(Diagnostic {
                code: "STE-SAFE-002".into(),
                severity: Severity::Blocked,
                message: format!(
                    "Safety instruction opening '{matched}' has competing approved dictionary identities; command role cannot be selected safely."
                ),
                span: Span {
                    start: content_start,
                    end: span_end,
                },
                rules: vec!["7.2".into()],
                evidence: Some(json!({
                    "opening": matched,
                    "candidate_count": candidates.len(),
                    "base_verb_candidates": base_verbs,
                    "requires_disambiguation": true,
                })),
                autofix: None,
            });
        } else {
            diagnostics.push(safety_error(content_start, span_end));
        }

        offset += line.len();
    }

    diagnostics
}

fn safety_error(start: usize, end: usize) -> Diagnostic {
    Diagnostic {
        code: "STE-SAFE-001".into(),
        severity: Severity::Error,
        message: "Safety instruction must start with a clear command or condition.".into(),
        span: Span { start, end },
        rules: vec!["7.2".into()],
        evidence: Some(json!({
            "required_opening": ["imperative_command", "condition"],
        })),
        autofix: None,
    }
}

fn leading_dictionary_match<'a>(
    content: &str,
    lexicon: &'a RuntimeLexicon,
) -> Option<(String, Vec<&'a ste_data::LexiconEntry>)> {
    let words = content
        .split_whitespace()
        .take(8)
        .map(|word| word.trim_matches(|character: char| character.is_ascii_punctuation()))
        .take_while(|word| !word.is_empty())
        .collect::<Vec<_>>();
    for width in (1..=words.len()).rev() {
        let phrase = words[..width].join(" ");
        let candidates = lexicon.lookup_form_candidates(&phrase);
        if !candidates.is_empty() {
            return Some((phrase, candidates));
        }
    }
    None
}

fn first_word(text: &str, start: usize, end: usize) -> Option<(&str, usize, usize)> {
    let slice = &text[start..end];
    let mut word_start = None;
    for (relative, character) in slice.char_indices() {
        if character.is_alphabetic() {
            word_start.get_or_insert(relative);
        } else if let Some(relative_start) = word_start {
            let absolute_start = start + relative_start;
            let absolute_end = start + relative;
            return Some((&text[absolute_start..absolute_end], absolute_start, absolute_end));
        }
    }
    word_start.map(|relative_start| {
        let absolute_start = start + relative_start;
        (&text[absolute_start..end], absolute_start, end)
    })
}

fn starts_condition(text: &str) -> bool {
    let first = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character: char| character.is_ascii_punctuation());
    matches!(
        first.to_ascii_lowercase().as_str(),
        "after" | "before" | "if" | "when" | "while"
    )
}

fn starts_label(text: &str, label: &str) -> bool {
    text.get(..label.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(label))
        && text[label.len()..].trim_start().starts_with(':')
}

fn safety_label_len(text: &str) -> Option<usize> {
    for label in ["WARNING", "CAUTION"] {
        if starts_label(text, label) {
            let colon = text[label.len()..].find(':')? + label.len();
            return Some(colon + 1);
        }
    }
    None
}
