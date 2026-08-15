use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::LintMode;
use crate::structure::word_limit_units;

pub(crate) fn check(text: &str, mode: LintMode) -> Vec<Diagnostic> {
    let (limit, code, rule) = match mode {
        LintMode::Procedural => (20, "STE-LEN-001", "5.1"),
        LintMode::Descriptive => (25, "STE-LEN-002", "6.3"),
    };

    word_limit_units(text)
        .into_iter()
        .filter_map(|unit| {
            (unit.word_count > limit).then(|| Diagnostic {
                code: code.into(),
                severity: Severity::Error,
                message: format!(
                    "Sentence has {} words; the limit is {limit}.",
                    unit.word_count
                ),
                span: Span {
                    start: unit.start,
                    end: unit.end,
                },
                rules: vec![rule.into(), "8.4".into(), "8.5".into(), "8.6".into(), "8.7".into()],
                evidence: Some(json!({
                    "counter": "issue9_mechanical_v1",
                    "word_count": unit.word_count,
                    "limit": limit,
                    "implemented_counting_rules": ["8.4", "8.5", "8.6", "8.7"],
                    "limitations": [
                        "arbitrary unquoted titles and headings require document structure and are not inferred from prose alone",
                        "proper-noun grouping requires external identity context and is not guessed"
                    ]
                })),
                autofix: None,
            })
        })
        .collect()
}
