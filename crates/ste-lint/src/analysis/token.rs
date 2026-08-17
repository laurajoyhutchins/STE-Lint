#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub sentence_id: Option<usize>,
}

/// Hyphen-aware source-token view retained for direct perfect-tense matching.
///
/// It uses the same source-coordinate scanner as `AnalysisToken`; the only
/// profile difference is that `-` remains inside a token so source-backed
/// multiword/compound participle matching preserves its historical behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HyphenAwareToken<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenProfile {
    Lexical,
    HyphenAware,
}

pub(crate) fn lexical_tokens(text: &str) -> Vec<AnalysisToken<'_>> {
    scan(text, TokenProfile::Lexical)
        .into_iter()
        .map(|token| AnalysisToken {
            text: token.text,
            start: token.start,
            end: token.end,
            sentence_id: None,
        })
        .collect()
}

pub(crate) fn hyphen_aware_tokens(text: &str) -> Vec<HyphenAwareToken<'_>> {
    scan(text, TokenProfile::HyphenAware)
}

fn scan(text: &str, profile: TokenProfile) -> Vec<HyphenAwareToken<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in text.char_indices() {
        let is_word =
            character.is_alphabetic() || (profile == TokenProfile::HyphenAware && character == '-');
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
