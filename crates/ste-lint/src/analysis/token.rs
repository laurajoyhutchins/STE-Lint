#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
}

/// Exact source-token view retained only for source-backed compound participle matching.
/// Generic linguistic token identity is owned by `LinguisticDocument`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HyphenAwareToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

pub(crate) fn hyphen_aware_tokens(text: &str) -> Vec<HyphenAwareToken<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in text.char_indices() {
        let source_form_character = character.is_alphabetic() || character == '-';
        match (start, source_form_character) {
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
