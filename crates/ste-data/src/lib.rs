use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Article,
    Preposition,
    Conjunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Approved,
    Unapproved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlternativeKind {
    ApprovedWord,
    ApprovedPhrase,
    TechnicalNoun,
    TechnicalVerb,
    NoDirectAlternative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairStrategy {
    WordReplacement,
    PhraseReplacement,
    SentenceReconstruction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationState {
    Structural,
    #[default]
    Interpreted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerbClassification {
    Lexical,
    IrregularAuxiliary,
    DefectiveModal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sense {
    pub meaning: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alternative {
    pub kind: AlternativeKind,
    pub text: String,
    pub part_of_speech: Option<PartOfSpeech>,
    pub strategy: RepairStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityProvenance {
    pub drive_file_id: String,
    pub source_sha256: String,
    pub source_byte_size: u64,
    pub physical_pages: u32,
    pub private_bundle_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryCardinalities {
    pub source_declared_approved_words: u32,
    pub source_declared_unapproved_words: u32,
    pub structural_approved_records: u32,
    pub structural_unapproved_records: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryProvenance {
    pub structural_record_index: u32,
    pub source_pages: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSemantics {
    pub word_cell: String,
    pub meaning_or_alternatives: String,
    pub ste_example: String,
    pub non_ste_example: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerbParadigm {
    pub classification: VerbClassification,
    pub source_sequence: Vec<String>,
    pub base_form: String,
    pub simple_present_variants: Vec<String>,
    pub simple_past_variants: Vec<String>,
    pub past_participle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconEntry {
    pub lemma: String,
    pub status: ApprovalStatus,
    pub part_of_speech: Option<PartOfSpeech>,
    pub forms: Vec<String>,
    #[serde(default)]
    pub verb_paradigm: Option<VerbParadigm>,
    pub senses: Vec<Sense>,
    pub alternatives: Vec<Alternative>,
    pub restrictions: Vec<String>,
    #[serde(default)]
    pub interpretation_state: InterpretationState,
    #[serde(default)]
    pub provenance: Option<EntryProvenance>,
    #[serde(default)]
    pub source_semantics: Option<SourceSemantics>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconMetadata {
    pub standard: String,
    pub issue: u8,
    pub date: String,
    pub scope: String,
    #[serde(default)]
    pub authority: Option<AuthorityProvenance>,
    #[serde(default)]
    pub dictionary_cardinalities: Option<DictionaryCardinalities>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconDocument {
    pub metadata: LexiconMetadata,
    pub entries: Vec<LexiconEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeIdentityManifest {
    pub standard: String,
    pub issue: u8,
    pub scope: String,
    pub source_sha256: String,
    pub private_bundle_sha256: String,
    pub runtime_lexicon_sha256: String,
    pub runtime_lexicon_bytes: u64,
    pub structural_records: u32,
    pub structural_approved_records: u32,
    pub structural_unapproved_records: u32,
    pub source_declared_approved_words: u32,
    pub source_declared_unapproved_words: u32,
}

impl RuntimeIdentityManifest {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Error)]
pub enum RuntimeDataError {
    #[error("runtime lexicon byte size mismatch: got {actual}, expected {expected}")]
    ByteSize { actual: u64, expected: u64 },
    #[error("runtime lexicon sha256 mismatch: got {actual}, expected {expected}")]
    Sha256 { actual: String, expected: String },
    #[error("runtime lexicon is not UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("runtime lexicon JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("runtime authority metadata is missing")]
    MissingAuthority,
    #[error("runtime dictionary cardinalities are missing")]
    MissingCardinalities,
    #[error("runtime {field} mismatch: got {actual}, expected {expected}")]
    Contract {
        field: &'static str,
        actual: String,
        expected: String,
    },
}

#[derive(Debug, Clone)]
pub struct RuntimeLexicon {
    document: LexiconDocument,
    by_form: HashMap<String, Vec<usize>>,
    by_lemma: HashMap<String, Vec<usize>>,
}

impl RuntimeLexicon {
    pub fn embedded() -> Result<Self, serde_json::Error> {
        Self::from_json(include_str!("../data/test-lexicon.json"))
    }

    pub fn verified_issue9_from_bytes(bytes: &[u8]) -> Result<Self, RuntimeDataError> {
        let manifest = RuntimeIdentityManifest::from_json(include_str!(
            "../../../data/issue9-runtime.manifest.json"
        ))?;
        Self::from_verified_bytes(bytes, &manifest)
    }

    pub fn from_verified_bytes(
        bytes: &[u8],
        manifest: &RuntimeIdentityManifest,
    ) -> Result<Self, RuntimeDataError> {
        let actual_bytes = bytes.len() as u64;
        if actual_bytes != manifest.runtime_lexicon_bytes {
            return Err(RuntimeDataError::ByteSize {
                actual: actual_bytes,
                expected: manifest.runtime_lexicon_bytes,
            });
        }

        let actual_sha256 = format!("{:x}", Sha256::digest(bytes));
        if actual_sha256 != manifest.runtime_lexicon_sha256 {
            return Err(RuntimeDataError::Sha256 {
                actual: actual_sha256,
                expected: manifest.runtime_lexicon_sha256.clone(),
            });
        }

        let json = std::str::from_utf8(bytes)?;
        let lexicon = Self::from_json(json)?;
        lexicon.validate_identity(manifest)?;
        Ok(lexicon)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let document: LexiconDocument = serde_json::from_str(json)?;
        let mut by_form: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_lemma: HashMap<String, Vec<usize>> = HashMap::new();

        for (index, entry) in document.entries.iter().enumerate() {
            by_lemma
                .entry(normalize(&entry.lemma))
                .or_default()
                .push(index);
            for form in &entry.forms {
                by_form.entry(normalize(form)).or_default().push(index);
            }
        }

        Ok(Self {
            document,
            by_form,
            by_lemma,
        })
    }

    pub fn metadata(&self) -> &LexiconMetadata {
        &self.document.metadata
    }

    pub fn entries(&self) -> &[LexiconEntry] {
        &self.document.entries
    }

    pub fn lookup_form(&self, form: &str) -> Option<&LexiconEntry> {
        let candidates = self.by_form.get(&normalize(form))?;
        if candidates.len() != 1 {
            return None;
        }
        Some(&self.document.entries[candidates[0]])
    }

    pub fn lookup_form_candidates(&self, form: &str) -> Vec<&LexiconEntry> {
        self.by_form
            .get(&normalize(form))
            .into_iter()
            .flatten()
            .map(|index| &self.document.entries[*index])
            .collect()
    }

    pub fn lookup_lemma(&self, lemma: &str) -> Vec<&LexiconEntry> {
        self.by_lemma
            .get(&normalize(lemma))
            .into_iter()
            .flatten()
            .map(|index| &self.document.entries[*index])
            .collect()
    }

    fn validate_identity(
        &self,
        manifest: &RuntimeIdentityManifest,
    ) -> Result<(), RuntimeDataError> {
        require_contract(
            "standard",
            &self.document.metadata.standard,
            &manifest.standard,
        )?;
        require_contract("issue", self.document.metadata.issue, manifest.issue)?;
        require_contract("scope", &self.document.metadata.scope, &manifest.scope)?;

        let authority = self
            .document
            .metadata
            .authority
            .as_ref()
            .ok_or(RuntimeDataError::MissingAuthority)?;
        require_contract(
            "source sha256",
            &authority.source_sha256,
            &manifest.source_sha256,
        )?;
        require_contract(
            "private bundle sha256",
            &authority.private_bundle_sha256,
            &manifest.private_bundle_sha256,
        )?;

        let cardinalities = self
            .document
            .metadata
            .dictionary_cardinalities
            .as_ref()
            .ok_or(RuntimeDataError::MissingCardinalities)?;
        require_contract(
            "source-declared approved words",
            cardinalities.source_declared_approved_words,
            manifest.source_declared_approved_words,
        )?;
        require_contract(
            "source-declared unapproved words",
            cardinalities.source_declared_unapproved_words,
            manifest.source_declared_unapproved_words,
        )?;
        require_contract(
            "structural approved records",
            cardinalities.structural_approved_records,
            manifest.structural_approved_records,
        )?;
        require_contract(
            "structural unapproved records",
            cardinalities.structural_unapproved_records,
            manifest.structural_unapproved_records,
        )?;

        let approved = self
            .document
            .entries
            .iter()
            .filter(|entry| entry.status == ApprovalStatus::Approved)
            .count() as u32;
        let unapproved = self
            .document
            .entries
            .iter()
            .filter(|entry| entry.status == ApprovalStatus::Unapproved)
            .count() as u32;
        require_contract(
            "structural record count",
            self.document.entries.len() as u32,
            manifest.structural_records,
        )?;
        require_contract(
            "derived approved structural record count",
            approved,
            manifest.structural_approved_records,
        )?;
        require_contract(
            "derived unapproved structural record count",
            unapproved,
            manifest.structural_unapproved_records,
        )?;

        Ok(())
    }
}

fn require_contract<T>(field: &'static str, actual: T, expected: T) -> Result<(), RuntimeDataError>
where
    T: PartialEq + ToString,
{
    if actual != expected {
        return Err(RuntimeDataError::Contract {
            field,
            actual: actual.to_string(),
            expected: expected.to_string(),
        });
    }
    Ok(())
}

fn normalize(value: &str) -> String {
    value.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_lexicon_resolves_only_explicit_forms() {
        let lexicon = RuntimeLexicon::embedded().unwrap();

        let ensure = lexicon.lookup_form("ensures").unwrap();
        assert_eq!(ensure.lemma, "ensure");
        assert_eq!(ensure.status, ApprovalStatus::Unapproved);

        let permitted = lexicon.lookup_form("permitted").unwrap();
        assert_eq!(permitted.lemma, "PERMITTED");
        assert_eq!(permitted.status, ApprovalStatus::Approved);

        assert!(lexicon.lookup_form("permitting").is_none());
    }

    #[test]
    fn embedded_document_round_trips_through_the_typed_model() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        assert_eq!(lexicon.metadata().issue, 9);
        assert_eq!(lexicon.metadata().scope, "first_slice_test_lexicon");
        assert!(lexicon.metadata().authority.is_none());
        assert!(lexicon.metadata().dictionary_cardinalities.is_none());
        assert!(!lexicon.entries().is_empty());
        assert!(
            lexicon
                .entries()
                .iter()
                .all(|entry| entry.interpretation_state == InterpretationState::Interpreted)
        );

        let encoded = serde_json::to_string(&lexicon.document).unwrap();
        let decoded: LexiconDocument = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, lexicon.document);
    }

    #[test]
    fn enriched_runtime_document_preserves_authority_and_source_semantics() {
        let json = r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_authority_mapping",
            "authority": {
              "drive_file_id": "synthetic-drive-object",
              "source_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              "source_byte_size": 123,
              "physical_pages": 4,
              "private_bundle_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            },
            "dictionary_cardinalities": {
              "source_declared_approved_words": 2,
              "source_declared_unapproved_words": 2,
              "structural_approved_records": 3,
              "structural_unapproved_records": 2
            }
          },
          "entries": [{
            "lemma": "CHECK AGAIN",
            "status": "approved",
            "part_of_speech": null,
            "forms": ["CHECK AGAIN"],
            "senses": [],
            "alternatives": [],
            "restrictions": [],
            "interpretation_state": "structural",
            "provenance": {"structural_record_index": 3, "source_pages": [7, 8]},
            "source_semantics": {
              "word_cell": "CHECK AGAIN",
              "meaning_or_alternatives": "synthetic source meaning",
              "ste_example": "CHECK AGAIN.",
              "non_ste_example": ""
            }
          }]
        }"#;

        let lexicon = RuntimeLexicon::from_json(json).unwrap();
        let entry = lexicon.lookup_form("check again").unwrap();
        assert_eq!(entry.part_of_speech, None);
        assert_eq!(entry.interpretation_state, InterpretationState::Structural);
        assert_eq!(entry.provenance.as_ref().unwrap().source_pages, vec![7, 8]);
        assert_eq!(
            lexicon
                .metadata()
                .dictionary_cardinalities
                .as_ref()
                .unwrap()
                .structural_approved_records,
            3
        );
        assert_eq!(
            entry
                .source_semantics
                .as_ref()
                .unwrap()
                .meaning_or_alternatives,
            "synthetic source meaning"
        );
    }

    #[test]
    fn verb_paradigm_round_trips_through_runtime_model() {
        let json = r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_verb_paradigm"
          },
          "entries": [{
            "lemma": "REMOVE",
            "status": "approved",
            "part_of_speech": "verb",
            "forms": ["REMOVE", "REMOVES", "REMOVED"],
            "verb_paradigm": {
              "classification": "lexical",
              "source_sequence": ["REMOVE", "REMOVES", "REMOVED", "REMOVED"],
              "base_form": "REMOVE",
              "simple_present_variants": ["REMOVES"],
              "simple_past_variants": ["REMOVED"],
              "past_participle": "REMOVED"
            },
            "senses": [],
            "alternatives": [],
            "restrictions": []
          }]
        }"#;

        let lexicon = RuntimeLexicon::from_json(json).unwrap();
        let entry = lexicon.lookup_form("removed").unwrap();
        let paradigm = entry.verb_paradigm.as_ref().unwrap();
        assert_eq!(paradigm.classification, VerbClassification::Lexical);
        assert_eq!(
            paradigm.source_sequence,
            vec!["REMOVE", "REMOVES", "REMOVED", "REMOVED"]
        );
        assert_eq!(paradigm.base_form, "REMOVE");
        assert_eq!(paradigm.simple_present_variants, vec!["REMOVES"]);
        assert_eq!(paradigm.simple_past_variants, vec!["REMOVED"]);
        assert_eq!(paradigm.past_participle.as_deref(), Some("REMOVED"));
    }

    #[test]
    fn ambiguous_forms_preserve_all_candidates_without_arbitrary_resolution() {
        let json = r#"{
          "metadata": {
            "standard": "ASD-STE100",
            "issue": 9,
            "date": "2025-01-15",
            "scope": "synthetic_collision"
          },
          "entries": [
            {
              "lemma": "CHECK",
              "status": "approved",
              "part_of_speech": "noun",
              "forms": ["CHECK"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            },
            {
              "lemma": "check",
              "status": "unapproved",
              "part_of_speech": "verb",
              "forms": ["check"],
              "senses": [],
              "alternatives": [],
              "restrictions": []
            }
          ]
        }"#;

        let lexicon = RuntimeLexicon::from_json(json).unwrap();
        let candidates = lexicon.lookup_form_candidates("check");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].part_of_speech, Some(PartOfSpeech::Noun));
        assert_eq!(candidates[1].part_of_speech, Some(PartOfSpeech::Verb));
        assert!(lexicon.lookup_form("check").is_none());
    }
}
