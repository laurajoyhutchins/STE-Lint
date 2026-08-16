use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

use crate::{
    AnalysisDocument, Resolution, SafetyEvidenceSource, SafetyLevel, SafetyLevelEvidence,
    SafetyLevelFact,
};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let Some(context) = analysis.context() else {
        return Vec::new();
    };

    let mut diagnostics = Vec::new();
    for safety in analysis.safety_semantics() {
        let Some(observed_level) = structural_level(&safety.level) else {
            continue;
        };

        let mut supplied_levels = Vec::new();
        let mut supplied_sources = Vec::new();
        for fact in &context.safety_facts {
            if fact.start != safety.span.start || fact.end != safety.span.end {
                continue;
            }
            let Some(level) = fact.safety_level else {
                continue;
            };
            let level = match level {
                SafetyLevelFact::Warning => SafetyLevel::Warning,
                SafetyLevelFact::Caution => SafetyLevel::Caution,
            };
            if !supplied_levels.contains(&level) {
                supplied_levels.push(level);
            }
            if !supplied_sources.contains(&fact.source) {
                supplied_sources.push(fact.source.clone());
            }
        }

        if supplied_levels.len() != 1 || supplied_levels[0] == observed_level {
            continue;
        }

        diagnostics.push(Diagnostic {
            code: "STE-SAFE-003".into(),
            severity: Severity::Error,
            message: "Safety label conflicts with the supplied project risk-level evidence.".into(),
            span: Span {
                start: safety.span.start,
                end: safety.span.end,
            },
            rules: vec!["7.1".into()],
            evidence: Some(json!({
                "observed_safety_level": safety_level_name(observed_level),
                "supplied_safety_level": safety_level_name(supplied_levels[0]),
                "sources": supplied_sources,
            })),
            autofix: None,
        });
    }

    diagnostics
}

fn structural_level(level: &Resolution<SafetyLevelEvidence>) -> Option<SafetyLevel> {
    match level {
        Resolution::Resolved(evidence)
            if matches!(evidence.source, SafetyEvidenceSource::Structure) =>
        {
            Some(evidence.level)
        }
        Resolution::Ambiguous(candidates) => candidates
            .iter()
            .find(|evidence| matches!(evidence.source, SafetyEvidenceSource::Structure))
            .map(|evidence| evidence.level),
        Resolution::Resolved(_) | Resolution::Unknown => None,
    }
}

fn safety_level_name(level: SafetyLevel) -> &'static str {
    match level {
        SafetyLevel::Warning => "warning",
        SafetyLevel::Caution => "caution",
    }
}
