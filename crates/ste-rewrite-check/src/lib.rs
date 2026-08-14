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
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code));
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
