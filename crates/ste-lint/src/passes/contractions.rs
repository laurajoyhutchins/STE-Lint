use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};

pub(crate) fn check(text: &str) -> Vec<Diagnostic> {
    word_spans(text)
        .into_iter()
        .filter_map(|(start, end)| {
            let raw = &text[start..end];
            is_contraction(raw).then(|| Diagnostic {
                code: "STE-SYN-001".into(),
                severity: Severity::Error,
                message: format!("Do not use the contraction '{raw}'; write the words in full."),
                span: Span { start, end },
                rules: vec!["4.2".into()],
                evidence: Some(json!({
                    "form": raw,
                    "autofix": "none because contraction expansions can be grammatically or semantically ambiguous"
                })),
                autofix: None,
            })
        })
        .collect()
}

fn is_contraction(raw: &str) -> bool {
    let normalized = raw.replace('’', "'").to_ascii_lowercase();
    if normalized.ends_with("n't")
        || normalized.ends_with("'re")
        || normalized.ends_with("'ve")
        || normalized.ends_with("'ll")
        || normalized.ends_with("'d")
        || normalized.ends_with("'m")
    {
        return true;
    }

    let Some(base) = normalized.strip_suffix("'s") else {
        return false;
    };
    matches!(
        base,
        "it" | "he"
            | "she"
            | "that"
            | "there"
            | "here"
            | "what"
            | "who"
            | "where"
            | "when"
            | "why"
            | "how"
            | "let"
    )
}

fn word_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;

    for (index, character) in text.char_indices() {
        let word_character = character.is_alphabetic() || matches!(character, '\'' | '’' | '-');
        match (start, word_character) {
            (None, true) => start = Some(index),
            (Some(word_start), false) => {
                spans.push((word_start, index));
                start = None;
            }
            _ => {}
        }
    }

    if let Some(word_start) = start {
        spans.push((word_start, text.len()));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_straight_and_curly_contractions() {
        assert!(is_contraction("DON'T"));
        assert!(is_contraction("we’re"));
        assert!(is_contraction("IT'S"));
    }

    #[test]
    fn does_not_treat_generic_possessive_as_contraction() {
        assert!(!is_contraction("engine's"));
        assert!(!is_contraction("operator’s"));
    }
}
