use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_rule_52_action_cardinality"
          },
          "entries": [
            {"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"VALVE","status":"approved","part_of_speech":"noun","forms":["VALVE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"AND","status":"approved","part_of_speech":"conjunction","forms":["AND"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"OPEN","status":"approved","part_of_speech":"verb","forms":["OPEN","OPENS","OPENED"],"verb_paradigm":{"classification":"lexical","source_sequence":["OPEN","OPENS","OPENED","OPENED"],"base_form":"OPEN","simple_present_variants":["OPENS"],"simple_past_variants":["OPENED"],"past_participle":"OPENED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CLOSE","status":"approved","part_of_speech":"verb","forms":["CLOSE","CLOSES","CLOSED"],"verb_paradigm":{"classification":"lexical","source_sequence":["CLOSE","CLOSES","CLOSED","CLOSED"],"base_form":"CLOSE","simple_present_variants":["CLOSES"],"simple_past_variants":["CLOSED"],"past_participle":"CLOSED"},"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}

fn lint(text: &str, mode: LintMode) -> ste_lint::LintResult {
    lint_text(
        text,
        &lexicon(),
        None,
        LintOptions { mode, fix: false },
    )
}

fn diagnostic<'a>(result: &'a ste_lint::LintResult, code: &str) -> Option<&'a ste_core::Diagnostic> {
    result.diagnostics.iter().find(|item| item.code == code)
}

#[test]
fn resolved_multiple_actions_emit_rule_52_diagnostic_with_exact_heads() {
    let result = lint(
        "OPEN THE VALVE AND CLOSE THE VALVE.",
        LintMode::Procedural,
    );
    let diagnostic = diagnostic(&result, "STE-PROC-003")
        .expect("resolved multiple procedural actions must be reported");

    assert_eq!(diagnostic.rules, vec!["5.2"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (0, 35));
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["action_heads"],
        serde_json::json!([
            {"start": 0, "end": 4},
            {"start": 19, "end": 24}
        ])
    );
}

#[test]
fn resolved_single_action_does_not_emit_rule_52_diagnostic() {
    let result = lint("OPEN THE VALVE.", LintMode::Procedural);
    assert!(diagnostic(&result, "STE-PROC-003").is_none());
}

#[test]
fn descriptive_multiple_action_wording_does_not_emit_rule_52_diagnostic() {
    let result = lint(
        "OPEN THE VALVE AND CLOSE THE VALVE.",
        LintMode::Descriptive,
    );
    assert!(diagnostic(&result, "STE-PROC-003").is_none());
}

#[test]
fn unresolved_non_base_opening_does_not_emit_rule_52_diagnostic() {
    let result = lint(
        "OPENS THE VALVE AND CLOSE THE VALVE.",
        LintMode::Procedural,
    );
    assert!(diagnostic(&result, "STE-PROC-003").is_none());
}

#[test]
fn unresolved_second_action_word_does_not_emit_rule_52_diagnostic() {
    let result = lint(
        "OPEN THE VALVE AND CLOSED THE VALVE.",
        LintMode::Procedural,
    );
    assert!(diagnostic(&result, "STE-PROC-003").is_none());
}
