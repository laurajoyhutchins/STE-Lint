use super::source::CanonicalSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderIdentity {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelIdentity {
    pub name: String,
    pub version: String,
    pub artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceProvenance {
    pub provider: ProviderIdentity,
    pub model: Option<ModelIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceTarget {
    Token(CanonicalSpan),
    Span(CanonicalSpan),
    Relation {
        source: CanonicalSpan,
        target: CanonicalSpan,
    },
    Sentence {
        id: usize,
        span: CanonicalSpan,
    },
    Paragraph {
        id: usize,
        span: CanonicalSpan,
    },
    Document,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAlternative<T> {
    pub value: T,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisEvidence<T> {
    pub value: T,
    pub target: EvidenceTarget,
    pub provenance: EvidenceProvenance,
    pub confidence: Option<f32>,
    pub alternatives: Vec<EvidenceAlternative<T>>,
}

impl<T> AnalysisEvidence<T> {
    pub fn new(value: T, target: EvidenceTarget, provenance: EvidenceProvenance) -> Self {
        Self {
            value,
            target,
            provenance,
            confidence: None,
            alternatives: Vec::new(),
        }
    }
}
