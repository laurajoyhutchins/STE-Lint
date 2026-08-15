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
fn parenthetical_group_counts_as_one_outer_word() {
    let mut words = vec!["USE"; 19];
    words.push("(THIS HAS MANY INNER WORDS)");
    let text = words.join(" ");
    assert!(!codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn hyphenated_group_counts_as_one_word() {
    let mut words = vec!["USE"; 19];
    words.push("soap-and-water");
    let text = words.join(" ");
    assert!(!codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn decimal_point_does_not_split_a_sentence() {
    let text = "USE 0.24 THEN USE THIS.";
    let result = codes(text, LintMode::Procedural);
    assert!(!result.contains(&"STE-LEN-001".to_string()));
}

#[test]
fn vertical_list_prefix_and_items_are_independent_sentence_units() {
    let prefix = vec!["USE"; 20].join(" ");
    let item = vec!["USE"; 20].join(" ");
    let text = format!("{prefix}:\n- {item}\n- {item}");
    assert!(!codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn vertical_list_item_over_limit_is_reported() {
    let prefix = vec!["USE"; 20].join(" ");
    let item = vec!["USE"; 21].join(" ");
    let text = format!("{prefix}:\n- {item}");
    assert!(codes(&text, LintMode::Procedural).contains(&"STE-LEN-001".to_string()));
}

#[test]
fn descriptive_paragraph_over_six_sentences_is_reported() {
    let text = "USE. USE. USE. USE. USE. USE. USE.";
    assert!(codes(text, LintMode::Descriptive).contains(&"STE-PARA-001".to_string()));
}

#[test]
fn six_sentence_descriptive_paragraph_is_allowed() {
    let text = "USE. USE. USE. USE. USE. USE.";
    assert!(!codes(text, LintMode::Descriptive).contains(&"STE-PARA-001".to_string()));
}

#[test]
fn contraction_is_reported_but_possessive_is_not() {
    assert!(codes("IT'S READY.", LintMode::Descriptive).contains(&"STE-SYN-001".to_string()));
    assert!(
        !codes("THE ENGINE'S COVER IS OPEN.", LintMode::Descriptive)
            .contains(&"STE-SYN-001".to_string())
    );
}

#[test]
fn ambiguous_d_contraction_is_reported_without_autofix() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text(
        "WE'D USE THIS.",
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: true,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-SYN-001")
        .unwrap();
    assert!(diagnostic.autofix.is_none());
    assert_eq!(result.text, "WE'D USE THIS.");
}
