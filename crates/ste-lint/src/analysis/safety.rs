use ste_data::{ApprovalStatus, PartOfSpeech, VerbClassification};

use crate::context::{SafetyFact, SafetyLevelFact, SafetySpanFact};
use crate::document_structure::{SafetyBlock, SafetyLabel, safety_blocks, starts_condition};

use super::document::{AnalysisDocument, Resolution};
use super::document_graph::DocumentSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyLevel {
    Warning,
    Caution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyEvidenceSource {
    Structure,
    Context(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyLevelEvidence {
    pub level: SafetyLevel,
    pub source: SafetyEvidenceSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetySpanEvidence {
    pub span: DocumentSpan,
    pub source: SafetyEvidenceSource,
    pub basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetySemantics {
    pub span: DocumentSpan,
    pub level: Resolution<SafetyLevelEvidence>,
    pub actor: Resolution<SafetySpanEvidence>,
    pub command: Resolution<SafetySpanEvidence>,
    pub hazard: Resolution<SafetySpanEvidence>,
    pub consequence: Resolution<SafetySpanEvidence>,
}

impl<'a> AnalysisDocument<'a> {
    pub fn safety_semantics(&self) -> Vec<SafetySemantics> {
        safety_semantics(self)
    }
}

fn safety_semantics(analysis: &AnalysisDocument<'_>) -> Vec<SafetySemantics> {
    safety_blocks(analysis.text())
        .into_iter()
        .map(|block| {
            let facts = matching_facts(analysis, block);
            SafetySemantics {
                span: DocumentSpan {
                    start: block.start,
                    end: block.end,
                },
                level: resolve_level(block.label, &facts),
                actor: resolve_context_spans(&facts, |fact| fact.actor, "actor"),
                command: combine_span_resolutions(
                    structural_command(analysis, block),
                    resolve_context_spans(&facts, |fact| fact.command, "command"),
                ),
                hazard: resolve_context_spans(&facts, |fact| fact.hazard, "hazard"),
                consequence: resolve_context_spans(
                    &facts,
                    |fact| fact.consequence,
                    "consequence",
                ),
            }
        })
        .collect()
}

fn matching_facts<'a>(
    analysis: &'a AnalysisDocument<'_>,
    block: SafetyBlock,
) -> Vec<&'a SafetyFact> {
    analysis.context().map_or_else(Vec::new, |context| {
        context
            .safety_facts
            .iter()
            .filter(|fact| fact.start == block.start && fact.end == block.end)
            .collect()
    })
}

fn resolve_level(
    structural: SafetyLabel,
    facts: &[&SafetyFact],
) -> Resolution<SafetyLevelEvidence> {
    let structural_level = match structural {
        SafetyLabel::Warning => SafetyLevel::Warning,
        SafetyLabel::Caution => SafetyLevel::Caution,
    };
    let structural_evidence = SafetyLevelEvidence {
        level: structural_level,
        source: SafetyEvidenceSource::Structure,
    };
    let mut levels = vec![structural_evidence.clone()];
    for fact in facts {
        let Some(level) = fact.safety_level else {
            continue;
        };
        let level = match level {
            SafetyLevelFact::Warning => SafetyLevel::Warning,
            SafetyLevelFact::Caution => SafetyLevel::Caution,
        };
        if level != structural_level && !levels.iter().any(|candidate| candidate.level == level) {
            levels.push(SafetyLevelEvidence {
                level,
                source: SafetyEvidenceSource::Context(fact.source.clone()),
            });
        }
    }
    if levels.len() == 1 {
        Resolution::Resolved(structural_evidence)
    } else {
        Resolution::Ambiguous(levels)
    }
}

fn structural_command(
    analysis: &AnalysisDocument<'_>,
    block: SafetyBlock,
) -> Resolution<SafetySpanEvidence> {
    if block.content_start >= block.content_end {
        return Resolution::Unknown;
    }
    let content = &analysis.text()[block.content_start..block.content_end];
    if starts_condition(content) {
        return Resolution::Unknown;
    }
    let Some(matched) =
        analysis.leading_dictionary_match_in_span(block.content_start, block.content_end, 8)
    else {
        return Resolution::Unknown;
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
    let evidence = SafetySpanEvidence {
        span: DocumentSpan {
            start: block.content_start,
            end: block.content_start + matched.text.len(),
        },
        source: SafetyEvidenceSource::Structure,
        basis: "imperative_command".into(),
    };
    if base_verbs > 0 && base_verbs == matched.candidates.len() {
        Resolution::Resolved(evidence)
    } else if base_verbs > 0 {
        Resolution::Ambiguous(vec![
            evidence,
            SafetySpanEvidence {
                span: DocumentSpan {
                    start: block.content_start,
                    end: block.content_start + matched.text.len(),
                },
                source: SafetyEvidenceSource::Structure,
                basis: "competing_dictionary_identity".into(),
            },
        ])
    } else {
        Resolution::Unknown
    }
}

fn resolve_context_spans<F>(
    facts: &[&SafetyFact],
    select: F,
    basis: &str,
) -> Resolution<SafetySpanEvidence>
where
    F: Fn(&SafetyFact) -> Option<SafetySpanFact>,
{
    let mut candidates = Vec::new();
    for fact in facts {
        let Some(span) = select(fact) else {
            continue;
        };
        if candidates
            .iter()
            .any(|candidate: &SafetySpanEvidence| {
                candidate.span.start == span.start && candidate.span.end == span.end
            })
        {
            continue;
        }
        candidates.push(SafetySpanEvidence {
            span: DocumentSpan {
                start: span.start,
                end: span.end,
            },
            source: SafetyEvidenceSource::Context(fact.source.clone()),
            basis: basis.into(),
        });
    }
    resolution_from_candidates(candidates)
}

fn combine_span_resolutions(
    left: Resolution<SafetySpanEvidence>,
    right: Resolution<SafetySpanEvidence>,
) -> Resolution<SafetySpanEvidence> {
    match (left, right) {
        (Resolution::Unknown, resolution) | (resolution, Resolution::Unknown) => resolution,
        (Resolution::Resolved(left), Resolution::Resolved(right)) => {
            if left.span == right.span {
                Resolution::Resolved(left)
            } else {
                Resolution::Ambiguous(vec![left, right])
            }
        }
        (Resolution::Ambiguous(mut candidates), Resolution::Resolved(candidate))
        | (Resolution::Resolved(candidate), Resolution::Ambiguous(mut candidates)) => {
            candidates.push(candidate);
            Resolution::Ambiguous(candidates)
        }
        (Resolution::Ambiguous(mut left), Resolution::Ambiguous(right)) => {
            left.extend(right);
            Resolution::Ambiguous(left)
        }
    }
}

fn resolution_from_candidates<T>(mut candidates: Vec<T>) -> Resolution<T> {
    match candidates.len() {
        0 => Resolution::Unknown,
        1 => Resolution::Resolved(candidates.pop().unwrap()),
        _ => Resolution::Ambiguous(candidates),
    }
}
