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
        .stdout(predicate::str::contains("\"status\": \"partial\""))
        .stdout(predicate::str::contains("\"status\": \"context_required\""))
        .stdout(predicate::str::contains("\"status\": \"not_implemented\""))
        .stdout(predicate::str::contains("STE-NOUN-002"))
        .stdout(predicate::str::contains("STE-DISC-001"))
        .stdout(predicate::str::contains("STE-CTX-001"))
        .stdout(predicate::str::contains("STE-CTX-002"))
        .stdout(predicate::str::contains("STE-CTX-003"));
}

#[test]
fn coverage_human_output_states_that_full_compliance_is_not_claimed() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .arg("coverage")
        .assert()
        .success()
        .stdout(predicate::str::contains("ASD-STE100 Issue 9"))
        .stdout(predicate::str::contains("53 rules tracked"))
        .stdout(predicate::str::contains("implemented: 2"))
        .stdout(predicate::str::contains("partial: 37"))
        .stdout(predicate::str::contains("context_required: 11"))
        .stdout(predicate::str::contains("not_implemented: 3"))
        .stdout(predicate::str::contains(
            "full Issue 9 compliance is not claimed",
        ));
}
