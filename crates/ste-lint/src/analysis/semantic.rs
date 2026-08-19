use super::document::Resolution;
use super::evidence::{AnalysisEvidence, EvidenceTarget};
use super::source::CanonicalSpan;

/// Shared STE-specific clause semantics that are useful to multiple rule passes.
///
/// These values are linguistic evidence only. They do not grant ASD-STE100,
/// terminology, lexical, form, meaning, or project/domain authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClauseKind {
    Condition,
    Requirement,
    LimitOrTolerance,
    WorkStepResult,
}

/// Minimum evidence policy for resolving an STE clause kind.
///
/// Evidence below the threshold is ignored. Any two distinct qualifying
/// candidates remain ambiguous regardless of their relative scores.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticResolutionPolicy {
    pub minimum_confidence: f32,
}

impl Default for SemanticResolutionPolicy {
    fn default() -> Self {
        Self {
            minimum_confidence: 0.80,
        }
    }
}

/// Resolve provider-neutral semantic evidence for one canonical source span.
///
/// Resolution is intentionally conservative:
/// - evidence without a usable confidence score cannot resolve the fact;
/// - low-confidence evidence remains unknown;
/// - distinct qualifying candidates remain ambiguous;
/// - agreement from multiple providers may resolve the same candidate.
pub fn resolve_clause_kind(
    evidence: &[AnalysisEvidence<ClauseKind>],
    span: CanonicalSpan,
    policy: SemanticResolutionPolicy,
) -> Resolution<ClauseKind> {
    if !(0.0..=1.0).contains(&policy.minimum_confidence) {
        return Resolution::Unknown;
    }

    let mut candidates = Vec::new();
    for observation in evidence {
        if evidence_span(observation.target) != Some(span) {
            continue;
        }
        if observation
            .confidence
            .is_some_and(|confidence| confidence >= policy.minimum_confidence)
        {
            push_unique(&mut candidates, observation.value);
        }
        for alternative in &observation.alternatives {
            if alternative
                .confidence
                .is_some_and(|confidence| confidence >= policy.minimum_confidence)
            {
                push_unique(&mut candidates, alternative.value);
            }
        }
    }

    candidates.sort_by_key(|kind| clause_kind_order(*kind));
    match candidates.as_slice() {
        [] => Resolution::Unknown,
        [only] => Resolution::Resolved(*only),
        many => Resolution::Ambiguous(many.to_vec()),
    }
}

fn evidence_span(target: EvidenceTarget) -> Option<CanonicalSpan> {
    match target {
        EvidenceTarget::Span(span) | EvidenceTarget::Token(span) => Some(span),
        EvidenceTarget::Sentence { span, .. } | EvidenceTarget::Paragraph { span, .. } => {
            Some(span)
        }
        EvidenceTarget::Relation { .. } | EvidenceTarget::Document => None,
    }
}

fn push_unique(candidates: &mut Vec<ClauseKind>, candidate: ClauseKind) {
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn clause_kind_order(kind: ClauseKind) -> u8 {
    match kind {
        ClauseKind::Condition => 0,
        ClauseKind::Requirement => 1,
        ClauseKind::LimitOrTolerance => 2,
        ClauseKind::WorkStepResult => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{
        EvidenceAlternative, EvidenceProvenance, ModelIdentity, ProviderIdentity,
    };

    fn evidence(
        kind: ClauseKind,
        span: CanonicalSpan,
        confidence: Option<f32>,
        provider: &str,
    ) -> AnalysisEvidence<ClauseKind> {
        let mut evidence = AnalysisEvidence::new(
            kind,
            EvidenceTarget::Span(span),
            EvidenceProvenance {
                provider: ProviderIdentity {
                    name: provider.into(),
                    version: Some("test".into()),
                },
                model: Some(ModelIdentity {
                    name: "source-safe-clause-kind".into(),
                    version: "v1".into(),
                    artifact_sha256: Some("0".repeat(64)),
                }),
            },
        );
        evidence.confidence = confidence;
        evidence
    }

    #[test]
    fn high_confidence_agreement_resolves_one_shared_fact() {
        let span = CanonicalSpan { start: 0, end: 12 };
        let evidence = vec![
            evidence(ClauseKind::Condition, span, Some(0.91), "provider-a"),
            evidence(ClauseKind::Condition, span, Some(0.88), "provider-b"),
        ];

        assert_eq!(
            resolve_clause_kind(&evidence, span, SemanticResolutionPolicy::default()),
            Resolution::Resolved(ClauseKind::Condition)
        );
    }

    #[test]
    fn low_confidence_evidence_remains_unknown() {
        let span = CanonicalSpan { start: 0, end: 12 };
        let evidence = vec![evidence(
            ClauseKind::Requirement,
            span,
            Some(0.79),
            "provider-a",
        )];

        assert_eq!(
            resolve_clause_kind(&evidence, span, SemanticResolutionPolicy::default()),
            Resolution::Unknown
        );
    }

    #[test]
    fn conflicting_qualifying_evidence_fails_closed_as_ambiguous() {
        let span = CanonicalSpan { start: 4, end: 20 };
        let evidence = vec![
            evidence(ClauseKind::Requirement, span, Some(0.96), "provider-a"),
            evidence(
                ClauseKind::LimitOrTolerance,
                span,
                Some(0.84),
                "provider-b",
            ),
        ];

        assert_eq!(
            resolve_clause_kind(&evidence, span, SemanticResolutionPolicy::default()),
            Resolution::Ambiguous(vec![
                ClauseKind::Requirement,
                ClauseKind::LimitOrTolerance,
            ])
        );
    }

    #[test]
    fn qualifying_alternative_prevents_false_resolution() {
        let span = CanonicalSpan { start: 2, end: 18 };
        let mut observation = evidence(
            ClauseKind::WorkStepResult,
            span,
            Some(0.90),
            "provider-a",
        );
        observation.alternatives.push(EvidenceAlternative {
            value: ClauseKind::Condition,
            confidence: Some(0.82),
        });

        assert_eq!(
            resolve_clause_kind(
                &[observation],
                span,
                SemanticResolutionPolicy::default()
            ),
            Resolution::Ambiguous(vec![
                ClauseKind::Condition,
                ClauseKind::WorkStepResult,
            ])
        );
    }

    #[test]
    fn evidence_for_another_span_cannot_resolve_this_span() {
        let requested = CanonicalSpan { start: 0, end: 10 };
        let other = CanonicalSpan { start: 11, end: 20 };
        let evidence = vec![evidence(
            ClauseKind::Condition,
            other,
            Some(0.99),
            "provider-a",
        )];

        assert_eq!(
            resolve_clause_kind(
                &evidence,
                requested,
                SemanticResolutionPolicy::default()
            ),
            Resolution::Unknown
        );
    }
}
