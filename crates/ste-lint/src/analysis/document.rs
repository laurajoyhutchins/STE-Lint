use ste_data::{LexiconEntry, PartOfSpeech, RuntimeLexicon};
use ste_glossary::{AliasKind, Glossary, GlossaryIdentityKind, TechnicalTerm, TermRole};

use crate::{LintContext, LintMode};

use super::evidence::{AnalysisEvidence, EvidenceTarget};
use super::grammar::{self, ObservedRoleEvidence};
use super::linguistic::{HarperProvider, LexicalObservation};
use super::sentence::{AnalysisSentence, build_sentences};
use super::source::{CanonicalSource, CanonicalSpan};
use super::token::AnalysisToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbFormRole {
    Base,
    SimplePresent,
    SimplePast,
    PastParticiple,
}

#[derive(Debug, Clone, Copy)]
pub struct VerbFormCandidate<'a> {
    pub entry: &'a LexiconEntry,
    pub role: VerbFormRole,
}

#[derive(Debug, Clone)]
pub struct DictionaryMatch<'a> {
    pub token_start: usize,
    pub token_width: usize,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub candidates: Vec<&'a LexiconEntry>,
    pub resolution: Resolution<&'a LexiconEntry>,
    pub possible_parts_of_speech: Vec<PartOfSpeech>,
    pub verb_forms: Vec<VerbFormCandidate<'a>>,
}

#[derive(Debug, Clone)]
pub struct GlossaryMatch<'a> {
    pub token_start: usize,
    pub token_width: usize,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub term: &'a TechnicalTerm,
    pub identity_kind: GlossaryIdentityKind,
    pub roles: &'a [TermRole],
    pub alias_kind: Option<AliasKind>,
}

#[derive(Debug)]
pub struct AnalysisDocument<'a> {
    text: &'a str,
    lexicon: &'a RuntimeLexicon,
    glossary: Option<&'a Glossary>,
    context: Option<&'a LintContext>,
    mode: LintMode,
    tokens: Vec<AnalysisToken<'a>>,
    source: CanonicalSource<'a>,
    lexical_evidence: Vec<AnalysisEvidence<LexicalObservation>>,
    sentences: Vec<AnalysisSentence>,
    max_dictionary_words: usize,
    max_glossary_words: usize,
}

impl<'a> AnalysisDocument<'a> {
    pub fn new(
        text: &'a str,
        lexicon: &'a RuntimeLexicon,
        glossary: Option<&'a Glossary>,
        context: Option<&'a LintContext>,
        mode: LintMode,
    ) -> Self {
        let source = CanonicalSource::with_context(text, context);
        let lexical_evidence = HarperProvider::analyze(&source);
        let mut tokens = lexical_evidence
            .iter()
            .map(|evidence| {
                let EvidenceTarget::Token(span) = evidence.target else {
                    unreachable!("Harper lexical evidence must target canonical tokens");
                };
                AnalysisToken {
                    text: &text[span.start..span.end],
                    start: span.start,
                    end: span.end,
                    sentence_id: None,
                }
            })
            .collect::<Vec<_>>();
        let sentences = build_sentences(text, &mut tokens);
        let max_dictionary_words = lexicon
            .entries()
            .iter()
            .flat_map(|entry| &entry.forms)
            .map(|form| source_form_token_count(form))
            .max()
            .unwrap_or(1);
        let max_glossary_words = glossary
            .map(|glossary| {
                glossary
                    .terms()
                    .iter()
                    .flat_map(|term| {
                        std::iter::once(term.canonical.as_str())
                            .chain(term.aliases.iter().map(|alias| alias.text.as_str()))
                            .chain(term.forms.iter().map(|form| form.text.as_str()))
                    })
                    .map(source_form_token_count)
                    .max()
                    .unwrap_or(1)
            })
            .unwrap_or(1);

        Self {
            text,
            lexicon,
            glossary,
            context,
            mode,
            tokens,
            source,
            lexical_evidence,
            sentences,
            max_dictionary_words,
            max_glossary_words,
        }
    }

