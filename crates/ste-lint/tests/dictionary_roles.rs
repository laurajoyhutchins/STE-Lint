use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_dictionary_roles"
          },
          "entries": [
            {
              "lemma": "TEST",
              "status": "approved",
              "part_of_speech": "noun",
              "forms": ["test"],
              "senses": [{"meaning": "An examination."}],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "ADJUST",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["adjust"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "CLEAN_VERB",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["clean"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "CLEAN_ADJECTIVE",
              "status": "approved",
              "part_of_speech": "adjective",
              "forms": ["clean"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "turn on",
              "status": "unapproved",
              "part_of_speech": "verb",
              "forms": ["turn on"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "TURN",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["turn"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "ON",
              "status": "approved",
              "part_of_speech": "preposition",
              "forms": ["on"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "MAKE SURE",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["make sure"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            }
          ]
        }"#,
    )
    .unwrap()
}

#[test]
fn approved_noun_in_strong_action_position_is_rejected() {
    let result = lint_text(
        "Test the system.",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-GRAM-001")
        .expect("approved noun used as a verb must be diagnosed in a strong action frame");
    assert_eq!(diagnostic.rules, vec!["1.2", "3.7"]);
    let evidence = diagnostic.evidence.as_ref().unwrap();
    assert_eq!(evidence["observed_role"], "verbal");
    assert_eq!(
        evidence["possible_parts_of_speech"],
        serde_json::json!(["noun"])
    );
}

#[test]
fn approved_verb_in_determiner_governed_noun_position_is_rejected() {
    let result = lint_text(
        "The adjust is complete.",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-GRAM-001")
        .expect("approved verb used as a noun must be diagnosed in a determiner frame");
    assert_eq!(diagnostic.rules, vec!["1.2"]);
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["observed_role"],
        "nominal"
    );
}

#[test]
fn compatible_approved_candidate_prevents_false_role_error() {
    let result = lint_text(
        "Clean the surface.",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|item| item.code == "STE-GRAM-001")
    );
}

#[test]
fn explicit_unapproved_multiword_verb_does_not_infer_phrasal_verb_rule() {
    let result = lint_text(
        "Turn on the system.",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-LEX-001" && item.span.start == 0)
        .expect("explicit unapproved multiword verb must retain lexical diagnostic");
    assert_eq!(diagnostic.rules, vec!["1.1"]);
}

#[test]
fn approved_multiword_verb_is_not_treated_as_prohibited_phrasal_verb() {
    let result = lint_text(
        "Make sure the system is ready.",
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|item| item.rules.contains(&"9.3".to_string()))
    );
}
