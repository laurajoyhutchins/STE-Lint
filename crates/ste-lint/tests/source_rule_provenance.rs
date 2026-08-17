use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text, lint_text_with_context};

fn options(mode: LintMode) -> LintOptions {
    LintOptions { mode, fix: false }
}

fn diagnostic<'a>(result: &'a ste_lint::LintResult, code: &str) -> &'a ste_core::Diagnostic {
    result
        .diagnostics
        .iter()
        .find(|item| item.code == code)
        .unwrap_or_else(|| panic!("missing diagnostic {code}: {:?}", result.diagnostics))
}

#[test]
fn approved_meaning_provenance_covers_rules_1_3_and_9_2() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{"occurrences":[{"start":0,"end":6,"source":"sense-review","dictionary_meaning":"not_approved"}]}"#,
    )
    .unwrap();
    let result = lint_text_with_context(
        "FOLLOW",
        &lexicon,
        None,
        Some(&context),
        options(LintMode::Descriptive),
    );
    assert_eq!(diagnostic(&result, "STE-CTX-001").rules, vec!["1.3", "9.2"]);
}

#[test]
fn sentence_length_provenance_includes_general_short_sentence_rule() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text(
        &vec!["USE"; 21].join(" "),
        &lexicon,
        None,
        options(LintMode::Procedural),
    );
    assert_eq!(
        diagnostic(&result, "STE-LEN-001").rules,
        vec!["4.1", "5.1", "8.4", "8.5", "8.6", "8.7"]
    );
}

#[test]
fn note_length_is_descriptive_length_not_rule_5_5() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = format!("NOTE: {}.", vec!["USE"; 26].join(" "));
    let result = lint_text(&text, &lexicon, None, options(LintMode::Procedural));
    assert_eq!(
        diagnostic(&result, "STE-LEN-002").rules,
        vec!["4.1", "6.3", "8.4", "8.5", "8.6", "8.7"]
    );
}

#[test]
fn comma_ended_list_item_does_not_claim_semicolon_rule() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text(
        "USE THESE:\n- FIRST,\n- SECOND.",
        &lexicon,
        None,
        options(LintMode::Procedural),
    );
    assert_eq!(diagnostic(&result, "STE-LIST-003").rules, vec!["4.3"]);
}

#[test]
fn semicolon_ended_list_item_keeps_rule_8_1_provenance() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text(
        "USE THESE:\n- FIRST;\n- SECOND.",
        &lexicon,
        None,
        options(LintMode::Procedural),
    );
    assert_eq!(
        diagnostic(&result, "STE-LIST-003").rules,
        vec!["4.3", "8.1"]
    );
}

#[test]
fn unapproved_multiword_dictionary_entry_does_not_claim_rules_9_2_or_9_3() {
    let lexicon = RuntimeLexicon::from_json(
        r#"{
          "metadata":{"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_rule_provenance"},
          "entries":[{
            "lemma":"PUT OUT",
            "status":"unapproved",
            "part_of_speech":"verb",
            "forms":["PUT OUT"],
            "senses":[],
            "alternatives":[],
            "restrictions":[]
          }]
        }"#,
    )
    .unwrap();
    let result = lint_text("PUT OUT", &lexicon, None, options(LintMode::Procedural));
    assert_eq!(diagnostic(&result, "STE-LEX-001").rules, vec!["1.1"]);
}

#[test]
fn deprecated_governed_technical_noun_carries_rule_1_8_provenance() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = Glossary::from_json(
        r#"{
          "terms":[{
            "term":"busway",
            "kind":"technical_noun",
            "definition":"Project electrical distribution term.",
            "domain":"electrical",
            "preferred":false,
            "aliases":[],
            "examples":[],
            "provenance":["terminology-board"],
            "status":"deprecated"
          }]
        }"#,
    )
    .unwrap();
    let result = lint_text(
        "busway",
        &lexicon,
        Some(&glossary),
        options(LintMode::Descriptive),
    );
    assert_eq!(diagnostic(&result, "STE-TERM-002").rules, vec!["1.8"]);
}
