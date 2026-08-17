use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_shared_token_spans"
          },
          "entries": [
            {
              "lemma": "TURN_OFF_VERB",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["TURN OFF"],
              "verb_paradigm": {
                "classification": "lexical",
                "source_sequence": ["TURN OFF"],
                "base_form": "TURN OFF",
                "simple_present_variants": [],
                "simple_past_variants": [],
                "past_participle": null
              },
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "TURN_OFF_NOUN",
              "status": "approved",
              "part_of_speech": "noun",
              "forms": ["TURN OFF"],
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
fn safety_opening_ambiguity_uses_exact_source_span_across_repeated_whitespace() {
    let text = "WARNING: TURN   OFF the unit.";
    let result = lint_text(
        text,
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
        .find(|item| item.code == "STE-SAFE-002")
        .expect("ambiguous multiword safety command must block");

    let start = text.find("TURN").unwrap();
    let end = text.find("OFF").unwrap() + "OFF".len();
    assert_eq!(diagnostic.span.start, start);
    assert_eq!(diagnostic.span.end, end);
}
