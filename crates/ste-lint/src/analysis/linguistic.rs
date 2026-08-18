use harper_core::Document;
use harper_core::spell::{Dictionary, FstDictionary};

use super::source::SourceDocument;
use super::token::AnalysisToken;

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
pub(crate) struct LinguisticTokenEvidence {
    pub start: usize,
    pub end: usize,
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
    pub verb_forms: Vec<GenericVerbForm>,
}

#[derive(Debug)]
pub(crate) struct LinguisticDocument<'a> {
    text: &'a str,
    tokens: Vec<LinguisticTokenEvidence>,
}

impl<'a> LinguisticDocument<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        let document = Document::new_plain_english_curated(text);
        let dictionary = FstDictionary::curated();
        let source = SourceDocument::new(text);
        let char_to_byte = char_to_byte_index(text);
        let mut tokens = Vec::new();

        for token in document.tokens().filter(|token| token.kind.is_word()) {
            let Some(&start) = char_to_byte.get(token.span.start) else {
                continue;
            };
            let Some(&end) = char_to_byte.get(token.span.end) else {
                continue;
            };
            if start >= end
                || source
                    .protected_ranges()
                    .iter()
                    .any(|span| span.intersects(start, end))
            {
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

            tokens.push(LinguisticTokenEvidence {
                start,
                end,
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
            });
        }

        Self { text, tokens }
    }

    #[cfg(test)]
    fn analysis_tokens(&self) -> Vec<AnalysisToken<'a>> {
        self.tokens
            .iter()
            .map(|token| AnalysisToken {
                text: &self.text[token.start..token.end],
                start: token.start,
                end: token.end,
                sentence_id: None,
            })
            .collect()
    }

    pub(crate) fn into_parts(self) -> (Vec<AnalysisToken<'a>>, Vec<LinguisticTokenEvidence>) {
        let analysis_tokens = self
            .tokens
            .iter()
            .map(|token| AnalysisToken {
                text: &self.text[token.start..token.end],
                start: token.start,
                end: token.end,
                sentence_id: None,
            })
            .collect();
        (analysis_tokens, self.tokens)
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

    #[test]
    fn converts_harper_character_spans_to_utf8_byte_spans() {
        let text = "CAFÉ valve";
        let document = LinguisticDocument::new(text);
        let tokens = document.analysis_tokens();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            (tokens[0].text, tokens[0].start, tokens[0].end),
            ("CAFÉ", 0, 5)
        );
        assert_eq!(
            (tokens[1].text, tokens[1].start, tokens[1].end),
            ("valve", 6, 11)
        );
    }

    #[test]
    fn protected_markdown_code_is_not_linguistic_prose() {
        let document = LinguisticDocument::new("USE `fluxcapacitor` here.");
        assert!(
            document
                .analysis_tokens()
                .iter()
                .all(|token| token.text != "fluxcapacitor")
        );
    }

    #[test]
    fn harper_lexical_pos_predicates_preserve_homograph_ambiguity() {
        let document = LinguisticDocument::new("Check the valve. The check is complete.");
        let checks = document
            .tokens
            .iter()
            .filter(|token| document.text[token.start..token.end].eq_ignore_ascii_case("check"))
            .collect::<Vec<_>>();
        assert_eq!(checks.len(), 2);
        assert!(checks[0].verb);
        assert!(checks[0].noun || checks[0].nominal);
        assert!(checks[1].noun || checks[1].nominal);
    }

    #[test]
    fn harper_curated_analysis_marks_bare_nominal_phrase_members() {
        let document = LinguisticDocument::new("Fuel pump pressure is stable.");
        let phrase = document
            .tokens
            .iter()
            .take(3)
            .map(|token| token.np_member)
            .collect::<Vec<_>>();
        assert_eq!(phrase, vec![true, true, true]);
    }
}
