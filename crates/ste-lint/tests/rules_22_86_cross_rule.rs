use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

fn options() -> LintOptions {
    LintOptions {
        mode: LintMode::Procedural,
        fix: false,
    }
}

fn has_code(result: &ste_lint::LintResult, code: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn multiple_long_entities_keep_independent_alias_identity() {
    let glossary = Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"fixture",
          "sources":{"fixture":{"title":"Independent multi-entity fixture authority"}},
          "terms":[
            {
              "id":"hydraulic-pressure-control-valve",
              "canonical":"hydraulic pressure control valve",
              "roles":["noun"],
              "definition":"Synthetic noun A.",
              "forms":[],
              "aliases":[{"text":"HPCV","kind":"abbreviation"}],
              "sources":[{"source":"fixture","supports":["admission","definition","role","alias","status"]}],
              "status":"approved"
            },
            {
              "id":"electrical-power-distribution-panel",
              "canonical":"electrical power distribution panel",
              "roles":["noun"],
              "definition":"Synthetic noun B.",
              "forms":[],
              "aliases":[{"text":"EPDP","kind":"abbreviation"}],
              "sources":[{"source":"fixture","supports":["admission","definition","role","alias","status"]}],
              "status":"approved"
            }
          ]
        }"#,
    )
    .unwrap();
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(
        "HYDRAULIC PRESSURE CONTROL VALVE. ELECTRICAL POWER DISTRIBUTION PANEL. HPCV. EPDP.",
        &lexicon,
        Some(&glossary),
        None,
        options(),
    );
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_abbreviation_identity_is_the_same_group_consumed_by_rule_86() {
    let glossary = Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"fixture",
          "sources":{"fixture":{"title":"Independent cross-rule fixture authority"}},
          "terms":[{
            "id":"hydraulic-pressure-control-valve",
            "canonical":"hydraulic pressure control valve",
            "roles":["noun"],
            "definition":"Synthetic cross-rule noun.",
            "forms":[],
            "aliases":[{"text":"HPC V","kind":"abbreviation"}],
            "sources":[{"source":"fixture","supports":["admission","definition","role","alias","status"]}],
            "status":"approved"
          }]
        }"#,
    )
    .unwrap();
    let text = format!(
        "HYDRAULIC PRESSURE CONTROL VALVE. {} HPC V.",
        vec!["USE"; 19].join(" ")
    );
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(&text, &lexicon, Some(&glossary), None, options());
    assert!(!has_code(&result, "STE-NOUN-002"));
    assert!(!has_code(&result, "STE-LEN-001"));
}

#[test]
fn rule_22_governed_hyphenated_short_form_can_share_rule_82_relation_authority() {
    let glossary = Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"fixture",
          "sources":{"fixture":{"title":"Independent hyphen fixture authority"}},
          "terms":[{
            "id":"hydraulic-pressure-control-valve",
            "canonical":"hydraulic pressure control valve",
            "roles":["noun"],
            "definition":"Synthetic cross-rule noun.",
            "forms":[],
            "aliases":[{"text":"pressure-control valve","kind":"short_form"}],
            "sources":[{"source":"fixture","supports":["admission","definition","role","alias","status"]}],
            "status":"approved"
          }]
        }"#,
    )
    .unwrap();
    let text = "HYDRAULIC PRESSURE CONTROL VALVE. PRESSURE-CONTROL VALVE.";
    let start = text.find("PRESSURE-CONTROL").unwrap();
    let end = start + "PRESSURE-CONTROL".len();
    let context = LintContext::from_json(&format!(
        r#"{{"occurrences":[{{"start":{start},"end":{end},"source":"governed direct-relation fixture","hyphen_direct_relation":true}}]}}"#
    ))
    .unwrap();
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(text, &lexicon, Some(&glossary), Some(&context), options());
    assert!(!has_code(&result, "STE-NOUN-002"));
    assert!(!has_code(&result, "STE-PUNC-002"));
    assert!(!has_code(&result, "STE-CTX-000"));
}
