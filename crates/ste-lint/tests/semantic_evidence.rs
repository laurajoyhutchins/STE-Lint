use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn semantic_collision() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_semantic_collision"
          },
          "entries": [
            {
              "lemma": "CHECK_NOUN",
              "status": "approved",
              "part_of_speech": "noun",
              "forms": ["check"],
              "senses": [{"meaning": "An inspection."}],
              "alternatives": [],
              "restrictions": ["Use only for the inspection sense."],
              "interpretation_state": "interpreted",
              "provenance": {"structural_record_index": 7, "source_pages": [150]}
            },
            {
              "lemma": "check_verb",
              "status": "unapproved",
              "part_of_speech": "verb",
              "forms": ["check"],
              "senses": [],
              "alternatives": [{
                "kind": "approved_word",
                "text": "INSPECT",
                "part_of_speech": "verb",
                "strategy": "word_replacement"
              }],
              "restrictions": ["Use INSPECT for this action."],
              "interpretation_state": "structural",
              "provenance": {"structural_record_index": 8, "source_pages": [151]}
            }
          ]
        }"#,
    )
    .unwrap()
}

#[test]
fn mixed_dictionary_identity_exposes_pos_role_and_semantic_evidence() {
    let lexicon = semantic_collision();
    let result = lint_text(
        "check",
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-LEX-002")
        .expect("mixed approval identity must block for disambiguation");
    let evidence = diagnostic.evidence.as_ref().unwrap();

    assert_eq!(evidence["requires_disambiguation"], true);
    assert_eq!(evidence["possible_parts_of_speech"], serde_json::json!(["noun", "verb"]));
    assert_eq!(evidence["role_evidence"], serde_json::json!(["nominal", "verbal"]));

    let candidates = evidence["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0]["senses"][0]["meaning"], "An inspection.");
    assert_eq!(
        candidates[0]["restrictions"][0],
        "Use only for the inspection sense."
    );
    assert_eq!(candidates[0]["interpretation_state"], "interpreted");
    assert_eq!(candidates[0]["provenance"]["source_pages"], serde_json::json!([150]));

    assert_eq!(candidates[1]["alternatives"][0]["text"], "INSPECT");
    assert_eq!(candidates[1]["alternatives"][0]["strategy"], "word_replacement");
    assert_eq!(candidates[1]["interpretation_state"], "structural");
    assert_eq!(candidates[1]["provenance"]["source_pages"], serde_json::json!([151]));
}
