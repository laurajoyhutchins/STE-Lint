use ste_data::{PartOfSpeech, VerbClassification};

use crate::LintMode;

use super::AnalysisToken;
use super::document::{AnalysisDocument, Resolution, VerbFormRole};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrammarSpan {
    pub token_start: usize,
    pub token_end: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NounPhrase {
    pub span: GrammarSpan,
    pub head_token: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubjectPredicate {
    pub sentence_id: usize,
    pub subject: GrammarSpan,
    pub predicate: GrammarSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxiliaryKind {
    Be,
    Have,
    Modal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxiliaryChain {
    pub span: GrammarSpan,
    pub auxiliaries: Vec<usize>,
    pub kinds: Vec<AuxiliaryKind>,
    pub lexical_head: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipleRole {
    PerfectVerb,
    PassiveVerb,
    Adjectival,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticipleUse {
    pub span: GrammarSpan,
    pub role: ParticipleRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngRole {
    Progressive,
    Nominal,
    Adjectival,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IngUse {
    pub span: GrammarSpan,
    pub role: IngRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCardinality {
    Single,
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionStructure {
    pub sentence_id: usize,
    pub action_heads: Vec<GrammarSpan>,
    pub cardinality: ActionCardinality,
}

impl<'a> AnalysisDocument<'a> {
    pub fn noun_phrase_at(&self, token_start: usize) -> Resolution<NounPhrase> {
        noun_phrase(self, token_start)
    }

    pub fn subject_predicate(&self, sentence_id: usize) -> Resolution<SubjectPredicate> {
        subject_predicate(self, sentence_id)
    }

    pub fn auxiliary_chain_at(&self, token_start: usize) -> Resolution<AuxiliaryChain> {
        auxiliary_chain(self, token_start)
    }

    pub fn participle_use_at(&self, token_index: usize) -> Resolution<ParticipleUse> {
        participle_use(self, token_index)
    }

    pub fn ing_use_at(&self, token_index: usize) -> Resolution<IngUse> {
        ing_use(self, token_index)
    }

    pub fn action_structure(&self, sentence_id: usize) -> Resolution<ActionStructure> {
        action_structure(self, sentence_id)
    }
}

fn noun_phrase(analysis: &AnalysisDocument<'_>, token_start: usize) -> Resolution<NounPhrase> {
    let tokens = analysis.tokens();
    let Some(first) = tokens.get(token_start) else {
        return Resolution::Unknown;
    };
    if !is_determiner_token(analysis, token_start) {
        return Resolution::Unknown;
    }

    let mut candidates = Vec::new();
    for index in token_start + 1..tokens.len() {
        if tokens[index].sentence_id != first.sentence_id
            || !separator_is_whitespace(analysis.text(), &tokens[index - 1], &tokens[index])
        {
            break;
        }

        let token = &tokens[index];
        let parts = token_parts_of_speech(analysis, index);
        if token.generic_is_verb
            || token.generic_is_preposition
            || token.generic_is_conjunction
            || token.generic_is_determiner
        {
            break;
        }

        if (token.generic_is_noun || parts.contains(&PartOfSpeech::Noun))
            && let Some(span) = grammar_span(tokens, token_start, index + 1)
        {
            candidates.push(NounPhrase {
                span,
                head_token: index,
            });
        }
    }

    resolution_from_candidates(candidates)
}

fn subject_predicate(
    analysis: &AnalysisDocument<'_>,
    sentence_id: usize,
) -> Resolution<SubjectPredicate> {
    let Some(sentence) = analysis
        .sentences()
        .iter()
        .find(|sentence| sentence.id == sentence_id)
    else {
        return Resolution::Unknown;
    };
    let (Some(first), Some(last)) = (sentence.first_token, sentence.last_token) else {
        return Resolution::Unknown;
    };

    match noun_phrase(analysis, first) {
        Resolution::Resolved(noun_phrase) => {
            clause_for_noun_phrase(analysis, sentence_id, noun_phrase, last)
                .map_or(Resolution::Unknown, Resolution::Resolved)
        }
        Resolution::Ambiguous(noun_phrases) => {
            let clauses = noun_phrases
                .into_iter()
                .filter_map(|noun_phrase| {
                    clause_for_noun_phrase(analysis, sentence_id, noun_phrase, last)
                })
                .collect::<Vec<_>>();
            resolution_from_candidates(clauses)
        }
        Resolution::Unknown => Resolution::Unknown,
    }
}

fn clause_for_noun_phrase(
    analysis: &AnalysisDocument<'_>,
    sentence_id: usize,
    noun_phrase: NounPhrase,
    sentence_token_end: usize,
) -> Option<SubjectPredicate> {
    let tokens = analysis.tokens();
    let predicate_start = (noun_phrase.span.token_end..sentence_token_end)
        .find(|index| token_has_part_of_speech(analysis, *index, PartOfSpeech::Verb))?;
    let predicate = grammar_span(tokens, predicate_start, sentence_token_end)?;

    Some(SubjectPredicate {
        sentence_id,
        subject: noun_phrase.span,
        predicate,
    })
}

fn auxiliary_chain(
    analysis: &AnalysisDocument<'_>,
    token_start: usize,
) -> Resolution<AuxiliaryChain> {
    let initial_kinds = auxiliary_kinds(analysis, token_start);
    if initial_kinds.is_empty() {
        return Resolution::Unknown;
    }

    let candidates = initial_kinds
        .into_iter()
        .filter_map(|kind| build_auxiliary_chain(analysis, token_start, kind))
        .collect::<Vec<_>>();
    resolution_from_candidates(candidates)
}

fn build_auxiliary_chain(
    analysis: &AnalysisDocument<'_>,
    token_start: usize,
    initial_kind: AuxiliaryKind,
) -> Option<AuxiliaryChain> {
    let tokens = analysis.tokens();
    let first = tokens.get(token_start)?;
    let mut auxiliaries = vec![token_start];
    let mut kinds = vec![initial_kind];
    let mut index = token_start + 1;

    while index < tokens.len()
        && tokens[index].sentence_id == first.sentence_id
        && separator_is_whitespace(analysis.text(), &tokens[index - 1], &tokens[index])
    {
        let next_kinds = auxiliary_kinds(analysis, index);
        if next_kinds.len() != 1 {
            break;
        }
        auxiliaries.push(index);
        kinds.push(next_kinds[0]);
        index += 1;
    }

    let lexical_head = (index < tokens.len()
        && tokens[index].sentence_id == first.sentence_id
        && separator_is_whitespace(analysis.text(), &tokens[index - 1], &tokens[index])
        && token_has_part_of_speech(analysis, index, PartOfSpeech::Verb))
    .then_some(index);
    let span_end = lexical_head.map_or(index, |head| head + 1);
    let span = grammar_span(tokens, token_start, span_end)?;

    Some(AuxiliaryChain {
        span,
        auxiliaries,
        kinds,
        lexical_head,
    })
}

fn participle_use(
    analysis: &AnalysisDocument<'_>,
    token_index: usize,
) -> Resolution<ParticipleUse> {
    let Some(matched) = analysis.dictionary_match_at(token_index, 1) else {
        return Resolution::Unknown;
    };
    if !matched
        .verb_forms
        .iter()
        .any(|candidate| candidate.role == VerbFormRole::PastParticiple)
    {
        return Resolution::Unknown;
    }
    let Some(span) = grammar_span(analysis.tokens(), token_index, token_index + 1) else {
        return Resolution::Unknown;
    };
    let previous_kinds = previous_auxiliary_kinds(analysis, token_index);

    if previous_kinds == vec![AuxiliaryKind::Have] {
        return Resolution::Resolved(ParticipleUse {
            span,
            role: ParticipleRole::PerfectVerb,
        });
    }

    if previous_kinds == vec![AuxiliaryKind::Be] {
        let mut candidates = vec![ParticipleUse {
            span,
            role: ParticipleRole::PassiveVerb,
        }];
        if matched
            .possible_parts_of_speech
            .contains(&PartOfSpeech::Adjective)
        {
            candidates.push(ParticipleUse {
                span,
                role: ParticipleRole::Adjectival,
            });
        }
        return resolution_from_candidates(candidates);
    }

    Resolution::Unknown
}

fn ing_use(analysis: &AnalysisDocument<'_>, token_index: usize) -> Resolution<IngUse> {
    let Some(token) = analysis.tokens().get(token_index) else {
        return Resolution::Unknown;
    };
    if !token.generic_is_progressive_form {
        return Resolution::Unknown;
    }
    let Some(matched) = analysis.dictionary_match_at(token_index, 1) else {
        return Resolution::Unknown;
    };
    let Some(span) = grammar_span(analysis.tokens(), token_index, token_index + 1) else {
        return Resolution::Unknown;
    };

    if previous_auxiliary_kinds(analysis, token_index) == vec![AuxiliaryKind::Be] {
        let mut candidates = Vec::new();
        if matched
            .possible_parts_of_speech
            .contains(&PartOfSpeech::Verb)
        {
            candidates.push(IngUse {
                span,
                role: IngRole::Progressive,
            });
        }
        if matched
            .possible_parts_of_speech
            .contains(&PartOfSpeech::Adjective)
        {
            candidates.push(IngUse {
                span,
                role: IngRole::Adjectival,
            });
        }
        return resolution_from_candidates(candidates);
    }

    if token_index > 0
        && token_index + 1 < analysis.tokens().len()
        && same_sentence_and_whitespace(analysis, token_index - 1, token_index)
        && same_sentence_and_whitespace(analysis, token_index, token_index + 1)
        && is_determiner_token(analysis, token_index - 1)
        && auxiliary_kinds(analysis, token_index + 1).contains(&AuxiliaryKind::Be)
        && matched
            .possible_parts_of_speech
            .contains(&PartOfSpeech::Noun)
    {
        return Resolution::Resolved(IngUse {
            span,
            role: IngRole::Nominal,
        });
    }

    Resolution::Unknown
}

fn action_structure(
    analysis: &AnalysisDocument<'_>,
    sentence_id: usize,
) -> Resolution<ActionStructure> {
    if analysis.mode() != LintMode::Procedural {
        return Resolution::Unknown;
    }
    let Some(sentence) = analysis
        .sentences()
        .iter()
        .find(|sentence| sentence.id == sentence_id)
    else {
        return Resolution::Unknown;
    };
    let (Some(first), Some(last)) = (sentence.first_token, sentence.last_token) else {
        return Resolution::Unknown;
    };
    if !is_base_form_verb(analysis, first) {
        return Resolution::Unknown;
    }

    let mut action_heads = vec![grammar_span(analysis.tokens(), first, first + 1).unwrap()];
    let mut index = first + 1;
    while index + 1 < last {
        if analysis.tokens()[index].text.eq_ignore_ascii_case("and")
            && same_sentence_and_whitespace(analysis, index, index + 1)
            && is_base_form_verb(analysis, index + 1)
            && let Some(span) = grammar_span(analysis.tokens(), index + 1, index + 2)
        {
            action_heads.push(span);
            index += 2;
            continue;
        }
        index += 1;
    }

    let cardinality = if action_heads.len() == 1 {
        ActionCardinality::Single
    } else {
        ActionCardinality::Multiple
    };
    Resolution::Resolved(ActionStructure {
        sentence_id,
        action_heads,
        cardinality,
    })
}

fn auxiliary_kinds(analysis: &AnalysisDocument<'_>, token_index: usize) -> Vec<AuxiliaryKind> {
    let Some(matched) = analysis.dictionary_match_at(token_index, 1) else {
        return Vec::new();
    };
    let mut kinds = Vec::new();
    for candidate in matched.candidates {
        let kind =
            if candidate.lemma.eq_ignore_ascii_case("be") {
                Some(AuxiliaryKind::Be)
            } else if candidate.lemma.eq_ignore_ascii_case("have") {
                Some(AuxiliaryKind::Have)
            } else if candidate.verb_paradigm.as_ref().is_some_and(|paradigm| {
                paradigm.classification == VerbClassification::DefectiveModal
            }) {
                Some(AuxiliaryKind::Modal)
            } else {
                None
            };
        if let Some(kind) = kind
            && !kinds.contains(&kind)
        {
            kinds.push(kind);
        }
    }
    kinds
}

fn previous_auxiliary_kinds(
    analysis: &AnalysisDocument<'_>,
    token_index: usize,
) -> Vec<AuxiliaryKind> {
    if token_index == 0 || !same_sentence_and_whitespace(analysis, token_index - 1, token_index) {
        return Vec::new();
    }
    auxiliary_kinds(analysis, token_index - 1)
}

fn is_base_form_verb(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    analysis
        .dictionary_match_at(token_index, 1)
        .is_some_and(|matched| {
            matched
                .verb_forms
                .iter()
                .any(|candidate| candidate.role == VerbFormRole::Base)
        })
}

fn is_determiner_token(analysis: &AnalysisDocument<'_>, token_index: usize) -> bool {
    analysis.tokens().get(token_index).is_some_and(|token| {
        token.generic_is_determiner
            || token_has_part_of_speech(analysis, token_index, PartOfSpeech::Article)
    })
}

fn token_has_part_of_speech(
    analysis: &AnalysisDocument<'_>,
    token_index: usize,
    part: PartOfSpeech,
) -> bool {
    token_parts_of_speech(analysis, token_index).contains(&part)
}

fn token_parts_of_speech(analysis: &AnalysisDocument<'_>, token_index: usize) -> Vec<PartOfSpeech> {
    analysis
        .dictionary_match_at(token_index, 1)
        .map_or_else(Vec::new, |matched| matched.possible_parts_of_speech)
}

fn grammar_span(
    tokens: &[AnalysisToken<'_>],
    token_start: usize,
    token_end: usize,
) -> Option<GrammarSpan> {
    if token_start >= token_end || token_end > tokens.len() {
        return None;
    }
    Some(GrammarSpan {
        token_start,
        token_end,
        start: tokens[token_start].start,
        end: tokens[token_end - 1].end,
    })
}

fn same_sentence_and_whitespace(
    analysis: &AnalysisDocument<'_>,
    left_index: usize,
    right_index: usize,
) -> bool {
    let tokens = analysis.tokens();
    let (Some(left), Some(right)) = (tokens.get(left_index), tokens.get(right_index)) else {
        return false;
    };
    left.sentence_id == right.sentence_id && separator_is_whitespace(analysis.text(), left, right)
}

fn resolution_from_candidates<T>(mut candidates: Vec<T>) -> Resolution<T> {
    match candidates.len() {
        0 => Resolution::Unknown,
        1 => Resolution::Resolved(candidates.pop().unwrap()),
        _ => Resolution::Ambiguous(candidates),
    }
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
        && tokens[index - 1].generic_is_determiner
        && tokens[next_index].generic_is_linking_verb
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
        && tokens[next_index].generic_is_determiner
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
        && tokens[index - 1].generic_is_determiner
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
        && tokens[next_index].generic_is_determiner
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

// Generic grammatical shape comes from Harper token metadata. STE approval remains dictionary-driven.
