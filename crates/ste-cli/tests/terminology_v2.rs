use std::fs;

use predicates::prelude::*;

#[test]
fn built_in_profile_exposes_v2_schema_and_stable_term_identity() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["profile", "show", "software-core", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"schema\": \"ste-terminology/v2\"",
        ))
        .stdout(predicate::str::contains("\"canonical\": \"runtime\""))
        .stdout(predicate::str::contains("\"roles\": ["))
        .stdout(predicate::str::contains("\"preferred\"").not());
}

#[test]
fn v2_project_glossary_resolves_structured_forms_and_aliases() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/terms.json"),
        r#"{
          "schema": "ste-terminology/v2",
          "domain": "project",
          "sources": {
            "project-spec": {
              "title": "Project specification",
              "reviewed_on": "2026-08-17"
            }
          },
          "terms": [{
            "id": "execution-receipt",
            "canonical": "execution receipt",
            "roles": ["noun"],
            "definition": "A durable record of one execution result.",
            "forms": [{"text":"execution receipts","roles":["noun"]}],
            "aliases": [{"text":"receipt record","kind":"short_form"}],
            "sources": [{
              "source": "project-spec",
              "supports": ["admission","definition","role","forms","alias","status"]
            }],
            "status": "approved"
          }]
        }"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "EXECUTION RECEIPTS. RECEIPT RECORD.").unwrap();

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
fn v2_deprecated_term_reports_stable_id_and_replacement() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/terms.json"),
        r#"{
          "schema": "ste-terminology/v2",
          "domain": "project",
          "sources": {
            "project-spec": {"title":"Project specification"}
          },
          "terms": [
            {
              "id": "new-name",
              "canonical": "newname",
              "roles": ["noun"],
              "definition": "The current project term.",
              "forms": [],
              "aliases": [],
              "sources": [{"source":"project-spec","supports":["admission","definition","role","status"]}],
              "status": "approved"
            },
            {
              "id": "old-name",
              "canonical": "oldname",
              "roles": ["noun"],
              "definition": "The retired project term.",
              "forms": [],
              "aliases": [],
              "sources": [{"source":"project-spec","supports":["admission","definition","role","status"]}],
              "status": "deprecated",
              "replacement": "new-name"
            }
          ]
        }"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "OLDNAME.").unwrap();

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
        .stdout(predicate::str::contains("\"term_id\": \"old-name\""))
        .stdout(predicate::str::contains("\"replacement\": \"new-name\""));
}

#[test]
fn v2_identity_collision_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ste")).unwrap();
    fs::write(
        root.path().join(".ste/terms.json"),
        r#"{
          "schema": "ste-terminology/v2",
          "domain": "project",
          "sources": {"spec":{"title":"Project specification"}},
          "terms": [
            {
              "id":"alpha",
              "canonical":"alpha",
              "roles":["noun"],
              "definition":"Alpha.",
              "forms":[],
              "aliases":[{"text":"shared name","kind":"synonym"}],
              "sources":[{"source":"spec","supports":["admission","definition","role","alias","status"]}],
              "status":"approved"
            },
            {
              "id":"beta",
              "canonical":"shared name",
              "roles":["noun"],
              "definition":"Beta.",
              "forms":[],
              "aliases":[],
              "sources":[{"source":"spec","supports":["admission","definition","role","status"]}],
              "status":"approved"
            }
          ]
        }"#,
    )
    .unwrap();
    let document = root.path().join("sample.txt");
    fs::write(&document, "ALPHA.").unwrap();

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["--allow-test-lexicon", "lint", document.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("TERM-ID-CONFLICT-001"));
}
