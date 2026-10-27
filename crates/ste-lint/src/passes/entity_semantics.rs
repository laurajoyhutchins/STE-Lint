use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_glossary::{AliasKind, GlossaryIdentityKind, TermRole, TermStatus};

use crate::Resolution;

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
        if canonical_word_count <= 3 {
            continue;
        }

        let representation = classify_representation(mention, canonical_word_count);
        match representation {
            Representation::FullForm => {}
            Representation::AuthorizedShortening => {
                if !full_form_seen_before(&mentions, mention, canonical_word_count) {
                    diagnostics.push(representation_diagnostic(
                        mention,
                        &governed.canonical,
                        domain,
                        canonical_word_count,
                        "full_form_required_first",
                    ));
                }
            }
            Representation::NotAuthorizedForRule22 => diagnostics.push(representation_diagnostic(
                mention,
                &governed.canonical,
                domain,
                canonical_word_count,
                "representation_not_authorized",
            )),
        }
    }

    diagnostics.extend(governed_term_consistency(analysis, &mentions));
    diagnostics
}

// Only identity-backed canonical words and explicit synonym aliases are
// comparable. Approved abbreviations, acronyms, short forms and grammatical
// inflections are not interchangeable spelling evidence for this rule.
fn governed_term_consistency(
    analysis: &AnalysisDocument<'_>,
    mentions: &[EntityMention],
) -> Vec<Diagnostic> {
    let Some(glossary) = analysis.glossary() else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();
    for (index, mention) in mentions.iter().enumerate() {
        let EntityIdentity::GovernedTerm { term, domain } = &mention.identity else {
            continue;
        };
        let Some(governed) = glossary.lookup_term(term) else {
            continue;
        };
        if governed.status != TermStatus::Approved
            || !governed.has_role(TermRole::Noun)
            || !is_comparable_term_mention(analysis, mention)
        {
            continue;
        }

        let first_different = mentions[..index].iter().find(|prior| {
            prior.identity == mention.identity
                && is_comparable_term_mention(analysis, prior)
                && !prior.surface.eq_ignore_ascii_case(&mention.surface)
        });
        let Some(prior) = first_different else {
            continue;
        };

        diagnostics.push(Diagnostic {
            code: "STE-TERM-005".into(),
            severity: Severity::Error,
            message: format!(
                "The governed technical noun forms '{}' and '{}' express the same governed term; use consistent terminology.",
                prior.surface, mention.surface
            ),
            span: Span {
                start: mention.span.start,
                end: mention.span.end,
            },
            rules: vec!["9.4".into()],
            evidence: Some(json!({
                "resolution": "governed_concept_different_synonyms",
                "term_id": governed.id,
                "canonical": governed.canonical,
                "domain": domain,
                "first_form": prior.surface,
                "first_span": {"start": prior.span.start, "end": prior.span.end},
                "different_form": mention.surface,
                "different_span": {"start": mention.span.start, "end": mention.span.end},
                "sources": governed.sources,
            })),
            autofix: None,
        });
    }
    diagnostics
}

fn is_comparable_term_mention(analysis: &AnalysisDocument<'_>, mention: &EntityMention) -> bool {
    if analysis.is_protected_span(mention.span.start, mention.span.end)
        || !matches!(
            analysis.entity_mention_at(mention.span.token_start),
            Resolution::Resolved(resolved) if resolved.identity == mention.identity
        )
    {
        return false;
    }
    match mention.glossary_identity_kind {
        Some(GlossaryIdentityKind::Canonical) => true,
        Some(GlossaryIdentityKind::Alias) => mention.alias_kind == Some(AliasKind::Synonym),
        Some(GlossaryIdentityKind::Form) | None => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Representation {
    FullForm,
    AuthorizedShortening,
    NotAuthorizedForRule22,
}

fn classify_representation(mention: &EntityMention, canonical_word_count: usize) -> Representation {
    match mention.glossary_identity_kind {
        Some(GlossaryIdentityKind::Canonical) => Representation::FullForm,
        Some(GlossaryIdentityKind::Form) => {
            if word_count(&mention.surface) >= canonical_word_count {
                Representation::FullForm
            } else {
                Representation::NotAuthorizedForRule22
            }
        }
        Some(GlossaryIdentityKind::Alias) => match mention.alias_kind {
            Some(AliasKind::Abbreviation | AliasKind::Acronym) => {
                Representation::AuthorizedShortening
            }
            Some(AliasKind::ShortForm) if word_count(&mention.surface) <= 3 => {
                Representation::AuthorizedShortening
            }
            Some(AliasKind::ShortForm | AliasKind::Synonym | AliasKind::Legacy) | None => {
                Representation::NotAuthorizedForRule22
            }
        },
        None => Representation::NotAuthorizedForRule22,
    }
}

fn full_form_seen_before(
    mentions: &[EntityMention],
    mention: &EntityMention,
    canonical_word_count: usize,
) -> bool {
    mentions.iter().any(|candidate| {
        candidate.span.start < mention.span.start
            && candidate.identity == mention.identity
            && classify_representation(candidate, canonical_word_count) == Representation::FullForm
    })
}

fn representation_diagnostic(
    mention: &EntityMention,
    canonical_term: &str,
    domain: &str,
    canonical_word_count: usize,
    reason: &str,
) -> Diagnostic {
    let message = if reason == "full_form_required_first" {
        format!(
            "Write the governed long technical noun '{canonical_term}' in full before using '{}'.",
            mention.surface
        )
    } else {
        format!(
            "The governed representation '{}' is not an authorized Rule 2.2 shortening of '{canonical_term}'.",
            mention.surface
        )
    };

    Diagnostic {
        code: "STE-NOUN-002".into(),
        severity: Severity::Error,
        message,
        span: Span {
            start: mention.span.start,
            end: mention.span.end,
        },
        rules: vec!["2.2".into()],
        evidence: Some(json!({
            "coverage": "governed_long_technical_noun_identity_v2",
            "canonical_term": canonical_term,
            "surface": mention.surface,
            "canonical_word_count": canonical_word_count,
            "surface_word_count": word_count(&mention.surface),
            "domain": domain,
            "identity_kind": mention.glossary_identity_kind,
            "alias_kind": mention.alias_kind,
            "provenance": mention.provenance,
            "reason": reason,
        })),
        autofix: None,
    }
}

fn word_count(value: &str) -> usize {
    value.split_whitespace().count()
}
