use ste_data::RuntimeLexicon;
use ste_lint::{LintContext, LintMode, LintOptions, LintResult, lint_text_with_context};

fn lint(text: &str, context: Option<&LintContext>) -> LintResult {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text_with_context(
        text,
        &lexicon,
        None,
        context,
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
}

fn contractions(result: &LintResult) -> Vec<(usize, usize)> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "STE-SYN-001")
        .map(|diagnostic| {
            assert_eq!(diagnostic.rules, vec!["4.2"]);
            assert!(diagnostic.autofix.is_none());
            (diagnostic.span.start, diagnostic.span.end)
        })
        .collect()
}

#[test]
fn authored_contractions_are_diagnosed_beside_inline_code() {
    let text = "DON'T COPY `DON'T` OR `WE'RE`. IT'S CLEAR.";
    let result = lint(text, None);

    let first = text.find("DON'T").unwrap();
    let last = text.find("IT'S").unwrap();
    assert_eq!(
        contractions(&result),
        vec![(first, first + 5), (last, last + 4)]
    );
}

#[test]
fn fenced_code_does_not_create_contraction_diagnostics() {
    let text = "DON'T.\n\n```text\nDON'T.\nWE'RE.\n```\n\nWE'LL.";
    let result = lint(text, None);
    let first = text.find("DON'T").unwrap();
    let last = text.rfind("WE'LL").unwrap();

    assert_eq!(
        contractions(&result),
        vec![(first, first + 5), (last, last + 5)]
    );
}

#[test]
fn governed_immutable_external_text_is_excluded_but_adjacent_authored_text_is_not() {
    let protected = "\"DON'T\"";
    let text = format!("{protected} DON'T.");
    let context = LintContext::from_json(
        &serde_json::json!({
            "occurrences": [{
                "start": 0,
                "end": protected.len(),
                "source": "immutable external contract",
                "text_authority": "quoted_external_text"
            }]
        })
        .to_string(),
    )
    .unwrap();
    context.validate(text.len()).unwrap();

    let result = lint(&text, Some(&context));
    let adjacent = text.rfind("DON'T").unwrap();
    assert_eq!(contractions(&result), vec![(adjacent, adjacent + 5)]);
}

#[test]
fn authored_titles_and_authored_quotations_remain_in_scope() {
    let text = "\"DON'T\" IT'S.";
    let context = LintContext::from_json(
        &serde_json::json!({
            "occurrences": [{
                "start": 0,
                "end": 7,
                "source": "author title",
                "text_authority": "title"
            }]
        })
        .to_string(),
    )
    .unwrap();
    context.validate(text.len()).unwrap();

    let result = lint(text, Some(&context));
    assert_eq!(contractions(&result), vec![(1, 6), (8, 12)]);
}

#[test]
fn curly_apostrophe_and_unicode_prefix_keep_byte_offsets() {
    let text = "ÉQUIPEMENT. DON'T. WE’RE.";
    let result = lint(text, None);
    let first = text.find("DON'T").unwrap();
    let second = text.find("WE’RE").unwrap();

    assert_eq!(
        contractions(&result),
        vec![
            (first, first + "DON'T".len()),
            (second, second + "WE’RE".len())
        ]
    );
}

#[test]
fn possessives_do_not_become_false_contraction_errors() {
    let result = lint("THE OPERATOR'S LOG IS READY.", None);
    assert!(contractions(&result).is_empty());
}
