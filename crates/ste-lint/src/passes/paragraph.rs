use std::collections::BTreeMap;

use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::structure::{paragraph_prose_sentence_count, paragraph_ranges};
use crate::{LintContext, LintMode, TopicFact};

pub(crate) fn check(
    text: &str,
    mode: LintMode,
    context: Option<&LintContext>,
) -> Vec<Diagnostic> {
    if mode != LintMode::Descriptive {
        return Vec::new();
    }

    let paragraphs = paragraph_ranges(text);
    let mut diagnostics = paragraphs
        .iter()
        .filter_map(|&(start, end)| {
            let sentence_count = paragraph_prose_sentence_count(&text[start..end]);
            (sentence_count > 6).then(|| Diagnostic {
                code: "STE-PARA-001".into(),
                severity: Severity::Error,
                message: format!(
                    "Paragraph has {sentence_count} sentences; the maximum is 6."
                ),
                span: Span { start, end },
                rules: vec!["6.6".into()],
                evidence: Some(json!({
                    "sentence_count": sentence_count,
                    "limit": 6,
                    "vertical_list_items": "excluded from paragraph sentence count per the Rule 6.6 structural example"
                })),
                autofix: None,
            })
        })
        .collect::<Vec<_>>();

    let Some(context) = context else {
        return diagnostics;
    };
    if context.validate(text.len()).is_err()
        || context
            .topics
            .iter()
            .any(|topic| !text.is_char_boundary(topic.start) || !text.is_char_boundary(topic.end))
    {
        return diagnostics;
    }

    for topic in &context.topics {
        if !paragraphs
            .iter()
            .any(|&(start, end)| topic.start >= start && topic.end <= end)
        {
            diagnostics.push(Diagnostic {
                code: "STE-CTX-000".into(),
                severity: Severity::Blocked,
                message: "Lint context topic span must be contained in one paragraph.".into(),
                span: Span { start: 0, end: 0 },
                rules: Vec::new(),
                evidence: Some(json!({
                    "source": topic.source,
                    "topic": topic.topic,
                    "span": {"start": topic.start, "end": topic.end},
                })),
                autofix: None,
            });
        }
    }

    for &(start, end) in &paragraphs {
        let mut topics = BTreeMap::<String, Vec<&TopicFact>>::new();
        for topic in context
            .topics
            .iter()
            .filter(|topic| topic.start >= start && topic.end <= end)
        {
            topics
                .entry(topic.topic.trim().to_string())
                .or_default()
                .push(topic);
        }

        if topics.len() > 1 {
            diagnostics.push(Diagnostic {
                code: "STE-PARA-002".into(),
                severity: Severity::Error,
                message: format!(
                    "Paragraph contains {} explicitly resolved topics; use one topic per paragraph.",
                    topics.len()
                ),
                span: Span { start, end },
                rules: vec!["6.5".into()],
                evidence: Some(json!({
                    "topics": topics,
                    "paragraph": {"start": start, "end": end},
                    "classification": "project-supplied topic evidence"
                })),
                autofix: None,
            });
        }
    }

    diagnostics
}
