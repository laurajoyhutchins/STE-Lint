use predicates::prelude::*;

#[test]
fn coverage_json_tracks_all_issue_nine_rules_without_runtime_data() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["coverage", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_rules\": 53"))
        .stdout(predicate::str::contains("\"id\": \"1.1\""))
        .stdout(predicate::str::contains("\"id\": \"1.14\""))
        .stdout(predicate::str::contains("\"id\": \"9.4\""))
        .stdout(predicate::str::contains("\"status\": \"implemented\""))
        .stdout(predicate::str::contains("\"status\": \"partial\""))
        .stdout(predicate::str::contains("\"status\": \"context_required\""))
        .stdout(predicate::str::contains("STE-NOUN-002"))
        .stdout(predicate::str::contains("STE-DISC-001"))
        .stdout(predicate::str::contains("STE-CTX-001"))
        .stdout(predicate::str::contains("STE-CTX-002"))
        .stdout(predicate::str::contains("STE-CTX-003"))
        .stdout(predicate::str::contains("STE-FORM-001"))
        .stdout(predicate::str::contains("STE-PHRASE-001"));
}

#[test]
fn coverage_human_output_states_that_full_compliance_is_not_claimed() {
    let json = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args(["coverage", "--format", "json"])
        .output()
        .unwrap();
    assert!(json.status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let rules = manifest["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 53);

    let human = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .arg("coverage")
        .output()
        .unwrap();
    assert!(human.status.success());
    let report = String::from_utf8(human.stdout).unwrap();
    assert!(report.contains("ASD-STE100 Issue 9"));
    assert!(report.contains("53 rules tracked"));
    for status in ["implemented", "partial", "context_required", "not_implemented"] {
        let count = rules
            .iter()
            .filter(|rule| rule["status"] == status)
            .count();
        assert!(report.contains(&format!("{status}: {count}")));
    }
    assert!(report.contains("full Issue 9 compliance is not claimed"));
}
