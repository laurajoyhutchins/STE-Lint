use std::fs;

use predicates::prelude::*;

#[test]
fn project_term_cannot_override_profile_canonical_identity() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["software-core"]}"#,
    )
    .unwrap();
    fs::write(
        root.path().join(".ste/terms.json"),
        r#"{"terms":[{"term":"runtime","kind":"technical_noun","definition":"A conflicting project meaning.","domain":"project","preferred":true,"aliases":[],"examples":[],"provenance":["project authority"],"status":"approved"}]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "RUNTIME.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("TERM-DUP-001"));
}

#[test]
fn project_alias_cannot_capture_a_profile_form() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/config.json"),
        r#"{"profiles":["software-core"]}"#,
    )
    .unwrap();
    fs::write(
        root.path().join(".ste/terms.json"),
        r#"{"terms":[{"term":"capsule","kind":"technical_noun","definition":"A project term with a conflicting alias.","domain":"project","preferred":true,"aliases":["runtimes"],"examples":[],"provenance":["project authority"],"status":"approved"}]}"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "CAPSULE.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("TERM-ID-CONFLICT-001"));
}
