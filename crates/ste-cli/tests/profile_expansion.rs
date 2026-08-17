use std::fs;

use predicates::prelude::*;

#[test]
fn github_profile_exposes_actions_terminology() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["profile", "show", "github", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workflow run"))
        .stdout(predicate::str::contains("self-hosted runner"))
        .stdout(predicate::str::contains("matrix strategy"))
        .stdout(predicate::str::contains("context"))
        .stdout(predicate::str::contains("workflow dispatch"));
}

#[test]
fn configured_github_profile_resolves_actions_multiword_forms() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["github"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(
        &document,
        "WORKFLOW RUNS. SELF-HOSTED RUNNERS. MATRIX STRATEGIES. RUNNER GROUPS. WORKFLOW DISPATCH.",
    )
    .unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
            "lint",
            document.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .stdout(predicate::str::contains("STE-TERM-001").not());
}

#[test]
fn software_core_exposes_second_pass_terminology() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["profile", "show", "software-core", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("compiler"))
        .stdout(predicate::str::contains("array"))
        .stdout(predicate::str::contains("payload"))
        .stdout(predicate::str::contains("retry"))
        .stdout(predicate::str::contains("fixture"))
        .stdout(predicate::str::contains("authentication"))
        .stdout(predicate::str::contains("telemetry"))
        .stdout(predicate::str::contains("plugin"))
        .stdout(predicate::str::contains("metadata"));
}

#[test]
fn configured_software_core_resolves_second_pass_forms() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["software-core"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(
        &document,
        "PAYLOADS. FIXTURES. RETRIES. MANIFESTS. CREDENTIALS. METRICS.",
    )
    .unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
            "lint",
            document.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .stdout(predicate::str::contains("STE-TERM-001").not());
}
