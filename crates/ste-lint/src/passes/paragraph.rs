use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::LintMode;
use crate::structure::{paragraph_prose_sentence_count, paragraph_ranges};

pub(crate) fn check(text: &str, mode: LintMode) -> Vec<Diagnostic> {
    if mode != LintMode::Descriptive {
        return Vec::new();
    }

    paragraph_ranges(text)
        .into_iter()
        .filter_map(|(start, end)| {
            let sentence_count = paragraph_prose_sentence_count(&text[start..end]);
            (sentence_count > 6).then(|| Diagnostic {
                code: "STE-PARA-001".into(),
                severity: Severity::Error,
                message: format!(
                    "Paragraph has {sentence_count} sentences; the maximum is 6."
                ),
                span: Span { start, end },
                rules: vec!["6.6".into()],
                evidence: Some(json!({
                    "sentence_count": sentence_count,
                    "limit": 6,
                    "vertical_list_items": "excluded from paragraph sentence count per the Rule 6.6 structural example"
                })),
                autofix: None,
            })
        })
        .collect()
}
