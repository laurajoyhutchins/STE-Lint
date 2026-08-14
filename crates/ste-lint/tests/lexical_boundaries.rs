use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn has_code(text: &str, code: &str) -> bool {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text(
        text,
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
    .diagnostics
    .iter()
    .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn sentence_final_punctuation_does_not_hide_unapproved_words() {
    assert!(has_code("acceptable.", "STE-LEX-001"));
}

#[test]
fn sentence_final_punctuation_does_not_hide_unknown_terms() {
    assert!(has_code("fluxcapacitor.", "STE-TERM-001"));
}
