use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_procedural_safety"
          },
          "entries": [
            {
              "lemma": "REMOVE",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["remove", "removes", "removed"],
              "verb_paradigm": {
                "classification": "lexical",
                "source_sequence": ["REMOVE", "REMOVES", "REMOVED", "REMOVED"],
                "base_form": "REMOVE",
                "simple_present_variants": ["REMOVE", "REMOVES"],
                "simple_past_variants": ["REMOVED"],
                "past_participle": "REMOVED"
              },
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "SET_VERB",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["set"],
              "verb_paradigm": {
                "classification": "lexical",
                "source_sequence": ["SET", "SETS", "SET", "SET"],
                "base_form": "SET",
                "simple_present_variants": ["SET", "SETS"],
                "simple_past_variants": ["SET"],
                "past_participle": "SET"
              },
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "SET_NOUN",
              "status": "approved",
              "part_of_speech": "noun",
              "forms": ["set"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            }
          ]
        }"#,
    )
    .unwrap()
}

fn lint(text: &str) -> ste_lint::LintResult {
    lint_text(
        text,
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    )
}

#[test]
fn non_base_approved_verb_at_instruction_start_is_rejected() {
    let result = lint("Removes the panel.");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-PROC-001")
        .expect("non-base instruction verb must be diagnosed");
    assert_eq!(diagnostic.rules, vec!["5.3"]);
    assert_eq!(diagnostic.evidence.as_ref().unwrap()["base_form"], "REMOVE");
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["observed_form"],
        "Removes"
    );
}

#[test]
fn base_form_imperative_is_accepted() {
    let result = lint("Remove the panel.");
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|item| item.code == "STE-PROC-001")
    );
}

#[test]
fn leading_if_condition_requires_comma_before_command() {
    let result = lint("If the valve is open remove the panel.");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-PROC-002")
        .expect("leading condition without comma must be diagnosed");
    assert_eq!(diagnostic.rules, vec!["5.4"]);

    let valid = lint("If the valve is open, remove the panel.");
    assert!(
        !valid
            .diagnostics
            .iter()
            .any(|item| item.code == "STE-PROC-002")
    );
}

#[test]
fn safety_block_must_start_with_command_or_condition() {
    let result = lint("WARNING: The solvent is dangerous. Remove the cover.");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-SAFE-001")
        .expect("safety block without command or condition opening must be diagnosed");
    assert_eq!(diagnostic.rules, vec!["7.2"]);
}

#[test]
fn safety_block_accepts_source_backed_command_or_condition() {
    for text in [
        "WARNING: Remove the cover. The solvent is dangerous.",
        "CAUTION: When the motor operates, remove the cover. Damage can occur.",
    ] {
        let result = lint(text);
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|item| item.code == "STE-SAFE-001" || item.code == "STE-SAFE-002")
        );
    }
}

#[test]
fn safety_command_with_competing_approved_identity_blocks() {
    let result = lint("WARNING: Set the switch.");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-SAFE-002")
        .expect("ambiguous command identity must block rather than guess");
    assert_eq!(diagnostic.rules, vec!["7.2"]);
}
