use harper_core::{Document, TokenStringExt};

use super::AnalysisToken;
use super::source::{SourceDocument, char_to_byte_offsets};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisSentence {
    pub id: usize,
    pub start: usize,
    pub end: usize,
    pub first_token: Option<usize>,
    pub last_token: Option<usize>,
}

pub(crate) fn build_sentences(
    text: &str,
    tokens: &mut [AnalysisToken<'_>],
) -> Vec<AnalysisSentence> {
    let source = SourceDocument::new(text);
    let offsets = char_to_byte_offsets(text);
    let document = Document::new_plain_english_curated(source.linguistic_projection());
    let mut sentences = Vec::new();

    for sentence in document.get_tokens().iter_sentences() {
        let Some(span) = sentence.span() else {
            continue;
        };
        let Some(end) = offsets.get(span.end).copied() else {
            continue;
        };
        let first_token = tokens
            .iter()
            .position(|token| token.start >= offsets[span.start] && token.end <= end);
        let last_token = tokens
            .iter()
            .rposition(|token| token.start >= offsets[span.start] && token.end <= end)
            .map(|index| index + 1);
        let (Some(first), Some(last)) = (first_token, last_token) else {
            continue;
        };
        let id = sentences.len();
        for token in &mut tokens[first..last] {
            token.sentence_id = Some(id);
        }
        sentences.push(AnalysisSentence {
            id,
            start: tokens[first].start,
            end,
            first_token: Some(first),
            last_token: Some(last),
        });
    }

    sentences
}