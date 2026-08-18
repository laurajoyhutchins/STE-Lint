use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::analysis::{AnalysisDocument, Resolution};
use crate::document_structure::{ListItem, simple_list_blocks};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemSyntax {
    Sentence,
    Fragment,
    Unresolved,
}

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let mut diagnostics = Vec::new();

    for block in simple_list_blocks(text) {
        if !block.introduced_by_colon {
            let first = block.items[0];
            diagnostics.push(Diagnostic {
                code: "STE-LIST-001".into(),
                severity: Severity::Error,
                message: "Put a colon before the first item in a vertical list.".into(),
                span: Span {
                    start: first.line_start,
                    end: first.line_end,
                },
                rules: vec!["4.3".into()],
                evidence: Some(json!({
                    "coverage": "structured_vertical_list_mechanics_v2",
                    "mechanic": "colon_before_list"
                })),
                autofix: None,
            });
        }

        for item in &block.items {
            let content = text[item.content_start..item.content_end].trim();
            if let Some(character) = content.chars().find(|character| character.is_alphabetic())
                && character.is_lowercase()
            {
                diagnostics.push(Diagnostic {
                    code: "STE-LIST-002".into(),
                    severity: Severity::Error,
                    message: "Start each vertical-list item with an uppercase letter.".into(),
                    span: Span {
                        start: item.content_start,
                        end: item.content_end,
                    },
                    rules: vec!["4.3".into()],
                    evidence: Some(json!({
                        "coverage": "structured_vertical_list_mechanics_v2",
                        "mechanic": "uppercase_item_start"
                    })),
                    autofix: None,
                });
            }

            if content.ends_with(',') || content.ends_with(';') {
                let mut rules = vec!["4.3".into()];
                if content.ends_with(';') {
                    rules.push("8.1".into());
                }
                diagnostics.push(Diagnostic {
                    code: "STE-LIST-003".into(),
                    severity: Severity::Error,
                    message: "Do not end a vertical-list item with a comma or semicolon.".into(),
                    span: Span {
                        start: item.content_start,
                        end: item.content_end,
                    },
                    rules,
                    evidence: Some(json!({
                        "coverage": "structured_vertical_list_mechanics_v2",
                        "mechanic": "forbidden_item_end_punctuation"
                    })),
                    autofix: None,
                });
                continue;
            }

            match item_syntax(analysis, *item) {
                ItemSyntax::Sentence if !content.ends_with('.') => {
                    diagnostics.push(terminal_punctuation_diagnostic(
                        *item,
                        Severity::Error,
                        "Put a period at the end of a full-sentence vertical-list item.",
                        "sentence_requires_period",
                    ));
                }
                ItemSyntax::Fragment if content.ends_with('.') => {
                    diagnostics.push(terminal_punctuation_diagnostic(
                        *item,
                        Severity::Error,
                        "Do not put a period at the end of a fragment vertical-list item.",
                        "fragment_forbids_period",
                    ));
                }
                ItemSyntax::Unresolved => {
                    diagnostics.push(terminal_punctuation_diagnostic(
                        *item,
                        Severity::Blocked,
                        "Cannot determine whether this vertical-list item is a full sentence or a fragment; terminal-period compliance is unresolved.",
                        "sentence_or_fragment_unresolved",
                    ));
                }
                ItemSyntax::Sentence | ItemSyntax::Fragment => {}
            }
        }
    }

    diagnostics
}

fn terminal_punctuation_diagnostic(
    item: ListItem,
    severity: Severity,
    message: &str,
    mechanic: &str,
) -> Diagnostic {
    Diagnostic {
        code: "STE-LIST-004".into(),
        severity,
        message: message.into(),
        span: Span {
            start: item.content_start,
            end: item.content_end,
        },
        rules: vec!["4.3".into()],
        evidence: Some(json!({
            "coverage": "structured_vertical_list_mechanics_v2",
            "mechanic": mechanic
        })),
        autofix: None,
    }
}

fn item_syntax(analysis: &AnalysisDocument<'_>, item: ListItem) -> ItemSyntax {
    let token_indices = analysis
        .tokens()
        .iter()
        .enumerate()
        .filter(|(_, token)| token.start >= item.content_start && token.end <= item.content_end)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let (Some(&first), Some(&last_inclusive)) = (token_indices.first(), token_indices.last())
    else {
        return ItemSyntax::Unresolved;
    };
    let last = last_inclusive + 1;
    let Some(sentence_id) = analysis.tokens()[first].sentence_id else {
        return ItemSyntax::Unresolved;
    };

    if matches!(
        analysis.action_structure(sentence_id),
        Resolution::Resolved(_) | Resolution::Ambiguous(_)
    ) || matches!(
        analysis.subject_predicate(sentence_id),
        Resolution::Resolved(_) | Resolution::Ambiguous(_)
    ) {
        return ItemSyntax::Sentence;
    }

    if determiner_led_nominal_fragment(analysis, &token_indices) {
        return ItemSyntax::Fragment;
    }

    match analysis.noun_phrase_at(first) {
        Resolution::Resolved(noun_phrase)
            if noun_phrase.span.token_start == first && noun_phrase.span.token_end == last =>
        {
            ItemSyntax::Fragment
        }
        Resolution::Ambiguous(noun_phrases)
            if !noun_phrases.is_empty()
                && noun_phrases.iter().all(|noun_phrase| {
                    noun_phrase.span.token_start == first && noun_phrase.span.token_end == last
                }) =>
        {
            ItemSyntax::Fragment
        }
        Resolution::Resolved(_) | Resolution::Ambiguous(_) | Resolution::Unknown => {
            ItemSyntax::Unresolved
        }
    }
}

fn determiner_led_nominal_fragment(
    analysis: &AnalysisDocument<'_>,
    token_indices: &[usize],
) -> bool {
    let Some((&first, rest)) = token_indices.split_first() else {
        return false;
    };
    if rest.is_empty()
        || !analysis
            .linguistic_token(first)
            .is_some_and(|evidence| evidence.determiner)
    {
        return false;
    }

    let nominal_tail = rest.iter().all(|&index| {
        analysis.linguistic_token(index).is_some_and(|evidence| {
            evidence.adjective || evidence.noun || evidence.nominal || evidence.np_member
        })
    });
    let nominal_head = rest.last().is_some_and(|&&index| {
        analysis
            .linguistic_token(index)
            .is_some_and(|evidence| evidence.noun || evidence.nominal)
    });

    nominal_tail && nominal_head
}
