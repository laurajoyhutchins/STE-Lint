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
fn rule_22_does_not_promote_an_explicit_form_into_shortening_authority() {
    let glossary = Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"fixture",
          "sources":{"fixture":{"title":"Independent explicit-form fixture authority"}},
          "terms":[{
            "id":"hydraulic-pressure-control-valve",
            "canonical":"hydraulic pressure control valve",
            "roles":["noun"],
            "definition":"A synthetic governed technical noun.",
            "forms":[{"text":"pressure valve","roles":["noun"]}],
            "aliases":[],
            "sources":[{"source":"fixture","supports":["admission","definition","role","forms","status"]}],
            "status":"approved"
          }]
        }"#,
    )
    .unwrap();
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(
        "HYDRAULIC PRESSURE CONTROL VALVE. PRESSURE VALVE.",
        &lexicon,
        Some(&glossary),
        None,
        options(),
    );
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_81_authored_title_remains_in_punctuation_scope_while_rule_86_counts_it_as_one() {
    let title = "ALPHA; BETA";
    let text = format!("{} {title}.", vec!["USE"; 19].join(" "));
    let start = text.find(title).unwrap();
    let end = start + title.len();
    let context = LintContext::from_json(&format!(
        r#"{{"occurrences":[{{"start":{start},"end":{end},"source":"authored title structure","text_authority":"title"}}]}}"#
    ))
    .unwrap();
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(&text, &lexicon, None, Some(&context), options());

    assert!(has_code(&result, "STE-PUNC-001"));
    assert!(!has_code(&result, "STE-LEN-001"));
}

#[test]
fn rule_81_protected_external_span_does_not_hide_adjacent_authored_semicolon() {
    let protected = "\"MODE;SAFE\"";
    let text = format!("{protected}; USE THIS.");
    let context = LintContext::from_json(&format!(
        r#"{{"occurrences":[{{"start":0,"end":{},"source":"immutable UI contract","text_authority":"quoted_external_text"}}]}}"#,
        protected.len()
    ))
    .unwrap();
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(&text, &lexicon, None, Some(&context), options());
    let semicolons = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "STE-PUNC-001")
        .collect::<Vec<_>>();

    assert_eq!(semicolons.len(), 1);
    assert_eq!(semicolons[0].span.start, protected.len());
}
