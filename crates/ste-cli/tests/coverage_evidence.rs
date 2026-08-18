use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use predicates::prelude::*;
use serde_json::Value;

#[test]
fn coverage_json_exposes_rule_evidence_and_claim_scope() {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args(["coverage", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"semantic_key\""))
        .stdout(predicate::str::contains("\"evidence_artifacts\""))
        .stdout(predicate::str::contains("\"unresolved_requirements\""))
        .stdout(predicate::str::contains("\"claim_scope\""));
}

#[test]
fn coverage_manifest_is_evidence_complete_and_path_valid() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest_path = repo.join("data/rules.json");
    let source = fs::read_to_string(&manifest_path).unwrap();
    let manifest: Value = serde_json::from_str(&source).unwrap();

    assert_eq!(manifest["total_rules"].as_u64(), Some(53));
    assert_eq!(manifest["full_compliance_claimed"].as_bool(), Some(false));

    let rules = manifest["rules"].as_array().expect("rules array");
    assert_eq!(rules.len(), 53);

    let implemented = rules
        .iter()
        .filter(|rule| rule["status"] == "implemented")
        .map(|rule| rule["id"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(implemented, BTreeSet::from(["6.6", "8.4", "8.5", "8.7"]));

    for rule in rules {
        let id = rule["id"].as_str().expect("rule id");
        let status = rule["status"].as_str().expect("rule status");
        let semantic_key = rule["semantic_key"].as_str().unwrap_or("").trim();
        assert!(
            !semantic_key.is_empty(),
            "rule {id} must state its source-audited semantic key"
        );
        let claim_scope = rule["claim_scope"].as_str().unwrap_or("").trim();
        assert!(
            !claim_scope.is_empty(),
            "rule {id} must state its claim scope"
        );

        let evidence = rule["evidence_artifacts"]
            .as_array()
            .unwrap_or_else(|| panic!("rule {id} must have an evidence_artifacts array"));
        if matches!(status, "implemented" | "partial") {
            assert!(
                !evidence.is_empty(),
                "executable rule {id} must cite repository evidence"
            );
        }
        for artifact in evidence {
            let relative = artifact.as_str().expect("evidence path string");
            assert!(
                repo.join(relative).exists(),
                "rule {id} evidence path does not exist: {relative}"
            );
        }

        let unresolved = rule["unresolved_requirements"]
            .as_array()
            .unwrap_or_else(|| panic!("rule {id} must have an unresolved_requirements array"));
        if status == "implemented" {
            assert!(
                unresolved.is_empty(),
                "implemented rule {id} must have no unresolved requirements in its stated scope"
            );
        } else {
            assert!(
                !unresolved.is_empty(),
                "non-implemented rule {id} must state what remains unresolved"
            );
        }
    }
}

#[test]
fn coverage_guide_inventory_matches_executable_manifest() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("data/rules.json")).expect("coverage manifest"),
    )
    .expect("valid coverage manifest");
    let guide = fs::read_to_string(repo.join("docs/rule-coverage.md")).expect("coverage guide");

    let mut counts = BTreeMap::<&str, usize>::new();
    for rule in manifest["rules"].as_array().expect("rules array") {
        *counts
            .entry(rule["status"].as_str().expect("rule status"))
            .or_default() += 1;
    }

    let inventory = format!(
        "The 53 rules classify as:\n\n- {} `implemented`;\n- {} `partial`;\n- {} `context_required`;\n- {} `not_implemented`.",
        counts.get("implemented").copied().unwrap_or_default(),
        counts.get("partial").copied().unwrap_or_default(),
        counts.get("context_required").copied().unwrap_or_default(),
        counts.get("not_implemented").copied().unwrap_or_default(),
    );
    assert!(
        guide.contains(&inventory),
        "public coverage inventory must match data/rules.json: {inventory}"
    );

    if counts.get("not_implemented").copied().unwrap_or_default() > 0 {
        assert!(
            !guide.contains("The current manifest has zero entries in this state."),
            "coverage guide must not claim zero not-implemented rules when the manifest contains some"
        );
    }
}
