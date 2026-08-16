use crate::structure::word_limit_units;

use super::AnalysisToken;

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
    let mut sentences = Vec::new();

    for (id, unit) in word_limit_units(text).into_iter().enumerate() {
        let first_token = tokens
            .iter()
            .position(|token| token.start >= unit.start && token.end <= unit.end);
        let last_token = tokens
            .iter()
            .rposition(|token| token.start >= unit.start && token.end <= unit.end)
            .map(|index| index + 1);

        if let (Some(first), Some(last)) = (first_token, last_token) {
            for token in &mut tokens[first..last] {
                if token.start >= unit.start && token.end <= unit.end {
                    token.sentence_id = Some(id);
                }
            }
        }

        sentences.push(AnalysisSentence {
            id,
            start: unit.start,
            end: unit.end,
            first_token,
            last_token,
        });
    }

    sentences
}
