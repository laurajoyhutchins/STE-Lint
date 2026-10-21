use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, LintResult, lint_text_with_context};

fn glossary() -> Glossary {
    Glossary::from_json(
        r#"{
            "schema":"ste-terminology/v2",
            "domain":"synthetic_electrical",
            "sources":{"test-source":{"title":"Synthetic project terminology authority"}},
            "terms":[
                {
                    "id":"busway",
                    "canonical":"busway",
                    "roles":["noun"],
                    "definition":"A synthetic busway.",
                    "aliases":[
                        {"text":"bus duct","kind":"synonym"},
                        {"text":"BWY","kind":"abbreviation"},
                        {"text":"the bus","kind":"short_form"}
                    ],
                    "forms":[{"text":"busways","roles":["noun"]}],
                    "sources":[{"source":"test-source","supports":["admission","definition","role","alias","forms","status"]}],
                    "status":"approved"
                },
                {
                    "id":"switchgear",
                    "canonical":"switchgear",
                    "roles":["noun"],
                    "definition":"Synthetic switchgear.",
                    "sources":[{"source":"test-source","supports":["admission","definition","role","status"]}],
                    "status":"approved"
                }
            ]
        }"#,
    )
    .unwrap()
}

fn lint(text: &str, context: Option<&LintContext>) -> LintResult {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = glossary();
    lint_text_with_context(
        text,
        &lexicon,
        Some(&glossary),
        context,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
}

fn terminology_diagnostics(result: &LintResult) -> Vec<&ste_core::Diagnostic> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "STE-TERM-005")
        .collect()
}

#[test]
fn governed_synonym_change_for_same_entity_is_reported() {
    let text = "THE BUSWAY IS READY. THE BUS DUCT IS READY.";
    let result = lint(text, None);
    let diagnostics = terminology_diagnostics(&result);

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = diagnostics[0];
    assert_eq!(diagnostic.rules, vec!["9.4"]);
    let second = text.find("BUS DUCT").unwrap();
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (second, second + 8));
    assert_eq!(
        diagnostic.evidence.as_ref().unwrap()["resolution"],
        "governed_concept_different_synonyms"
    );
    assert!(diagnostic.autofix.is_none());
}

#[test]
fn reverse_order_still_reports_only_the_later_conflicting_form() {
    let text = "THE BUS DUCT IS READY. THE BUSWAY IS READY.";
    let result = lint(text, None);
    let diagnostics = terminology_diagnostics(&result);

    assert_eq!(diagnostics.len(), 1);
    let second = text.rfind("BUSWAY").unwrap();
    assert_eq!((diagnostics[0].span.start, diagnostics[0].span.end), (second, second + 6));
}

#[test]
fn repeated_same_term_is_not_a_false_positive() {
    let result = lint("THE BUSWAY IS READY. THE BUSWAY IS INSTALLED.", None);
    assert!(terminology_diagnostics(&result).is_empty());
}

#[test]
fn short_forms_abbreviations_and_inflections_remain_exempt() {
    let result = lint("THE BUSWAY IS READY. THE BWY IS READY. THE BUSWAYS ARE READY.", None);
    assert!(terminology_diagnostics(&result).is_empty());
}

#[test]
fn different_governed_entities_do_not_collide() {
    let result = lint("THE BUSWAY IS READY. THE SWITCHGEAR IS READY.", None);
    assert!(terminology_diagnostics(&result).is_empty());
}

#[test]
fn protected_code_and_explicit_external_evidence_do_not_trigger_drift() {
    let text = "THE BUSWAY IS READY. `BUS DUCT` IS A CODE TOKEN.";
    let result = lint(text, None);
    assert!(terminology_diagnostics(&result).is_empty());

    let text = "THE BUSWAY IS READY. BUS DUCT IS AN EXTERNAL LABEL.";
    let start = text.find("BUS DUCT").unwrap();
    let end = start + "BUS DUCT".len();
    let context = LintContext::from_json(
        &serde_json::json!({
            "occurrences":[{
                "start": start,
                "end": end,
                "source":"immutable manufacturer term",
                "text_authority":"quoted_external_text"
            }]
        })
        .to_string(),
    )
    .unwrap();
    context.validate(text.len()).unwrap();
    let result = lint(text, Some(&context));
    assert!(terminology_diagnostics(&result).is_empty());
}

#[test]
fn ambiguous_competing_official_name_authority_does_not_force_a_verdict() {
    let text = "THE BUSWAY IS READY. THE BUS DUCT IS READY.";
    let start = text.find("BUS DUCT").unwrap();
    let context = LintContext::from_json(
        &serde_json::json!({
            "occurrences":[{
                "start":start,
                "end":start + 8,
                "source":"official name registry",
                "official_technical_name":true
            }]
        })
        .to_string(),
    )
    .unwrap();
    context.validate(text.len()).unwrap();

    let result = lint(text, Some(&context));
    assert!(terminology_diagnostics(&result).is_empty());
}

#[test]
fn unicode_prefix_retains_byte_accurate_diagnostic() {
    let text = "É. THE BUSWAY IS READY. THE BUS DUCT IS READY.";
    let result = lint(text, None);
    let second = text.find("BUS DUCT").unwrap();
    let diagnostics = terminology_diagnostics(&result);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!((diagnostics[0].span.start, diagnostics[0].span.end), (second, second + 8));
}

#[test]
fn no_project_glossary_means_no_synthetic_identity_inference() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let result = lint_text_with_context(
        "THE BUSWAY IS READY. THE BUS DUCT IS READY.",
        &lexicon,
        None,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );
    assert!(terminology_diagnostics(&result).is_empty());
}
