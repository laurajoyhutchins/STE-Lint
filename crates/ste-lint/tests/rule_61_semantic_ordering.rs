use ste_data::RuntimeLexicon;
use ste_lint::{lint_text_with_context, LintContext, LintMode, LintOptions};

#[test]
fn reversed_resolved_project_ordering_reports_rule_61() {
    let text = "Use this. Use that.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"sentence","start":10,"end":19},
            "after": {"kind":"sentence","start":0,"end":9},
            "source": "project-information-order"
          }]
        }"#,
    )
    .unwrap();
    context.validate(text.len()).unwrap();

    let result = lint_text_with_context(
        text,
        &lexicon,
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-DISC-001")
        .expect("reversed resolved ordering must report Rule 6.1");
    assert_eq!(diagnostic.rules, vec!["6.1"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (0, 19));
}