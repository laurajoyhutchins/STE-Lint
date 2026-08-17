use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

const TEXT: &str = "Use this. Use that.";

#[test]
fn reversed_resolved_project_ordering_reports_rule_61() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = reversed_sentence_ordering();

    let result = lint_text_with_context(
        TEXT,
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

    let evidence = diagnostic.evidence.as_ref().expect("ordering evidence");
    assert_eq!(evidence["resolution"], "resolved_reversed");
    assert_eq!(evidence["source"], "project-information-order");
    assert_eq!(evidence["expected_before"]["kind"], "sentence");
    assert_eq!(evidence["expected_before"]["index"].as_u64(), Some(1));
    assert_eq!(evidence["expected_before"]["start"].as_u64(), Some(10));
    assert_eq!(evidence["expected_before"]["end"].as_u64(), Some(19));
    assert_eq!(evidence["expected_after"]["kind"], "sentence");
    assert_eq!(evidence["expected_after"]["index"].as_u64(), Some(0));
    assert_eq!(evidence["expected_after"]["start"].as_u64(), Some(0));
    assert_eq!(evidence["expected_after"]["end"].as_u64(), Some(9));
}

#[test]
fn satisfied_project_ordering_is_not_reported() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = context(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"sentence","start":0,"end":9},
            "after": {"kind":"sentence","start":10,"end":19},
            "source": "project-information-order"
          }]
        }"#,
    );

    assert_no_rule_61(lint_text_with_context(
        TEXT,
        &lexicon,
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    ));
}

#[test]
fn unresolved_project_ordering_target_is_not_guessed() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = context(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"sentence","start":10,"end":18},
            "after": {"kind":"sentence","start":0,"end":9},
            "source": "project-information-order"
          }]
        }"#,
    );

    assert_no_rule_61(lint_text_with_context(
        TEXT,
        &lexicon,
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    ));
}

#[test]
fn overlapping_project_ordering_targets_are_not_reported() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = context(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"sentence","start":0,"end":9},
            "after": {"kind":"paragraph","start":0,"end":19},
            "source": "project-information-order"
          }]
        }"#,
    );

    assert_no_rule_61(lint_text_with_context(
        TEXT,
        &lexicon,
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    ));
}

#[test]
fn self_referential_project_ordering_is_not_reported() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = context(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"sentence","start":0,"end":9},
            "after": {"kind":"sentence","start":0,"end":9},
            "source": "project-information-order"
          }]
        }"#,
    );

    assert_no_rule_61(lint_text_with_context(
        TEXT,
        &lexicon,
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    ));
}

#[test]
fn procedural_mode_does_not_apply_descriptive_rule_61_ordering() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = reversed_sentence_ordering();

    assert_no_rule_61(lint_text_with_context(
        TEXT,
        &lexicon,
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    ));
}

fn reversed_sentence_ordering() -> LintContext {
    context(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"sentence","start":10,"end":19},
            "after": {"kind":"sentence","start":0,"end":9},
            "source": "project-information-order"
          }]
        }"#,
    )
}

fn context(source: &str) -> LintContext {
    let context = LintContext::from_json(source).unwrap();
    context.validate(TEXT.len()).unwrap();
    context
}

fn assert_no_rule_61(result: ste_lint::LintResult) {
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-DISC-001")
    );
}
