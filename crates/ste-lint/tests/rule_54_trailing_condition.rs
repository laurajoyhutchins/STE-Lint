use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, LintResult, lint_text};

fn lexicon(ambiguous_condition: bool, ambiguous_command: bool) -> RuntimeLexicon {
    let mut entries = vec![
        serde_json::json!({
            "lemma": "REMOVE",
            "status": "approved",
            "part_of_speech": "verb",
            "forms": ["REMOVE", "REMOVES", "REMOVED"],
            "verb_paradigm": {
                "classification": "lexical",
                "source_sequence": ["REMOVE", "REMOVES", "REMOVED", "REMOVED"],
                "base_form": "REMOVE",
                "simple_present_variants": ["REMOVE", "REMOVES"],
                "simple_past_variants": ["REMOVED"],
                "past_participle": "REMOVED"
            },
            "senses": [], "alternatives": [], "restrictions": []
        }),
        serde_json::json!({
            "lemma": "IF", "status": "approved", "part_of_speech": "conjunction",
            "forms": ["IF"], "senses": [], "alternatives": [], "restrictions": []
        }),
        serde_json::json!({
            "lemma": "WHEN", "status": "approved", "part_of_speech": "conjunction",
            "forms": ["WHEN"], "senses": [], "alternatives": [], "restrictions": []
        }),
    ];
    if ambiguous_condition {
        entries.push(serde_json::json!({
            "lemma": "WHEN", "status": "approved", "part_of_speech": "noun",
            "forms": ["WHEN"], "senses": [], "alternatives": [], "restrictions": []
        }));
    }
    if ambiguous_command {
        entries.push(serde_json::json!({
            "lemma": "REMOVE", "status": "approved", "part_of_speech": "noun",
            "forms": ["REMOVE"], "senses": [], "alternatives": [], "restrictions": []
        }));
    }

    RuntimeLexicon::from_json(
        &serde_json::json!({
            "metadata": {
                "standard": "ASD-STE100", "issue": 9,
                "date": "2025-01-15", "scope": "synthetic_rule_54_condition_order"
            },
            "entries": entries
        })
        .to_string(),
    )
    .unwrap()
}

fn lint(text: &str, mode: LintMode, ambiguous_condition: bool, ambiguous_command: bool) -> LintResult {
    let runtime = lexicon(ambiguous_condition, ambiguous_command);
    lint_text(
        text,
        &runtime,
        None,
        LintOptions { mode, fix: false },
    )
}

fn trailing<'a>(result: &'a LintResult) -> Vec<&'a ste_core::Diagnostic> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "STE-PROC-005")
        .collect()
}

#[test]
fn command_followed_by_if_condition_is_reported_with_source_coordinates() {
    let text = "REMOVE POWER IF READY.";
    let result = lint(text, LintMode::Procedural, false, false);
    let diagnostics = trailing(&result);

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = diagnostics[0];
    assert_eq!(diagnostic.rules, vec!["5.4"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (13, 15));
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["grammar_resolution"],
        "uniquely_approved_imperative_and_conjunction"
    );
    assert!(diagnostic.autofix.is_none());
}

#[test]
fn command_followed_by_when_condition_is_reported() {
    let result = lint("REMOVE POWER WHEN READY.", LintMode::Procedural, false, false);
    assert_eq!(trailing(&result).len(), 1);
}

#[test]
fn leading_condition_is_not_reported_as_trailing() {
    let result = lint("IF READY, REMOVE POWER.", LintMode::Procedural, false, false);
    assert!(trailing(&result).is_empty());
}

#[test]
fn descriptive_text_does_not_use_procedural_ordering() {
    let result = lint("REMOVE POWER IF READY.", LintMode::Descriptive, false, false);
    assert!(trailing(&result).is_empty());
}

#[test]
fn ambiguous_condition_word_is_not_assumed_to_be_a_conjunction() {
    let result = lint("REMOVE POWER WHEN READY.", LintMode::Procedural, true, false);
    assert!(trailing(&result).is_empty());
}

#[test]
fn ambiguous_command_identity_does_not_create_ordering_verdict() {
    let result = lint("REMOVE POWER IF READY.", LintMode::Procedural, false, true);
    assert!(trailing(&result).is_empty());
}

#[test]
fn code_span_condition_marker_is_not_prose_evidence() {
    let result = lint("REMOVE POWER `IF READY`.", LintMode::Procedural, false, false);
    assert!(trailing(&result).is_empty());
}

#[test]
fn quoted_word_is_not_assumed_to_introduce_a_condition() {
    let result = lint("REMOVE \"WHEN READY\" LABEL.", LintMode::Procedural, false, false);
    assert!(trailing(&result).is_empty());
}

#[test]
fn utf8_prefix_preserves_condition_byte_span() {
    let text = "ÉQUIPEMENT.\nREMOVE POWER IF READY.";
    let result = lint(text, LintMode::Procedural, false, false);
    let diagnostics = trailing(&result);
    assert_eq!(diagnostics.len(), 1);
    let expected = text.find("IF READY").unwrap();
    assert_eq!((diagnostics[0].span.start, diagnostics[0].span.end), (expected, expected + 2));
}

#[test]
fn condition_word_without_a_following_clause_is_not_diagnosed() {
    let result = lint("REMOVE IF.", LintMode::Procedural, false, false);
    assert!(trailing(&result).is_empty());
}
