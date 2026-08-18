use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::analysis::linguistic::GenericPos;
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
    let mut index = 0usize;

    while index < analysis.tokens().len() {
        if !is_multiword_noun_member(analysis, index) {
            index += 1;
            continue;
        }

        let sentence_id = analysis.tokens()[index].sentence_id;
        let start = index;
        let mut end = index + 1;
        while end < analysis.tokens().len()
            && analysis.tokens()[end].sentence_id == sentence_id
            && is_multiword_noun_member(analysis, end)
            && compound_separator_is_valid(analysis, end - 1, end)
        {
            end += 1;
        }

        let head = end - 1;
        let head_is_noun = analysis
            .linguistic_token(head)
            .and_then(|evidence| evidence.occurrence_pos)
            .is_some_and(|pos| matches!(pos, GenericPos::Noun | GenericPos::ProperNoun));
        if head_is_noun && end - start > 1 {
            let word_count = multiword_noun_word_count(analysis, start, end);
            if word_count > 3 {
                diagnostics.push(Diagnostic {
                    code: "STE-NOUN-001".into(),
                    severity: Severity::Error,
                    message: "Multi-word noun contains more than three words.".into(),
                    span: Span {
                        start: analysis.tokens()[start].start,
                        end: analysis.tokens()[head].end,
                    },
                    rules: vec!["2.1".into()],
                    evidence: Some(json!({
                        "grammar_resolution": "harper_brill_noun_adjective_compound",
                        "word_count": word_count,
                        "token_start": start,
                        "token_end": end,
                        "head_token": head,
                        "hyphenated_source_groups_count_as_one": true,
                    })),
                    autofix: None,
                });
            }
        }

        index = end.max(index + 1);
    }

    diagnostics
}

fn is_multiword_noun_member(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    analysis
        .linguistic_token(token_index)
        .and_then(|evidence| evidence.occurrence_pos)
        .is_some_and(|pos| {
            matches!(
                pos,
                GenericPos::Adjective | GenericPos::Noun | GenericPos::ProperNoun
            )
        })
}

fn compound_separator_is_valid(
    analysis: &AnalysisDocument<'_>,
    left: usize,
    right: usize,
) -> bool {
    let separator = &analysis.text()[analysis.tokens()[left].end..analysis.tokens()[right].start];
    !separator.is_empty()
        && separator
            .chars()
            .all(|character| character.is_whitespace() || character == '-')
}

fn multiword_noun_word_count(
    analysis: &AnalysisDocument<'_>,
    token_start: usize,
    token_end: usize,
) -> usize {
    let mut count = 1usize;
    for index in token_start + 1..token_end {
        let separator = &analysis.text()
            [analysis.tokens()[index - 1].end..analysis.tokens()[index].start];
        let hyphenated = separator.contains('-') && !separator.chars().any(char::is_whitespace);
        if !hyphenated {
            count += 1;
        }
    }
    count
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
            rules: vec!["3.3".into(), "3.6".into()],
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
