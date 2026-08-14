#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_serializes_with_stable_external_field_names() {
        let diagnostic = Diagnostic {
            code: "STE-PUNC-001".into(),
            severity: Severity::Error,
            message: "Semicolons are not permitted.".into(),
            span: Span { start: 4, end: 5 },
            rules: vec!["8.1".into()],
            evidence: None,
            autofix: Some(Fix {
                span: Span { start: 4, end: 5 },
                replacement: ".".into(),
            }),
        };

        let value = serde_json::to_value(diagnostic).unwrap();
        assert_eq!(value["code"], "STE-PUNC-001");
        assert_eq!(value["severity"], "error");
        assert_eq!(value["span"]["start"], 4);
        assert_eq!(value["autofix"]["replacement"], ".");
    }
}
