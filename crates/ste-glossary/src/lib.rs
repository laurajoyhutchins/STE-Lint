use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ste_core::{Diagnostic, Severity, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalTermKind {
    TechnicalNoun,
    TechnicalVerb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermStatus {
    Approved,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnicalTerm {
    pub term: String,
    pub kind: TechnicalTermKind,
    pub definition: String,
    pub domain: String,
    pub preferred: bool,
    pub aliases: Vec<String>,
    pub examples: Vec<String>,
    pub provenance: Vec<String>,
    pub status: TermStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glossary {
    pub terms: Vec<TechnicalTerm>,
}

impl Glossary {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn contains_term(&self, value: &str) -> bool {
        let wanted = normalize_identity(value);
        self.terms.iter().any(|term| {
            normalize_identity(&term.term) == wanted
                || term
                    .aliases
                    .iter()
                    .any(|alias| normalize_identity(alias) == wanted)
        })
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut seen: HashMap<String, &str> = HashMap::new();
        let mut diagnostics = Vec::new();

        for term in &self.terms {
            let identity = normalize_identity(&term.term);
            if let Some(first) = seen.get(&identity) {
                diagnostics.push(Diagnostic {
                    code: "TERM-DUP-001".into(),
                    severity: Severity::Error,
                    message: "Technical glossary contains duplicate term identities.".into(),
                    span: Span { start: 0, end: 0 },
                    rules: Vec::new(),
                    evidence: Some(serde_json::json!({
                        "first": first,
                        "second": term.term,
                        "normalized": identity,
                    })),
                    autofix: None,
                });
            } else {
                seen.insert(identity, &term.term);
            }
        }

        diagnostics
    }
}

fn normalize_identity(value: &str) -> String {
    value
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_glossary_recognizes_project_term() {
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/valid.json")).unwrap();
        assert!(glossary.contains_term("busway"));
        assert!(glossary.validate().is_empty());
    }

    #[test]
    fn aliases_are_recognized_as_project_terms() {
        let glossary = Glossary::from_json(
            r#"{
              "terms": [{
                "term": "busway",
                "kind": "technical_noun",
                "definition": "A project term.",
                "domain": "electrical",
                "preferred": true,
                "aliases": ["bus duct"],
                "examples": [],
                "provenance": ["fixture"],
                "status": "approved"
              }]
            }"#,
        )
        .unwrap();

        assert!(glossary.contains_term("BUS   DUCT"));
    }

    #[test]
    fn duplicate_identity_is_rejected_with_stable_code() {
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/duplicate.json")).unwrap();
        let diagnostics = glossary.validate();
        assert_eq!(diagnostics[0].code, "TERM-DUP-001");
    }
}
