use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

fn lint(text: &str, context: Option<&LintContext>) -> ste_lint::LintResult {
    lint_text_with_context(
        text,
        &RuntimeLexicon::embedded().unwrap(),
        None,
        context,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
}

fn has_code(result: &ste_lint::LintResult, code: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn explicit_proper_noun_group_counts_as_one_word_for_rule_8_6() {
    let prefix = vec!["USE"; 22].join(" ");
    let phrase = "North Atlantic Treaty Organization";
    let text = format!("{prefix} {phrase}.");
    let start = text.find(phrase).unwrap();
    let end = start + phrase.len();

    assert!(has_code(&lint(&text, None), "STE-LEN-002"));

    let context = LintContext::from_json(&format!(
        r#"{{
          "occurrences": [{{
            "start": {start},
            "end": {end},
            "source": "document identity review",
            "count_group": "proper_noun"
          }}]
        }}"#
    ))
    .unwrap();
    assert!(!has_code(&lint(&text, Some(&context)), "STE-LEN-002"));
}

#[test]
fn explicit_unrelated_hyphen_use_violates_rule_8_2() {
    let text = "power-operated";
    let context = LintContext::from_json(&format!(
        r#"{{
          "occurrences": [{{
            "start": 0,
            "end": {},
            "source": "editor review",
            "hyphen_direct_relation": false
          }}]
        }}"#,
        text.len()
    ))
    .unwrap();

    let result = lint(text, Some(&context));
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-PUNC-002")
        .expect("explicit unrelated hyphen evidence must be enforced");
    assert_eq!(diagnostic.rules, vec!["8.2"]);
}

#[test]
fn explicit_other_parenthesis_use_violates_rule_8_3() {
    let text = "(marketing flourish)";
    let context = LintContext::from_json(&format!(
        r#"{{
          "occurrences": [{{
            "start": 0,
            "end": {},
            "source": "editor review",
            "parenthesis_use": "other"
          }}]
        }}"#,
        text.len()
    ))
    .unwrap();

    let result = lint(text, Some(&context));
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-PUNC-003")
        .expect("explicit non-listed parenthesis use must be enforced");
    assert_eq!(diagnostic.rules, vec!["8.3"]);
}

#[test]
fn listed_parenthesis_use_is_allowed() {
    let text = "(alternate)";
    let context = LintContext::from_json(&format!(
        r#"{{
          "occurrences": [{{
            "start": 0,
            "end": {},
            "source": "editor review",
            "parenthesis_use": "alternative"
          }}]
        }}"#,
        text.len()
    ))
    .unwrap();

    assert!(!has_code(&lint(text, Some(&context)), "STE-PUNC-003"));
}
