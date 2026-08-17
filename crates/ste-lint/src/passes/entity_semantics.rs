use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_glossary::{TermRole, TermStatus};

use crate::{AnalysisDocument, EntityIdentity, EntityMention};

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let Some(glossary) = analysis.glossary() else {
        return Vec::new();
    };
    let mentions = analysis.entity_mentions();
    let mut diagnostics = Vec::new();

    for mention in &mentions {
        let EntityIdentity::GovernedTerm { term, domain } = &mention.identity else {
            continue;
        };
        let Some(governed) = glossary.lookup_term(term) else {
            continue;
        };
        if !governed.has_role(TermRole::Noun) || governed.status != TermStatus::Approved {
            continue;
        }

        let canonical_word_count = word_count(&governed.canonical);
        let alias_word_count = word_count(&mention.surface);
        if canonical_word_count <= 3
            || surfaces_match(&mention.surface, &governed.canonical)
            || alias_word_count > 3
            || alias_word_count >= canonical_word_count
        {
            continue;
        }

        let full_form_seen_before = mentions.iter().any(|candidate| {
            candidate.span.start < mention.span.start
                && candidate.identity == mention.identity
                && surfaces_match(&candidate.surface, &governed.canonical)
        });
        if full_form_seen_before {
            continue;
        }

        diagnostics.push(alias_before_full_form_diagnostic(
            mention,
            &governed.canonical,
            domain,
            canonical_word_count,
            alias_word_count,
        ));
    }

    diagnostics
}

fn alias_before_full_form_diagnostic(
    mention: &EntityMention,
    canonical_term: &str,
    domain: &str,
    canonical_word_count: usize,
    alias_word_count: usize,
) -> Diagnostic {
    Diagnostic {
        code: "STE-NOUN-002".into(),
        severity: Severity::Error,
        message: format!(
            "Write the governed long technical noun '{canonical_term}' in full before using its shorter form '{}'.",
            mention.surface
        ),
        span: Span {
            start: mention.span.start,
            end: mention.span.end,
        },
        rules: vec!["2.2".into()],
        evidence: Some(json!({
            "coverage": "governed_long_technical_noun_first_use_v1",
            "canonical_term": canonical_term,
            "alias_surface": mention.surface,
            "canonical_word_count": canonical_word_count,
            "alias_word_count": alias_word_count,
            "domain": domain,
            "provenance": mention.provenance,
            "full_form_seen_before": false,
            "limitations": [
                "only explicit governed glossary aliases are evaluated",
                "canonical technical noun must contain more than three words",
                "shorter alias must contain no more than three words"
            ]
        })),
        autofix: None,
    }
}

fn word_count(value: &str) -> usize {
    value.split_whitespace().count()
}

fn surfaces_match(left: &str, right: &str) -> bool {
    normalize_surface(left).eq_ignore_ascii_case(&normalize_surface(right))
}

fn normalize_surface(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
