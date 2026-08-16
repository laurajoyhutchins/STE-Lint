use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::{DictionaryMeaningUse, LintContext, ParenthesisUseKind, SpellingUse, TechnicalNounScope};

pub(crate) fn check(text: &str, context: Option<&LintContext>) -> Vec<Diagnostic> {
    let Some(context) = context else {
        return Vec::new();
    };
    if let Err(error) = context.validate(text.len()) {
        return vec![Diagnostic {
            code: "STE-CTX-000".into(),
            severity: Severity::Blocked,
            message: format!("Lint context is invalid: {error}"),
            span: Span { start: 0, end: 0 },
            rules: Vec::new(),
            evidence: Some(json!({ "error": error })),
            autofix: None,
        }];
    }

    let mut diagnostics = Vec::new();
    for topic in &context.topics {
        if !text.is_char_boundary(topic.start) || !text.is_char_boundary(topic.end) {
            diagnostics.push(Diagnostic {
                code: "STE-CTX-000".into(),
                severity: Severity::Blocked,
                message: "Lint context topic span is not on UTF-8 character boundaries.".into(),
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

    for occurrence in &context.occurrences {
        if !text.is_char_boundary(occurrence.start) || !text.is_char_boundary(occurrence.end) {
            diagnostics.push(Diagnostic {
                code: "STE-CTX-000".into(),
                severity: Severity::Blocked,
                message: "Lint context occurrence span is not on UTF-8 character boundaries."
                    .into(),
                span: Span { start: 0, end: 0 },
                rules: Vec::new(),
                evidence: Some(json!({
                    "source": occurrence.source,
                    "span": {"start": occurrence.start, "end": occurrence.end},
                })),
                autofix: None,
            });
            continue;
        }

        let span = Span {
            start: occurrence.start,
            end: occurrence.end,
        };
        let text_value = &text[occurrence.start..occurrence.end];

        if occurrence.dictionary_meaning == Some(DictionaryMeaningUse::NotApproved) {
            diagnostics.push(Diagnostic {
                code: "STE-CTX-001".into(),
                severity: Severity::Error,
                message: format!(
                    "'{text_value}' is explicitly resolved to a meaning that is not approved for this dictionary word."
                ),
                span,
                rules: vec!["1.3".into()],
                evidence: Some(json!({
                    "source": occurrence.source,
                    "fact": "dictionary_meaning",
                    "value": occurrence.dictionary_meaning,
                    "span": {"start": occurrence.start, "end": occurrence.end},
                })),
                autofix: None,
            });
        }

        if matches!(
            occurrence.technical_noun_scope,
            Some(
                TechnicalNounScope::Regional
                    | TechnicalNounScope::Slang
                    | TechnicalNounScope::Jargon
            )
        ) {
            diagnostics.push(Diagnostic {
                code: "STE-CTX-002".into(),
                severity: Severity::Error,
                message: format!(
                    "'{text_value}' is explicitly classified as regional, slang, or jargon technical terminology."
                ),
                span,
                rules: vec!["1.10".into()],
                evidence: Some(json!({
                    "source": occurrence.source,
                    "fact": "technical_noun_scope",
                    "value": occurrence.technical_noun_scope,
                    "span": {"start": occurrence.start, "end": occurrence.end},
                })),
                autofix: None,
            });
        }

        if occurrence.spelling == Some(SpellingUse::NonAmerican)
            && !occurrence.official_technical_name
        {
            diagnostics.push(Diagnostic {
                code: "STE-CTX-003".into(),
                severity: Severity::Error,
                message: format!(
                    "'{text_value}' is explicitly classified as non-American spelling and is not an official technical name."
                ),
                span,
                rules: vec!["1.14".into()],
                evidence: Some(json!({
                    "source": occurrence.source,
                    "fact": "spelling",
                    "value": occurrence.spelling,
                    "official_technical_name": occurrence.official_technical_name,
                    "span": {"start": occurrence.start, "end": occurrence.end},
                })),
                autofix: None,
            });
        }

        if let Some(direct_relation) = occurrence.hyphen_direct_relation {
            if !text_value.contains('-') {
                diagnostics.push(invalid_occurrence_shape(
                    occurrence,
                    "hyphen_direct_relation evidence must identify text that contains a hyphen",
                ));
            } else if !direct_relation {
                diagnostics.push(Diagnostic {
                    code: "STE-PUNC-002".into(),
                    severity: Severity::Error,
                    message: format!(
                        "'{text_value}' is explicitly classified as a hyphen use between words that are not directly related."
                    ),
                    span,
                    rules: vec!["8.2".into()],
                    evidence: Some(json!({
                        "source": occurrence.source,
                        "fact": "hyphen_direct_relation",
                        "value": direct_relation,
                        "span": {"start": occurrence.start, "end": occurrence.end},
                    })),
                    autofix: None,
                });
            }
        }

        if let Some(parenthesis_use) = occurrence.parenthesis_use {
            let trimmed = text_value.trim();
            if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
                diagnostics.push(invalid_occurrence_shape(
                    occurrence,
                    "parenthesis_use evidence must identify a parenthesized text span",
                ));
            } else if parenthesis_use == ParenthesisUseKind::Other {
                diagnostics.push(Diagnostic {
                    code: "STE-PUNC-003".into(),
                    severity: Severity::Error,
                    message: "This parenthetical use is explicitly classified outside the Rule 8.3 allowed categories."
                        .into(),
                    span,
                    rules: vec!["8.3".into()],
                    evidence: Some(json!({
                        "source": occurrence.source,
                        "fact": "parenthesis_use",
                        "value": parenthesis_use,
                        "span": {"start": occurrence.start, "end": occurrence.end},
                    })),
                    autofix: None,
                });
            }
        }
    }

    diagnostics
}

fn invalid_occurrence_shape(occurrence: &crate::OccurrenceFact, message: &str) -> Diagnostic {
    Diagnostic {
        code: "STE-CTX-000".into(),
        severity: Severity::Blocked,
        message: message.into(),
        span: Span { start: 0, end: 0 },
        rules: Vec::new(),
        evidence: Some(json!({
            "source": occurrence.source,
            "span": {"start": occurrence.start, "end": occurrence.end},
        })),
        autofix: None,
    }
}
