use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn has_long_multiword_noun(text: &str) -> bool {
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
    .any(|diagnostic| diagnostic.code == "STE-NOUN-001")
}

#[test]
fn four_word_noun_compound_is_reported_without_a_determiner_dependency() {
    assert!(has_long_multiword_noun(
        "Fuel pump pressure sensor is stable."
    ));
}

#[test]
fn homographic_terminal_word_is_not_forced_into_a_noun_compound() {
    assert!(!has_long_multiword_noun("SMALL METAL VALVE COVER."));
}

#[test]
fn three_word_noun_adjective_compound_is_allowed() {
    assert!(!has_long_multiword_noun("METAL VALVE COVER."));
}

#[test]
fn preposition_ends_one_multiword_noun_before_another() {
    assert!(!has_long_multiword_noun("COVER OF THE METAL VALVE UNIT."));
}

#[test]
fn conjunction_breaks_the_compound() {
    assert!(!has_long_multiword_noun("METAL VALVE AND PUMP COVER."));
}

#[test]
fn hyphenated_source_group_counts_as_one_word() {
    assert!(!has_long_multiword_noun("SMALL METAL-VALVE COVER."));
}
