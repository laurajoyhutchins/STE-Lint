use crate::OccurrenceFact;

use super::document::{AnalysisDocument, GlossaryMatch, Resolution};
use super::grammar::GrammarSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityIdentity {
    GovernedTerm { term: String, domain: String },
    OfficialTechnicalName { normalized: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityMentionKind {
    GovernedTechnicalTerm,
    OfficialTechnicalName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMention {
    pub identity: EntityIdentity,
    pub kind: EntityMentionKind,
    pub span: GrammarSpan,
    pub surface: String,
    pub definition: Option<String>,
    pub provenance: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceBasis {
    SameSentenceUniqueEntity,
    PreviousSentenceUniqueEntity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLink {
    pub reference: GrammarSpan,
    pub antecedent: EntityMention,
    pub basis: ReferenceBasis,
}

impl<'a> AnalysisDocument<'a> {
    pub fn entity_mentions(&self) -> Vec<EntityMention> {
        let mut mentions = Vec::new();
        let mut token_index = 0;

        while token_index < self.tokens().len() {
            if let Some(matched) = self.longest_glossary_match_at(token_index) {
                let width = matched.token_width;
                mentions.push(mention_from_glossary(matched));
                token_index += width;
            } else {
                token_index += 1;
            }
        }

        if let Some(context) = self.context() {
            for occurrence in &context.occurrences {
                if !occurrence.official_technical_name {
                    continue;
                }
                if let Some(mention) = mention_from_official_name(self, occurrence)
                    && !mentions.iter().any(|existing| {
                        existing.span.start == mention.span.start
                            && existing.span.end == mention.span.end
                    })
                {
                    mentions.push(mention);
                }
            }
        }

        mentions.sort_by_key(|mention| (mention.span.start, mention.span.end));
        mentions
    }

    pub fn entity_mention_at(&self, token_start: usize) -> Resolution<EntityMention> {
        let Some(token) = self.tokens().get(token_start) else {
            return Resolution::Unknown;
        };
        let mut candidates = Vec::new();

        if let Some(matched) = self.longest_glossary_match_at(token_start) {
            candidates.push(mention_from_glossary(matched));
        }

        if let Some(context) = self.context() {
            for occurrence in &context.occurrences {
                if occurrence.official_technical_name
                    && occurrence.start == token.start
                    && let Some(mention) = mention_from_official_name(self, occurrence)
                    && !candidates.iter().any(|existing: &EntityMention| {
                        existing.span.start == mention.span.start
                            && existing.span.end == mention.span.end
                    })
                {
                    candidates.push(mention);
                }
            }
        }

        resolution_from_candidates(candidates)
    }

    pub fn reference_at(&self, token_index: usize) -> Resolution<ReferenceLink> {
        let Some(token) = self.tokens().get(token_index) else {
            return Resolution::Unknown;
        };
        if !matches!(token.text.to_ascii_lowercase().as_str(), "it" | "its") {
            return Resolution::Unknown;
        }
        let Some(sentence_id) = token.sentence_id else {
            return Resolution::Unknown;
        };

        let reference = GrammarSpan {
            token_start: token_index,
            token_end: token_index + 1,
            start: token.start,
            end: token.end,
        };
        let mentions = self.entity_mentions();

        let same_sentence = unique_entity_mentions(
            mentions
                .iter()
                .filter(|mention| {
                    mention.span.end <= token.start
                        && mention_sentence_id(self, mention) == Some(sentence_id)
                })
                .cloned()
                .collect(),
        );
        if !same_sentence.is_empty() {
            return reference_resolution(
                same_sentence,
                reference,
                ReferenceBasis::SameSentenceUniqueEntity,
            );
        }

        if sentence_id == 0 {
            return Resolution::Unknown;
        }
        let previous_sentence = unique_entity_mentions(
            mentions
                .into_iter()
                .filter(|mention| {
                    mention.span.end <= token.start
                        && mention_sentence_id(self, mention) == Some(sentence_id - 1)
                })
                .collect(),
        );
        reference_resolution(
            previous_sentence,
            reference,
            ReferenceBasis::PreviousSentenceUniqueEntity,
        )
    }
}

fn mention_from_glossary(matched: GlossaryMatch<'_>) -> EntityMention {
    EntityMention {
        identity: EntityIdentity::GovernedTerm {
            term: matched.term.term.clone(),
            domain: matched.term.domain.clone(),
        },
        kind: EntityMentionKind::GovernedTechnicalTerm,
        span: GrammarSpan {
            token_start: matched.token_start,
            token_end: matched.token_start + matched.token_width,
            start: matched.start,
            end: matched.end,
        },
        surface: matched.text,
        definition: (!matched.term.definition.trim().is_empty())
            .then(|| matched.term.definition.clone()),
        provenance: matched.term.provenance.clone(),
    }
}

fn mention_from_official_name(
    analysis: &AnalysisDocument<'_>,
    occurrence: &OccurrenceFact,
) -> Option<EntityMention> {
    let span = exact_analysis_span(analysis, occurrence.start, occurrence.end)?;
    let surface = analysis.text()[occurrence.start..occurrence.end].to_string();
    Some(EntityMention {
        identity: EntityIdentity::OfficialTechnicalName {
            normalized: normalize_identity(&surface),
        },
        kind: EntityMentionKind::OfficialTechnicalName,
        span,
        surface,
        definition: None,
        provenance: vec![occurrence.source.clone()],
    })
}

fn exact_analysis_span(
    analysis: &AnalysisDocument<'_>,
    start: usize,
    end: usize,
) -> Option<GrammarSpan> {
    if start >= end || end > analysis.text().len() {
        return None;
    }
    let tokens = analysis.tokens();
    let token_start = tokens.iter().position(|token| token.start == start)?;
    let sentence_id = tokens[token_start].sentence_id;

    for token_end in token_start + 1..=tokens.len() {
        let last = &tokens[token_end - 1];
        if last.end > end || last.sentence_id != sentence_id {
            return None;
        }
        if token_end > token_start + 1 {
            let previous = &tokens[token_end - 2];
            if !analysis.text()[previous.end..last.start]
                .chars()
                .all(char::is_whitespace)
            {
                return None;
            }
        }
        if last.end == end {
            return Some(GrammarSpan {
                token_start,
                token_end,
                start,
                end,
            });
        }
    }
    None
}

fn mention_sentence_id(
    analysis: &AnalysisDocument<'_>,
    mention: &EntityMention,
) -> Option<usize> {
    analysis
        .tokens()
        .get(mention.span.token_start)
        .and_then(|token| token.sentence_id)
}

fn unique_entity_mentions(mentions: Vec<EntityMention>) -> Vec<EntityMention> {
    let mut unique = Vec::new();
    for mention in mentions {
        if !unique
            .iter()
            .any(|existing: &EntityMention| existing.identity == mention.identity)
        {
            unique.push(mention);
        }
    }
    unique
}

fn reference_resolution(
    mentions: Vec<EntityMention>,
    reference: GrammarSpan,
    basis: ReferenceBasis,
) -> Resolution<ReferenceLink> {
    let candidates = mentions
        .into_iter()
        .map(|antecedent| ReferenceLink {
            reference,
            antecedent,
            basis,
        })
        .collect();
    resolution_from_candidates(candidates)
}

fn resolution_from_candidates<T>(mut candidates: Vec<T>) -> Resolution<T> {
    match candidates.len() {
        0 => Resolution::Unknown,
        1 => Resolution::Resolved(candidates.pop().unwrap()),
        _ => Resolution::Ambiguous(candidates),
    }
}

fn normalize_identity(value: &str) -> String {
    value
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
