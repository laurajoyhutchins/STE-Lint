use ste_data::{ApprovalStatus, InterpretationState, PartOfSpeech};

use super::document::{AnalysisDocument, Resolution, VerbFormRole};
use super::grammar::{GrammarSpan, ObservedRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseIdentity {
    pub entry_index: usize,
    pub sense_index: usize,
    pub lemma: String,
    pub part_of_speech: Option<PartOfSpeech>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SenseRestrictionTag {
    pub entry_index: usize,
    pub restriction_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseProvenance {
    pub structural_record_index: u32,
    pub source_pages: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseEvidence {
    pub identity: SenseIdentity,
    pub span: GrammarSpan,
    pub approval_status: ApprovalStatus,
    pub verb_forms: Vec<VerbFormRole>,
    pub restriction_tags: Vec<SenseRestrictionTag>,
    pub provenance: Option<SenseProvenance>,
}

impl<'a> AnalysisDocument<'a> {
    pub fn sense_resolution_at(
        &self,
        token_start: usize,
        token_width: usize,
    ) -> Resolution<SenseEvidence> {
        let Some(matched) = self.dictionary_match_at(token_start, token_width) else {
            return Resolution::Unknown;
        };
        let observed_role = self
            .dictionary_role_at(token_start, token_width)
            .map(|evidence| evidence.role);
        let span = GrammarSpan {
            token_start,
            token_end: token_start + token_width,
            start: matched.start,
            end: matched.end,
        };
        let entries = self.lexicon().entries();
        let mut candidates = Vec::new();

        for entry in matched.candidates {
            if entry.interpretation_state != InterpretationState::Interpreted
                || !role_allows_entry(observed_role, entry.part_of_speech)
            {
                continue;
            }
            let Some(entry_index) = entries.iter().position(|known| std::ptr::eq(known, entry))
            else {
                continue;
            };
            let verb_forms = verb_forms_for_entry(&matched.verb_forms, entry);
            let restriction_tags = (0..entry.restrictions.len())
                .map(|restriction_index| SenseRestrictionTag {
                    entry_index,
                    restriction_index,
                })
                .collect::<Vec<_>>();
            let provenance = entry.provenance.as_ref().map(|provenance| SenseProvenance {
                structural_record_index: provenance.structural_record_index,
                source_pages: provenance.source_pages.clone(),
            });

            for sense_index in 0..entry.senses.len() {
                candidates.push(SenseEvidence {
                    identity: SenseIdentity {
                        entry_index,
                        sense_index,
                        lemma: entry.lemma.clone(),
                        part_of_speech: entry.part_of_speech,
                    },
                    span,
                    approval_status: entry.status,
                    verb_forms: verb_forms.clone(),
                    restriction_tags: restriction_tags.clone(),
                    provenance: provenance.clone(),
                });
            }
        }

        resolution_from_candidates(candidates)
    }
}

fn role_allows_entry(observed_role: Option<ObservedRole>, part_of_speech: Option<PartOfSpeech>) -> bool {
    match observed_role {
        Some(ObservedRole::Nominal) => part_of_speech == Some(PartOfSpeech::Noun),
        Some(ObservedRole::Verbal) => part_of_speech == Some(PartOfSpeech::Verb),
        None => true,
    }
}

fn verb_forms_for_entry(
    candidates: &[super::document::VerbFormCandidate<'_>],
    entry: &ste_data::LexiconEntry,
) -> Vec<VerbFormRole> {
    let mut roles = Vec::new();
    for candidate in candidates {
        if std::ptr::eq(candidate.entry, entry) && !roles.contains(&candidate.role) {
            roles.push(candidate.role);
        }
    }
    roles
}

fn resolution_from_candidates<T>(mut candidates: Vec<T>) -> Resolution<T> {
    match candidates.len() {
        0 => Resolution::Unknown,
        1 => Resolution::Resolved(candidates.pop().unwrap()),
        _ => Resolution::Ambiguous(candidates),
    }
}
