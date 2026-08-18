use std::collections::BTreeMap;

use serde_json::{Value, json};
use ste_benchmark::{
    BenchmarkDiagnostic, BenchmarkResult, Cohort, PageObservation, SourceManifest, SteClaimKind,
    SuiteManifest,
};
use ste_core::{Outcome, Severity, Span};
use ste_lint::LintMode;

fn valid_source() -> SourceManifest {
    serde_json::from_str(include_str!("fixtures/source-valid.json")).unwrap()
}

fn source_index() -> BTreeMap<String, SourceManifest> {
    let source = valid_source();
    BTreeMap::from([(source.id.clone(), source)])
}

#[test]
fn source_manifest_accepts_the_strict_fixture() {
    valid_source().validate().unwrap();
}

#[test]
fn source_manifest_rejects_unknown_fields() {
    let mut value: Value =
        serde_json::from_str(include_str!("fixtures/source-valid.json")).unwrap();
    value["unexpected"] = json!(true);
    assert!(serde_json::from_value::<SourceManifest>(value).is_err());
}

#[test]
fn source_manifest_requires_https_pdf_identity_and_real_retrieval_date() {
    let mut source = valid_source();
    source.url = "http://example.com/manual.pdf".into();
    assert!(source.validate().is_err());

    let mut source = valid_source();
    source.media_type = "text/plain".into();
    assert!(source.validate().is_err());

    let mut source = valid_source();
    source.retrieval_date = "2026-02-30".into();
    assert!(source.validate().is_err());

    let mut source = valid_source();
    source.identity.sha256 = "A".repeat(64);
    assert!(source.validate().is_err());

    let mut source = valid_source();
    source.identity.byte_size = 0;
    assert!(source.validate().is_err());

    let mut source = valid_source();
    source.identity.physical_pages = 0;
    assert!(source.validate().is_err());
}

#[test]
fn explicit_claim_requires_bounded_evidence() {
    let mut source = valid_source();
    source.ste_claim.evidence = None;
    assert!(source.validate().is_err());

    let mut source = valid_source();
    source.ste_claim.evidence.as_mut().unwrap().physical_page = 43;
    assert!(source.validate().is_err());
}

#[test]
fn claim_none_serializes_without_non_ste_semantics() {
    let serialized = serde_json::to_string(&SteClaimKind::None).unwrap();
    assert_eq!(serialized, "\"none\"");
    assert!(!serialized.contains("non_ste"));
}

#[test]
fn source_manifest_rejects_unknown_enum_values() {
    let json = include_str!("fixtures/source-valid.json")
        .replace("\"explicit_ste\"", "\"publisher_says_ste\"");
    assert!(serde_json::from_str::<SourceManifest>(&json).is_err());
}

#[test]
fn suite_accepts_the_strict_fixture() {
    let suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.validate(&source_index()).unwrap();
}

#[test]
fn suite_rejects_unknown_fields_and_enum_values() {
    let mut value: Value = serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    value["unexpected"] = json!(true);
    assert!(serde_json::from_value::<SuiteManifest>(value).is_err());

    let invalid_cohort = include_str!("fixtures/suite-valid.json")
        .replace("\"declared_ste_deep\"", "\"unclassified\"");
    assert!(serde_json::from_str::<SuiteManifest>(&invalid_cohort).is_err());

    let invalid_mode =
        include_str!("fixtures/suite-valid.json").replace("\"procedural\"", "\"mixed\"");
    assert!(serde_json::from_str::<SuiteManifest>(&invalid_mode).is_err());
}

#[test]
fn suite_rejects_zero_based_reversed_or_out_of_bounds_pages() {
    let mut suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections[0].first_page = 0;
    assert!(suite.validate(&source_index()).is_err());

    let mut suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections[0].first_page = 30;
    suite.selections[0].last_page = 29;
    assert!(suite.validate(&source_index()).is_err());

    let mut suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections[0].last_page = 43;
    assert!(suite.validate(&source_index()).is_err());
}

#[test]
fn suite_rejects_unknown_sources_duplicate_ids_and_empty_match_groups() {
    let mut suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections[0].source_id = "missing".into();
    assert!(suite.validate(&source_index()).is_err());

    let mut suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections.push(suite.selections[0].clone());
    assert!(suite.validate(&source_index()).is_err());

    let mut suite: SuiteManifest =
        serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections[0].match_group = Some("   ".into());
    assert!(suite.validate(&source_index()).is_err());
}

#[test]
fn result_contract_is_rights_safe_and_strict() {
    let result = BenchmarkResult {
        schema_version: 1,
        suite_id: "synthetic-suite".into(),
        authoritative_runtime: false,
        pages: vec![PageObservation {
            source_id: "synthetic-source".into(),
            selection_id: "synthetic-selection".into(),
            cohort: Cohort::ClaimNoneControl,
            match_group: Some("pair-01".into()),
            physical_page: 1,
            mode: LintMode::Descriptive,
            normalized_text_sha256: "b".repeat(64),
            normalized_byte_count: 4,
            word_count: 1,
            outcome: Outcome::Error,
            diagnostics: vec![BenchmarkDiagnostic {
                code: "STE-TEST-001".into(),
                severity: Severity::Error,
                rules: vec!["1.1".into()],
                span: Span { start: 0, end: 4 },
            }],
        }],
    };
    result.validate().unwrap();

    let serialized = serde_json::to_string(&result).unwrap();
    assert!(!serialized.contains("\"text\""));
    assert!(!serialized.contains("\"message\""));
    assert!(!serialized.contains("\"evidence\""));
    assert!(!serialized.contains("\"autofix\""));
    assert!(!serialized.contains("\"replacement\""));

    let mut value = serde_json::to_value(result).unwrap();
    value["unexpected"] = json!(true);
    assert!(serde_json::from_value::<BenchmarkResult>(value).is_err());
}

#[test]
fn result_rejects_invalid_page_identity_and_spans() {
    let result = BenchmarkResult {
        schema_version: 1,
        suite_id: "synthetic-suite".into(),
        authoritative_runtime: false,
        pages: vec![PageObservation {
            source_id: "synthetic-source".into(),
            selection_id: "synthetic-selection".into(),
            cohort: Cohort::DeclaredSteBroad,
            match_group: None,
            physical_page: 1,
            mode: LintMode::Procedural,
            normalized_text_sha256: "c".repeat(64),
            normalized_byte_count: 3,
            word_count: 1,
            outcome: Outcome::Error,
            diagnostics: vec![BenchmarkDiagnostic {
                code: "STE-TEST-001".into(),
                severity: Severity::Error,
                rules: Vec::new(),
                span: Span { start: 0, end: 4 },
            }],
        }],
    };
    assert!(result.validate().is_err());
}
