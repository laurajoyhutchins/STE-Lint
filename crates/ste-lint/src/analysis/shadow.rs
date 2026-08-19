use std::fmt;

use serde::Deserialize;

use super::document::AnalysisDocument;
use super::evidence::{
    AnalysisEvidence, EvidenceProvenance, EvidenceTarget, ModelIdentity, ProviderIdentity,
};
use super::source::CanonicalSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticObservation {
    Dependency { relation: String },
    Constituency { label: String },
    NamedEntity { class: String },
    Coreference { representative: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowEvidenceIdentity {
    pub source_sha256: String,
    pub source_bytes: usize,
    pub provider: ProviderIdentity,
    pub model: ModelIdentity,
    pub configuration: String,
    pub configuration_sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowEvidenceSet {
    pub identity: ShadowEvidenceIdentity,
    pub evidence: Vec<AnalysisEvidence<SemanticObservation>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowEvidenceError {
    InvalidJson(String),
    UnsupportedSchema(u32),
    InvalidIdentity(&'static str),
    SourceLengthMismatch {
        expected: usize,
        actual: usize,
    },
    SourceDigestMismatch,
    ConfigurationDigestMismatch,
    InvalidSpan {
        kind: String,
        start: usize,
        end: usize,
    },
    SurfaceMismatch {
        kind: String,
        start: usize,
        end: usize,
    },
    InvalidConfidence {
        kind: String,
    },
}

impl fmt::Display for ShadowEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => {
                write!(formatter, "invalid shadow evidence JSON: {message}")
            }
            Self::UnsupportedSchema(version) => {
                write!(
                    formatter,
                    "unsupported shadow evidence schema version {version}"
                )
            }
            Self::InvalidIdentity(field) => {
                write!(
                    formatter,
                    "invalid or missing shadow evidence identity field {field}"
                )
            }
            Self::SourceLengthMismatch { expected, actual } => write!(
                formatter,
                "shadow evidence source byte length mismatch: expected {expected}, got {actual}"
            ),
            Self::SourceDigestMismatch => {
                write!(
                    formatter,
                    "shadow evidence source SHA-256 does not match the analysis text"
                )
            }
            Self::ConfigurationDigestMismatch => write!(
                formatter,
                "shadow evidence configuration SHA-256 does not match the recorded configuration"
            ),
            Self::InvalidSpan { kind, start, end } => {
                write!(formatter, "invalid {kind} canonical span {start}:{end}")
            }
            Self::SurfaceMismatch { kind, start, end } => write!(
                formatter,
                "{kind} surface does not match canonical source bytes at {start}:{end}"
            ),
            Self::InvalidConfidence { kind } => {
                write!(formatter, "{kind} confidence must be between 0 and 1")
            }
        }
    }
}

impl std::error::Error for ShadowEvidenceError {}

impl<'a> AnalysisDocument<'a> {
    pub fn import_shadow_evidence_json(
        &self,
        json: &str,
    ) -> Result<ShadowEvidenceSet, ShadowEvidenceError> {
        import_shadow_evidence(self, json)
    }
}

#[derive(Debug, Deserialize)]
struct RawBundle {
    schema_version: u32,
    source: RawSourceIdentity,
    provider: RawProviderIdentity,
    model: RawModelIdentity,
    configuration: String,
    configuration_sha256: String,
    evidence: Vec<RawEvidence>,
}

#[derive(Debug, Deserialize)]
struct RawSourceIdentity {
    sha256: String,
    bytes: usize,
}

#[derive(Debug, Deserialize)]
struct RawProviderIdentity {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct RawModelIdentity {
    name: String,
    version: String,
    artifact_sha256: String,
}

#[derive(Debug, Deserialize)]
struct RawSpan {
    start: usize,
    end: usize,
    surface: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RawEvidence {
    Dependency {
        relation: String,
        source: RawSpan,
        target: RawSpan,
        #[serde(default)]
        confidence: Option<f32>,
    },
    Constituency {
        label: String,
        span: RawSpan,
        #[serde(default)]
        confidence: Option<f32>,
    },
    NamedEntity {
        class: String,
        span: RawSpan,
        #[serde(default)]
        confidence: Option<f32>,
    },
    Coreference {
        representative: String,
        source: RawSpan,
        target: RawSpan,
        #[serde(default)]
        confidence: Option<f32>,
    },
}

fn import_shadow_evidence(
    analysis: &AnalysisDocument<'_>,
    json: &str,
) -> Result<ShadowEvidenceSet, ShadowEvidenceError> {
    let raw: RawBundle = serde_json::from_str(json)
        .map_err(|error| ShadowEvidenceError::InvalidJson(error.to_string()))?;
    if raw.schema_version != 1 {
        return Err(ShadowEvidenceError::UnsupportedSchema(raw.schema_version));
    }

    require_identity("source.sha256", &raw.source.sha256)?;
    require_identity("provider.name", &raw.provider.name)?;
    require_identity("provider.version", &raw.provider.version)?;
    require_identity("model.name", &raw.model.name)?;
    require_identity("model.version", &raw.model.version)?;
    require_identity("model.artifact_sha256", &raw.model.artifact_sha256)?;
    require_identity("configuration", &raw.configuration)?;
    require_identity("configuration_sha256", &raw.configuration_sha256)?;
    if !is_sha256(&raw.source.sha256) || !is_sha256(&raw.model.artifact_sha256) {
        return Err(ShadowEvidenceError::InvalidIdentity("sha256"));
    }
    if !is_sha256(&raw.configuration_sha256) {
        return Err(ShadowEvidenceError::InvalidIdentity("configuration_sha256"));
    }

    let actual_bytes = analysis.text().len();
    if raw.source.bytes != actual_bytes {
        return Err(ShadowEvidenceError::SourceLengthMismatch {
            expected: raw.source.bytes,
            actual: actual_bytes,
        });
    }
    if !raw
        .source
        .sha256
        .eq_ignore_ascii_case(&sha256_hex(analysis.text().as_bytes()))
    {
        return Err(ShadowEvidenceError::SourceDigestMismatch);
    }
    if !raw
        .configuration_sha256
        .eq_ignore_ascii_case(&sha256_hex(raw.configuration.as_bytes()))
    {
        return Err(ShadowEvidenceError::ConfigurationDigestMismatch);
    }

    let provider = ProviderIdentity {
        name: raw.provider.name,
        version: Some(raw.provider.version),
    };
    let model = ModelIdentity {
        name: raw.model.name,
        version: raw.model.version,
        artifact_sha256: Some(raw.model.artifact_sha256),
    };
    let provenance = EvidenceProvenance {
        provider: provider.clone(),
        model: Some(model.clone()),
    };

    let mut evidence = Vec::with_capacity(raw.evidence.len());
    for item in raw.evidence {
        let (observation, target, confidence, kind) = match item {
            RawEvidence::Dependency {
                relation,
                source,
                target,
                confidence,
            } => {
                require_identity("evidence.dependency.relation", &relation)?;
                let source = checked_span(analysis, "dependency source", source)?;
                let target = checked_span(analysis, "dependency target", target)?;
                (
                    SemanticObservation::Dependency { relation },
                    EvidenceTarget::Relation { source, target },
                    confidence,
                    "dependency",
                )
            }
            RawEvidence::Constituency {
                label,
                span,
                confidence,
            } => {
                require_identity("evidence.constituency.label", &label)?;
                let span = checked_span(analysis, "constituency", span)?;
                (
                    SemanticObservation::Constituency { label },
                    EvidenceTarget::Span(span),
                    confidence,
                    "constituency",
                )
            }
            RawEvidence::NamedEntity {
                class,
                span,
                confidence,
            } => {
                require_identity("evidence.named_entity.class", &class)?;
                let span = checked_span(analysis, "named entity", span)?;
                (
                    SemanticObservation::NamedEntity { class },
                    EvidenceTarget::Span(span),
                    confidence,
                    "named_entity",
                )
            }
            RawEvidence::Coreference {
                representative,
                source,
                target,
                confidence,
            } => {
                require_identity("evidence.coreference.representative", &representative)?;
                let source = checked_span(analysis, "coreference mention", source)?;
                let target_surface = target.surface.clone();
                let target = checked_span(analysis, "coreference antecedent", target)?;
                if representative != target_surface {
                    return Err(ShadowEvidenceError::SurfaceMismatch {
                        kind: "coreference representative".into(),
                        start: target.start,
                        end: target.end,
                    });
                }
                (
                    SemanticObservation::Coreference { representative },
                    EvidenceTarget::Relation { source, target },
                    confidence,
                    "coreference",
                )
            }
        };
        if confidence.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
            return Err(ShadowEvidenceError::InvalidConfidence { kind: kind.into() });
        }
        let mut item = AnalysisEvidence::new(observation, target, provenance.clone());
        item.confidence = confidence;
        evidence.push(item);
    }

    Ok(ShadowEvidenceSet {
        identity: ShadowEvidenceIdentity {
            source_sha256: raw.source.sha256,
            source_bytes: raw.source.bytes,
            provider,
            model,
            configuration: raw.configuration,
            configuration_sha256: raw.configuration_sha256,
        },
        evidence,
    })
}

fn checked_span(
    analysis: &AnalysisDocument<'_>,
    kind: &str,
    raw: RawSpan,
) -> Result<CanonicalSpan, ShadowEvidenceError> {
    let span = analysis.canonical_span(raw.start, raw.end).ok_or_else(|| {
        ShadowEvidenceError::InvalidSpan {
            kind: kind.into(),
            start: raw.start,
            end: raw.end,
        }
    })?;
    if analysis.text()[span.start..span.end] != raw.surface {
        return Err(ShadowEvidenceError::SurfaceMismatch {
            kind: kind.into(),
            start: span.start,
            end: span.end,
        });
    }
    Ok(span)
}

fn require_identity(field: &'static str, value: &str) -> Result<(), ShadowEvidenceError> {
    if value.trim().is_empty() {
        Err(ShadowEvidenceError::InvalidIdentity(field))
    } else {
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sha256_hex(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut message = input.to_vec();
    message.push(0x80);
    while !(message.len() + 8).is_multiple_of(64) {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, word) in words[..16].iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sigma1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sigma0.wrapping_add(majority);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    state
        .iter()
        .map(|word| format!("{word:08x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
