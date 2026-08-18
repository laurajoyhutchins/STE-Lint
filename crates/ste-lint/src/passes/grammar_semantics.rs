use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, PartOfSpeech};

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
    let mut diagnostics = determiner_led_multiword_noun_diagnostics(analysis);
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
        if is_multiword_noun_head(analysis, head) && end - start > 1 {
            let word_count = multiword_noun_word_count(analysis, start, end);
            if word_count > 3 {
                diagnostics.push(multiword_noun_diagnostic(
                    analysis,
                    start,
                    end,
                    head,
                    word_count,
                    "harper_brill_noun_phrase_chunk_with_lexical_nominal_compatibility",
                ));
            }
        }

        index = end.max(index + 1);
    }

    diagnostics
}

fn determiner_led_multiword_noun_diagnostics(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut index = 0usize;

    while index < analysis.tokens().len() {
        if !has_only_approved_pos(analysis, index, &[PartOfSpeech::Article]) {
            index += 1;
            continue;
        }

        let sentence_id = analysis.tokens()[index].sentence_id;
        let content_start = index + 1;
        let mut end = content_start;
        while end < analysis.tokens().len()
            && analysis.tokens()[end].sentence_id == sentence_id
            && (end == content_start || compound_separator_is_valid(analysis, end - 1, end))
            && has_only_approved_pos(
                analysis,
                end,
                &[PartOfSpeech::Adjective, PartOfSpeech::Noun],
            )
        {
            end += 1;
        }

        if end > content_start && has_only_approved_pos(analysis, end - 1, &[PartOfSpeech::Noun]) {
            let content_word_count = multiword_noun_word_count(analysis, content_start, end);
            if content_word_count > 3 {
                diagnostics.push(multiword_noun_diagnostic(
                    analysis,
                    index,
                    end,
                    end - 1,
                    content_word_count,
                    "authoritative_determiner_led_noun_phrase",
                ));
            }
        }

        index = end.max(index + 1);
    }

    diagnostics
}

fn has_only_approved_pos(
    analysis: &AnalysisDocument<'_>,
    token_index: usize,
    allowed: &[PartOfSpeech],
) -> bool {
    let Some(matched) = analysis.dictionary_match_at(token_index, 1) else {
        return false;
    };
    !matched.candidates.is_empty()
        && matched.candidates.iter().all(|entry| {
            entry.status == ApprovalStatus::Approved
                && entry
                    .part_of_speech
                    .is_some_and(|part_of_speech| allowed.contains(&part_of_speech))
        })
}

fn multiword_noun_diagnostic(
    analysis: &AnalysisDocument<'_>,
    start: usize,
    end: usize,
    head: usize,
    content_word_count: usize,
    grammar_resolution: &str,
) -> Diagnostic {
    Diagnostic {
        code: "STE-NOUN-001".into(),
        severity: Severity::Error,
        message: "Multi-word noun contains more than three words.".into(),
        span: Span {
            start: analysis.tokens()[start].start,
            end: analysis.tokens()[head].end,
        },
        rules: vec!["2.1".into()],
        evidence: Some(json!({
            "grammar_resolution": grammar_resolution,
            "content_word_count": content_word_count,
            "word_count": content_word_count,
            "token_start": start,
            "token_end": end,
            "head_token": head,
            "hyphenated_source_groups_count_as_one": true,
        })),
        autofix: None,
    }
}

fn is_multiword_noun_member(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    let Some(evidence) = analysis.linguistic_token(token_index) else {
        return false;
    };
    if !evidence.np_member {
        return false;
    }

    evidence.occurrence_pos.is_some_and(|pos| {
        matches!(
            pos,
            GenericPos::Adjective | GenericPos::Noun | GenericPos::ProperNoun
        )
    }) || evidence.adjective
        || evidence.noun
        || evidence.nominal
}

fn is_multiword_noun_head(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    let Some(evidence) = analysis.linguistic_token(token_index) else {
        return false;
    };
    evidence.np_member
        && (evidence
            .occurrence_pos
            .is_some_and(|pos| matches!(pos, GenericPos::Noun | GenericPos::ProperNoun))
            || evidence.noun
            || evidence.nominal)
}

fn compound_separator_is_valid(analysis: &AnalysisDocument<'_>, left: usize, right: usize) -> bool {
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
        let separator =
            &analysis.text()[analysis.tokens()[index - 1].end..analysis.tokens()[index].start];
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
