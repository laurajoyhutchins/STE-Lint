use ste_data::RuntimeLexicon;
use ste_lint::{AnalysisDocument, LintMode};

#[test]
fn multiline_commonmark_code_span_does_not_fragment_sentence_identity() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = "USE `alpha.\nbeta` in this sentence. USE this.";
    let analysis = AnalysisDocument::new(text, &lexicon, None, None, LintMode::Descriptive);

    assert_eq!(analysis.sentences().len(), 2);
    let first = analysis.sentences()[0];
    assert_eq!(
        &text[first.start..first.end],
        "USE `alpha.\nbeta` in this sentence."
    );
}

#[test]
fn markdown_syntax_is_not_exposed_as_analysis_tokens() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let text = "# TITLE\n\nUSE **this** and `that`.";
    let analysis = AnalysisDocument::new(text, &lexicon, None, None, LintMode::Descriptive);
    let tokens = analysis
        .tokens()
        .iter()
        .map(|token| token.text)
        .collect::<Vec<_>>();

    assert!(tokens.contains(&"TITLE"));
    assert!(tokens.contains(&"USE"));
    assert!(tokens.contains(&"this"));
    assert!(!tokens.contains(&"that"));
}