use crate::LintMode;

use super::AnalysisToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedRole {
    Nominal,
    Verbal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedRoleEvidence {
    pub role: ObservedRole,
    pub basis: &'static str,
}

pub(crate) fn dictionary_role(
    text: &str,
    tokens: &[AnalysisToken<'_>],
    index: usize,
    width: usize,
    mode: LintMode,
) -> Option<ObservedRoleEvidence> {
    let next_index = index + width;

    if index > 0
        && next_index < tokens.len()
        && separator_is_whitespace(text, &tokens[index - 1], &tokens[index])
        && separator_is_whitespace(text, &tokens[next_index - 1], &tokens[next_index])
        && is_determiner(tokens[index - 1].text)
        && is_copula(tokens[next_index].text)
    {
        return Some(ObservedRoleEvidence {
            role: ObservedRole::Nominal,
            basis: "determiner_term_copula",
        });
    }

    if mode == LintMode::Procedural
        && sentence_start(tokens, index)
        && next_index < tokens.len()
        && separator_is_whitespace(text, &tokens[next_index - 1], &tokens[next_index])
        && is_determiner(tokens[next_index].text)
    {
        return Some(ObservedRoleEvidence {
            role: ObservedRole::Verbal,
            basis: "procedural_sentence_initial_term_followed_by_determiner",
        });
    }

    None
}

pub(crate) fn technical_role(
    text: &str,
    tokens: &[AnalysisToken<'_>],
    index: usize,
    width: usize,
    mode: LintMode,
) -> Option<ObservedRoleEvidence> {
    if index > 0
        && separator_is_whitespace(text, &tokens[index - 1], &tokens[index])
        && is_determiner(tokens[index - 1].text)
    {
        return Some(ObservedRoleEvidence {
            role: ObservedRole::Nominal,
            basis: "preceded_by_determiner",
        });
    }

    let next_index = index + width;
    if mode == LintMode::Procedural
        && sentence_start(tokens, index)
        && next_index < tokens.len()
        && separator_is_whitespace(text, &tokens[next_index - 1], &tokens[next_index])
        && is_determiner(tokens[next_index].text)
    {
        return Some(ObservedRoleEvidence {
            role: ObservedRole::Verbal,
            basis: "procedural_sentence_initial_term_followed_by_determiner",
        });
    }

    None
}

fn sentence_start(tokens: &[AnalysisToken<'_>], index: usize) -> bool {
    let Some(sentence_id) = tokens.get(index).and_then(|token| token.sentence_id) else {
        return index == 0;
    };
    tokens[..index]
        .iter()
        .all(|token| token.sentence_id != Some(sentence_id))
}

fn separator_is_whitespace(
    text: &str,
    left: &AnalysisToken<'_>,
    right: &AnalysisToken<'_>,
) -> bool {
    text[left.end..right.start].chars().all(char::is_whitespace)
}

fn is_determiner(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "a" | "an" | "the" | "this" | "that" | "these" | "those"
    )
}

fn is_copula(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "am" | "are"
            | "be"
            | "became"
            | "become"
            | "becomes"
            | "is"
            | "stay"
            | "stays"
            | "was"
            | "were"
    )
}
