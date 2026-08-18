use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, lint_text};

fn codes(text: &str, mode: LintMode) -> Vec<String> {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text(text, &lexicon, None, LintOptions { mode, fix: false })
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn procedural_prefix_and_each_nested_item_are_independent_twenty_word_units() {
    let prefix = vec!["USE"; 20].join(" ");
    let outer = vec!["USE"; 20].join(" ");
    let nested = vec!["USE"; 20].join(" ");
    let sibling = vec!["USE"; 20].join(" ");
    let text = format!("{prefix}:\n- {outer}:\n  - {nested}\n- {sibling}");
    assert!(!codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn procedural_nested_item_over_twenty_words_is_reported() {
    let prefix = vec!["USE"; 20].join(" ");
    let outer = vec!["USE"; 20].join(" ");
    let nested = vec!["USE"; 21].join(" ");
    let text = format!("{prefix}:\n- {outer}:\n  - {nested}");
    assert!(codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn descriptive_prefix_and_nested_items_use_twenty_five_word_units() {
    let prefix = vec!["USE"; 25].join(" ");
    let outer = vec!["USE"; 25].join(" ");
    let nested = vec!["USE"; 25].join(" ");
    let text = format!("{prefix}:\n- {outer}:\n  - {nested}");
    assert!(!codes(&text, LintMode::Descriptive).contains(&"STE-LEN-002".to_string()));
}

#[test]
fn wrapped_list_item_is_one_count_unit_not_one_unit_per_source_line() {
    let first = ["USE"; 10].join(" ");
    let continuation = ["USE"; 11].join(" ");
    let text = format!("USE:\n- {first}\n  {continuation}");
    assert!(codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn legacy_ste_list_markers_use_the_same_colon_boundary_model() {
    let prefix = vec!["USE"; 20].join(" ");
    let first = vec!["USE"; 20].join(" ");
    let second = vec!["USE"; 20].join(" ");
    let text = format!("{prefix}:\n(a) {first}\n(b) {second}");
    assert!(!codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}
