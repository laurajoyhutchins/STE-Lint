use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn gate3_semantic_provider_eval_on_research_pull_request() {
    if env::var("GITHUB_EVENT_NAME").as_deref() != Ok("pull_request")
        || env::var("GITHUB_HEAD_REF").as_deref() != Ok("research/semantic-provider-evaluation")
    {
        return;
    }

    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let status = Command::new("bash")
        .arg("tools/semantic-eval/run.sh")
        .current_dir(repository_root)
        .status()
        .expect("semantic provider evaluation runner must start");

    assert!(status.success(), "semantic provider evaluation must succeed");
}
