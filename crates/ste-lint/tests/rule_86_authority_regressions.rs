use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

fn lint(text: &str, mode: LintMode, context: Option<&LintContext>) -> ste_lint::LintResult {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text_with_context(
        text,
        &lexicon,
        None,
        context,
        LintOptions { mode, fix: false },
    )
}

fn has_length_error(result: &ste_lint::LintResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.code.as_str(), "STE-LEN-001" | "STE-LEN-002"))
}

fn uses(count: usize) -> String {
    vec!["USE"; count].join(" ")
}

#[test]
fn rule_86_all_governed_proper_noun_classes_count_as_one() {
    for (class, surface) in [
        ("person", "Ada Example Lovelace"),
        ("group", "North Test Working Group"),
        ("organization", "Example Aerospace Standards Council"),
        ("geopolitical_entity", "United Test Territories"),
    ] {
        let context = LintContext::from_json(&format!(
            r#"{{
              "named_entities":[{{
                "id":"fixture-{class}",
                "canonical":"{surface}",
                "class":"{class}",
                "forms":[],
                "source":"independent proper-noun fixture authority"
              }}]
            }}"#
        ))
        .unwrap();
        let text = format!("{} {surface}.", uses(19));
        let result = lint(&text, LintMode::Procedural, Some(&context));
        assert!(!has_length_error(&result), "class {class}: {text}");
    }
}

#[test]
fn rule_86_project_measurement_unit_identity_counts_number_and_unit_as_one() {
    let context = LintContext::from_json(
        r#"{
          "measurement_units":[{
            "id":"widget-flux",
            "canonical":"widget flux",
            "forms":["wf"],
            "source":"independent project measurement authority"
          }]
        }"#,
    )
    .unwrap();

    for unit in ["widget flux", "wf"] {
        let text = format!("{} 10 {unit}.", uses(19));
        let result = lint(&text, LintMode::Procedural, Some(&context));
        assert!(!has_length_error(&result), "unit {unit}: {text}");
    }
}

#[test]
fn rule_86_does_not_guess_an_unknown_word_is_a_measurement_unit() {
    let text = format!("{} 10 widgets.", uses(19));
    let result = lint(&text, LintMode::Procedural, None);
    assert!(has_length_error(&result));
}

#[test]
fn rule_86_numeric_forms_count_as_one() {
    for number in ["+10.5", "−10.5", "0.125", "3/8", "10-12", "10–12"] {
        let text = format!("{} {number}.", uses(19));
        let result = lint(&text, LintMode::Procedural, None);
        assert!(!has_length_error(&result), "number {number}: {text}");
    }
}

#[test]
fn rule_86_alphanumeric_identifiers_count_as_one_without_ner() {
    for identifier in ["A320", "PANEL-3A", "ABC_12", "R2/D2"] {
        let text = format!("{} {identifier}.", uses(19));
        let result = lint(&text, LintMode::Procedural, None);
        assert!(!has_length_error(&result), "identifier {identifier}: {text}");
    }
}

#[test]
fn rule_86_curly_quoted_text_counts_as_one() {
    let text = format!("{} “CONTROL PANEL READY”.", uses(19));
    let result = lint(&text, LintMode::Procedural, None);
    assert!(!has_length_error(&result));
}

#[test]
fn governed_entity_surface_collision_fails_closed() {
    let context = LintContext::from_json(
        r#"{
          "named_entities":[
            {"id":"one","canonical":"Example Test Group","class":"group","forms":[],"source":"fixture A"},
            {"id":"two","canonical":"Different Canonical","class":"organization","forms":["Example Test Group"],"source":"fixture B"}
          ]
        }"#,
    );
    assert!(context.is_err());
}

#[test]
fn governed_measurement_surface_collision_fails_closed() {
    let context = LintContext::from_json(
        r#"{
          "measurement_units":[
            {"id":"one","canonical":"widget flux","forms":["wf"],"source":"fixture A"},
            {"id":"two","canonical":"wobble factor","forms":["wf"],"source":"fixture B"}
          ]
        }"#,
    );
    assert!(context.is_err());
}
