use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::{AnalysisDocument, DocumentNode, DocumentNodeKind, Resolution};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let graph = analysis.document_graph();
    let mut diagnostics = Vec::new();

    for ordering in &graph.semantic_orderings {
        let before_id = match &ordering.before {
            Resolution::Resolved(id) => *id,
            Resolution::Ambiguous(_) | Resolution::Unknown => continue,
        };
        let after_id = match &ordering.after {
            Resolution::Resolved(id) => *id,
            Resolution::Ambiguous(_) | Resolution::Unknown => continue,
        };
        let Some(before) = graph.nodes.iter().find(|node| node.id == before_id) else {
            continue;
        };
        let Some(after) = graph.nodes.iter().find(|node| node.id == after_id) else {
            continue;
        };

        if after.span.end > before.span.start {
            continue;
        }

        diagnostics.push(Diagnostic {
            code: "STE-DISC-001".into(),
            severity: Severity::Error,
            message: "Descriptive information appears in the reverse of an explicit project-supplied semantic ordering.".into(),
            span: Span {
                start: after.span.start,
                end: before.span.end,
            },
            rules: vec!["6.1".into()],
            evidence: Some(json!({
                "resolution": "resolved_reversed",
                "source": ordering.source,
                "expected_before": node_evidence(before),
                "expected_after": node_evidence(after),
            })),
            autofix: None,
        });
    }

    diagnostics
}

fn node_evidence(node: &DocumentNode) -> serde_json::Value {
    json!({
        "kind": kind_name(node.id.kind),
        "index": node.id.index,
        "start": node.span.start,
        "end": node.span.end,
    })
}

fn kind_name(kind: DocumentNodeKind) -> &'static str {
    match kind {
        DocumentNodeKind::Sentence => "sentence",
        DocumentNodeKind::Paragraph => "paragraph",
        DocumentNodeKind::Topic => "topic",
        DocumentNodeKind::EntityMention => "entity_mention",
    }
}
