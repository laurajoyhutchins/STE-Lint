use ste_data::{PartOfSpeech, RuntimeLexicon};
use ste_glossary::Glossary;
use ste_lint::{AnalysisDocument, LintMode, Resolution, VerbFormRole};

#[test]
fn analysis_document_preserves_spans_sentence_identity_and_runtime_evidence() {
    let text = "USE busway. USE fluxcapacitor.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary =
        Glossary::from_json(include_str!("../../../fixtures/glossary/valid.json")).unwrap();
    let analysis =
        AnalysisDocument::new(text, &lexicon, Some(&glossary), None, LintMode::Procedural);

    let tokens = analysis.tokens();
    assert_eq!(tokens.len(), 4);
    assert_eq!(
        (tokens[0].text, tokens[0].start, tokens[0].end),
        ("USE", 0, 3)
    );
    assert_eq!(tokens[0].sentence_id, Some(0));
    assert_eq!(tokens[1].sentence_id, Some(0));
    assert_eq!(tokens[2].sentence_id, Some(1));
    assert_eq!(tokens[3].sentence_id, Some(1));

    let use_match = analysis.longest_dictionary_match_at(0).unwrap();
    assert_eq!(use_match.text, "USE");
    assert_eq!((use_match.start, use_match.end), (0, 3));
    assert_eq!(use_match.possible_parts_of_speech, vec![PartOfSpeech::Verb]);
    assert!(matches!(use_match.resolution, Resolution::Resolved(_)));
    assert!(
        use_match
            .verb_forms
            .iter()
            .any(|candidate| candidate.role == VerbFormRole::Base)
    );

    let busway = analysis.longest_glossary_match_at(1).unwrap();
    assert_eq!(busway.text, "busway");
    assert_eq!(busway.term.term, "busway");

    assert!(matches!(
        analysis.dictionary_resolution_at(3, 1),
        Resolution::Unknown
    ));
}

#[test]
fn analysis_document_preserves_competing_dictionary_identities() {
    let lexicon = RuntimeLexicon::from_json(
        r#"{
          "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_analysis_ir"},
          "entries": [
            {"lemma":"COMPLETE","status":"approved","part_of_speech":"verb","forms":["COMPLETE","COMPLETES","COMPLETED"],"verb_paradigm":{"classification":"lexical","source_sequence":["COMPLETE","COMPLETES","COMPLETED","COMPLETED"],"base_form":"COMPLETE","simple_present_variants":["COMPLETES"],"simple_past_variants":["COMPLETED"],"past_participle":"COMPLETED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"COMPLETE","status":"approved","part_of_speech":"adjective","forms":["COMPLETE"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap();
    let analysis =
        AnalysisDocument::new("COMPLETE THIS.", &lexicon, None, None, LintMode::Procedural);

    let matched = analysis.longest_dictionary_match_at(0).unwrap();
    assert!(matches!(
        &matched.resolution,
        Resolution::Ambiguous(candidates) if candidates.len() == 2
    ));
    assert_eq!(
        matched.possible_parts_of_speech,
        vec![PartOfSpeech::Verb, PartOfSpeech::Adjective]
    );
}
