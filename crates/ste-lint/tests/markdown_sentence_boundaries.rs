use ste_data::RuntimeLexicon;
use ste_lint::{AnalysisDocument, LintMode, LintOptions, lint_text};

#[test]
fn inline_code_terminal_punctuation_does_not_fragment_analysis_sentence_identity() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = "USE the endpoint `/api/v1/operator/runs/:runId/...` to inspect the run. USE this.";
    let analysis = AnalysisDocument::new(text, &lexicon, None, None, LintMode::Descriptive);

    assert_eq!(analysis.sentences().len(), 2);
    let first = analysis.sentences()[0];
    assert_eq!(
        &text[first.start..first.end],
        "USE the endpoint `/api/v1/operator/runs/:runId/...` to inspect the run."
    );
}

#[test]
fn question_and_exclamation_marks_inside_multi_backtick_code_stay_protected() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = "USE the value ``ready?!`` in this sentence. USE this.";
    let analysis = AnalysisDocument::new(text, &lexicon, None, None, LintMode::Descriptive);

    assert_eq!(analysis.sentences().len(), 2);
    let first = analysis.sentences()[0];
    assert_eq!(
        &text[first.start..first.end],
        "USE the value ``ready?!`` in this sentence."
    );
}

#[test]
fn descriptive_length_uses_the_complete_sentence_around_inline_code() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = format!(
        "{} `/api/v1/operator/runs/:runId/...` {}.",
        vec!["USE"; 13].join(" "),
        vec!["USE"; 13].join(" ")
    );
    let result = lint_text(
        &text,
        &lexicon,
        None,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    );

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "STE-LEN-002")
    );
}
