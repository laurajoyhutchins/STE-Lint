use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, LintResult, lint_text};

fn lint(text: &str, mode: LintMode) -> LintResult {
    let lexicon = lexicon();
    lint_text(
        text,
        &lexicon,
        None,
        LintOptions { mode, fix: false },
    )
}

fn diagnostic<'a>(result: &'a LintResult, code: &str) -> Option<&'a ste_core::Diagnostic> {
    result.diagnostics.iter().find(|item| item.code == code)
}

#[test]
fn resolved_unapproved_verbal_form_is_reported_for_rule_3_1() {
    let result = lint("ENSURE THE VALVE.", LintMode::Procedural);
    let diagnostic = diagnostic(&result, "STE-VERB-003")
        .expect("resolved unapproved verbal form must be reported for Rule 3.1");

    assert_eq!(diagnostic.rules, vec!["3.1"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (0, 6));
    let evidence = diagnostic.evidence.as_ref().expect("verb-form evidence");
    assert_eq!(evidence["observed_role"], "verbal");
    assert_eq!(
        evidence["role_basis"],
        "procedural_sentence_initial_term_followed_by_determiner"
    );
}

#[test]
fn approved_verbal_form_is_not_reported() {
    let result = lint("USE THE VALVE.", LintMode::Procedural);
    assert!(diagnostic(&result, "STE-VERB-003").is_none());
}

#[test]
fn competing_approved_verbal_candidate_stays_fail_closed() {
    let result = lint("CHECK THE VALVE.", LintMode::Procedural);
    assert!(diagnostic(&result, "STE-VERB-003").is_none());
}

#[test]
fn unapproved_verb_without_bounded_verbal_role_stays_outside_slice() {
    let result = lint("THE ENSURE IS READY.", LintMode::Descriptive);
    assert!(diagnostic(&result, "STE-VERB-003").is_none());
}

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_rule_31_dictionary_verb_forms"
          },
          "entries":[
            {"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"VALVE","status":"approved","part_of_speech":"noun","forms":["VALVE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"READY","status":"approved","part_of_speech":"adjective","forms":["READY"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["IS"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"USE","status":"approved","part_of_speech":"verb","forms":["USE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"ENSURE","status":"unapproved","part_of_speech":"verb","forms":["ENSURE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CHECK","status":"approved","part_of_speech":"verb","forms":["CHECK"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CHECK_ALT","status":"unapproved","part_of_speech":"verb","forms":["CHECK"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}
