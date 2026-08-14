use std::collections::BTreeMap;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedChange {
    pub original: String,
    pub proposed: String,
    pub target_diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewriteCheckResult {
    pub accepted: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn check_rewrite(change: &ProposedChange) -> RewriteCheckResult {
    let mut diagnostics = Vec::new();

    let original_modality = protected_word_counts(
        &change.original,
        &["may", "can", "could", "must", "should", "will"],
    );
    let proposed_modality = protected_word_counts(
        &change.proposed,
        &["may", "can", "could", "must", "should", "will"],
    );
    if original_modality != proposed_modality {
        diagnostics.push(semantic_diagnostic(
            "SEM-MODALITY-001",
            "The proposed rewrite changes protected modality or epistemic language.",
            json!({
                "original": original_modality,
                "proposed": proposed_modality,
            }),
        ));
    }

    let original_negation =
        protected_word_counts(&change.original, &["not", "no", "never", "cannot"]);
    let proposed_negation =
        protected_word_counts(&change.proposed, &["not", "no", "never", "cannot"]);
    if original_negation != proposed_negation {
        diagnostics.push(semantic_diagnostic(
            "SEM-NEGATION-001",
            "The proposed rewrite changes protected negation.",
            json!({
                "original": original_negation,
                "proposed": proposed_negation,
            }),
        ));
    }

    let original_numbers = numeric_literals(&change.original);
    let proposed_numbers = numeric_literals(&change.proposed);
    if original_numbers != proposed_numbers {
        diagnostics.push(semantic_diagnostic(
            "SEM-QUANTITY-001",
            "The proposed rewrite changes numeric literals.",
            json!({
                "original": original_numbers,
                "proposed": proposed_numbers,
            }),
        ));
    }

    RewriteCheckResult {
        accepted: diagnostics.is_empty(),
        diagnostics,
    }
}

fn semantic_diagnostic(code: &str, message: &str, evidence: serde_json::Value) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        severity: Severity::Error,
        message: message.into(),
        span: Span { start: 0, end: 0 },
        rules: Vec::new(),
        evidence: Some(evidence),
        autofix: None,
    }
}

fn protected_word_counts(text: &str, protected: &[&str]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in text
        .split(|character: char| !character.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
    {
        if protected.contains(&word.as_str()) {
            *counts.entry(word).or_insert(0) += 1;
        }
    }
    counts
}

fn numeric_literals(text: &str) -> Vec<String> {
    number_regex()
        .find_iter(text)
        .map(|matched| matched.as_str().to_owned())
        .collect()
}

fn number_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"[-+]?(?:\d+(?:\.\d+)?|\.\d+)").expect("valid number regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rejected(original: &str, proposed: &str, code: &str) {
        let result = check_rewrite(&ProposedChange {
            original: original.into(),
            proposed: proposed.into(),
            target_diagnostics: Vec::new(),
        });
        assert!(!result.accepted);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == code)
        );
    }

    #[test]
    fn modality_strengthening_is_rejected() {
        assert_rejected(
            "The request may fail.",
            "The request fails.",
            "SEM-MODALITY-001",
        );
    }

    #[test]
    fn dropping_negation_is_rejected() {
        assert_rejected(
            "Do not open the valve.",
            "Open the valve.",
            "SEM-NEGATION-001",
        );
    }

    #[test]
    fn changing_numeric_literal_is_rejected() {
        assert_rejected(
            "Keep the pressure below 10 psi.",
            "Keep the pressure below 20 psi.",
            "SEM-QUANTITY-001",
        );
    }

    #[test]
    fn punctuation_only_repair_is_accepted() {
        let result = check_rewrite(&ProposedChange {
            original: "USE THIS; USE THIS.".into(),
            proposed: "USE THIS. USE THIS.".into(),
            target_diagnostics: vec!["STE-PUNC-001".into()],
        });
        assert!(result.accepted);
        assert!(result.diagnostics.is_empty());
    }
}
