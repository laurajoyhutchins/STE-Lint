use std::fs;
use std::io::Write;
use std::path::PathBuf;

use predicates::prelude::*;

#[test]
fn version_identifies_issue_nine_runtime() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ASD-STE100 Issue 9"))
        .stdout(predicate::str::contains(
            "runtime source: embedded test lexicon",
        ));
}

#[test]
fn lint_without_verified_runtime_fails_closed() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["lint", file.path().to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("no verified runtime lexicon"))
        .stderr(predicate::str::contains("--allow-test-lexicon"));
}

#[test]
fn explicit_missing_runtime_path_is_invalid_data_without_fallback() {
    let missing = tempfile::tempdir()
        .unwrap()
        .path()
        .join("missing-runtime.json");
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--lexicon", missing.to_str().unwrap(), "version"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("configured runtime lexicon"))
        .stderr(predicate::str::contains("could not be read"));
}

#[test]
fn environment_runtime_path_is_verified_without_fallback() {
    let mut runtime = tempfile::NamedTempFile::new().unwrap();
    writeln!(runtime, "{{}}").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .env("STE_LINT_LEXICON", runtime.path())
        .arg("version")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("failed verification"));
}

#[test]
fn lint_json_reports_stable_diagnostic_and_exit_code() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "USE THIS; USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
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

#[test]
fn lint_fix_does_not_apply_unsafe_semicolon_replacement() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "USE THIS; USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
            "lint",
            file.path().to_str().unwrap(),
            "--fix",
            "--mode",
            "procedural",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("STE-PUNC-001"));

    assert_eq!(
        fs::read_to_string(file.path()).unwrap(),
        "USE THIS; USE THIS.\n"
    );
}

#[test]
fn lint_fix_json_preserves_semicolons_and_does_not_offer_autofix() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "USE THIS; USE THIS.").unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            file.path().to_str().unwrap(),
            "--fix",
            "--format",
            "json",
            "--mode",
            "procedural",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["text"], "USE THIS; USE THIS.\n");
    let semicolon = result["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "STE-PUNC-001")
        .expect("expected semicolon diagnostic");
    assert!(semicolon["autofix"].is_null());
    assert_eq!(
        fs::read_to_string(file.path()).unwrap(),
        "USE THIS; USE THIS.\n"
    );
}

#[test]
fn lint_discovers_nearest_project_context() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("docs");
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::create_dir_all(&nested).unwrap();
    let document = nested.join("sample.txt");
    fs::write(&document, "USE.").unwrap();
    fs::write(
        root.path().join(".ste/context.json"),
        r#"{
  "occurrences": [
    {
      "start": 0,
      "end": 3,
      "source": "project terminology review",
      "spelling": "non_american",
      "official_technical_name": false
    }
  ]
}"#,
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
        .code(1)
        .stdout(predicate::str::contains("STE-CTX-003"))
        .stdout(predicate::str::contains("project terminology review"));
}

#[test]
fn malformed_project_context_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "USE.").unwrap();
    fs::write(root.path().join(".ste/context.json"), "{not-json").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("invalid lint context"))
        .stderr(predicate::str::contains(".ste/context.json"));
}

#[test]
fn missing_runtime_json_mode_returns_typed_application_error() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "USE THIS.").unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args(["lint", file.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["schema"], "ste-cli-error/v1");
    assert_eq!(error["error"]["kind"], "invalid_data");
    assert_eq!(error["error"]["exit_code"], 3);
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("no verified runtime lexicon")
    );
}

#[test]
fn invalid_profile_json_mode_returns_typed_application_error() {
    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args(["profile", "show", "not-a-profile", "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["schema"], "ste-cli-error/v1");
    assert_eq!(error["error"]["kind"], "invalid_data");
    assert_eq!(error["error"]["exit_code"], 3);
}

#[test]
fn invalid_project_context_json_mode_returns_typed_application_error() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    let document = root.path().join("instructions.txt");
    fs::write(&document, "USE.").unwrap();
    fs::write(root.path().join(".ste/context.json"), "{not-json").unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            document.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["schema"], "ste-cli-error/v1");
    assert_eq!(error["error"]["kind"], "invalid_data");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("invalid lint context")
    );
}

#[test]
fn malformed_glossary_json_mode_returns_typed_application_error() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{{not-json").unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "glossary",
            "check",
            file.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["schema"], "ste-cli-error/v1");
    assert_eq!(error["error"]["kind"], "invalid_data");
}

#[test]
fn missing_file_json_mode_returns_typed_io_error() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing.txt");

    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "check-rewrite",
            missing.to_str().unwrap(),
            missing.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stderr.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["schema"], "ste-cli-error/v1");
    assert_eq!(error["error"]["kind"], "io_or_internal");
    assert_eq!(error["error"]["exit_code"], 4);
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("could not read")
    );
}

#[test]
fn check_rewrite_rejects_modality_change() {
    let mut before = tempfile::NamedTempFile::new().unwrap();
    let mut after = tempfile::NamedTempFile::new().unwrap();
    writeln!(before, "The request may fail.").unwrap();
    writeln!(after, "The request fails.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "check-rewrite",
            before.path().to_str().unwrap(),
            after.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("SEM-MODALITY-001"));
}

#[test]
fn dictionary_lookup_exposes_structured_alternatives() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
            "dictionary",
            "lookup",
            "acceptable",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PERMITTED"));
}

#[test]
fn human_dictionary_lookup_shows_status_forms_meaning_and_alternatives() {
    let mut approved = assert_cmd::cargo::cargo_bin_cmd!("ste");
    approved
        .args(["--allow-test-lexicon", "dictionary", "lookup", "USE"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USE — approved verb"))
        .stdout(predicate::str::contains("forms: USE, USES, USED"))
        .stdout(predicate::str::contains(
            "meaning: Employ something for a purpose",
        ));

    let mut unapproved = assert_cmd::cargo::cargo_bin_cmd!("ste");
    unapproved
        .args(["--allow-test-lexicon", "dictionary", "lookup", "acceptable"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "acceptable — unapproved adjective",
        ))
        .stdout(predicate::str::contains("alternative: PERMITTED"))
        .stdout(predicate::str::contains("strategy: word_replacement"));
}

#[test]
fn glossary_check_accepts_valid_project_terms() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/glossary/valid.json");
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "glossary",
            "check",
            fixture.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}
