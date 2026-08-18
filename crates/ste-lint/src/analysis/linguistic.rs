pub(crate) use super::source;
pub(crate) use super::token;

use super::evidence::{AnalysisEvidence, EvidenceProvenance, EvidenceTarget, ProviderIdentity};
use super::source::CanonicalSource;

// Keep provider-specific Harper/Brill extraction behind the repository-owned evidence IR.
#[path = "linguistic_impl.rs"]
mod implementation;

pub(crate) use implementation::{GenericPos, GenericVerbForm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalObservation {
    pub lemma: Option<String>,
    pub(crate) occurrence_pos: Option<GenericPos>,
    pub determiner: bool,
    pub conjunction: bool,
    pub noun: bool,
    pub nominal: bool,
    pub adjective: bool,
    pub verb: bool,
    pub auxiliary_verb: bool,
    pub linking_verb: bool,
    pub np_member: bool,
    pub comparative_adjective: bool,
    pub superlative_adjective: bool,
    pub(crate) verb_forms: Vec<GenericVerbForm>,
}

pub(crate) type LinguisticTokenEvidence = LexicalObservation;

pub(crate) struct HarperProvider;

impl HarperProvider {
    pub(crate) fn analyze(
        source: &CanonicalSource<'_>,
    ) -> Vec<AnalysisEvidence<LexicalObservation>> {
        let provenance = EvidenceProvenance {
            provider: ProviderIdentity {
                name: "harper-core".into(),
                version: Some("2.7.0".into()),
            },
            model: None,
        };
        let (_, observations) = implementation::LinguisticDocument::new(source.text()).into_parts();

        observations
            .into_iter()
            .filter_map(|observation| {
                let span = source.span(observation.start, observation.end)?;
                if source.is_protected(span) {
                    return None;
                }
                Some(AnalysisEvidence::new(
                    LexicalObservation {
                        lemma: observation.lemma,
                        occurrence_pos: observation.occurrence_pos,
                        determiner: observation.determiner,
                        conjunction: observation.conjunction,
                        noun: observation.noun,
                        nominal: observation.nominal,
                        adjective: observation.adjective,
                        verb: observation.verb,
                        auxiliary_verb: observation.auxiliary_verb,
                        linking_verb: observation.linking_verb,
                        np_member: observation.np_member,
                        comparative_adjective: observation.comparative_adjective,
                        superlative_adjective: observation.superlative_adjective,
                        verb_forms: observation.verb_forms,
                    },
                    EvidenceTarget::Token(span),
                    provenance.clone(),
                ))
            })
            .collect()
    }
}
