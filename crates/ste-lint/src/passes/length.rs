use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::LintMode;
use crate::document_structure::{note_blocks, overlaps_note};
use crate::structure::{CountUnit, word_limit_units};

pub(crate) fn check(text: &str, mode: LintMode) -> Vec<Diagnostic> {
    let notes = note_blocks(text);
    let (normal_limit, normal_code, normal_rule) = match mode {
        LintMode::Procedural => (20, "STE-LEN-001", "5.1"),
        LintMode::Descriptive => (25, "STE-LEN-002", "6.3"),
    };
    let mut diagnostics = Vec::new();

    for unit in word_limit_units(text)
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
        let content = &text[note.content_start..note.end];
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
            "limitations": [
                "arbitrary unquoted titles and headings require document structure and are not inferred from prose alone",
                "proper-noun grouping requires external identity context and is not guessed"
            ]
        })),
        autofix: None,
    }
}
