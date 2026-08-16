use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, TopicFact, lint_text_with_context};

fn options() -> LintOptions {
    LintOptions {
        mode: LintMode::Descriptive,
        fix: false,
    }
}

fn topic(start: usize, end: usize, value: &str) -> TopicFact {
    TopicFact {
        start,
        end,
        topic: value.into(),
        source: "document topic review".into(),
    }
}

#[test]
fn two_explicit_topics_in_one_paragraph_violate_rule_6_5() {
    let text = "The pump is hot. The valve is open.";
    let context = LintContext {
        occurrences: Vec::new(),
        topics: vec![
            topic(0, 16, "pump condition"),
            topic(17, 35, "valve condition"),
        ],
    };
    let lexicon = RuntimeLexicon::embedded().unwrap();

    let result = lint_text_with_context(text, &lexicon, None, Some(&context), options());

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-PARA-002")
        .expect("multiple explicit topics in one paragraph must be reported");
    assert_eq!(diagnostic.rules, vec!["6.5"]);
    assert_eq!(diagnostic.span.start, 0);
    assert_eq!(diagnostic.span.end, text.len());
}

#[test]
fn repeated_explicit_topic_in_one_paragraph_is_allowed() {
    let text = "The pump is hot. The pump is loud.";
    let context = LintContext {
        occurrences: Vec::new(),
        topics: vec![topic(0, 16, "pump"), topic(17, text.len(), "pump")],
    };
    let lexicon = RuntimeLexicon::embedded().unwrap();

    let result = lint_text_with_context(text, &lexicon, None, Some(&context), options());

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-PARA-002")
    );
}

#[test]
fn different_topics_in_different_paragraphs_are_allowed() {
    let text = "The pump is hot.\n\nThe valve is open.";
    let valve = text.find("The valve").unwrap();
    let context = LintContext {
        occurrences: Vec::new(),
        topics: vec![topic(0, 16, "pump"), topic(valve, text.len(), "valve")],
    };
    let lexicon = RuntimeLexicon::embedded().unwrap();

    let result = lint_text_with_context(text, &lexicon, None, Some(&context), options());

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-PARA-002")
    );
}
