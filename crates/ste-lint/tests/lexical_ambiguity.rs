use ste_core::Severity;
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
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

fn empty_lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_empty"
          },
          "entries": []
        }"#,
    )
    .unwrap()
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

fn glossary(status: &str) -> Glossary {
    Glossary::from_json(&format!(
        r#"{{
          "terms": [{{
            "term": "bus duct",
            "kind": "technical_noun",
            "definition": "Synthetic project term.",
            "domain": "electrical",
            "preferred": true,
            "aliases": [],
            "examples": [],
            "provenance": ["fixture"],
            "status": "{status}"
          }}]
        }}"#
    ))
    .unwrap()
}

fn check_glossary(status: &str) -> Glossary {
    Glossary::from_json(&format!(
        r#"{{
          "terms": [{{
            "term": "check",
            "kind": "technical_noun",
            "definition": "Synthetic technical noun.",
            "domain": "inspection",
            "preferred": true,
            "aliases": [],
            "examples": [],
            "provenance": ["fixture"],
            "status": "{status}"
          }}]
        }}"#
    ))
    .unwrap()
}

fn diagnostics_with_glossary(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
) -> Vec<ste_core::Diagnostic> {
    lint_text(
        text,
        lexicon,
        glossary,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
    .diagnostics
}

fn diagnostics_for(text: &str, lexicon: &RuntimeLexicon) -> Vec<ste_core::Diagnostic> {
    diagnostics_with_glossary(text, lexicon, None)
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

#[test]
fn approved_multiword_glossary_term_suppresses_component_unknowns() {
    let lexicon = empty_lexicon();
    let glossary = glossary("approved");
    let diagnostics = diagnostics_with_glossary("bus duct", &lexicon, Some(&glossary));
    assert!(diagnostics.is_empty());
}

#[test]
fn deprecated_multiword_glossary_term_is_rejected_as_one_phrase() {
    let lexicon = empty_lexicon();
    let glossary = glossary("deprecated");
    let diagnostics = diagnostics_with_glossary("bus duct", &lexicon, Some(&glossary));
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "STE-TERM-002")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.span.start, 0);
    assert_eq!(diagnostic.span.end, 8);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn approved_technical_term_overrides_unapproved_general_dictionary_record() {
    let lexicon = collision(&["unapproved"]);
    let glossary = check_glossary("approved");
    let diagnostics = diagnostics_with_glossary("check", &lexicon, Some(&glossary));
    assert!(diagnostics.is_empty());
}

#[test]
fn deprecated_governed_term_overrides_general_dictionary_status() {
    let lexicon = collision(&["approved"]);
    let glossary = check_glossary("deprecated");
    let diagnostics = diagnostics_with_glossary("check", &lexicon, Some(&glossary));
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "STE-TERM-002")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostics.len(), 1);
}
