use ste_data::{PartOfSpeech, RuntimeLexicon};
use ste_glossary::Glossary;
use ste_lint::{
    ActionCardinality, AnalysisDocument, IngRole, LintMode, ParticipleRole, Resolution,
    VerbFormRole,
};

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

#[test]
fn grammar_v1_resolves_determiner_led_subject_predicate() {
    let lexicon = grammar_lexicon();
    let analysis = AnalysisDocument::new(
        "THE MAIN PUMP IS READY.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    let Resolution::Resolved(noun_phrase) = analysis.noun_phrase_at(0) else {
        panic!("expected resolved determiner-led noun phrase");
    };
    assert_eq!(noun_phrase.head_token, 2);
    assert_eq!((noun_phrase.span.start, noun_phrase.span.end), (0, 13));

    let Resolution::Resolved(clause) = analysis.subject_predicate(0) else {
        panic!("expected resolved subject/predicate structure");
    };
    assert_eq!((clause.subject.start, clause.subject.end), (0, 13));
    assert_eq!((clause.predicate.start, clause.predicate.end), (14, 22));
}

#[test]
fn grammar_v1_distinguishes_perfect_from_passive_adjective_ambiguity() {
    let lexicon = grammar_lexicon();
    let perfect = AnalysisDocument::new(
        "THE VALVE HAS CLOSED.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    let Resolution::Resolved(chain) = perfect.auxiliary_chain_at(2) else {
        panic!("expected auxiliary chain");
    };
    assert_eq!(chain.auxiliaries, vec![2]);
    assert_eq!(chain.lexical_head, Some(3));

    let Resolution::Resolved(participle) = perfect.participle_use_at(3) else {
        panic!("HAVE plus a source-backed past participle is a bounded perfect frame");
    };
    assert_eq!(participle.role, ParticipleRole::PerfectVerb);

    let passive = AnalysisDocument::new(
        "THE VALVE IS CLOSED.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );
    let Resolution::Ambiguous(candidates) = passive.participle_use_at(3) else {
        panic!("BE plus a spelling that is both participle and adjective must stay ambiguous");
    };
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.role == ParticipleRole::PassiveVerb)
    );
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.role == ParticipleRole::Adjectival)
    );
}

#[test]
fn grammar_v1_keeps_ing_roles_fail_closed() {
    let lexicon = grammar_lexicon();
    let nominal = AnalysisDocument::new(
        "THE TESTING IS READY.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );
    let Resolution::Resolved(observation) = nominal.ing_use_at(1) else {
        panic!("dictionary noun evidence in a bounded determiner/copula frame should resolve");
    };
    assert_eq!(observation.role, IngRole::Nominal);

    let ambiguous = AnalysisDocument::new(
        "THE PUMP IS LEAKING.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );
    let Resolution::Ambiguous(candidates) = ambiguous.ing_use_at(3) else {
        panic!("BE plus competing verb/adjective identities must remain ambiguous");
    };
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.role == IngRole::Progressive)
    );
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.role == IngRole::Adjectival)
    );
}

#[test]
fn grammar_v1_counts_only_bounded_procedural_action_heads() {
    let lexicon = grammar_lexicon();
    let single = AnalysisDocument::new(
        "OPEN THE VALVE.",
        &lexicon,
        None,
        None,
        LintMode::Procedural,
    );
    let Resolution::Resolved(action) = single.action_structure(0) else {
        panic!("expected a bounded procedural action");
    };
    assert_eq!(action.cardinality, ActionCardinality::Single);
    assert_eq!(action.action_heads.len(), 1);

    let multiple = AnalysisDocument::new(
        "OPEN THE VALVE AND CLOSE THE VALVE.",
        &lexicon,
        None,
        None,
        LintMode::Procedural,
    );
    let Resolution::Resolved(action) = multiple.action_structure(0) else {
        panic!("expected bounded coordinated procedural actions");
    };
    assert_eq!(action.cardinality, ActionCardinality::Multiple);
    assert_eq!(action.action_heads.len(), 2);
    assert_eq!(
        action
            .action_heads
            .iter()
            .map(|span| span.token_start)
            .collect::<Vec<_>>(),
        vec![0, 4]
    );
}

fn grammar_lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_grammar_v1"},
          "entries": [
            {"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"MAIN","status":"approved","part_of_speech":"adjective","forms":["MAIN"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"PUMP","status":"approved","part_of_speech":"noun","forms":["PUMP"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"VALVE","status":"approved","part_of_speech":"noun","forms":["VALVE"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"READY","status":"approved","part_of_speech":"adjective","forms":["READY"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"AND","status":"approved","part_of_speech":"conjunction","forms":["AND"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["BE","IS","ARE","WAS","WERE","BEEN"],"verb_paradigm":{"classification":"irregular_auxiliary","source_sequence":["BE","IS","ARE","WAS","WERE","BEEN"],"base_form":"BE","simple_present_variants":["IS","ARE"],"simple_past_variants":["WAS","WERE"],"past_participle":"BEEN"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"HAVE","status":"approved","part_of_speech":"verb","forms":["HAVE","HAS","HAD"],"verb_paradigm":{"classification":"lexical","source_sequence":["HAVE","HAS","HAD","HAD"],"base_form":"HAVE","simple_present_variants":["HAS"],"simple_past_variants":["HAD"],"past_participle":"HAD"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"OPEN","status":"approved","part_of_speech":"verb","forms":["OPEN","OPENS","OPENED"],"verb_paradigm":{"classification":"lexical","source_sequence":["OPEN","OPENS","OPENED","OPENED"],"base_form":"OPEN","simple_present_variants":["OPENS"],"simple_past_variants":["OPENED"],"past_participle":"OPENED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CLOSE","status":"approved","part_of_speech":"verb","forms":["CLOSE","CLOSES","CLOSED"],"verb_paradigm":{"classification":"lexical","source_sequence":["CLOSE","CLOSES","CLOSED","CLOSED"],"base_form":"CLOSE","simple_present_variants":["CLOSES"],"simple_past_variants":["CLOSED"],"past_participle":"CLOSED"},"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"CLOSED","status":"approved","part_of_speech":"adjective","forms":["CLOSED"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"TESTING","status":"approved","part_of_speech":"noun","forms":["TESTING"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"LEAKING","status":"approved","part_of_speech":"verb","forms":["LEAKING"],"senses":[],"alternatives":[],"restrictions":[]},
            {"lemma":"LEAKING","status":"approved","part_of_speech":"adjective","forms":["LEAKING"],"senses":[],"alternatives":[],"restrictions":[]}
          ]
        }"#,
    )
    .unwrap()
}
