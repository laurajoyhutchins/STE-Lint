use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, LintResult, lint_text_with_context};

#[derive(Clone, Copy)]
struct GoldenCase {
    id: &'static str,
    mode: LintMode,
    context_file: Option<&'static str>,
    glossary_file: Option<&'static str>,
}

const CASES: &[GoldenCase] = &[
    GoldenCase {
        id: "clean-procedure",
        mode: LintMode::Procedural,
        context_file: None,
        glossary_file: None,
    },
    GoldenCase {
        id: "mixed-procedure",
        mode: LintMode::Procedural,
        context_file: None,
        glossary_file: None,
    },
    GoldenCase {
        id: "dictionary-semantics",
        mode: LintMode::Descriptive,
        context_file: None,
        glossary_file: None,
    },
    GoldenCase {
        id: "semantic-interaction",
        mode: LintMode::Descriptive,
        context_file: Some("semantic-interaction.context.json"),
        glossary_file: None,
    },
    GoldenCase {
        id: "safety-procedure",
        mode: LintMode::Procedural,
        context_file: None,
        glossary_file: None,
    },
    GoldenCase {
        id: "ambiguity-boundary",
        mode: LintMode::Descriptive,
        context_file: None,
        glossary_file: None,
    },
];

#[test]
fn curated_golden_documents_match_semantic_contract() {
    let root = golden_root();
    let lexicon = RuntimeLexicon::embedded().expect("embedded test lexicon");
    let update = std::env::var_os("STE_UPDATE_GOLDENS").is_some();
    let mut mismatches = Vec::new();

    assert_eq!(CASES.len(), 6, "keep the initial golden slice deliberately small");

    for case in CASES {
        let text = read_fixture(&root, &format!("{}.txt", case.id));
        let glossary = case.glossary_file.map(|path| {
            Glossary::from_json(&read_fixture(&root, path)).expect("golden glossary must parse")
        });
        let context = case.context_file.map(|path| {
            let context = LintContext::from_json(&read_fixture(&root, path))
                .expect("golden context must parse");
            context
                .validate(text.len())
                .expect("golden context must match the fixture text");
            context
        });

        let result = lint_text_with_context(
            &text,
            &lexicon,
            glossary.as_ref(),
            context.as_ref(),
            LintOptions {
                mode: case.mode,
                fix: false,
            },
        );

        if case.id == "ambiguity-boundary" {
            assert!(
                result
                    .diagnostics
                    .iter()
                    .all(|diagnostic| !diagnostic.rules.iter().any(|rule| rule == "3.1")),
                "Rule 3.1 must remain absent until the source-backed form-linkage gate is satisfied"
            );
        }

        let actual = normalize_result(&result);
        let expected_path = root.join(format!("{}.json", case.id));

        if update {
            fs::write(&expected_path, pretty_json(&actual))
                .unwrap_or_else(|error| panic!("could not update {}: {error}", expected_path.display()));
            continue;
        }

        let expected_source = fs::read_to_string(&expected_path).unwrap_or_else(|error| {
            panic!(
                "golden expectation {} is required: {error}",
                expected_path.display()
            )
        });
        let expected: Value = serde_json::from_str(&expected_source)
            .unwrap_or_else(|error| panic!("invalid golden {}: {error}", expected_path.display()));

        if expected != actual {
            mismatches.push(format!(
                "{}\nexpected:\n{}actual:\n{}",
                case.id,
                pretty_json(&expected),
                pretty_json(&actual)
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "golden mismatch(es):\n\n{}\nintentional updates: STE_UPDATE_GOLDENS=1 cargo test -p ste-lint --test golden",
        mismatches.join("\n")
    );
}

fn normalize_result(result: &LintResult) -> Value {
    let diagnostics = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let mut value = json!({
                "code": &diagnostic.code,
                "severity": diagnostic.severity,
                "span": diagnostic.span,
                "rules": &diagnostic.rules,
            });
            if let Some(evidence) = &diagnostic.evidence {
                value["evidence"] = evidence.clone();
            }
            if let Some(autofix) = &diagnostic.autofix {
                value["autofix"] = serde_json::to_value(autofix).expect("autofix serializes");
            }
            value
        })
        .collect::<Vec<_>>();

    json!({
        "outcome": result.outcome,
        "diagnostics": diagnostics,
    })
}

fn golden_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/golden")
}

fn read_fixture(root: &Path, relative: &str) -> String {
    let path = root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("could not read golden fixture {}: {error}", path.display()))
}

fn pretty_json(value: &Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(value).expect("golden JSON serializes")
    )
}