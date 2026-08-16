use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::{AnalysisDocument, IngRole, LintMode, ParticipleRole, Resolution};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = multiword_noun_diagnostics(analysis);
    diagnostics.extend(progressive_ing_diagnostics(analysis));
    if analysis.mode() == LintMode::Procedural {
        diagnostics.extend(procedural_passive_diagnostics(analysis));
    }
    diagnostics
}

fn multiword_noun_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for token_index in 0..analysis.tokens().len() {
        let Resolution::Resolved(noun_phrase) = analysis.noun_phrase_at(token_index) else {
            continue;
        };
        let content_word_count = noun_phrase
            .span
            .token_end
            .saturating_sub(noun_phrase.span.token_start + 1);
        if content_word_count <= 3 {
            continue;
        }

        diagnostics.push(Diagnostic {
            code: "STE-NOUN-001".into(),
            severity: Severity::Error,
            message: "Resolved multi-word noun contains more than three content words.".into(),
            span: Span {
                start: noun_phrase.span.start,
                end: noun_phrase.span.end,
            },
            rules: vec!["2.1".into()],
            evidence: Some(json!({
                "grammar_resolution": "resolved_multiword_noun",
                "content_word_count": content_word_count,
                "token_start": noun_phrase.span.token_start,
                "token_end": noun_phrase.span.token_end,
                "head_token": noun_phrase.head_token,
            })),
            autofix: None,
        });
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
