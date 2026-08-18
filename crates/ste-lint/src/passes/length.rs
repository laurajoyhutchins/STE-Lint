use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::document_structure::{note_blocks, overlaps_note};
use crate::structure::{CountUnit, word_limit_units};
use crate::{AnalysisDocument, CountGroupProjection, LintMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CountedUnit {
    start: usize,
    end: usize,
    word_count: usize,
}

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let notes = note_blocks(text);
    let projection = CountGroupProjection::from_analysis(analysis);
    let (normal_limit, normal_code, normal_rule) = match analysis.mode() {
        LintMode::Procedural => (20, "STE-LEN-001", "5.1"),
        LintMode::Descriptive => (25, "STE-LEN-002", "6.3"),
    };
    let mut diagnostics = Vec::new();

    for unit in word_limit_units(text)
        .into_iter()
        .filter(|unit| !overlaps_note(unit.start, unit.end, &notes))
    {
        let counted = recount(unit, 0, &projection);
        if counted.word_count > normal_limit {
            diagnostics.push(length_diagnostic(
                counted,
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
            let counted = recount(unit, note.content_start, &projection);
            if counted.word_count > 25 {
                diagnostics.push(length_diagnostic(
                    counted,
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

fn recount(unit: CountUnit, offset: usize, projection: &CountGroupProjection<'_>) -> CountedUnit {
    CountedUnit {
        start: unit.start,
        end: unit.end,
        word_count: projection.count_range(offset + unit.start, offset + unit.end),
    }
}

fn length_diagnostic(
    unit: CountedUnit,
    offset: usize,
    limit: usize,
    code: &str,
    rule: &str,
    note: bool,
) -> Diagnostic {
    let rules = vec![
        "4.1".into(),
        rule.into(),
        "8.4".into(),
        "8.5".into(),
        "8.6".into(),
        "8.7".into(),
    ];
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
            "counter": "issue9_canonical_count_groups_v2",
            "word_count": unit.word_count,
            "limit": limit,
            "note_uses_descriptive_limit": note,
            "implemented_counting_rules": ["8.4", "8.5", "8.6", "8.7"],
            "authority": [
                "document-native structure",
                "verified terminology identity",
                "governed named-entity authority",
                "governed measurement-unit authority",
                "explicit project context",
                "bounded lexical syntax"
            ]
        })),
        autofix: None,
    }
}
