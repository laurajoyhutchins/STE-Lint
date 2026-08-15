use ste_core::Severity;
use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_procedural_structure"},
          "entries": [
            {"lemma":"REMOVE","status":"approved","part_of_speech":"verb","forms":["REMOVE","REMOVES","REMOVED"],"verb_paradigm":{"classification":"lexical","source_sequence":["REMOVE","REMOVES","REMOVED","REMOVED"],"base_form":"REMOVE","simple_present_variants":["REMOVES"],"simple_past_variants":["REMOVED"],"past_participle":"REMOVED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CHECK","status":"approved","part_of_speech":"verb","forms":["CHECK","CHECKS","CHECKED"],"verb_paradigm":{"classification":"lexical","source_sequence":["CHECK","CHECKS","CHECKED","CHECKED"],"base_form":"CHECK","simple_present_variants":["CHECKS"],"simple_past_variants":["CHECKED"],"past_participle":"CHECKED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CHECK","status":"approved","part_of_speech":"noun","forms":["CHECK"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}

fn lint(text: &str) -> Vec<ste_core::Diagnostic> {
    lint_text(
        text,
        &lexicon(),
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    )
    .diagnostics
}

fn has_code(diagnostics: &[ste_core::Diagnostic], code: &str) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.code == code)
}

#[test]
fn imperative_at_start_of_note_is_an_error() {
    let diagnostics = lint("NOTE: REMOVE THE COVER.");
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-NOTE-001")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(diagnostic.autofix.is_none());
}

#[test]
fn note_is_recognized_without_a_blank_line_before_it() {
    let diagnostics = lint("REMOVE THE COVER.\nNOTE: REMOVE THE SEAL.");
    assert!(has_code(&diagnostics, "STE-NOTE-001"));
}

#[test]
fn indented_note_continuation_stays_inside_note_block() {
    let diagnostics = lint("NOTE: THE UNIT IS STABLE.\n  REMOVE THE SEAL.\nREMOVE THE COVER.");
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "STE-NOTE-001")
            .count(),
        1
    );
}

#[test]
fn ambiguous_note_initial_form_blocks_instead_of_guessing() {
    let diagnostics = lint("NOTE: CHECK THE INDICATION.");
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-NOTE-002")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Blocked);
}

#[test]
fn descriptive_note_does_not_get_an_imperative_diagnostic() {
    let diagnostics = lint("NOTE: THE UNIT IS STABLE.");
    assert!(!has_code(&diagnostics, "STE-NOTE-001"));
    assert!(!has_code(&diagnostics, "STE-NOTE-002"));
}

#[test]
fn note_in_procedure_uses_twenty_five_word_limit_not_twenty() {
    let note_23 = format!("NOTE: {}.", vec!["ALPHA"; 23].join(" "));
    let diagnostics = lint(&note_23);
    assert!(!has_code(&diagnostics, "STE-LEN-001"));
    assert!(!has_code(&diagnostics, "STE-LEN-002"));

    let note_26 = format!("NOTE: {}.", vec!["ALPHA"; 26].join(" "));
    let diagnostics = lint(&note_26);
    assert!(!has_code(&diagnostics, "STE-LEN-001"));
    assert!(has_code(&diagnostics, "STE-LEN-002"));
}

#[test]
fn simple_vertical_list_requires_colon_before_first_item() {
    let diagnostics = lint("REMOVE THESE PARTS.\n- The cover\n- The seal.");
    assert!(has_code(&diagnostics, "STE-LIST-001"));
}

#[test]
fn simple_vertical_list_items_start_with_uppercase() {
    let diagnostics = lint("REMOVE THESE PARTS:\n- the cover\n- The seal.");
    assert!(has_code(&diagnostics, "STE-LIST-002"));
}

#[test]
fn simple_vertical_list_items_cannot_end_with_comma_or_semicolon() {
    let diagnostics = lint("REMOVE THESE PARTS:\n- The cover,\n- The seal;\n- The bolt.");
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "STE-LIST-003")
            .count(),
        2
    );
}

#[test]
fn last_simple_vertical_list_item_requires_period() {
    let diagnostics = lint("REMOVE THESE PARTS:\n- The cover\n- The seal");
    assert!(has_code(&diagnostics, "STE-LIST-004"));
}

#[test]
fn well_formed_simple_vertical_list_has_no_list_diagnostics() {
    let diagnostics = lint("REMOVE THESE PARTS:\n- The cover\n- The seal.");
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.code.starts_with("STE-LIST-"))
    );
}
