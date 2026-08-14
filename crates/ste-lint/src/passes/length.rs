use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::LintMode;

pub(crate) fn check(text: &str, mode: LintMode) -> Vec<Diagnostic> {
    let (limit, code, rule) = match mode {
        LintMode::Procedural => (20, "STE-LEN-001", "5.1"),
        LintMode::Descriptive => (25, "STE-LEN-002", "6.3"),
    };

    sentence_spans(text)
        .into_iter()
        .filter_map(|(start, end)| {
            let sentence = &text[start..end];
            let word_count = count_words(sentence);
            (word_count > limit).then(|| Diagnostic {
                code: code.into(),
                severity: Severity::Error,
                message: format!("Sentence has {word_count} words; the limit is {limit}."),
                span: Span { start, end },
                rules: vec![rule.into()],
                evidence: Some(json!({
                    "counter": "first_slice_whitespace",
                    "word_count": word_count,
                    "limit": limit,
                })),
                autofix: None,
            })
        })
        .collect()
}

fn sentence_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0;

    for (index, character) in text.char_indices() {
        if matches!(character, '.' | '?' | '!') {
            let end = index + character.len_utf8();
            if !text[start..end].trim().is_empty() {
                spans.push((start, end));
            }
            start = end;
        }
    }

    if start < text.len() && !text[start..].trim().is_empty() {
        spans.push((start, text.len()));
    }

    spans
}

fn count_words(sentence: &str) -> usize {
    sentence
        .split_whitespace()
        .filter(|token| {
            !token
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .is_empty()
        })
        .count()
}
