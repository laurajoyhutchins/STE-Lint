use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, LintResult, lint_text};

fn lint(text: &str) -> LintResult {
    let lexicon = lexicon();
    lint_text(
        text,
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
}

fn diagnostic<'a>(result: &'a LintResult, code: &str) -> Option<&'a ste_core::Diagnostic> {
    result.diagnostics.iter().find(|item| item.code == code)
}

#[test]
fn resolved_four_word_noun_phrase_is_reported_for_rule_2_1() {
    let result = lint("THE HYDRAULIC PRIMARY CONTROL VALVE IS READY.");
    let diagnostic = diagnostic(&result, "STE-NOUN-001")
        .expect("resolved multi-word noun over the three-word maximum must be reported");

    assert_eq!(diagnostic.rules, vec!["2.1"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (0, 35));
    let evidence = diagnostic.evidence.as_ref().expect("noun phrase evidence");
    assert_eq!(evidence["content_word_count"], 4);
    assert_eq!(evidence["token_start"], 0);
    assert_eq!(evidence["token_end"], 5);
    assert_eq!(evidence["head_token"], 4);
}

#[test]
fn resolved_three_word_noun_phrase_is_within_rule_2_1_limit() {
    let result = lint("THE PRIMARY CONTROL VALVE IS READY.");
    assert!(diagnostic(&result, "STE-NOUN-001").is_none());
}

#[test]
fn ambiguous_noun_head_boundary_stays_fail_closed() {
    let result = lint("THE POWER CONTROL VALVE IS READY.");
    assert!(diagnostic(&result, "STE-NOUN-001").is_none());
}

#[test]
fn non_determiner_led_noun_cluster_stays_outside_bounded_slice() {
    let result = lint("HYDRAULIC PRIMARY CONTROL VALVE IS READY.");
    assert!(diagnostic(&result, "STE-NOUN-001").is_none());
}

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_rule_21_multiword_noun"
          },
          "entries":[
            {"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"HYDRAULIC","status":"approved","part_of_speech":"adjective","forms":["HYDRAULIC"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"PRIMARY","status":"approved","part_of_speech":"adjective","forms":["PRIMARY"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CONTROL","status":"approved","part_of_speech":"adjective","forms":["CONTROL"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"POWER","status":"approved","part_of_speech":"noun","forms":["POWER"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"VALVE","status":"approved","part_of_speech":"noun","forms":["VALVE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["BE","IS","WAS","BEEN"],"verb_paradigm":{"classification":"irregular_auxiliary","source_sequence":["BE","IS","WAS","BEEN"],"base_form":"BE","simple_present_variants":["BE","IS"],"simple_past_variants":["WAS"],"past_participle":"BEEN"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"READY","status":"approved","part_of_speech":"adjective","forms":["READY"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}
