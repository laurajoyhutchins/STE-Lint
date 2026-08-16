use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    synthetic: bool,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    category: String,
    mode: LintMode,
    text_file: String,
    #[serde(default)]
    glossary_file: Option<String>,
    #[serde(default)]
    context_file: Option<String>,
    expected_outcome: String,
    expected_codes: Vec<String>,
}

#[test]
fn representative_engineering_corpus_matches_declared_results() {
    let root = corpus_root();
    let manifest_path = root.join("manifest.json");
    let source = fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
        panic!(
            "representative engineering corpus manifest {} is required: {error}",
            manifest_path.display()
        )
    });
    let manifest: CorpusManifest = serde_json::from_str(&source).expect("corpus manifest JSON");

    assert!(
        manifest.synthetic,
        "public regression prose must be synthetic"
    );
    assert!(
        manifest.cases.len() >= 10,
        "corpus must contain at least ten representative cases"
    );

    let ids = manifest
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(
        ids.len(),
        manifest.cases.len(),
        "corpus case ids must be unique"
    );

    let categories = manifest
        .cases
        .iter()
        .map(|case| case.category.as_str())
        .collect::<HashSet<_>>();
    for required in [
        "clean_control",
        "punctuation",
        "lexical",
        "blocker",
        "note",
        "list",
        "context",
        "counting",
        "paragraph",
        "safety",
    ] {
        assert!(
            categories.contains(required),
            "representative corpus is missing category {required}"
        );
    }

    let lexicon = RuntimeLexicon::embedded().unwrap();
    for case in &manifest.cases {
        let text = read_fixture(&root, &case.text_file);
        let glossary = case
            .glossary_file
            .as_deref()
            .map(|path| Glossary::from_json(&read_fixture(&root, path)).expect("corpus glossary"));
        let context = case.context_file.as_deref().map(|path| {
            LintContext::from_json(&read_fixture(&root, path)).expect("corpus context")
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

        let observed_outcome = serde_json::to_value(result.outcome)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let observed_codes = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<BTreeSet<_>>();
        let expected_codes = case.expected_codes.iter().cloned().collect::<BTreeSet<_>>();

        assert_eq!(
            (observed_outcome.as_str(), &observed_codes),
            (case.expected_outcome.as_str(), &expected_codes),
            "unexpected lint result for corpus case {}",
            case.id
        );
    }
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/corpus")
}

fn read_fixture(root: &Path, relative: &str) -> String {
    let path = root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("could not read corpus fixture {}: {error}", path.display()))
}
