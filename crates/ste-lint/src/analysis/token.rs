#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
}

pub(crate) fn lexical_tokens(text: &str) -> Vec<AnalysisToken<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in text.char_indices() {
        match (start, character.is_alphabetic()) {
            (None, true) => start = Some(index),
            (Some(word_start), false) => {
                tokens.push(AnalysisToken {
                    text: &text[word_start..index],
                    start: word_start,
                    end: index,
                    sentence_id: None,
                });
                start = None;
            }
            _ => {}
        }
    }

    if let Some(word_start) = start {
        tokens.push(AnalysisToken {
            text: &text[word_start..],
            start: word_start,
            end: text.len(),
            sentence_id: None,
        });
    }

    tokens
}