    pub fn text(&self) -> &'a str {
        self.text
    }

    pub fn lexicon(&self) -> &'a RuntimeLexicon {
        self.lexicon
    }

    pub fn glossary(&self) -> Option<&'a Glossary> {
        self.glossary
    }

    pub fn context(&self) -> Option<&'a LintContext> {
        self.context
    }

    pub fn mode(&self) -> LintMode {
        self.mode
    }

    pub fn tokens(&self) -> &[AnalysisToken<'a>] {
        &self.tokens
    }

    pub fn canonical_span(&self, start: usize, end: usize) -> Option<CanonicalSpan> {
        self.source.span(start, end)
    }

    pub fn lexical_evidence(&self) -> &[AnalysisEvidence<LexicalObservation>] {
        &self.lexical_evidence
    }

    pub(crate) fn linguistic_token(&self, index: usize) -> Option<&LexicalObservation> {
        self.lexical_evidence
            .get(index)
            .map(|evidence| &evidence.value)
    }

    pub fn sentences(&self) -> &[AnalysisSentence] {
        &self.sentences
    }

    pub fn dictionary_resolution_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Resolution<&'a LexiconEntry> {
        self.dictionary_match_at(token_start, token_width)
            .map_or(Resolution::Unknown, |matched| matched.resolution)
    }

    pub fn dictionary_match_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Option<DictionaryMatch<'a>> {
        if token_width == 0 || token_start + token_width > self.tokens.len() {
            return None;
        }
        let window = &self.tokens[token_start..token_start + token_width];
        if !analysis_tokens_whitespace_joined(self.text, window) {
            return None;
        }
        let phrase = window
            .iter()
            .map(|token| token.text)
            .collect::<Vec<_>>()
            .join(" ");
        self.make_dictionary_match(
            token_start,
            token_width,
            phrase,
            window[0].start,
            window[token_width - 1].end,
        )
    }

    pub fn longest_dictionary_match_at(&self, token_start: usize) -> Option<DictionaryMatch<'a>> {
        if token_start >= self.tokens.len() {
            return None;
        }
        let max_width = self
            .max_dictionary_words
            .min(self.tokens.len() - token_start);
        for width in (1..=max_width).rev() {
            if let Some(matched) = self.dictionary_match_at(token_start, width) {
                return Some(matched);
            }
        }
        None
    }

    pub fn longest_glossary_match_at(&self, token_start: usize) -> Option<GlossaryMatch<'a>> {
        let glossary = self.glossary?;
        if token_start >= self.tokens.len() {
            return None;
        }
        let max_width = self.max_glossary_words.min(self.tokens.len() - token_start);
        for width in (1..=max_width).rev() {
            let window = &self.tokens[token_start..token_start + width];
            if !analysis_tokens_source_form_joined(self.text, window) {
                continue;
            }
            let start = window[0].start;
            let end = window[width - 1].end;
            let phrase = normalize_source_form(&self.text[start..end]);
            if let Some(matched) = glossary.lookup_identity(&phrase) {
                return Some(GlossaryMatch {
                    token_start,
                    token_width: width,
                    text: phrase,
                    start,
                    end,
                    term: matched.term,
                    identity_kind: matched.identity_kind,
                    roles: matched.roles,
                    alias_kind: matched.alias_kind,
                });
            }
        }
        None
    }

    pub fn dictionary_role_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Option<ObservedRoleEvidence> {
        grammar::dictionary_role(self, token_start, token_width)
    }

    pub fn technical_role_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Option<ObservedRoleEvidence> {
        grammar::technical_role(self, token_start, token_width)
    }

    pub(crate) fn first_token_in_span(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(usize, &AnalysisToken<'a>)> {
        self.tokens
            .iter()
            .enumerate()
            .find(|(_, token)| token.start >= start && token.end <= end)
    }

    pub(crate) fn leading_dictionary_match_in_span(
        &self,
        start: usize,
        end: usize,
        max_words: usize,
    ) -> Option<DictionaryMatch<'a>> {
        if start >= end || end > self.text.len() || max_words == 0 {
            return None;
        }
        let (token_start, _) = self.first_token_in_span(start, end)?;
        let available = self.tokens[token_start..]
            .iter()
            .take_while(|token| token.end <= end)
            .count()
            .min(max_words)
            .min(self.max_dictionary_words);

        for width in (1..=available).rev() {
            let Some(matched) = self.dictionary_match_at(token_start, width) else {
                continue;
            };
            if matched.start >= start && matched.end <= end {
                return Some(matched);
            }
        }
        None
    }

    pub(crate) fn source_dictionary_match_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Option<DictionaryMatch<'a>> {
        if token_width == 0 || token_start + token_width > self.tokens.len() {
            return None;
        }
        let window = &self.tokens[token_start..token_start + token_width];
        if !analysis_tokens_source_form_joined(self.text, window) {
            return None;
        }
        let start = window[0].start;
        let end = window[token_width - 1].end;
        let phrase = normalize_source_form(&self.text[start..end]);
        self.make_dictionary_match(token_start, token_width, phrase, start, end)
    }

    fn make_dictionary_match(
        &self,
        token_start: usize,
        token_width: usize,
        phrase: String,
        start: usize,
        end: usize,
    ) -> Option<DictionaryMatch<'a>> {
        let candidates = self.lexicon.lookup_form_candidates(&phrase);
        if candidates.is_empty() {
            return None;
        }
        let resolution = resolution(&candidates);
        let possible_parts_of_speech = possible_parts_of_speech(&candidates);
        let verb_forms = verb_forms(&phrase, &candidates);

        Some(DictionaryMatch {
            token_start,
            token_width,
            text: phrase,
            start,
            end,
            candidates,
            resolution,
            possible_parts_of_speech,
            verb_forms,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution<T> {
    Resolved(T),
    Ambiguous(Vec<T>),
    Unknown,
}

fn resolution<'a>(candidates: &[&'a LexiconEntry]) -> Resolution<&'a LexiconEntry> {
    match candidates {
        [] => Resolution::Unknown,
        [only] => Resolution::Resolved(*only),
        many => Resolution::Ambiguous(many.to_vec()),
    }
}

