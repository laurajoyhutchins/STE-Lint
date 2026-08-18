use ste_core::Severity;
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lint_dictionary(text: &str, lexicon: &RuntimeLexicon, mode: LintMode) -> Vec<ste_core::Diagnostic> {
    lint_text(text, lexicon, None, LintOptions { mode, fix: false }).diagnostics
}

fn role_lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata":{"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_role_resolution"},
          "entries":[
            {"lemma":"CHECK","status":"approved","part_of_speech":"noun","forms":["CHECK"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CLEAR","status":"approved","part_of_speech":"noun","forms":["CLEAR"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"VALVE","status":"approved","part_of_speech":"noun","forms":["VALVE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"UNIT","status":"approved","part_of_speech":"noun","forms":["UNIT"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["IS"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"COMPLETE","status":"approved","part_of_speech":"adjective","forms":["COMPLETE"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}

#[test]
fn rule_1_2_rejects_approved_noun_used_as_imperative_verb() {
    let diagnostics = lint_dictionary("CHECK THE VALVE.", &role_lexicon(), LintMode::Procedural);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-GRAM-001")
        .expect("noun CHECK used as a verb must be rejected");
    assert!(diagnostic.rules.iter().any(|rule| rule == "1.2"));
    assert_eq!(diagnostic.evidence.as_ref().unwrap()["observed_role"], "a verb");
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["role_basis"],
        "harper_brill_pos_tag"
    );
}

#[test]
fn rule_1_2_accepts_same_spelling_in_its_approved_noun_role() {
    let diagnostics = lint_dictionary(
        "THE CHECK IS COMPLETE.",
        &role_lexicon(),
        LintMode::Descriptive,
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-GRAM-001")
    );
}

#[test]
fn rule_1_2_checks_adjective_occurrence_role_not_only_noun_and_verb_frames() {
    let diagnostics = lint_dictionary(
        "THE UNIT IS CLEAR.",
        &role_lexicon(),
        LintMode::Descriptive,
    );
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-GRAM-001")
        .expect("noun-only CLEAR used as adjective must be rejected");
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["observed_role"],
        "an adjective"
    );
}

fn technical_glossary() -> Glossary {
    Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"synthetic-role-resolution",
          "sources":{"fixture":{"title":"Synthetic role fixture"}},
          "terms":[
            {
              "id":"busway",
              "canonical":"busway",
              "roles":["noun"],
              "definition":"Synthetic governed technical noun.",
              "forms":[],
              "aliases":[],
              "sources":[{"source":"fixture","supports":["admission","definition","role","forms","alias","status"]}],
              "status":"approved"
            },
            {
              "id":"torque",
              "canonical":"torque",
              "roles":["verb"],
              "definition":"Synthetic governed technical verb.",
              "forms":[],
              "aliases":[],
              "sources":[{"source":"fixture","supports":["admission","definition","role","forms","alias","status"]}],
              "status":"approved"
            }
          ]
        }"#,
    )
    .unwrap()
}

#[test]
fn rule_1_7_rejects_governed_technical_noun_used_as_verb() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = technical_glossary();
    let diagnostics = lint_text(
        "BUSWAY THE FEED.",
        &lexicon,
        Some(&glossary),
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    )
    .diagnostics;
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-TERM-003")
        .expect("technical noun used as a verb must be rejected or the parse must block");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.rules, vec!["1.7"]);
}

#[test]
fn rule_1_13_rejects_governed_technical_verb_used_as_noun() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = technical_glossary();
    let diagnostics = lint_text(
        "THE TORQUE IS STABLE.",
        &lexicon,
        Some(&glossary),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
    .diagnostics;
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-TERM-004")
        .expect("technical verb used as a noun must be rejected");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.rules, vec!["1.13"]);
}
