use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_glossary::{Glossary, TechnicalTerm, TechnicalTermKind, TermStatus};

use crate::LintMode;

pub(crate) fn check(
    text: &str,
    glossary: Option<&Glossary>,
    mode: LintMode,
) -> Vec<Diagnostic> {
    let Some(glossary) = glossary else {
        return Vec::new();
    };

    let tokens = tokens(text);
    if tokens.is_empty() {
        return Vec::new();
    }

    let max_term_words = glossary
        .terms
        .iter()
        .flat_map(|term| std::iter::once(&term.term).chain(term.aliases.iter()))
        .map(|value| value.split_whitespace().count())
        .max()
        .unwrap_or(1);

    let mut diagnostics = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let max_window = max_term_words.min(tokens.len() - index);
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
            if let Some(term) = glossary.lookup_term(&phrase) {
                matched = Some((width, term, phrase));
                break;
            }
        }

        let Some((width, term, matched_text)) = matched else {
            index += 1;
            continue;
        };

        if term.status == TermStatus::Approved
            && let Some((observed_role, role_basis)) =
                observed_role(text, &tokens, index, width, mode)
            && let Some(diagnostic) = role_diagnostic(
                term,
                &matched_text,
                tokens[index].start,
                tokens[index + width - 1].end,
                observed_role,
                role_basis,
            )
        {
            diagnostics.push(diagnostic);
        }

        index += width;
    }

    diagnostics
}

fn observed_role(
    text: &str,
    tokens: &[Token<'_>],
    index: usize,
    width: usize,
    mode: LintMode,
) -> Option<(&'static str, &'static str)> {
    if index > 0
        && separator_is_whitespace(text, &tokens[index - 1], &tokens[index])
        && is_determiner(tokens[index - 1].text)
    {
        return Some(("nominal", "preceded_by_determiner"));
    }

    let next_index = index + width;
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

fn role_diagnostic(
    term: &TechnicalTerm,
    matched_text: &str,
    start: usize,
    end: usize,
    observed_role: &str,
    role_basis: &str,
) -> Option<Diagnostic> {
    let (code, rule, message) = match (term.kind, observed_role) {
        (TechnicalTermKind::TechnicalNoun, "verbal") => (
            "STE-TERM-003",
            "1.7",
            format!(
                "Technical noun '{matched_text}' is used in a bounded imperative verb position."
            ),
        ),
        (TechnicalTermKind::TechnicalVerb, "nominal") => (
            "STE-TERM-004",
            "1.13",
            format!("Technical verb '{matched_text}' is used in a bounded noun position."),
        ),
        _ => return None,
    };

    Some(Diagnostic {
        code: code.into(),
        severity: Severity::Error,
        message,
        span: Span { start, end },
        rules: vec![rule.into()],
        evidence: Some(json!({
            "canonical_term": &term.term,
            "matched_text": matched_text,
            "governed_kind": term.kind,
            "observed_role": observed_role,
            "role_basis": role_basis,
            "domain": &term.domain,
            "provenance": &term.provenance,
        })),
        autofix: None,
    })
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
