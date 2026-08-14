use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
pub struct LexiconEntry {
    pub lemma: String,
    pub status: ApprovalStatus,
    pub part_of_speech: PartOfSpeech,
    pub forms: Vec<String>,
    pub senses: Vec<Sense>,
    pub alternatives: Vec<Alternative>,
    pub restrictions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconMetadata {
    pub standard: String,
    pub issue: u8,
    pub date: String,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconDocument {
    pub metadata: LexiconMetadata,
    pub entries: Vec<LexiconEntry>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLexicon {
    document: LexiconDocument,
    by_form: HashMap<String, usize>,
    by_lemma: HashMap<String, Vec<usize>>,
}

impl RuntimeLexicon {
    pub fn embedded() -> Result<Self, serde_json::Error> {
        Self::from_json(include_str!("../data/test-lexicon.json"))
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let document: LexiconDocument = serde_json::from_str(json)?;
        let mut by_form = HashMap::new();
        let mut by_lemma: HashMap<String, Vec<usize>> = HashMap::new();

        for (index, entry) in document.entries.iter().enumerate() {
            by_lemma
                .entry(normalize(&entry.lemma))
                .or_default()
                .push(index);
            for form in &entry.forms {
                by_form.insert(normalize(form), index);
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
        self.by_form
            .get(&normalize(form))
            .map(|index| &self.document.entries[*index])
    }

    pub fn lookup_lemma(&self, lemma: &str) -> Vec<&LexiconEntry> {
        self.by_lemma
            .get(&normalize(lemma))
            .into_iter()
            .flatten()
            .map(|index| &self.document.entries[*index])
            .collect()
    }
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
        assert!(!lexicon.entries().is_empty());

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
}
