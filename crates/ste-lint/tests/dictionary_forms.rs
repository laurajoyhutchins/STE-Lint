use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_form_inventory"},
          "entries": [
            {"lemma":"REMOVE","status":"approved","part_of_speech":"verb","forms":["REMOVE","REMOVES","REMOVED"],"verb_paradigm":{"classification":"lexical","source_sequence":["REMOVE","REMOVES","REMOVED","REMOVED"],"base_form":"REMOVE","simple_present_variants":["REMOVES"],"simple_past_variants":["REMOVED"],"past_participle":"REMOVED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"SMALL","status":"approved","part_of_speech":"adjective","forms":["SMALL"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}

#[test]
fn progressive_form_linked_to_approved_verb_but_absent_from_source_inventory_is_rejected() {
    let result = lint_text(
        "REMOVING",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-FORM-001")
        .expect("source-linked out-of-inventory verb form must be rejected");
    assert_eq!(diagnostic.rules, vec!["1.4", "3.1"]);
    assert_eq!(diagnostic.span.start, 0);
    assert_eq!(diagnostic.span.end, "REMOVING".len());
}

#[test]
fn source_supplied_verb_form_is_not_rejected_by_generic_morphology() {
    let result = lint_text(
        "REMOVED",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-FORM-001")
    );
}

#[test]
fn comparative_adjective_absent_from_source_inventory_is_rejected() {
    let result = lint_text(
        "SMALLER",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-FORM-001")
        .expect("source-linked out-of-inventory adjective form must be rejected");
    assert_eq!(diagnostic.rules, vec!["1.4"]);
}
