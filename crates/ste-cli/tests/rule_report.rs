use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn run_lint(path: &std::path::Path) -> (std::process::Output, serde_json::Value) {
    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            path.to_str().unwrap(),
            "--format",
            "json",
            "--mode",
            "procedural",
            "--rule-report",
        ])
        .output()
        .unwrap();
    let payload = serde_json::from_slice(&output.stdout).unwrap();
    (output, payload)
}

fn rule<'a>(payload: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    payload["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap()
}

#[test]
fn rule_report_retains_lint_diagnostics_and_distinguishes_unchecked_rules() {
    let (output, payload) = run_lint(&fixtures().join("lint/semicolon.txt"));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert_eq!(payload["schema"], "ste-lint-rule-report/v1");
    assert_eq!(payload["full_compliance_claimed"], false);
    assert_eq!(payload["rules"].as_array().unwrap().len(), 53);
    assert!(
        payload["lint"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "STE-PUNC-001")
    );

    assert_eq!(rule(&payload, "8.1")["assessment"], "violation_detected");
    assert_eq!(
        rule(&payload, "8.1")["observed_diagnostic_codes"],
        serde_json::json!(["STE-PUNC-001"])
    );
    assert_eq!(rule(&payload, "5.4")["assessment"], "unresolved_coverage");
    assert_eq!(rule(&payload, "7.3")["assessment"], "not_evaluated");
    assert!(
        !rule(&payload, "5.4")["unresolved_requirements"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn clean_implemented_slice_does_not_clear_partial_or_context_required_rules() {
    let (output, payload) = run_lint(&fixtures().join("corpus/clean-control.txt"));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(rule(&payload, "8.1")["assessment"], "checked_within_scope");
    assert_eq!(rule(&payload, "5.4")["assessment"], "unresolved_coverage");
    assert_eq!(rule(&payload, "7.3")["assessment"], "not_evaluated");
    assert_eq!(payload["full_compliance_claimed"], false);
}

#[test]
fn implemented_rules_not_applicable_to_the_requested_mode_are_not_reported_as_checked() {
    let (output, procedural) = run_lint(&fixtures().join("corpus/clean-control.txt"));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        rule(&procedural, "5.1")["assessment"],
        "checked_within_scope"
    );
    assert_eq!(rule(&procedural, "6.3")["assessment"], "not_evaluated");
    assert_eq!(rule(&procedural, "6.6")["assessment"], "not_evaluated");

    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            fixtures()
                .join("corpus/clean-control.txt")
                .to_str()
                .unwrap(),
            "--format",
            "json",
            "--mode",
            "descriptive",
            "--rule-report",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let descriptive: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rule(&descriptive, "5.1")["assessment"], "not_evaluated");
    assert_eq!(
        rule(&descriptive, "6.3")["assessment"],
        "checked_within_scope"
    );
    assert_eq!(
        rule(&descriptive, "6.6")["assessment"],
        "checked_within_scope"
    );
}

#[test]
fn standard_lint_json_output_keeps_its_existing_schema() {
    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            fixtures().join("lint/semicolon.txt").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("diagnostics").is_some());
    assert!(json.get("rules").is_none());
    assert!(json.get("schema").is_none());
}

#[test]
fn human_mode_cannot_silently_discard_requested_rule_report() {
    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            fixtures().join("lint/semicolon.txt").to_str().unwrap(),
            "--rule-report",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--format json"));
}
