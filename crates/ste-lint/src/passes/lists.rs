use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::document_structure::simple_list_blocks;

pub(crate) fn check(text: &str) -> Vec<Diagnostic> {
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
                    "coverage": "simple_vertical_list_mechanics_v1",
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
                        "coverage": "simple_vertical_list_mechanics_v1",
                        "mechanic": "uppercase_item_start"
                    })),
                    autofix: None,
                });
            }

            if content.ends_with(',') || content.ends_with(';') {
                diagnostics.push(Diagnostic {
                    code: "STE-LIST-003".into(),
                    severity: Severity::Error,
                    message: "Do not end a vertical-list item with a comma or semicolon.".into(),
                    span: Span {
                        start: item.content_start,
                        end: item.content_end,
                    },
                    rules: vec!["4.3".into(), "8.1".into()],
                    evidence: Some(json!({
                        "coverage": "simple_vertical_list_mechanics_v1",
                        "mechanic": "forbidden_item_end_punctuation"
                    })),
                    autofix: None,
                });
            }
        }

        if let Some(last) = block.items.last() {
            let content = text[last.content_start..last.content_end].trim_end();
            if !content.ends_with('.') {
                diagnostics.push(Diagnostic {
                    code: "STE-LIST-004".into(),
                    severity: Severity::Error,
                    message: "Put a period at the end of the last item in a vertical list.".into(),
                    span: Span {
                        start: last.content_start,
                        end: last.content_end,
                    },
                    rules: vec!["4.3".into()],
                    evidence: Some(json!({
                        "coverage": "simple_vertical_list_mechanics_v1",
                        "mechanic": "last_item_period"
                    })),
                    autofix: None,
                });
            }
        }
    }

    diagnostics
}
