use ste_core::Severity;
use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn collision(statuses: &[&str]) -> RuntimeLexicon {
    let entries = statuses
        .iter()
        .enumerate()
        .map(|(index, status)| {
            format!(
                r#"{{
                  "lemma": "CHECK_{index}",
                  "status": "{status}",
                  "part_of_speech": "noun",
                  "forms": ["check"],
                  "senses": [],
                  "alternatives": [],
                  "restrictions": []
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        r#"{{
          "metadata": {{
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_collision"
          }},
          "entries": [{entries}]
        }}"#
    );
    RuntimeLexicon::from_json(&json).unwrap()
}

fn phrase_lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_phrases"
          },
          "entries": [
            {
              "lemma": "AWAY FROM",
              "status": "approved",
              "part_of_speech": "preposition",
              "forms": ["AWAY FROM"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "FROM",
              "status": "approved",
              "part_of_speech": "preposition",
              "forms": ["FROM"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "have to",
              "status": "unapproved",
              "part_of_speech": "verb",
              "forms": ["have to"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "HAVE",
              "status": "approved",
              "part_of_speech": "verb",
              "forms": ["HAVE"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "TO",
              "status": "approved",
              "part_of_speech": "preposition",
              "forms": ["TO"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            }
          ]
        }"#,
    )
    .unwrap()
}

fn diagnostics_for(text: &str, lexicon: &RuntimeLexicon) -> Vec<ste_core::Diagnostic> {
    lint_text(
        text,
        lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
    .diagnostics
}

fn diagnostics(lexicon: &RuntimeLexicon) -> Vec<ste_core::Diagnostic> {
    diagnostics_for("check", lexicon)
}

#[test]
fn all_approved_form_candidates_are_lexically_accepted() {
    let lexicon = collision(&["approved", "approved"]);
    let diagnostics = diagnostics(&lexicon);
    assert!(!diagnostics.iter().any(|item| item.code == "STE-TERM-001"));
    assert!(!diagnostics.iter().any(|item| item.code == "STE-LEX-001"));
    assert!(!diagnostics.iter().any(|item| item.code == "STE-LEX-002"));
}

#[test]
fn all_unapproved_form_candidates_emit_unapproved_word_error() {
    let lexicon = collision(&["unapproved", "unapproved"]);
    let diagnostics = diagnostics(&lexicon);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "STE-LEX-001")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(!diagnostics.iter().any(|item| item.code == "STE-TERM-001"));
}

#[test]
fn mixed_approval_form_candidates_block_for_disambiguation() {
    let lexicon = collision(&["approved", "unapproved"]);
    let diagnostics = diagnostics(&lexicon);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "STE-LEX-002")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Blocked);
    assert!(!diagnostics.iter().any(|item| item.code == "STE-TERM-001"));
}

#[test]
fn approved_phrase_suppresses_unknown_component_diagnostics() {
    let lexicon = phrase_lexicon();
    let diagnostics = diagnostics_for("away from", &lexicon);
    assert!(diagnostics.is_empty());
}

#[test]
fn unapproved_phrase_is_detected_even_when_components_are_approved() {
    let lexicon = phrase_lexicon();
    let diagnostics = diagnostics_for("have to", &lexicon);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "STE-LEX-001")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.span.start, 0);
    assert_eq!(diagnostic.span.end, 7);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn phrase_matching_does_not_cross_sentence_punctuation() {
    let lexicon = phrase_lexicon();
    let diagnostics = diagnostics_for("have. to", &lexicon);
    assert!(diagnostics.is_empty());
}
