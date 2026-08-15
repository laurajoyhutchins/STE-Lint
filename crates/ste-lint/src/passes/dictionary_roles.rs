use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, LexiconEntry, PartOfSpeech, RuntimeLexicon};
use ste_glossary::Glossary;

use super::semantic::dictionary_evidence;
use crate::LintMode;

pub(crate) fn check(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
    mode: LintMode,
) -> Vec<Diagnostic> {
    let tokens = tokens(text);
    let max_dictionary_words = lexicon
        .entries()
        .iter()
        .flat_map(|entry| &entry.forms)
        .map(|form| form.split_whitespace().count())
        .max()
        .unwrap_or(1);

    let mut diagnostics = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let max_window = max_dictionary_words.min(tokens.len() - index);
        let mut matched = None;

        for width in (1..=max_window).rev() {
            let window = &tokens[index..index + width];
            if !window.windows(2).all(|pair| {
                text[pair[0].end..pair[1].start]
                    .chars()
                    .all(char::is_whitespace)
            }) {
                continue;
            }
            let phrase = window
                .iter()
                .map(|token| token.text)
                .collect::<Vec<_>>()
                .join(" ");
            let candidates = lexicon.lookup_form_candidates(&phrase);
            if !candidates.is_empty() {
                matched = Some((width, phrase, candidates));
                break;
            }
        }

        let Some((width, matched_text, candidates)) = matched else {
            index += 1;
            continue;
        };

        if glossary
            .and_then(|governed| governed.lookup_term(&matched_text))
            .is_some()
        {
            index += width;
            continue;
        }

        if candidates
            .iter()
            .all(|entry| entry.status == ApprovalStatus::Approved)
            && let Some((observed_role, role_basis)) =
                observed_role(text, &tokens, index, width, mode)
            && !role_has_compatible_candidate(observed_role, &candidates)
        {
            diagnostics.push(role_diagnostic(
                &matched_text,
                tokens[index].start,
                tokens[index + width - 1].end,
                observed_role,
                role_basis,
                &candidates,
            ));
        }

        index += width;
    }

    diagnostics
}

fn role_diagnostic(
    matched_text: &str,
    start: usize,
    end: usize,
    observed_role: &str,
    role_basis: &str,
    candidates: &[&LexiconEntry],
) -> Diagnostic {
    let mut rules = vec!["1.2".into()];
    if observed_role == "verbal" {
        rules.push("3.7".into());
    }
    let mut evidence = dictionary_evidence(candidates, false);
    evidence["observed_role"] = json!(observed_role);
    evidence["role_basis"] = json!(role_basis);

    Diagnostic {
        code: "STE-GRAM-001".into(),
        severity: Severity::Error,
        message: format!(
            "Approved dictionary word '{matched_text}' is used in a bounded {observed_role} role that is incompatible with its approved part of speech."
        ),
        span: Span { start, end },
        rules,
        evidence: Some(evidence),
        autofix: None,
    }
}

fn role_has_compatible_candidate(role: &str, candidates: &[&LexiconEntry]) -> bool {
    candidates
        .iter()
        .any(|entry| match (role, entry.part_of_speech) {
            ("verbal", Some(PartOfSpeech::Verb)) => true,
            ("nominal", Some(PartOfSpeech::Noun | PartOfSpeech::Pronoun)) => true,
            _ => false,
        })
}

fn observed_role(
    text: &str,
    tokens: &[Token<'_>],
    index: usize,
    width: usize,
    mode: LintMode,
) -> Option<(&'static str, &'static str)> {
    let next_index = index + width;

    if index > 0
        && next_index < tokens.len()
        && separator_is_whitespace(text, &tokens[index - 1], &tokens[index])
        && separator_is_whitespace(text, &tokens[next_index - 1], &tokens[next_index])
        && is_determiner(tokens[index - 1].text)
        && is_copula(tokens[next_index].text)
    {
        return Some(("nominal", "determiner_term_copula"));
    }

    if mode == LintMode::Procedural
        && sentence_start(text, tokens[index].start)
        && next_index < tokens.len()
        && separator_is_whitespace(text, &tokens[next_index - 1], &tokens[next_index])
        && is_determiner(tokens[next_index].text)
    {
        return Some((
            "verbal",
            "procedural_sentence_initial_term_followed_by_determiner",
        ));
    }

    None
}

fn sentence_start(text: &str, start: usize) -> bool {
    let prefix = &text[..start];
    if prefix.trim().is_empty() {
        return true;
    }
    prefix
        .chars()
        .rev()
        .find(|character| !character.is_whitespace())
        .is_some_and(|character| matches!(character, '.' | '?' | '!'))
}

fn separator_is_whitespace(text: &str, left: &Token<'_>, right: &Token<'_>) -> bool {
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

struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

fn tokens(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in text.char_indices() {
        if character.is_alphabetic() {
            start.get_or_insert(index);
        } else if let Some(word_start) = start.take() {
            tokens.push(Token {
                text: &text[word_start..index],
                start: word_start,
                end: index,
            });
        }
    }

    if let Some(word_start) = start {
        tokens.push(Token {
            text: &text[word_start..],
            start: word_start,
            end: text.len(),
        });
    }

    tokens
}
