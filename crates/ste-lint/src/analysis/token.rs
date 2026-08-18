use harper_core::Document;

use super::source::{SourceDocument, char_to_byte_offsets};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
    pub(crate) generic_is_determiner: bool,
    pub(crate) generic_is_linking_verb: bool,
    pub(crate) generic_is_conjunction: bool,
    pub(crate) generic_is_preposition: bool,
    pub(crate) generic_is_verb: bool,
    pub(crate) generic_is_noun: bool,
    pub(crate) generic_is_adjective: bool,
    pub(crate) generic_is_progressive_form: bool,
}

// Dictionary matching uses the canonical Harper token stream.

pub(crate) fn lexical_tokens(text: &str) -> Vec<AnalysisToken<'_>> {
    let source = SourceDocument::new(text);
    let offsets = char_to_byte_offsets(text);
    let document = Document::new_plain_english_curated(source.linguistic_projection());

    document
        .tokens()
        .filter(|token| token.kind.is_word())
        .filter_map(|token| {
            let start = *offsets.get(token.span.start)?;
            let end = *offsets.get(token.span.end)?;
            let kind = &token.kind;
            (start < end).then_some(AnalysisToken {
                text: &text[start..end],
                start,
                end,
                sentence_id: None,
                generic_is_determiner: kind.is_determiner(),
                generic_is_linking_verb: kind.is_linking_verb(),
                generic_is_conjunction: kind.is_conjunction(),
                generic_is_preposition: kind.is_preposition(),
                generic_is_verb: kind.is_verb(),
                generic_is_noun: kind.is_noun(),
                generic_is_adjective: kind.is_adjective(),
                generic_is_progressive_form: kind.is_verb_progressive_form(),
            })
        })
        .collect()
}

// Hyphenated dictionary forms are reconstructed from canonical token source spans.
