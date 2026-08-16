use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::{AnalysisDocument, IngRole, LintMode, ParticipleRole, Resolution};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = progressive_ing_diagnostics(analysis);
    if analysis.mode() == LintMode::Procedural {
        diagnostics.extend(procedural_passive_diagnostics(analysis));
    }
    diagnostics
}

fn progressive_ing_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for token_index in 0..analysis.tokens().len() {
        let Resolution::Resolved(ing_use) = analysis.ing_use_at(token_index) else {
            continue;
        };
        if ing_use.role != IngRole::Progressive {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: "STE-GRAM-002".into(),
            severity: Severity::Error,
            message: "Resolved progressive -ing verb use is outside this bounded STE grammar rule."
                .into(),
            span: Span {
                start: ing_use.span.start,
                end: ing_use.span.end,
            },
            rules: vec!["3.5".into()],
            evidence: Some(json!({
                "grammar_resolution": "resolved_progressive",
                "token_start": ing_use.span.token_start,
                "token_end": ing_use.span.token_end,
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn procedural_passive_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for token_index in 0..analysis.tokens().len() {
        let Resolution::Resolved(participle_use) = analysis.participle_use_at(token_index) else {
            continue;
        };
        if participle_use.role != ParticipleRole::PassiveVerb {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: "STE-GRAM-003".into(),
            severity: Severity::Error,
            message: "Resolved passive construction is outside this bounded procedural active-voice rule."
                .into(),
            span: Span {
                start: participle_use.span.start,
                end: participle_use.span.end,
            },
            rules: vec!["3.6".into()],
            evidence: Some(json!({
                "grammar_resolution": "resolved_passive_verb",
                "mode": "procedural",
                "token_start": participle_use.span.token_start,
                "token_end": participle_use.span.token_end,
            })),
            autofix: None,
        });
    }
    diagnostics
}
