#![rustfmt::skip]
use std::fs;
use std::path::PathBuf;

#[test]
fn lint_json_matches_exact_public_contract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let fixture = root.join("lint/semicolon.txt");
    let expected = fs::read_to_string(root.join("golden/cli-semicolon.json"))
        .expect("exact CLI golden is required");

    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ste");
    command
        .args([
            "--allow-test-lexicon",
            "lint",
            fixture.to_str().unwrap(),
            "--format",
            "json",
            "--mode",
            "procedural",
        ])
        .assert()
        .code(1)
        .stdout(expected);
}