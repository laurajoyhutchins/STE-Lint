use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

#[test]
fn generic_determiner_tag_does_not_manufacture_ste_article_identity() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text(
        "WARNING: THIS CB-1.",
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        },
    );

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-GRAM-001"),
        "generic DET evidence must not override the runtime's approved pronoun identity"
    );
}
