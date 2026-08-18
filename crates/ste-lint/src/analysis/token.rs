use harper_core::Document;

use super::source::{SourceDocument, char_to_byte_offsets};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
}

/// Hyphen-aware source-token view retained temporarily for direct perfect-tense matching.
///
/// Canonical generic token identity now comes from Harper. This compatibility view remains only
/// until source-backed compound matching is moved onto the canonical token stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HyphenAwareToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

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
            (start < end).then_some(AnalysisToken {
                text: &text[start..end],
                start,
                end,
                sentence_id: None,
            })
        })
        .collect()
}

pub(crate) fn hyphen_aware_tokens(text: &str) -> Vec<HyphenAwareToken<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in text.char_indices() {
        let is_word = character.is_alphabetic() || character == '-';
        match (start, is_word) {
            (None, true) => start = Some(index),
            (Some(word_start), false) => {
                tokens.push(HyphenAwareToken {
                    text: &text[word_start..index],
                    start: word_start,
                    end: index,
                });
                start = None;
            }
            _ => {}
        }
    }

    if let Some(word_start) = start {
        tokens.push(HyphenAwareToken {
            text: &text[word_start..],
            start: word_start,
            end: text.len(),
        });
    }

    tokens
}