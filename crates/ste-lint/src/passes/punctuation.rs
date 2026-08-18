use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::AnalysisDocument;
use crate::analysis::source::SourceDocument;

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let source = SourceDocument::with_context(text, analysis.context());

    text.match_indices(';')
        .filter(|(start, _)| !source.is_protected(*start, *start + 1))
        .map(|(start, _)| Diagnostic {
            code: "STE-PUNC-001".into(),
            severity: Severity::Error,
            message: "Do not use semicolons in STE-authored text. Write separate sentences where required.".into(),
            span: Span {
                start,
                end: start + 1,
            },
            rules: vec!["8.1".into()],
            evidence: Some(json!({
                "coverage": "authored_text_semicolon_prohibition_v2",
                "punctuation": ";",
                "scope": "ste_authored_text"
            })),
            autofix: None,
        })
        .collect()
}
