use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, LintResult, lint_text};

fn lint(text: &str, mode: LintMode) -> LintResult {
    let lexicon = lexicon();
    lint_text(text, &lexicon, None, LintOptions { mode, fix: false })
}

fn has_code(result: &LintResult, code: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn resolved_multiple_procedural_actions_are_reported_for_rule_5_2() {
    let text = "DISCONNECT POWER AND OPEN VALVE.";
    let result = lint(text, LintMode::Procedural);

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-PROC-003")
        .expect("resolved multiple-action procedural instruction must be reported");
    assert_eq!(diagnostic.rules, vec!["5.2"]);
    assert_eq!(
        (diagnostic.span.start, diagnostic.span.end),
        (0, text.len())
    );
    let evidence = diagnostic.evidence.as_ref().expect("action evidence");
    assert_eq!(evidence["action_count"], 2);
    assert_eq!(evidence["action_heads"][0]["start"], 0);
    assert_eq!(evidence["action_heads"][0]["end"], 10);
    assert_eq!(evidence["action_heads"][1]["start"], 21);
    assert_eq!(evidence["action_heads"][1]["end"], 25);
}

#[test]
fn resolved_single_procedural_action_is_allowed() {
    let result = lint("DISCONNECT POWER.", LintMode::Procedural);

    assert!(!has_code(&result, "STE-PROC-003"));
}

#[test]
fn non_base_form_second_verb_does_not_create_a_second_action() {
    let result = lint("DISCONNECT POWER AND OPENED VALVE.", LintMode::Procedural);

    assert!(!has_code(&result, "STE-PROC-003"));
}

#[test]
fn non_base_form_opening_does_not_claim_action_cardinality() {
    let result = lint("DISCONNECTED POWER AND OPEN VALVE.", LintMode::Procedural);

    assert!(!has_code(&result, "STE-PROC-003"));
}

#[test]
fn descriptive_mode_does_not_apply_procedural_action_cardinality() {
    let result = lint("DISCONNECT POWER AND OPEN VALVE.", LintMode::Descriptive);

    assert!(!has_code(&result, "STE-PROC-003"));
}

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_procedural_action_cardinality"
          },
          "entries":[
            {
              "lemma":"DISCONNECT",
              "status":"approved",
              "part_of_speech":"verb",
              "forms":["DISCONNECT","DISCONNECTS","DISCONNECTED"],
              "verb_paradigm":{
                "classification":"lexical",
                "source_sequence":["DISCONNECT","DISCONNECTS","DISCONNECTED","DISCONNECTED"],
                "base_form":"DISCONNECT",
                "simple_present_variants":["DISCONNECT","DISCONNECTS"],
                "simple_past_variants":["DISCONNECTED"],
                "past_participle":"DISCONNECTED"
              },
              "senses":[],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"OPEN",
              "status":"approved",
              "part_of_speech":"verb",
              "forms":["OPEN","OPENS","OPENED"],
              "verb_paradigm":{
                "classification":"lexical",
                "source_sequence":["OPEN","OPENS","OPENED","OPENED"],
                "base_form":"OPEN",
                "simple_present_variants":["OPEN","OPENS"],
                "simple_past_variants":["OPENED"],
                "past_participle":"OPENED"
              },
              "senses":[],"alternatives":[],"restrictions":[]
            },
            {"lemma":"POWER","status":"approved","part_of_speech":"noun","forms":["POWER"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"VALVE","status":"approved","part_of_speech":"noun","forms":["VALVE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"AND","status":"approved","part_of_speech":"conjunction","forms":["AND"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}
