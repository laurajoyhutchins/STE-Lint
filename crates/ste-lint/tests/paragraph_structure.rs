use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn has_paragraph_limit(text: &str) -> bool {
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
    .any(|diagnostic| diagnostic.code == "STE-PARA-001")
}

#[test]
fn wrapped_source_lines_remain_one_paragraph() {
    assert!(has_paragraph_limit("USE. USE. USE.\nUSE. USE. USE. USE."));
}

#[test]
fn blank_line_separates_paragraphs() {
    assert!(!has_paragraph_limit(
        "USE. USE. USE. USE.\n\nUSE. USE. USE. USE."
    ));
}

#[test]
fn heading_separates_adjacent_paragraphs() {
    assert!(!has_paragraph_limit(
        "USE. USE. USE. USE.\n\n# HEADING\n\nUSE. USE. USE. USE."
    ));
}

#[test]
fn list_items_are_not_folded_into_the_surrounding_paragraph() {
    let items = (1..=7)
        .map(|index| format!("- USE ITEM {index}."))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!has_paragraph_limit(&format!("USE THESE ITEMS:\n{items}")));
}

#[test]
fn blockquote_prose_keeps_paragraph_identity() {
    assert!(has_paragraph_limit("> USE. USE. USE. USE. USE. USE. USE."));
}

#[test]
fn note_text_is_counted_as_a_descriptive_paragraph() {
    assert!(has_paragraph_limit(
        "NOTE: USE. USE. USE. USE. USE. USE. USE."
    ));
}

#[test]
fn fenced_verbatim_content_is_not_a_prose_paragraph() {
    assert!(!has_paragraph_limit(
        "```text\nUSE. USE. USE. USE. USE. USE. USE.\n```"
    ));
}
