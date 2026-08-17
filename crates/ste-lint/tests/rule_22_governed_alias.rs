use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintMode, LintOptions, lint_text};

fn options() -> LintOptions {
    LintOptions {
        mode: LintMode::Descriptive,
        fix: false,
    }
}

fn glossary(kind: &str, status: &str, alias: &str) -> Glossary {
    Glossary::from_json(&format!(
        r#"{{
          "terms":[{{
            "term":"hydraulic pressure control valve",
            "kind":"{kind}",
            "definition":"Synthetic governed long-form term.",
            "domain":"hydraulic",
            "preferred":true,
            "aliases":["{alias}"],
            "examples":[],
            "provenance":["fixture:rule-2.2"],
            "status":"{status}"
          }}]
        }}"#
    ))
    .unwrap()
}

fn has_rule_22(text: &str, glossary: Option<&Glossary>) -> bool {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text(text, &lexicon, glossary, options())
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "STE-NOUN-002")
}

#[test]
fn reports_short_governed_alias_before_long_full_form() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = glossary("technical_noun", "approved", "pressure valve");
    let text = "PRESSURE VALVE. HYDRAULIC PRESSURE CONTROL VALVE.";
    let result = lint_text(text, &lexicon, Some(&glossary), options());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-NOUN-002")
        .expect("short governed alias before the required full form must be reported");

    assert_eq!(diagnostic.rules, vec!["2.2"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (0, 14));
    let evidence = diagnostic.evidence.as_ref().unwrap();
    assert_eq!(
        evidence["canonical_term"],
        "hydraulic pressure control valve"
    );
    assert_eq!(evidence["alias_surface"], "PRESSURE VALVE");
    assert_eq!(evidence["canonical_word_count"], 4);
    assert_eq!(evidence["alias_word_count"], 2);
    assert_eq!(evidence["domain"], "hydraulic");
    assert_eq!(
        evidence["provenance"],
        serde_json::json!(["fixture:rule-2.2"])
    );
    assert_eq!(evidence["full_form_seen_before"], false);
}

#[test]
fn canonical_full_form_before_short_alias_is_clean() {
    let glossary = glossary("technical_noun", "approved", "pressure valve");
    assert!(!has_rule_22(
        "HYDRAULIC PRESSURE CONTROL VALVE. PRESSURE VALVE.",
        Some(&glossary)
    ));
}

#[test]
fn canonical_full_form_without_alias_is_clean() {
    let glossary = glossary("technical_noun", "approved", "pressure valve");
    assert!(!has_rule_22(
        "HYDRAULIC PRESSURE CONTROL VALVE.",
        Some(&glossary)
    ));
}

#[test]
fn nontechnical_noun_governed_term_is_out_of_scope() {
    let glossary = glossary("technical_verb", "approved", "pressure valve");
    assert!(!has_rule_22(
        "PRESSURE VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        Some(&glossary)
    ));
}

#[test]
fn deprecated_governed_term_is_out_of_scope() {
    let glossary = glossary("technical_noun", "deprecated", "pressure valve");
    assert!(!has_rule_22(
        "PRESSURE VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        Some(&glossary)
    ));
}

#[test]
fn alias_that_is_not_short_enough_is_out_of_scope() {
    let glossary = glossary(
        "technical_noun",
        "approved",
        "alternate hydraulic pressure control valve",
    );
    assert!(!has_rule_22(
        "ALTERNATE HYDRAULIC PRESSURE CONTROL VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        Some(&glossary)
    ));
}

#[test]
fn ungoverned_text_is_out_of_scope() {
    assert!(!has_rule_22(
        "PRESSURE VALVE. HYDRAULIC PRESSURE CONTROL VALVE.",
        None
    ));
}
