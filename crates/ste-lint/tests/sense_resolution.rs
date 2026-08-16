use ste_data::{ApprovalStatus, PartOfSpeech, RuntimeLexicon};
use ste_lint::{
    AnalysisDocument, LintMode, Resolution, SenseIdentity, SenseProvenance, SenseRestrictionTag,
    VerbFormRole,
};

#[test]
fn sense_resolution_exposes_only_source_safe_identity_and_coordinates() {
    let lexicon = lexicon(
        r#"{
          "lemma":"OPEN",
          "status":"approved",
          "part_of_speech":"verb",
          "forms":["OPEN","OPENS","OPENED"],
          "verb_paradigm":{"classification":"lexical","source_sequence":["OPEN","OPENS","OPENED","OPENED"],"base_form":"OPEN","simple_present_variants":["OPENS"],"simple_past_variants":["OPENED"],"past_participle":"OPENED"},
          "senses":[{"meaning":"synthetic private meaning placeholder"}],
          "alternatives":[],
          "restrictions":["synthetic restriction alpha","synthetic restriction beta"],
          "provenance":{"structural_record_index":7,"source_pages":[2,3]}
        }"#,
    );
    let analysis = AnalysisDocument::new(
        "OPEN THE VALVE.",
        &lexicon,
        None,
        None,
        LintMode::Procedural,
    );

    let Resolution::Resolved(sense) = analysis.sense_resolution_at(0, 1) else {
        panic!("one interpreted, grammar-compatible sense should resolve");
    };
    assert_eq!(
        sense.identity,
        SenseIdentity {
            entry_index: 0,
            sense_index: 0,
            lemma: "OPEN".into(),
            part_of_speech: Some(PartOfSpeech::Verb),
        }
    );
    assert_eq!((sense.span.start, sense.span.end), (0, 4));
    assert_eq!(sense.approval_status, ApprovalStatus::Approved);
    assert_eq!(sense.verb_forms, vec![VerbFormRole::Base]);
    assert_eq!(
        sense.restriction_tags,
        vec![
            SenseRestrictionTag {
                entry_index: 0,
                restriction_index: 0,
            },
            SenseRestrictionTag {
                entry_index: 0,
                restriction_index: 1,
            },
        ]
    );
    assert_eq!(
        sense.provenance,
        Some(SenseProvenance {
            structural_record_index: 7,
            source_pages: vec![2, 3],
        })
    );
}

#[test]
fn grammar_role_filters_competing_dictionary_entries_before_sense_resolution() {
    let lexicon = lexicon(
        r#"{
          "lemma":"TEST",
          "status":"approved",
          "part_of_speech":"noun",
          "forms":["TEST"],
          "senses":[{"meaning":"synthetic noun sense"}],
          "alternatives":[],
          "restrictions":[]
        },{
          "lemma":"TEST",
          "status":"approved",
          "part_of_speech":"verb",
          "forms":["TEST"],
          "senses":[{"meaning":"synthetic verb sense"}],
          "alternatives":[],
          "restrictions":[]
        }"#,
    );
    let analysis = AnalysisDocument::new(
        "THE TEST IS READY.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    let Resolution::Resolved(sense) = analysis.sense_resolution_at(1, 1) else {
        panic!("bounded nominal grammar evidence should select the noun entry");
    };
    assert_eq!(sense.identity.entry_index, 0);
    assert_eq!(sense.identity.part_of_speech, Some(PartOfSpeech::Noun));
}

#[test]
fn multiple_senses_in_one_interpreted_entry_remain_ambiguous() {
    let lexicon = lexicon(
        r#"{
          "lemma":"SET",
          "status":"approved",
          "part_of_speech":"noun",
          "forms":["SET"],
          "senses":[
            {"meaning":"synthetic sense one"},
            {"meaning":"synthetic sense two"}
          ],
          "alternatives":[],
          "restrictions":[]
        }"#,
    );
    let analysis = AnalysisDocument::new(
        "THE SET IS READY.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    let Resolution::Ambiguous(candidates) = analysis.sense_resolution_at(1, 1) else {
        panic!("multiple viable senses must not be guessed apart");
    };
    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.identity.sense_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn competing_entries_without_grammar_evidence_remain_ambiguous() {
    let lexicon = lexicon(
        r#"{
          "lemma":"TEST",
          "status":"approved",
          "part_of_speech":"noun",
          "forms":["TEST"],
          "senses":[{"meaning":"synthetic noun sense"}],
          "alternatives":[],
          "restrictions":[]
        },{
          "lemma":"TEST",
          "status":"approved",
          "part_of_speech":"verb",
          "forms":["TEST"],
          "senses":[{"meaning":"synthetic verb sense"}],
          "alternatives":[],
          "restrictions":[]
        }"#,
    );
    let analysis =
        AnalysisDocument::new("TEST.", &lexicon, None, None, LintMode::Descriptive);

    let Resolution::Ambiguous(candidates) = analysis.sense_resolution_at(0, 1) else {
        panic!("dictionary identity alone cannot choose between competing entries");
    };
    assert_eq!(candidates.len(), 2);
}

#[test]
fn structurally_uninterpreted_entry_stays_unknown() {
    let lexicon = lexicon(
        r#"{
          "lemma":"CHECK AGAIN",
          "status":"approved",
          "part_of_speech":null,
          "forms":["CHECK AGAIN"],
          "senses":[{"meaning":"synthetic uninterpreted payload"}],
          "alternatives":[],
          "restrictions":["synthetic restriction"],
          "interpretation_state":"structural",
          "provenance":{"structural_record_index":3,"source_pages":[7,8]}
        }"#,
    );
    let analysis = AnalysisDocument::new(
        "CHECK AGAIN.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    assert!(matches!(
        analysis.sense_resolution_at(0, 2),
        Resolution::Unknown
    ));
}

#[test]
fn absent_dictionary_evidence_has_unknown_sense() {
    let lexicon = lexicon("");
    let analysis = AnalysisDocument::new(
        "FLUXCAPACITOR.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    assert!(matches!(
        analysis.sense_resolution_at(0, 1),
        Resolution::Unknown
    ));
}

fn lexicon(entries: &str) -> RuntimeLexicon {
    RuntimeLexicon::from_json(&format!(
        r#"{{
          "metadata": {{
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_sense_resolution"
          }},
          "entries":[{entries}]
        }}"#
    ))
    .unwrap()
}
