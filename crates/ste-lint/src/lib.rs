#[cfg(test)]
mod tests {
    use super::*;
    use ste_data::RuntimeLexicon;

    #[test]
    fn semicolon_is_reported_and_can_be_safely_fixed() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let options = LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        };
        let result = lint_text("USE THIS; USE THIS.", &lexicon, None, options);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "STE-PUNC-001"));

        let fixed = lint_text(
            "USE THIS; USE THIS.",
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Procedural,
                fix: true,
            },
        );
        assert_eq!(fixed.text, "USE THIS. USE THIS.");
        assert!(!fixed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "STE-PUNC-001"));
    }
}
