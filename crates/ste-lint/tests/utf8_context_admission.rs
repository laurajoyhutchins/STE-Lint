use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::embedded().unwrap()
}

#[test]
fn malformed_utf8_spans_are_blocked_before_other_passes_consume_offsets() {
    let source = "É T";
    for fact in [
        r#"{"occurrences":[{"start":1,"end":2,"source":"test","count_group":"title"}]}"#,
        r#"{"topics":[{"start":0,"end":1,"topic":"x","source":"test"}]}"#,
        r#"{"semantic_orderings":[{"before":{"kind":"sentence","start":0,"end":1},"after":{"kind":"sentence","start":3,"end":4},"source":"test"}]}"#,
        r#"{"safety_facts":[{"start":0,"end":4,"source":"test","actor":{"start":1,"end":2}}]}"#,
    ] {
        let context = LintContext::from_json(fact).unwrap();
        assert!(context.validate_text(source).is_err(), "{fact}");
        let result = lint_text_with_context(
            source, &lexicon(), None, Some(&context),
            LintOptions { mode: LintMode::Descriptive, fix: true },
        );
        assert_eq!(result.text, source);
        assert_eq!(result.diagnostics.len(), 1, "{fact}");
        assert_eq!(result.diagnostics[0].code, "STE-CTX-000", "{fact}");
    }
}

#[test]
fn valid_multibyte_and_empty_contexts_are_accepted() {
    let source = "É T";
    for fact in [
        r#"{"occurrences":[{"start":0,"end":2,"source":"test","count_group":"title"}]}"#,
        r#"{"occurrences":[]}"#,
    ] {
        assert!(LintContext::from_json(fact).unwrap().validate_text(source).is_ok());
    }
}
