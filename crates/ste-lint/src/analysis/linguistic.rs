use harper_brill::{Chunker, Tagger, UPOS, brill_chunker, brill_tagger};
use harper_core::parsers::{Parser, PlainEnglish};
use harper_core::spell::{Dictionary, FstDictionary};
use harper_core::{DictWordMetadata, Token};

use super::source::SourceDocument;
use super::token::AnalysisToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenericPos {
    Adjective,
    Adposition,
    Adverb,
    Auxiliary,
    Conjunction,
    Determiner,
    Interjection,
    Noun,
    Numeral,
    Particle,
    Pronoun,
    ProperNoun,
    Symbol,
    Verb,
}

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
    pub occurrence_pos: Option<GenericPos>,
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
        let chars = text.chars().collect::<Vec<_>>();
        let raw_tokens = PlainEnglish.parse(&chars);
        let dictionary = FstDictionary::curated();
        let tagger = brill_tagger();
        let chunker = brill_chunker();
        let source = SourceDocument::new(text);
        let char_to_byte = char_to_byte_index(text);
        let mut tokens = Vec::new();
        let mut sentence_start = 0usize;

        for boundary in 0..=raw_tokens.len() {
            let at_end = boundary == raw_tokens.len();
            let at_sentence_end = !at_end && raw_tokens[boundary].kind.is_sentence_terminator();
            if !at_end && !at_sentence_end {
                continue;
            }
            let sentence_end = if at_sentence_end {
                boundary + 1
            } else {
                boundary
            };
            append_sentence_evidence(
                &raw_tokens[sentence_start..sentence_end],
                &chars,
                &char_to_byte,
                &dictionary,
                tagger.as_ref(),
                chunker.as_ref(),
                &source,
                &mut tokens,
            );
            sentence_start = sentence_end;
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

#[allow(clippy::too_many_arguments)]
fn append_sentence_evidence(
    sentence: &[Token],
    chars: &[char],
    char_to_byte: &[usize],
    dictionary: &FstDictionary,
    tagger: &impl Tagger,
    chunker: &impl Chunker,
    source: &SourceDocument,
    output: &mut Vec<LinguisticTokenEvidence>,
) {
    let visible = sentence
        .iter()
        .filter(|token| !token.kind.is_whitespace())
        .collect::<Vec<_>>();
    if visible.is_empty() {
        return;
    }
    let strings = visible
        .iter()
        .map(|token| {
            chars[token.span.start..token.span.end]
                .iter()
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    let tags = tagger.tag_sentence(&strings);
    let noun_phrase_flags = chunker.chunk_sentence(&strings, &tags);

    for ((token, tag), noun_phrase_member) in visible
        .into_iter()
        .zip(tags.into_iter())
        .zip(noun_phrase_flags.into_iter())
    {
        if !token.kind.is_word() {
            continue;
        }
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

        let metadata = dictionary.get_word_metadata(&chars[token.span.start..token.span.end]);
        let metadata = metadata.as_deref();
        output.push(token_evidence(
            start,
            end,
            &text_slice(chars, token),
            metadata,
            tag.and_then(generic_pos_from_upos),
            noun_phrase_member,
            dictionary,
        ));
    }
}

fn token_evidence(
    start: usize,
    end: usize,
    token_text: &str,
    metadata: Option<&DictWordMetadata>,
    occurrence_pos: Option<GenericPos>,
    np_member: bool,
    dictionary: &FstDictionary,
) -> LinguisticTokenEvidence {
    let lemma = metadata.and_then(|metadata| {
        if let Some(derived_from) = metadata.derived_from.as_ref() {
            dictionary
                .get_word_from_id(derived_from)
                .map(|word| word.iter().collect::<String>())
        } else if metadata.is_verb_lemma() {
            Some(token_text.to_ascii_lowercase())
        } else {
            None
        }
    });
    let mut verb_forms = Vec::new();
    if metadata.is_some_and(DictWordMetadata::is_verb_lemma) {
        verb_forms.push(GenericVerbForm::Lemma);
    }
    if metadata.is_some_and(DictWordMetadata::is_verb_past_form) {
        verb_forms.push(GenericVerbForm::Past);
    }
    if metadata.is_some_and(DictWordMetadata::is_verb_simple_past_form) {
        verb_forms.push(GenericVerbForm::SimplePast);
    }
    if metadata.is_some_and(DictWordMetadata::is_verb_past_participle_form) {
        verb_forms.push(GenericVerbForm::PastParticiple);
    }
    if metadata.is_some_and(DictWordMetadata::is_verb_progressive_form) {
        verb_forms.push(GenericVerbForm::Progressive);
    }
    if metadata.is_some_and(DictWordMetadata::is_verb_third_person_singular_present_form) {
        verb_forms.push(GenericVerbForm::ThirdPersonSingularPresent);
    }

    LinguisticTokenEvidence {
        start,
        end,
        lemma,
        occurrence_pos,
        determiner: metadata.is_some_and(DictWordMetadata::is_determiner),
        conjunction: metadata.is_some_and(DictWordMetadata::is_conjunction),
        noun: metadata.is_some_and(DictWordMetadata::is_noun),
        nominal: metadata.is_some_and(DictWordMetadata::is_nominal),
        adjective: metadata.is_some_and(DictWordMetadata::is_adjective),
        verb: metadata.is_some_and(DictWordMetadata::is_verb),
        auxiliary_verb: metadata.is_some_and(DictWordMetadata::is_auxiliary_verb),
        linking_verb: metadata.is_some_and(DictWordMetadata::is_linking_verb),
        np_member,
        comparative_adjective: metadata.is_some_and(DictWordMetadata::is_comparative_adjective),
        superlative_adjective: metadata.is_some_and(DictWordMetadata::is_superlative_adjective),
        verb_forms,
    }
}

fn text_slice(chars: &[char], token: &Token) -> String {
    chars[token.span.start..token.span.end].iter().collect()
}

fn generic_pos_from_upos(pos: UPOS) -> Option<GenericPos> {
    match pos {
        UPOS::ADJ => Some(GenericPos::Adjective),
        UPOS::ADP => Some(GenericPos::Adposition),
        UPOS::ADV => Some(GenericPos::Adverb),
        UPOS::AUX => Some(GenericPos::Auxiliary),
        UPOS::CCONJ | UPOS::SCONJ => Some(GenericPos::Conjunction),
        UPOS::DET => Some(GenericPos::Determiner),
        UPOS::INTJ => Some(GenericPos::Interjection),
        UPOS::NOUN => Some(GenericPos::Noun),
        UPOS::NUM => Some(GenericPos::Numeral),
        UPOS::PART => Some(GenericPos::Particle),
        UPOS::PRON => Some(GenericPos::Pronoun),
        UPOS::PROPN => Some(GenericPos::ProperNoun),
        UPOS::SYM => Some(GenericPos::Symbol),
        UPOS::VERB => Some(GenericPos::Verb),
        UPOS::PUNCT => None,
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
    fn harper_pos_tag_disambiguates_homograph_occurrences() {
        let document = LinguisticDocument::new("Check the valve. The check is complete.");
        let checks = document
            .tokens
            .iter()
            .filter(|token| document.text[token.start..token.end].eq_ignore_ascii_case("check"))
            .collect::<Vec<_>>();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].occurrence_pos, Some(GenericPos::Verb));
        assert_eq!(checks[1].occurrence_pos, Some(GenericPos::Noun));
    }

    #[test]
    fn deterministic_brill_chunker_marks_bare_nominal_phrase_members() {
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
