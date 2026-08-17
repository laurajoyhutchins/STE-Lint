use std::collections::BTreeSet;

use serde_json::Value;

fn software_core_profile() -> Value {
    let output = assert_cmd::cargo::cargo_bin_cmd!("ste")
        .args(["profile", "show", "software-core", "--format", "json"])
        .output()
        .expect("run profile show");
    assert!(
        output.status.success(),
        "profile show failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse software-core profile JSON")
}

fn term_by_id<'a>(profile: &'a Value, id: &str) -> &'a Value {
    profile["terms"]
        .as_array()
        .expect("terms array")
        .iter()
        .find(|term| term["id"] == id)
        .unwrap_or_else(|| panic!("missing term {id}"))
}

#[test]
fn software_core_v3_has_exact_subject_field_term_set() {
    let profile = software_core_profile();
    assert_eq!(profile["schema"], "ste-terminology/v2");
    assert_eq!(profile["profile"]["version"], 3);

    let actual = profile["terms"]
        .as_array()
        .expect("terms array")
        .iter()
        .map(|term| term["id"].as_str().expect("term id"))
        .collect::<BTreeSet<_>>();

    let expected = [
        "api", "application", "client", "command", "configuration", "database", "dependency",
        "directory", "endpoint", "environment", "file", "identifier", "interface", "library",
        "module", "package", "path", "program", "runtime", "schema", "server", "service",
        "version", "cli", "argument", "binary", "class", "code", "constant", "function",
        "method", "object", "parameter", "property", "string", "value", "variable", "alias",
        "array", "boolean", "byte", "field", "namespace", "record", "table", "cache", "event",
        "execution", "error", "job", "log", "message", "process", "request", "response", "state",
        "warning", "worker", "buffer", "header", "index", "payload", "protocol", "query", "queue",
        "retry", "session", "stream", "thread", "timeout", "transaction", "artifact", "test-case",
        "assertion", "benchmark", "build", "compiler", "coverage", "deserialization", "fixture",
        "framework", "manifest", "migration", "parser", "persistence", "plugin", "regression",
        "release", "serialization", "authentication", "authorization", "credential", "identity",
        "metadata", "metric", "permission", "secret", "telemetry", "trace", "compile", "deploy",
        "deserialize", "execute", "export", "import", "install", "load", "parse", "serialize",
        "validate",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 110);
}

#[test]
fn removed_general_words_do_not_receive_software_core_authority() {
    let profile = software_core_profile();
    let actual = profile["terms"]
        .as_array()
        .expect("terms array")
        .iter()
        .map(|term| term["id"].as_str().expect("term id"))
        .collect::<BTreeSet<_>>();

    for excluded in [
        "component", "data", "implementation", "input", "output", "status", "task", "test",
        "compatibility", "entity", "integration", "result", "configure", "verify", "persist",
    ] {
        assert!(
            !actual.contains(excluded),
            "{excluded} must not be owned by software-core"
        );
    }
}

#[test]
fn broad_noun_verb_terms_are_narrowed_to_nouns() {
    let profile = software_core_profile();

    for id in [
        "cache", "log", "process", "request", "buffer", "index", "query", "queue", "retry",
        "stream", "benchmark", "build", "release", "trace",
    ] {
        let term = term_by_id(&profile, id);
        assert_eq!(term["roles"], serde_json::json!(["noun"]), "{id} roles");
        for form in term["forms"].as_array().expect("forms array") {
            assert_eq!(
                form["roles"],
                serde_json::json!(["noun"]),
                "{id} form roles"
            );
        }
    }
}

#[test]
fn export_import_and_configuration_keep_only_bounded_authority() {
    let profile = software_core_profile();

    for id in ["export", "import"] {
        assert_eq!(
            term_by_id(&profile, id)["roles"],
            serde_json::json!(["verb"])
        );
    }

    let configuration = term_by_id(&profile, "configuration");
    let aliases = configuration["aliases"].as_array().expect("aliases array");
    assert!(
        aliases.iter().all(|alias| alias["text"] != "config"),
        "config convenience alias must not be admitted"
    );
}

#[test]
fn every_term_has_fresh_admission_authority_not_v1_v2_self_baselines() {
    let profile = software_core_profile();

    for term in profile["terms"].as_array().expect("terms array") {
        let id = term["id"].as_str().expect("term id");
        let sources = term["sources"].as_array().expect("source refs");
        assert!(
            sources.iter().any(|source| {
                source["source"] == "ste-lint-issue-57-owner-approved-review"
                    && source["supports"]
                        .as_array()
                        .expect("supports")
                        .iter()
                        .any(|support| support == "admission")
            }),
            "{id} lacks v3 admission authority"
        );
        assert!(
            sources.iter().all(|source| {
                let source_id = source["source"].as_str().expect("source id");
                !source_id.contains("software-core-v1-curated-baseline")
                    && !source_id.contains("software-core-v2-curated-baseline")
            }),
            "{id} still relies on a prior curated baseline"
        );
    }
}
