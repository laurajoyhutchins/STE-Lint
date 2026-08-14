use ste_core::{Diagnostic, Fix, Severity, Span};

pub(crate) fn check(text: &str) -> Vec<Diagnostic> {
    text.match_indices(';')
        .map(|(start, _)| Diagnostic {
            code: "STE-PUNC-001".into(),
            severity: Severity::Error,
            message: "Semicolons are not permitted in STE.".into(),
            span: Span {
                start,
                end: start + 1,
            },
            rules: vec!["8.1".into()],
            evidence: None,
            autofix: Some(Fix {
                span: Span {
                    start,
                    end: start + 1,
                },
                replacement: ".".into(),
            }),
        })
        .collect()
}
