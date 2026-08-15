use ste_data::{RuntimeIdentityManifest, RuntimeLexicon};

const SYNTHETIC_RUNTIME: &str = r#"{
  "metadata": {
    "standard": "ASD-STE100",
    "issue": 9,
    "date": "2025-01-15",
    "scope": "issue9_private_authority_runtime",
    "authority": {
      "drive_file_id": "synthetic-drive",
      "source_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "source_byte_size": 123,
      "physical_pages": 4,
      "private_bundle_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    },
    "dictionary_cardinalities": {
      "source_declared_approved_words": 1,
      "source_declared_unapproved_words": 1,
      "structural_approved_records": 1,
      "structural_unapproved_records": 1
    }
  },
  "entries": [
    {
      "lemma": "CHECK",
      "status": "approved",
      "part_of_speech": "verb",
      "forms": ["CHECK"],
      "senses": [],
      "alternatives": [],
      "restrictions": [],
      "interpretation_state": "structural",
      "provenance": {"structural_record_index": 0, "source_pages": [1]},
      "source_semantics": {
        "word_cell": "CHECK",
        "meaning_or_alternatives": "synthetic",
        "ste_example": "CHECK.",
        "non_ste_example": ""
      }
    },
    {
      "lemma": "badword",
      "status": "unapproved",
      "part_of_speech": "noun",
      "forms": ["badword"],
      "senses": [],
      "alternatives": [],
      "restrictions": [],
      "interpretation_state": "structural",
      "provenance": {"structural_record_index": 1, "source_pages": [2]},
      "source_semantics": {
        "word_cell": "badword",
        "meaning_or_alternatives": "synthetic",
        "ste_example": "USE CHECK.",
        "non_ste_example": "badword"
      }
    }
  ]
}
"#;

const SYNTHETIC_MANIFEST: &str = r#"{
  "standard": "ASD-STE100",
  "issue": 9,
  "scope": "issue9_private_authority_runtime",
  "source_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "private_bundle_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "runtime_lexicon_sha256": "37529bfad47aab92160c86a61ee1e25d8c255883d6d37945498520c00f09c2fe",
  "runtime_lexicon_bytes": 1685,
  "structural_records": 2,
  "structural_approved_records": 1,
  "structural_unapproved_records": 1,
  "source_declared_approved_words": 1,
  "source_declared_unapproved_words": 1
}
"#;

#[test]
fn exact_runtime_identity_is_accepted() {
    let manifest = RuntimeIdentityManifest::from_json(SYNTHETIC_MANIFEST).unwrap();
    let lexicon = RuntimeLexicon::from_verified_bytes(SYNTHETIC_RUNTIME.as_bytes(), &manifest).unwrap();

    assert_eq!(lexicon.entries().len(), 2);
    assert_eq!(lexicon.metadata().scope, "issue9_private_authority_runtime");
}

#[test]
fn same_size_runtime_tampering_is_rejected_by_digest() {
    let manifest = RuntimeIdentityManifest::from_json(SYNTHETIC_MANIFEST).unwrap();
    let tampered = SYNTHETIC_RUNTIME.replacen("CHECK.", "CHOCK.", 1);
    assert_eq!(tampered.len(), SYNTHETIC_RUNTIME.len());

    let error = RuntimeLexicon::from_verified_bytes(tampered.as_bytes(), &manifest).unwrap_err();
    assert!(error.to_string().contains("sha256"));
}

#[test]
fn metadata_mismatch_is_rejected_even_for_matching_bytes() {
    let altered_manifest = SYNTHETIC_MANIFEST.replace(
        "\"scope\": \"issue9_private_authority_runtime\"",
        "\"scope\": \"wrong_runtime_scope___________\"",
    );
    let altered_manifest = altered_manifest.replace(
        "37529bfad47aab92160c86a61ee1e25d8c255883d6d37945498520c00f09c2fe",
        "37529bfad47aab92160c86a61ee1e25d8c255883d6d37945498520c00f09c2fe",
    );
    let manifest = RuntimeIdentityManifest::from_json(&altered_manifest).unwrap();

    let error = RuntimeLexicon::from_verified_bytes(SYNTHETIC_RUNTIME.as_bytes(), &manifest).unwrap_err();
    assert!(error.to_string().contains("scope"));
}
