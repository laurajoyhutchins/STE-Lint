use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ste_core::{Diagnostic, Severity, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalTermKind {
    TechnicalNoun,
    TechnicalVerb,
    TechnicalNounAndVerb,
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
    #[serde(default)]
    pub forms: Vec<String>,
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

    pub fn lookup_term(&self, value: &str) -> Option<&TechnicalTerm> {
        let wanted = normalize_identity(value);
        self.terms.iter().find(|term| {
            normalize_identity(&term.term) == wanted
                || term
                    .forms
                    .iter()
                    .chain(term.aliases.iter())
                    .any(|identity| normalize_identity(identity) == wanted)
        })
    }

    pub fn contains_term(&self, value: &str) -> bool {
        self.lookup_term(value).is_some()
    }

    pub fn compose(glossaries: &[Glossary]) -> Result<Self, Vec<Diagnostic>> {
        let glossary = Self {
            terms: glossaries
                .iter()
                .flat_map(|glossary| glossary.terms.iter().cloned())
                .collect(),
        };
        let diagnostics = glossary.validate();
        if diagnostics.is_empty() {
            Ok(glossary)
        } else {
            Err(diagnostics)
        }
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut canonical_seen: HashMap<String, &str> = HashMap::new();
        let mut identity_seen: HashMap<String, (usize, &str, &str)> = HashMap::new();
        let mut diagnostics = Vec::new();

        for (index, term) in self.terms.iter().enumerate() {
            let canonical = normalize_identity(&term.term);
            if let Some(first) = canonical_seen.get(&canonical) {
                diagnostics.push(Diagnostic {
                    code: "TERM-DUP-001".into(),
                    severity: Severity::Error,
                    message: "Technical glossary contains duplicate term identities.".into(),
                    span: Span { start: 0, end: 0 },
                    rules: Vec::new(),
                    evidence: Some(serde_json::json!({
                        "first": first,
                        "second": term.term,
                        "normalized": canonical,
                    })),
                    autofix: None,
                });
            } else {
                canonical_seen.insert(canonical, &term.term);
            }

            for (kind, value) in term_identities(term) {
                let normalized = normalize_identity(value);
                if let Some((first_index, first_term, first_kind)) = identity_seen.get(&normalized)
                {
                    if *first_index != index && !(*first_kind == "term" && kind == "term") {
                        diagnostics.push(identity_conflict(
                            first_term,
                            first_kind,
                            &term.term,
                            kind,
                            &normalized,
                        ));
                    }
                } else {
                    identity_seen.insert(normalized, (index, &term.term, kind));
                }
            }
        }

        diagnostics
    }
}

fn term_identities(term: &TechnicalTerm) -> Vec<(&str, &str)> {
    std::iter::once(("term", term.term.as_str()))
        .chain(term.aliases.iter().map(|value| ("alias", value.as_str())))
        .chain(term.forms.iter().map(|value| ("form", value.as_str())))
        .collect()
}

fn identity_conflict(
    first_term: &str,
    first_kind: &str,
    second_term: &str,
    second_kind: &str,
    normalized: &str,
) -> Diagnostic {
    Diagnostic {
        code: "TERM-ID-CONFLICT-001".into(),
        severity: Severity::Error,
        message:
            "Technical glossary contains a conflicting canonical term, alias, or form identity."
                .into(),
        span: Span { start: 0, end: 0 },
        rules: Vec::new(),
        evidence: Some(serde_json::json!({
            "first": {
                "term": first_term,
                "identity_kind": first_kind,
            },
            "second": {
                "term": second_term,
                "identity_kind": second_kind,
            },
            "normalized": normalized,
        })),
        autofix: None,
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
        assert_eq!(
            glossary.lookup_term("busway").unwrap().status,
            TermStatus::Approved
        );
        assert!(glossary.validate().is_empty());
    }

    #[test]
    fn aliases_resolve_to_the_governed_term() {
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
                "status": "deprecated"
              }]
            }"#,
        )
        .unwrap();

        let term = glossary.lookup_term("BUS   DUCT").unwrap();
        assert_eq!(term.term, "busway");
        assert_eq!(term.status, TermStatus::Deprecated);
        assert!(glossary.contains_term("BUS   DUCT"));
    }

    #[test]
    fn same_term_can_be_governed_as_noun_and_verb() {
        let glossary = Glossary::from_json(
            r#"{
              "terms": [{
                "term": "plate",
                "kind": "technical_noun_and_verb",
                "definition": "A synthetic term with both grammatical uses.",
                "domain": "manufacturing",
                "preferred": true,
                "aliases": [],
                "examples": [],
                "provenance": ["fixture"],
                "status": "approved"
              }]
            }"#,
        )
        .unwrap();

        assert_eq!(
            glossary.lookup_term("plate").unwrap().kind,
            TechnicalTermKind::TechnicalNounAndVerb
        );
    }

    #[test]
    fn duplicate_identity_is_rejected_with_stable_code() {
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/duplicate.json")).unwrap();
        let diagnostics = glossary.validate();
        assert_eq!(diagnostics[0].code, "TERM-DUP-001");
    }
}