fn possible_parts_of_speech(candidates: &[&LexiconEntry]) -> Vec<PartOfSpeech> {
    let mut parts = Vec::new();
    for entry in candidates {
        if let Some(part) = entry.part_of_speech
            && !parts.contains(&part)
        {
            parts.push(part);
        }
    }
    parts
}

fn verb_forms<'a>(phrase: &str, candidates: &[&'a LexiconEntry]) -> Vec<VerbFormCandidate<'a>> {
    let mut forms = Vec::new();
    for entry in candidates {
        let Some(paradigm) = &entry.verb_paradigm else {
            continue;
        };
        if phrase.eq_ignore_ascii_case(&paradigm.base_form) {
            forms.push(VerbFormCandidate {
                entry,
                role: VerbFormRole::Base,
            });
        }
        if paradigm
            .simple_present_variants
            .iter()
            .any(|form| phrase.eq_ignore_ascii_case(form))
        {
            forms.push(VerbFormCandidate {
                entry,
                role: VerbFormRole::SimplePresent,
            });
        }
        if paradigm
            .simple_past_variants
            .iter()
            .any(|form| phrase.eq_ignore_ascii_case(form))
        {
            forms.push(VerbFormCandidate {
                entry,
                role: VerbFormRole::SimplePast,
            });
        }
        if paradigm
            .past_participle
            .as_deref()
            .is_some_and(|form| phrase.eq_ignore_ascii_case(form))
        {
            forms.push(VerbFormCandidate {
                entry,
                role: VerbFormRole::PastParticiple,
            });
        }
    }
    forms
}

fn analysis_tokens_whitespace_joined(text: &str, tokens: &[AnalysisToken<'_>]) -> bool {
    tokens.windows(2).all(|pair| {
        text[pair[0].end..pair[1].start]
            .chars()
            .all(char::is_whitespace)
    })
}

fn analysis_tokens_source_form_joined(text: &str, tokens: &[AnalysisToken<'_>]) -> bool {
    tokens.windows(2).all(|pair| {
        let separator = &text[pair[0].end..pair[1].start];
        !separator.is_empty()
            && separator
                .chars()
                .all(|character| character.is_whitespace() || character == '-')
    })
}

fn normalize_source_form(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn source_form_token_count(value: &str) -> usize {
    value
        .split(|character: char| character.is_whitespace() || character == '-')
        .filter(|part| !part.is_empty())
        .count()
        .max(1)
}
