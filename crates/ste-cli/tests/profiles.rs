use std::fs;

use predicates::prelude::*;

#[test]
fn profile_list_exposes_built_in_profiles() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["profile", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("software-core"))
        .stdout(predicate::str::contains("git"))
        .stdout(predicate::str::contains("github"));
}

#[test]
fn profile_show_exposes_profile_metadata_and_terms() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["profile", "show", "github", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"github\""))
        .stdout(predicate::str::contains("pull request"));
}

#[test]
fn unknown_profile_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["does-not-exist"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("unknown terminology profile"));
}

#[test]
fn duplicate_profile_selection_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["git","git"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("duplicate terminology profile"));
}

#[test]
fn malformed_project_profile_config_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(root.path().join(".ste/config.json"), "{not-json").unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "USE THIS.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("invalid STE project config"));
}

#[test]
fn configured_software_profile_resolves_explicit_technical_forms() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["software-core"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "RUNTIMES.").unwrap();

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
fn configured_git_profile_resolves_git_terminology() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["git"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "REPOSITORIES.").unwrap();

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
fn configured_github_profile_resolves_multiword_form() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["github"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "PULL REQUESTS.").unwrap();

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
fn profiles_leave_project_specific_terminology_blocked() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["software-core","git","github"]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "CAPSULE.").unwrap();

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
        .code(2)
        .stdout(predicate::str::contains("STE-TERM-001"));
}

#[test]
fn no_config_does_not_enable_profiles_implicitly() {
    let document = tempfile::NamedTempFile::new().unwrap();
    fs::write(document.path(), "RUNTIMES.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
            "lint",
            document.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("STE-TERM-001"));
}

#[test]
fn effective_glossary_reports_selected_profiles_and_project_terms() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["git","github"]}"#,
    )
    .unwrap();
    fs::write(
        root.path().join(".ste/terms.json"),
        r#"{"terms":[{"term":"capsule","kind":"technical_noun","definition":"A project-specific capsule.","domain":"project","preferred":true,"aliases":[],"examples":[],"provenance":["project authority"],"status":"approved"}]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "CAPSULE.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "glossary",
            "effective",
            document.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"git\""))
        .stdout(predicate::str::contains("\"github\""))
        .stdout(predicate::str::contains("capsule"));
}
