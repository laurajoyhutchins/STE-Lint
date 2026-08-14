use std::io::Write;

use predicates::prelude::*;

#[test]
fn version_identifies_issue_nine_runtime() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ASD-STE100 Issue 9"));
}

#[test]
fn lint_json_reports_stable_diagnostic_and_exit_code() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "USE THIS; USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "lint",
            file.path().to_str().unwrap(),
            "--format",
            "json",
            "--mode",
            "procedural",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("STE-PUNC-001"));
}
