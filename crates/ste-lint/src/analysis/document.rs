use ste_data::{LexiconEntry, PartOfSpeech, RuntimeLexicon};
use ste_glossary::{Glossary, TechnicalTerm};

use crate::{LintContext, LintMode};

use super::grammar::{self, ObservedRoleEvidence};
use super::sentence::{AnalysisSentence, build_sentences};
use super::token::{AnalysisToken, WordToken, lexical_tokens, word_tokens};

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
}

#[derive(Debug)]
pub struct AnalysisDocument<'a> {
    text: &'a str,
    lexicon: &'a RuntimeLexicon,
    glossary: Option<&'a Glossary>,
    context: Option<&'a LintContext>,
    mode: LintMode,
    tokens: Vec<AnalysisToken<'a>>,
    word_tokens: Vec<WordToken<'a>>,
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
        let mut tokens = lexical_tokens(text);
        let sentences = build_sentences(text, &mut tokens);
        let max_dictionary_words = lexicon
            .entries()
            .iter()
            .flat_map(|entry| &entry.forms)
            .map(|form| form.split_whitespace().count())
            .max()
            .unwrap_or(1);
        let max_glossary_words = glossary
            .map(|glossary| {
                glossary
                    .terms
                    .iter()
                    .flat_map(|term| std::iter::once(&term.term).chain(term.aliases.iter()))
                    .map(|value| value.split_whitespace().count())
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
            word_tokens: word_tokens(text),
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
            if !analysis_tokens_whitespace_joined(self.text, window) {
                continue;
            }
            let phrase = window
                .iter()
                .map(|token| token.text)
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(term) = glossary.lookup_term(&phrase) {
                return Some(GlossaryMatch {
                    token_start,
                    token_width: width,
                    text: phrase,
                    start: window[0].start,
                    end: window[width - 1].end,
                    term,
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
        grammar::dictionary_role(self.text, &self.tokens, token_start, token_width, self.mode)
    }

    pub fn technical_role_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Option<ObservedRoleEvidence> {
        grammar::technical_role(self.text, &self.tokens, token_start, token_width, self.mode)
    }

    pub(crate) fn word_tokens(&self) -> &[WordToken<'a>] {
        &self.word_tokens
    }

    pub(crate) fn word_dictionary_match_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Option<DictionaryMatch<'a>> {
        if token_width == 0 || token_start + token_width > self.word_tokens.len() {
            return None;
        }
        let window = &self.word_tokens[token_start..token_start + token_width];
        if !word_tokens_whitespace_joined(self.text, window) {
            return None;
        }
        let start = window[0].start;
        let end = window[token_width - 1].end;
        let phrase = self.text[start..end].to_string();
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

fn word_tokens_whitespace_joined(text: &str, tokens: &[WordToken<'_>]) -> bool {
    tokens.windows(2).all(|pair| {
        text[pair[0].end..pair[1].start]
            .chars()
            .all(char::is_whitespace)
    })
}
