use harper_core::Document;

use super::source::{SourceDocument, char_to_byte_offsets};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
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
            (start < end).then_some(AnalysisToken {
                text: &text[start..end],
                start,
                end,
                sentence_id: None,
            })
        })
        .collect()
}

// Hyphenated dictionary forms are reconstructed from canonical token source spans.
