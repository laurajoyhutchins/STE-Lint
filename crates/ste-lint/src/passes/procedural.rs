use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, PartOfSpeech, VerbClassification};

use crate::document_structure::{safety_blocks, starts_condition};
use crate::{
    ActionCardinality, AnalysisDocument, LintMode, Resolution, SafetyEvidenceSource, SafetyLevel,
    SafetyLevelFact,
};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = safety_level_mismatches(analysis);
    diagnostics.extend(safety_openings(analysis));
    if analysis.mode() != LintMode::Procedural {
        return diagnostics;
    }

    diagnostics.extend(action_cardinality(analysis));
    diagnostics.extend(condition_commas(analysis));
    diagnostics.extend(imperative_forms(analysis));
    diagnostics
}

fn action_cardinality(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for sentence in analysis.sentences() {
        let Resolution::Resolved(action) = analysis.action_structure(sentence.id) else {
            continue;
        };
        if action.cardinality != ActionCardinality::Multiple {
            continue;
        }

        let action_heads = action
            .action_heads
            .iter()
            .map(|head| {
                json!({
                    "start": head.start,
                    "end": head.end,
                    "token_start": head.token_start,
                    "token_end": head.token_end,
                })
            })
            .collect::<Vec<_>>();
        diagnostics.push(Diagnostic {
            code: "STE-PROC-003".into(),
            severity: Severity::Error,
            message: "Resolved procedural instruction contains more than one action.".into(),
            span: Span {
                start: sentence.start,
                end: sentence.end,
            },
            rules: vec!["5.2".into()],
            evidence: Some(json!({
                "action_resolution": "resolved_multiple",
                "action_count": action.action_heads.len(),
                "action_heads": action_heads,
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn safety_level_mismatches(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let Some(context) = analysis.context() else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();

    for safety in analysis.safety_semantics() {
        let Resolution::Ambiguous(levels) = &safety.level else {
            continue;
        };
        let Some(structural) = levels
            .iter()
            .find(|candidate| matches!(&candidate.source, SafetyEvidenceSource::Structure))
        else {
            continue;
        };

        let mut supplied_levels = Vec::new();
        let mut supplied_sources = Vec::new();
        for fact in context
            .safety_facts
            .iter()
            .filter(|fact| fact.start == safety.span.start && fact.end == safety.span.end)
        {
            let Some(level) = fact.safety_level else {
                continue;
            };
            let level = safety_level_from_fact(level);
            if !supplied_levels.contains(&level) {
                supplied_levels.push(level);
            }
            if !supplied_sources
                .iter()
                .any(|source: &String| source == &fact.source)
            {
                supplied_sources.push(fact.source.clone());
            }
        }

        if supplied_levels.len() != 1 {
            continue;
        }
        let supplied_level = supplied_levels[0];
        if supplied_level == structural.level
            || !levels.iter().any(|candidate| {
                candidate.level == supplied_level
                    && matches!(&candidate.source, SafetyEvidenceSource::Context(_))
            })
        {
            continue;
        }

        diagnostics.push(Diagnostic {
            code: "STE-SAFE-003".into(),
            severity: Severity::Error,
            message:
                "Visible safety label does not agree with the unambiguous supplied project risk level."
                    .into(),
            span: Span {
                start: safety.span.start,
                end: safety.span.end,
            },
            rules: vec!["7.1".into()],
            evidence: Some(json!({
                "safety_resolution": "structural_context_level_mismatch",
                "visible_level": safety_level_name(structural.level),
                "supplied_level": safety_level_name(supplied_level),
                "context_sources": supplied_sources,
            })),
            autofix: None,
        });
    }

    diagnostics
}

fn safety_level_from_fact(level: SafetyLevelFact) -> SafetyLevel {
    match level {
        SafetyLevelFact::Warning => SafetyLevel::Warning,
        SafetyLevelFact::Caution => SafetyLevel::Caution,
    }
}

fn safety_level_name(level: SafetyLevel) -> &'static str {
    match level {
        SafetyLevel::Warning => "warning",
        SafetyLevel::Caution => "caution",
    }
}

fn imperative_forms(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let mut diagnostics = Vec::new();
    for sentence_span in analysis.sentences() {
        let start = sentence_span.start;
        let end = sentence_span.end;
        let raw_sentence = &text[start..end];
        let sentence = raw_sentence.trim_start();
        if sentence.is_empty()
            || starts_label(sentence, "NOTE")
            || starts_label(sentence, "WARNING")
            || starts_label(sentence, "CAUTION")
        {
            continue;
        }

        let command_start = if starts_condition(sentence) {
            let Some(comma) = raw_sentence.find(',') else {
                // Rule 5.4 owns the missing condition separator. Without the
                // separator there is no deterministic command boundary for 5.3.
                continue;
            };
            start + comma + 1
        } else {
            start
        };

        let Some((token_index, token)) = analysis.first_token_in_span(command_start, end) else {
            continue;
        };
        let Some(matched) = analysis.dictionary_match_at(token_index, 1) else {
            // Unknown terminology is already fail-closed in the lexical pass.
            // Do not manufacture an imperative identity from generic syntax.
            continue;
        };

        let approved_base_verbs = matched
            .candidates
            .iter()
            .filter(|entry| {
                entry.status == ApprovalStatus::Approved
                    && entry.part_of_speech == Some(PartOfSpeech::Verb)
                    && entry.verb_paradigm.as_ref().is_some_and(|paradigm| {
                        paradigm.classification == VerbClassification::Lexical
                            && token.text.eq_ignore_ascii_case(&paradigm.base_form)
                    })
            })
            .collect::<Vec<_>>();

        if !approved_base_verbs.is_empty() && approved_base_verbs.len() == matched.candidates.len()
        {
            continue;
        }

        if !approved_base_verbs.is_empty() {
            diagnostics.push(Diagnostic {
                code: "STE-PROC-004".into(),
                severity: Severity::Blocked,
                message: format!(
                    "Procedural command head '{}' has competing authoritative identities; imperative status cannot be selected safely.",
                    token.text
                ),
                span: Span {
                    start: token.start,
                    end: token.end,
                },
                rules: vec!["5.3".into()],
                evidence: Some(json!({
                    "command_head": token.text,
                    "candidate_count": matched.candidates.len(),
                    "approved_base_verb_candidates": approved_base_verbs.len(),
                    "requires_disambiguation": true,
                })),
                autofix: None,
            });
            continue;
        }

        let source_backed_verbs = matched
            .candidates
            .iter()
            .filter(|entry| {
                entry.status == ApprovalStatus::Approved
                    && entry.part_of_speech == Some(PartOfSpeech::Verb)
                    && entry.verb_paradigm.is_some()
            })
            .collect::<Vec<_>>();
        let base_forms = source_backed_verbs
            .iter()
            .filter_map(|entry| entry.verb_paradigm.as_ref())
            .map(|paradigm| paradigm.base_form.as_str())
            .collect::<Vec<_>>();
        let base_form = if base_forms.len() == 1 {
            base_forms.first().copied()
        } else {
            None
        };

        diagnostics.push(Diagnostic {
            code: "STE-PROC-001".into(),
            severity: Severity::Error,
            message: if base_forms.len() == 1 {
                format!(
                    "Procedural command head '{}' is not the source-backed imperative base form '{}'.",
                    token.text, base_forms[0]
                )
            } else {
                format!(
                    "Procedural instruction does not start its command with a uniquely approved imperative base-form verb; observed '{}'.",
                    token.text
                )
            },
            span: Span {
                start: token.start,
                end: token.end,
            },
            rules: vec!["5.3".into()],
            evidence: Some(json!({
                "command_head": token.text,
                "observed_form": token.text,
                "candidate_count": matched.candidates.len(),
                "source_backed_verb_candidates": source_backed_verbs.len(),
                "base_form": base_form,
                "base_forms": base_forms,
                "condition_prefix": starts_condition(sentence),
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn condition_commas(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let mut diagnostics = Vec::new();
    for sentence_span in analysis.sentences() {
        let start = sentence_span.start;
        let end = sentence_span.end;
        let sentence = text[start..end].trim_start();
        if !starts_condition(sentence) || sentence.contains(',') {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: "STE-PROC-002".into(),
            severity: Severity::Error,
            message: "Leading procedural condition must be separated from its command by a comma."
                .into(),
            span: Span { start, end },
            rules: vec!["5.4".into()],
            evidence: Some(json!({
                "condition_position": "before_command",
                "required_separator": "comma",
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn safety_openings(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let text = analysis.text();
    let mut diagnostics = Vec::new();

    for block in safety_blocks(text) {
        if block.content_start >= block.content_end {
            continue;
        }
        let content = &text[block.content_start..block.content_end];
        if starts_condition(content) {
            continue;
        }

        let Some(matched) =
            analysis.leading_dictionary_match_in_span(block.content_start, block.content_end, 8)
        else {
            if let Some((_, token)) =
                analysis.first_token_in_span(block.content_start, block.content_end)
            {
                diagnostics.push(safety_error(token.start, token.end));
            } else {
                diagnostics.push(safety_error(
                    block.content_start,
                    (block.content_start + 1).min(block.content_end),
                ));
            }
            continue;
        };

        let base_verbs = matched
            .candidates
            .iter()
            .filter(|entry| {
                entry.status == ApprovalStatus::Approved
                    && entry.part_of_speech == Some(PartOfSpeech::Verb)
                    && entry.verb_paradigm.as_ref().is_some_and(|paradigm| {
                        paradigm.classification == VerbClassification::Lexical
                            && matched.text.eq_ignore_ascii_case(&paradigm.base_form)
                    })
            })
            .count();

        if base_verbs > 0 && base_verbs == matched.candidates.len() {
            continue;
        }

        if base_verbs > 0 {
            diagnostics.push(Diagnostic {
                code: "STE-SAFE-002".into(),
                severity: Severity::Blocked,
                message: format!(
                    "Safety instruction opening '{}' has competing approved dictionary identities; command role cannot be selected safely.",
                    matched.text
                ),
                span: Span {
                    start: matched.start,
                    end: matched.end,
                },
                rules: vec!["7.2".into()],
                evidence: Some(json!({
                    "opening": matched.text,
                    "candidate_count": matched.candidates.len(),
                    "base_verb_candidates": base_verbs,
                    "requires_disambiguation": true,
                })),
                autofix: None,
            });
        } else {
            diagnostics.push(safety_error(matched.start, matched.end));
        }
    }

    diagnostics
}

fn safety_error(start: usize, end: usize) -> Diagnostic {
    Diagnostic {
        code: "STE-SAFE-001".into(),
        severity: Severity::Error,
        message: "Safety instruction must start with a clear command or condition.".into(),
        span: Span { start, end },
        rules: vec!["7.2".into()],
        evidence: Some(json!({
            "required_opening": ["imperative_command", "condition"],
        })),
        autofix: None,
    }
}

fn starts_label(text: &str, label: &str) -> bool {
    text.get(..label.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(label))
        && text[label.len()..].trim_start().starts_with(':')
}
