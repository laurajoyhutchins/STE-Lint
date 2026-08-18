use harper_core::Document;
use harper_core::spell::{Dictionary, FstDictionary};

use super::evidence::{AnalysisEvidence, EvidenceProvenance, EvidenceTarget, ProviderIdentity};
use super::source::CanonicalSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenericVerbForm {
    Lemma,
    Past,
    SimplePast,
    PastParticiple,
    Progressive,
    ThirdPersonSingularPresent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalObservation {
    pub lemma: Option<String>,
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
    pub(crate) fn analyze(source: &CanonicalSource<'_>) -> Vec<AnalysisEvidence<LexicalObservation>> {
        let text = source.text();
        let document = Document::new_plain_english_curated(text);
        let dictionary = FstDictionary::curated();
        let char_to_byte = char_to_byte_index(text);
        let provenance = EvidenceProvenance {
            provider: ProviderIdentity {
                name: "harper-core".into(),
                version: Some("2.7.0".into()),
            },
            model: None,
        };
        let mut evidence = Vec::new();

        for token in document.tokens().filter(|token| token.kind.is_word()) {
            let Some(&start) = char_to_byte.get(token.span.start) else {
                continue;
            };
            let Some(&end) = char_to_byte.get(token.span.end) else {
                continue;
            };
            let Some(span) = source.span(start, end) else {
                continue;
            };
            if source.is_protected(span) {
                continue;
            }

            let metadata = token.kind.as_word().and_then(|word| word.as_ref());
            let lemma = metadata.and_then(|metadata| {
                if let Some(derived_from) = metadata.derived_from.as_ref() {
                    dictionary
                        .get_word_from_id(derived_from)
                        .map(|word| word.iter().collect::<String>())
                } else if metadata.is_verb_lemma() {
                    Some(text[start..end].to_ascii_lowercase())
                } else {
                    None
                }
            });
            let mut verb_forms = Vec::new();
            if token.kind.is_verb_lemma() {
                verb_forms.push(GenericVerbForm::Lemma);
            }
            if token.kind.is_verb_past_form() {
                verb_forms.push(GenericVerbForm::Past);
            }
            if token.kind.is_verb_simple_past_form() {
                verb_forms.push(GenericVerbForm::SimplePast);
            }
            if token.kind.is_verb_past_participle_form() {
                verb_forms.push(GenericVerbForm::PastParticiple);
            }
            if token.kind.is_verb_progressive_form() {
                verb_forms.push(GenericVerbForm::Progressive);
            }
            if token.kind.is_verb_third_person_singular_present_form() {
                verb_forms.push(GenericVerbForm::ThirdPersonSingularPresent);
            }

            evidence.push(AnalysisEvidence::new(
                LexicalObservation {
                    lemma,
                    determiner: token.kind.is_determiner(),
                    conjunction: token.kind.is_conjunction(),
                    noun: token.kind.is_noun(),
                    nominal: token.kind.is_nominal(),
                    adjective: token.kind.is_adjective(),
                    verb: token.kind.is_verb(),
                    auxiliary_verb: token.kind.is_auxiliary_verb(),
                    linking_verb: token.kind.is_linking_verb(),
                    np_member: token.kind.is_np_member(),
                    comparative_adjective: token.kind.is_comparative_adjective(),
                    superlative_adjective: token.kind.is_superlative_adjective(),
                    verb_forms,
                },
                EvidenceTarget::Token(span),
                provenance.clone(),
            ));
        }

        evidence
    }
}

fn char_to_byte_index(text: &str) -> Vec<usize> {
    let mut table = text
        .char_indices()
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    table.push(text.len());
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_spans(text: &str) -> Vec<(String, usize, usize)> {
        let source = CanonicalSource::new(text);
        HarperProvider::analyze(&source)
            .into_iter()
            .map(|evidence| {
                let EvidenceTarget::Token(span) = evidence.target else {
                    unreachable!("Harper lexical evidence must target canonical tokens");
                };
                (text[span.start..span.end].to_string(), span.start, span.end)
            })
            .collect()
    }

    #[test]
    fn converts_harper_character_spans_to_utf8_byte_spans() {
        let tokens = token_spans("CAFÉ valve");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], ("CAFÉ".into(), 0, 5));
        assert_eq!(tokens[1], ("valve".into(), 6, 11));
    }

    #[test]
    fn protected_markdown_code_is_not_linguistic_prose() {
        let tokens = token_spans("USE `fluxcapacitor` here.");
        assert!(tokens.iter().all(|(text, _, _)| text != "fluxcapacitor"));
    }
}
