use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub rules: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofix: Option<Fix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Clean,
    Fixed,
    Error,
    Blocked,
}

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
