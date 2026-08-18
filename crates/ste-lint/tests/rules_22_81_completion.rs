use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{
    LintContext, LintMode, LintOptions, lint_text, lint_text_with_context,
};

fn options() -> LintOptions {
    LintOptions {
        mode: LintMode::Descriptive,
        fix: false,
    }
}

fn long_noun_glossary(alias: &str, alias_kind: &str, status: &str) -> Glossary {
    Glossary::from_json(&format!(
        r#"{{
          "schema":"ste-terminology/v2",
          "domain":"hydraulic",
          "sources":{{"fixture":{{"title":"Independent Rule 2.2 fixture authority"}}}},
          "terms":[{{
            "id":"hydraulic-pressure-control-valve",
            "canonical":"hydraulic pressure control valve",
            "roles":["noun"],
            "definition":"A synthetic governed technical noun.",
            "forms":[],
            "aliases":[{{"text":"{alias}","kind":"{alias_kind}"}}],
            "sources":[{{
              "source":"fixture",
              "supports":["admission","definition","role","alias","status"]
            }}],
            "status":"{status}"
          }}]
        }}"#
    ))
    .unwrap()
}

fn lint_with_glossary(text: &str, glossary: &Glossary) -> ste_lint::LintResult {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text(text, &lexicon, Some(glossary), options())
}

fn has_code(result: &ste_lint::LintResult, code: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn rule_22_allows_canonical_then_short_form() {
    let glossary = long_noun_glossary("pressure valve", "short_form", "approved");
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE. PRESSURE VALVE.",
        &glossary,
    );
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_allows_canonical_then_abbreviation() {
    let glossary = long_noun_glossary("HPCV", "abbreviation", "approved");
    let result = lint_with_glossary("HYDRAULIC PRESSURE CONTROL VALVE. HPCV.", &glossary);
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_rejects_short_form_before_canonical() {
    let glossary = long_noun_glossary("pressure valve", "short_form", "approved");
    let result = lint_with_glossary(
        "PRESSURE VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        &glossary,
    );
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_rejects_abbreviation_before_canonical() {
    let glossary = long_noun_glossary("HPCV", "abbreviation", "approved");
    let result = lint_with_glossary("HPCV. HYDRAULIC PRESSURE CONTROL VALVE.", &glossary);
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_rejects_synonym_even_after_canonical() {
    let glossary = long_noun_glossary("regulating valve", "synonym", "approved");
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE. REGULATING VALVE.",
        &glossary,
    );
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_rejects_legacy_alias_even_after_canonical() {
    let glossary = long_noun_glossary("old pressure valve", "legacy", "approved");
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE. OLD PRESSURE VALVE.",
        &glossary,
    );
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_rejects_long_short_form_that_does_not_satisfy_the_shortening_method() {
    let glossary = long_noun_glossary(
        "alternate hydraulic pressure valve",
        "short_form",
        "approved",
    );
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE. ALTERNATE HYDRAULIC PRESSURE VALVE.",
        &glossary,
    );
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_allows_governed_hyphenated_representation_after_full_form() {
    let glossary = long_noun_glossary("pressure-control valve", "hyphenated", "approved");
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE. PRESSURE-CONTROL VALVE.",
        &glossary,
    );
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_requires_full_form_before_governed_hyphenated_representation() {
    let glossary = long_noun_glossary("pressure-control valve", "hyphenated", "approved");
    let result = lint_with_glossary(
        "PRESSURE-CONTROL VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        &glossary,
    );
    assert!(has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_repeated_canonical_form_is_clean() {
    let glossary = long_noun_glossary("HPCV", "abbreviation", "approved");
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        &glossary,
    );
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_identity_survives_paragraph_and_list_boundaries() {
    let glossary = long_noun_glossary("HPCV", "abbreviation", "approved");
    let result = lint_with_glossary(
        "HYDRAULIC PRESSURE CONTROL VALVE.\n\nITEMS:\n- HPCV.\n- HPCV.",
        &glossary,
    );
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_does_not_match_alias_as_substring_of_unrelated_text() {
    let glossary = long_noun_glossary("pressure valve", "short_form", "approved");
    let result = lint_with_glossary(
        "OVERPRESSURE VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        &glossary,
    );
    assert!(!has_code(&result, "STE-NOUN-002"));
}

#[test]
fn rule_22_deprecated_term_alias_still_uses_terminology_status_enforcement() {
    let glossary = long_noun_glossary("HPCV", "abbreviation", "deprecated");
    let result = lint_with_glossary("HPCV.", &glossary);
    assert!(has_code(&result, "STE-TERM-002"));
}

#[test]
fn rule_81_reports_semicolon_in_authored_prose() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text("USE THIS; USE THIS.", &lexicon, None, options());
    assert!(has_code(&result, "STE-PUNC-001"));
}

#[test]
fn rule_81_reports_multiple_semicolons_and_list_item_semicolon() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text(
        "USE THIS; USE THIS; USE THIS.\n- USE THIS; USE THIS.",
        &lexicon,
        None,
        options(),
    );
    let count = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "STE-PUNC-001")
        .count();
    assert_eq!(count, 3);
}

#[test]
fn rule_81_does_not_lint_semicolon_inside_inline_code() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text("USE `alpha;beta` HERE.", &lexicon, None, options());
    assert!(!has_code(&result, "STE-PUNC-001"));
}

#[test]
fn rule_81_does_not_lint_semicolon_inside_code_block() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text("```text\nalpha;beta\n```", &lexicon, None, options());
    assert!(!has_code(&result, "STE-PUNC-001"));
}

#[test]
fn rule_81_does_not_lint_governed_immutable_quoted_external_text() {
    let text = "USE \"MODE;SAFE\" NOW.";
    let start = text.find('"').unwrap();
    let end = text.rfind('"').unwrap() + 1;
    let context = LintContext::from_json(&format!(
        r#"{{
          "occurrences":[{{
            "start":{start},
            "end":{end},
            "source":"immutable UI contract",
            "official_technical_name":true,
            "text_authority":"quoted_external_text"
          }}]
        }}"#
    ))
    .unwrap();
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(text, &lexicon, None, Some(&context), options());
    assert!(!has_code(&result, "STE-PUNC-001"));
}

#[test]
fn rule_81_still_lints_semicolon_inside_authored_quotation() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text("WRITE \"USE THIS; USE THAT\".", &lexicon, None, options());
    assert!(has_code(&result, "STE-PUNC-001"));
}

#[test]
fn rule_81_unicode_adjacency_preserves_utf8_byte_span() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = "USE café; USE THIS.";
    let result = lint_text(text, &lexicon, None, options());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-PUNC-001")
        .unwrap();
    let semicolon = text.find(';').unwrap();
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (semicolon, semicolon + 1));
}

#[test]
fn rule_81_semicolon_has_no_automatic_replacement() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text("USE THIS; USE THIS.", &lexicon, None, options());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-PUNC-001")
        .unwrap();
    assert!(diagnostic.autofix.is_none());

    let fixed = lint_text(
        "USE THIS; USE THIS.",
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: true,
        },
    );
    assert_eq!(fixed.text, "USE THIS; USE THIS.");
    assert!(has_code(&fixed, "STE-PUNC-001"));
}
