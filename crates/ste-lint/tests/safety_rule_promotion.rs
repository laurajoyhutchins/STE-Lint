use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, LintResult, lint_text_with_context};

fn lint(text: &str, context: Option<&LintContext>) -> LintResult {
    let lexicon = lexicon();
    lint_text_with_context(
        text,
        &lexicon,
        None,
        context,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    )
}

fn has_code(result: &LintResult, code: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn warning_label_conflicting_with_supplied_caution_level_is_reported() {
    let text = "WARNING: DISCONNECT POWER.";
    let context = context(text, r#""caution""#);
    let result = lint(text, Some(&context));

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-SAFE-003")
        .expect("supplied risk level that conflicts with the visible safety label must be reported");
    assert_eq!(diagnostic.rules, vec!["7.1"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (0, text.len()));
}

#[test]
fn caution_label_conflicting_with_supplied_warning_level_is_reported() {
    let text = "CAUTION: DISCONNECT POWER.";
    let context = context(text, r#""warning""#);
    let result = lint(text, Some(&context));

    assert!(has_code(&result, "STE-SAFE-003"));
}

#[test]
fn matching_supplied_and_visible_safety_levels_are_allowed() {
    let text = "WARNING: DISCONNECT POWER.";
    let context = context(text, r#""warning""#);
    let result = lint(text, Some(&context));

    assert!(!has_code(&result, "STE-SAFE-003"));
}

#[test]
fn missing_supplied_risk_level_is_not_guessed() {
    let text = "WARNING: DISCONNECT POWER.";
    let result = lint(text, None);

    assert!(!has_code(&result, "STE-SAFE-003"));
}

#[test]
fn conflicting_project_risk_levels_are_not_guessed() {
    let text = "WARNING: DISCONNECT POWER.";
    let context = LintContext::from_json(&format!(
        r#"{{
          "safety_facts": [
            {{"start":0,"end":{},"source":"risk-analysis-a","safety_level":"warning"}},
            {{"start":0,"end":{},"source":"risk-analysis-b","safety_level":"caution"}}
          ]
        }}"#,
        text.len(),
        text.len()
    ))
    .unwrap();
    context.validate(text.len()).unwrap();
    let result = lint(text, Some(&context));

    assert!(!has_code(&result, "STE-SAFE-003"));
}

#[test]
fn supplied_risk_level_does_not_create_a_safety_block() {
    let text = "DISCONNECT POWER.";
    let context = context(text, r#""warning""#);
    let result = lint(text, Some(&context));

    assert!(!has_code(&result, "STE-SAFE-003"));
}

fn context(text: &str, level: &str) -> LintContext {
    let context = LintContext::from_json(&format!(
        r#"{{
          "safety_facts": [{{
            "start": 0,
            "end": {},
            "source": "project-hazard-analysis",
            "safety_level": {}
          }}]
        }}"#,
        text.len(),
        level
    ))
    .unwrap();
    context.validate(text.len()).unwrap();
    context
}

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_safety_rule_promotion"
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
              "lemma":"POWER",
              "status":"approved",
              "part_of_speech":"noun",
              "forms":["POWER"],
              "senses":[],"alternatives":[],"restrictions":[]
            }
          ]
        }"#,
    )
    .unwrap()
}
