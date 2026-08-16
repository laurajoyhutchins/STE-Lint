use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::document_structure::{note_blocks, overlaps_note};
use crate::structure::{CountUnit, word_limit_units};
use crate::{LintContext, LintMode};

pub(crate) fn check(text: &str, mode: LintMode, context: Option<&LintContext>) -> Vec<Diagnostic> {
    let notes = note_blocks(text);
    let counting_text = counting_projection(text, context);
    let (normal_limit, normal_code, normal_rule) = match mode {
        LintMode::Procedural => (20, "STE-LEN-001", "5.1"),
        LintMode::Descriptive => (25, "STE-LEN-002", "6.3"),
    };
    let mut diagnostics = Vec::new();

    for unit in word_limit_units(&counting_text)
        .into_iter()
        .filter(|unit| !overlaps_note(unit.start, unit.end, &notes))
    {
        if unit.word_count > normal_limit {
            diagnostics.push(length_diagnostic(
                unit,
                0,
                normal_limit,
                normal_code,
                normal_rule,
                false,
            ));
        }
    }

    for note in notes {
        if note.content_start >= note.end {
            continue;
        }
        let content = &counting_text[note.content_start..note.end];
        for unit in word_limit_units(content) {
            if unit.word_count > 25 {
                diagnostics.push(length_diagnostic(
                    unit,
                    note.content_start,
                    25,
                    "STE-LEN-002",
                    "6.3",
                    true,
                ));
            }
        }
    }

    diagnostics
}

fn counting_projection(text: &str, context: Option<&LintContext>) -> String {
    let Some(context) = context else {
        return text.to_owned();
    };
    if context.validate(text.len()).is_err() {
        return text.to_owned();
    }
    if context.occurrences.iter().any(|occurrence| {
        occurrence.count_group.is_some()
            && (!text.is_char_boundary(occurrence.start) || !text.is_char_boundary(occurrence.end))
    }) {
        return text.to_owned();
    }

    let mut bytes = text.as_bytes().to_vec();
    for occurrence in context
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.count_group.is_some())
    {
        let mut marker = None;
        for (index, byte) in bytes
            .iter_mut()
            .enumerate()
            .take(occurrence.end)
            .skip(occurrence.start)
        {
            match *byte {
                b'\n' | b'\r' => {}
                value if value.is_ascii_whitespace() => *byte = b' ',
                _ => {
                    marker.get_or_insert(index);
                    *byte = b' ';
                }
            }
        }
        if let Some(index) = marker {
            bytes[index] = b'X';
        }
    }

    String::from_utf8(bytes).expect("counting projection replaces complete UTF-8 spans with ASCII")
}

fn length_diagnostic(
    unit: CountUnit,
    offset: usize,
    limit: usize,
    code: &str,
    rule: &str,
    note: bool,
) -> Diagnostic {
    let mut rules = vec![
        rule.into(),
        "8.4".into(),
        "8.5".into(),
        "8.6".into(),
        "8.7".into(),
    ];
    if note {
        rules.insert(0, "5.5".into());
    }
    Diagnostic {
        code: code.into(),
        severity: Severity::Error,
        message: format!(
            "Sentence has {} words; the limit is {limit}.",
            unit.word_count
        ),
        span: Span {
            start: offset + unit.start,
            end: offset + unit.end,
        },
        rules,
        evidence: Some(json!({
            "counter": "issue9_mechanical_v1",
            "word_count": unit.word_count,
            "limit": limit,
            "note_uses_descriptive_limit": note,
            "implemented_counting_rules": ["8.4", "8.5", "8.6", "8.7"],
            "context_count_groups": "explicit Rule 8.6 title, heading, placard, label, abbreviation, and proper-noun spans are counted as one word",
            "limitations": [
                "titles, headings, placards, labels, abbreviations, and proper nouns are not inferred from prose alone"
            ]
        })),
        autofix: None,
    }
}
