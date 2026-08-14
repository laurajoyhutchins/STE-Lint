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
    fn duplicate_identity_is_rejected_with_stable_code() {
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/duplicate.json")).unwrap();
        let diagnostics = glossary.validate();
        assert_eq!(diagnostics[0].code, "TERM-DUP-001");
    }
}
