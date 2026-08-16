#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WordToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

pub(crate) fn lexical_tokens(text: &str) -> Vec<AnalysisToken<'_>> {
    scan(text, false)
        .into_iter()
        .map(|token| AnalysisToken {
            text: token.text,
            start: token.start,
            end: token.end,
            sentence_id: None,
        })
        .collect()
}

pub(crate) fn word_tokens(text: &str) -> Vec<WordToken<'_>> {
    scan(text, true)
}

fn scan(text: &str, include_hyphen: bool) -> Vec<WordToken<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in text.char_indices() {
        let is_word = character.is_alphabetic() || (include_hyphen && character == '-');
        match (start, is_word) {
            (None, true) => start = Some(index),
            (Some(word_start), false) => {
                tokens.push(WordToken {
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
        tokens.push(WordToken {
            text: &text[word_start..],
            start: word_start,
            end: text.len(),
        });
    }

    tokens
}
