use std::fs;

#[test]
fn malformed_multibyte_context_is_a_typed_cli_error_and_does_not_rewrite_input() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".ste")).unwrap();
    let document = root.path().join("source.txt");
    fs::write(&document, "É T").unwrap();
    fs::write(
        root.path().join(".ste/context.json"),
        r#"{"occurrences":[{"start":1,"end":2,"source":"test","count_group":"title"}]}"#,
    )
    .unwrap();
    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args([
            "--allow-test-lexicon",
            "lint",
            document.to_str().unwrap(),
            "--fix",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["schema"], "ste-cli-error/v1");
    assert_eq!(data["error"]["kind"], "invalid_data");
    assert!(data["error"]["message"].as_str().unwrap().contains("UTF-8"));
    assert_eq!(fs::read_to_string(document).unwrap(), "É T");
}
