use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text, lint_text_with_context};

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::embedded().unwrap()
}

#[test]
fn existing_lint_api_is_unchanged_without_context() {
    let options = LintOptions {
        mode: LintMode::Descriptive,
        fix: false,
    };
    let old = lint_text("USE THIS.", &lexicon(), None, options);
    let context = LintContext::from_json(r#"{"occurrences":[]}"#).unwrap();
    let contextual = lint_text_with_context("USE THIS.", &lexicon(), None, Some(&context), options);
    assert_eq!(old, contextual);
}

#[test]
fn nonapproved_meaning_evidence_enforces_rules_1_3_and_9_2() {
    let context = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 6,
            "source": "human-sense-review",
            "dictionary_meaning": "not_approved"
          }]
        }"#,
    )
    .unwrap();
    let result = lint_text_with_context(
        "FOLLOW",
        &lexicon(),
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-CTX-001")
        .expect("explicit non-approved meaning evidence must be enforced");
    assert_eq!(diagnostic.rules, vec!["1.3", "9.2"]);
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["source"],
        "human-sense-review"
    );
}

#[test]
fn regional_slang_or_jargon_evidence_enforces_rule_1_10() {
    for scope in ["regional", "slang", "jargon"] {
        let context = LintContext::from_json(&format!(
            r#"{{
              "occurrences": [{{
                "start": 0,
                "end": 6,
                "source": "terminology-review",
                "technical_noun_scope": "{scope}"
              }}]
            }}"#
        ))
        .unwrap();
        let result = lint_text_with_context(
            "BUSWAY",
            &lexicon(),
            None,
            Some(&context),
            LintOptions {
                mode: LintMode::Descriptive,
                fix: false,
            },
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|item| item.code == "STE-CTX-002" && item.rules == vec!["1.10"])
        );
    }
}

#[test]
fn nonamerican_spelling_requires_official_name_exception() {
    let bad = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 6,
            "source": "spelling-review",
            "spelling": "non_american"
          }]
        }"#,
    )
    .unwrap();
    let bad_result = lint_text_with_context(
        "COLOUR",
        &lexicon(),
        None,
        Some(&bad),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    assert!(
        bad_result
            .diagnostics
            .iter()
            .any(|item| item.code == "STE-CTX-003" && item.rules == vec!["1.14"])
    );

    let exception = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 6,
            "source": "official-name-register",
            "spelling": "non_american",
            "official_technical_name": true
          }]
        }"#,
    )
    .unwrap();
    let exception_result = lint_text_with_context(
        "COLOUR",
        &lexicon(),
        None,
        Some(&exception),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    assert!(
        !exception_result
            .diagnostics
            .iter()
            .any(|item| item.code == "STE-CTX-003")
    );
}

#[test]
fn context_validation_rejects_invalid_spans_and_missing_source() {
    let invalid_span = LintContext::from_json(
        r#"{"occurrences":[{"start":5,"end":2,"source":"review","dictionary_meaning":"approved"}]}"#,
    )
    .unwrap();
    assert!(invalid_span.validate(10).is_err());

    assert!(
        LintContext::from_json(
            r#"{"occurrences":[{"start":0,"end":2,"source":"","dictionary_meaning":"approved"}]}"#
        )
        .is_err()
    );
}

#[test]
fn explicit_phrasal_verb_evidence_enforces_rule_9_3() {
    let context = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 8,
            "source": "controlled-language review",
            "phrasal_verb": true
          }]
        }"#,
    )
    .unwrap();
    let result = lint_text_with_context(
        "TURN OFF",
        &lexicon(),
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|item| item.code == "STE-PHRASE-001")
        .expect("explicit phrasal-verb classification must be enforced");
    assert_eq!(diagnostic.rules, vec!["9.3"]);
    assert_eq!(diagnostic.evidence.as_ref().unwrap()["source"], "controlled-language review");
}

#[test]
fn phrasal_verb_evidence_requires_a_multiword_span() {
    let context = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 4,
            "source": "controlled-language review",
            "phrasal_verb": true
          }]
        }"#,
    )
    .unwrap();
    let result = lint_text_with_context(
        "TURN",
        &lexicon(),
        None,
        Some(&context),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    assert!(result.diagnostics.iter().any(|item| item.code == "STE-CTX-000"));
}
