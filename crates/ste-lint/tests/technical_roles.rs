use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintMode, LintOptions, lint_text};

fn empty_lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_technical_roles"
          },
          "entries": []
        }"#,
    )
    .unwrap()
}

fn glossary(term: &str, kind: &str) -> Glossary {
    Glossary::from_json(&format!(
        r#"{{
          "terms": [{{
            "term": "{term}",
            "kind": "{kind}",
            "definition": "Synthetic governed term.",
            "domain": "test",
            "preferred": true,
            "aliases": [],
            "examples": [],
            "provenance": ["fixture"],
            "status": "approved"
          }}]
        }}"#
    ))
    .unwrap()
}

fn diagnostics(text: &str, kind: &str, mode: LintMode) -> Vec<ste_core::Diagnostic> {
    let lexicon = empty_lexicon();
    let glossary = glossary(text.split_whitespace().find(|word| word.chars().all(char::is_alphabetic)).unwrap_or("term"), kind);
    lint_text(
        text,
        &lexicon,
        Some(&glossary),
        LintOptions { mode, fix: false },
    )
    .diagnostics
}

#[test]
fn technical_noun_in_clear_command_role_is_rejected() {
    let lexicon = empty_lexicon();
    let glossary = glossary("oil", "technical_noun");
    let result = lint_text(
        "Oil surface.",
        &lexicon,
        Some(&glossary),
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-TERM-003")
        .expect("technical noun used as an imperative verb must be diagnosed");
    assert_eq!(diagnostic.rules, vec!["1.7"]);
    let evidence = diagnostic.evidence.as_ref().unwrap();
    assert_eq!(evidence["canonical_term"], "oil");
    assert_eq!(evidence["governed_kind"], "technical_noun");
    assert_eq!(evidence["observed_role"], "verbal");
}

#[test]
fn technical_verb_in_determiner_governed_noun_role_is_rejected() {
    let lexicon = empty_lexicon();
    let glossary = glossary("ream", "technical_verb");
    let result = lint_text(
        "The ream is complete.",
        &lexicon,
        Some(&glossary),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-TERM-004")
        .expect("technical verb used after a determiner as a noun must be diagnosed");
    assert_eq!(diagnostic.rules, vec!["1.13"]);
    let evidence = diagnostic.evidence.as_ref().unwrap();
    assert_eq!(evidence["canonical_term"], "ream");
    assert_eq!(evidence["governed_kind"], "technical_verb");
    assert_eq!(evidence["observed_role"], "nominal");
}

#[test]
fn dual_role_governed_term_is_valid_in_both_bounded_roles() {
    let lexicon = empty_lexicon();
    let glossary = glossary("drill", "technical_noun_and_verb");
    for (text, mode) in [
        ("Drill hole.", LintMode::Procedural),
        ("The drill is ready.", LintMode::Descriptive),
    ] {
        let result = lint_text(
            text,
            &lexicon,
            Some(&glossary),
            LintOptions { mode, fix: false },
        );
        assert!(!result
            .diagnostics
            .iter()
            .any(|item| item.code == "STE-TERM-003" || item.code == "STE-TERM-004"));
    }
}

#[test]
fn governed_single_role_term_is_not_rejected_when_local_role_is_compatible() {
    let lexicon = empty_lexicon();
    let noun_glossary = glossary("oil", "technical_noun");
    let noun_result = lint_text(
        "The oil is clean.",
        &lexicon,
        Some(&noun_glossary),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    assert!(!noun_result
        .diagnostics
        .iter()
        .any(|item| item.code == "STE-TERM-003"));

    let verb_glossary = glossary("ream", "technical_verb");
    let verb_result = lint_text(
        "Ream hole.",
        &lexicon,
        Some(&verb_glossary),
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );
    assert!(!verb_result
        .diagnostics
        .iter()
        .any(|item| item.code == "STE-TERM-004"));
}
