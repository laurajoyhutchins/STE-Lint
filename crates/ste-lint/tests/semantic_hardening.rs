use ste_data::{PartOfSpeech, RuntimeLexicon};
use ste_glossary::Glossary;
use ste_lint::{
    AnalysisDocument, DocumentNodeKind, LintContext, LintMode, Resolution, SafetyEvidenceSource,
};

#[test]
fn resolved_noun_phrase_and_sense_keep_the_same_head_token_span() {
    let lexicon = lexicon();
    let analysis = AnalysisDocument::new(
        "THE PUMP IS READY.",
        &lexicon,
        None,
        None,
        LintMode::Descriptive,
    );

    let Resolution::Resolved(noun_phrase) = analysis.noun_phrase_at(0) else {
        panic!("bounded noun phrase should resolve");
    };
    let Resolution::Resolved(sense) = analysis.sense_resolution_at(noun_phrase.head_token, 1)
    else {
        panic!("resolved noun identity should expose one source-safe sense");
    };
    let head = &analysis.tokens()[noun_phrase.head_token];

    assert_eq!(sense.identity.part_of_speech, Some(PartOfSpeech::Noun));
    assert_eq!((sense.span.start, sense.span.end), (head.start, head.end));
}

#[test]
fn overlapping_entity_authority_stays_ambiguous_in_document_ordering() {
    let text = "PUMP.";
    let lexicon = lexicon();
    let glossary = glossary();
    let context = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 4,
            "source": "official-name-register",
            "official_technical_name": true
          }],
          "semantic_orderings": [{
            "before": {"kind":"entity_mention","start":0,"end":4},
            "after": {"kind":"sentence","start":0,"end":5},
            "source": "project-information-order"
          }]
        }"#,
    )
    .unwrap();
    context.validate(text.len()).unwrap();
    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        Some(&glossary),
        Some(&context),
        LintMode::Descriptive,
    );

    let Resolution::Ambiguous(mentions) = analysis.entity_mention_at(0) else {
        panic!("governed and official identities on one span must remain ambiguous");
    };
    assert_eq!(mentions.len(), 2);

    let graph = analysis.document_graph();
    assert_eq!(
        graph
            .nodes
            .iter()
            .filter(|node| node.id.kind == DocumentNodeKind::EntityMention)
            .count(),
        2
    );
    let Resolution::Ambiguous(targets) = &graph.semantic_orderings[0].before else {
        panic!("graph ordering must preserve same-span entity ambiguity");
    };
    assert_eq!(targets.len(), 2);
    assert!(
        targets
            .iter()
            .all(|target| target.kind == DocumentNodeKind::EntityMention)
    );
}

#[test]
fn safety_command_and_actor_spans_align_with_dictionary_and_entity_evidence() {
    let text = "WARNING: DISCONNECT PUMP.";
    let lexicon = lexicon();
    let glossary = glossary();
    let context = LintContext::from_json(
        r#"{
          "safety_facts": [{
            "start": 0,
            "end": 25,
            "source": "project-hazard-analysis",
            "actor": {"start":20,"end":24}
          }]
        }"#,
    )
    .unwrap();
    context.validate(text.len()).unwrap();
    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        Some(&glossary),
        Some(&context),
        LintMode::Procedural,
    );

    let safety = &analysis.safety_semantics()[0];
    let Resolution::Resolved(command) = &safety.command else {
        panic!("source-backed safety command should resolve");
    };
    let dictionary = analysis
        .longest_dictionary_match_at(1)
        .expect("command must retain dictionary evidence");
    assert_eq!(
        (command.span.start, command.span.end),
        (dictionary.start, dictionary.end)
    );
    assert_eq!(command.source, SafetyEvidenceSource::Structure);

    let Resolution::Resolved(actor) = &safety.actor else {
        panic!("explicit safety actor should resolve");
    };
    let Resolution::Resolved(entity) = analysis.entity_mention_at(2) else {
        panic!("governed actor span should retain entity identity");
    };
    assert_eq!(
        (actor.span.start, actor.span.end),
        (entity.span.start, entity.span.end)
    );
}

#[test]
fn unresolved_sense_does_not_fabricate_entity_or_graph_identity() {
    let lexicon = lexicon();
    let analysis = AnalysisDocument::new("TEST.", &lexicon, None, None, LintMode::Descriptive);

    assert!(matches!(
        analysis.sense_resolution_at(0, 1),
        Resolution::Ambiguous(_)
    ));
    assert!(matches!(analysis.entity_mention_at(0), Resolution::Unknown));
    assert!(
        analysis
            .document_graph()
            .nodes
            .iter()
            .all(|node| node.id.kind != DocumentNodeKind::EntityMention)
    );
}

fn glossary() -> Glossary {
    Glossary::from_json(
        r#"{
          "terms": [{
            "term":"PUMP",
            "kind":"technical_noun",
            "definition":"Synthetic governed component.",
            "domain":"mechanical",
            "preferred":true,
            "aliases":[],
            "examples":[],
            "provenance":["fixture:pump"],
            "status":"approved"
          }]
        }"#,
    )
    .unwrap()
}

fn lexicon() -> RuntimeLexicon {
    RuntimeLexicon::from_json(
        r#"{
          "metadata": {
            "standard":"ASD-STE100",
            "issue":9,
            "date":"2025-01-15",
            "scope":"synthetic_semantic_hardening"
          },
          "entries": [
            {
              "lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],
              "senses":[],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"TEST","status":"approved","part_of_speech":"noun","forms":["TEST"],
              "senses":[{"meaning":"synthetic noun sense"}],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"TEST","status":"approved","part_of_speech":"verb","forms":["TEST"],
              "senses":[{"meaning":"synthetic verb sense"}],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"BE","status":"approved","part_of_speech":"verb","forms":["BE","IS"],
              "verb_paradigm":{"classification":"irregular_auxiliary","source_sequence":["BE","IS","WAS","BEEN"],"base_form":"BE","simple_present_variants":["IS"],"simple_past_variants":["WAS"],"past_participle":"BEEN"},
              "senses":[],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"READY","status":"approved","part_of_speech":"adjective","forms":["READY"],
              "senses":[],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"DISCONNECT","status":"approved","part_of_speech":"verb","forms":["DISCONNECT","DISCONNECTS","DISCONNECTED"],
              "verb_paradigm":{"classification":"lexical","source_sequence":["DISCONNECT","DISCONNECTS","DISCONNECTED","DISCONNECTED"],"base_form":"DISCONNECT","simple_present_variants":["DISCONNECTS"],"simple_past_variants":["DISCONNECTED"],"past_participle":"DISCONNECTED"},
              "senses":[],"alternatives":[],"restrictions":[]
            },
            {
              "lemma":"PUMP","status":"approved","part_of_speech":"noun","forms":["PUMP"],
              "senses":[{"meaning":"synthetic pump sense"}],"alternatives":[],"restrictions":[]
            }
          ]
        }"#,
    )
    .unwrap()
}
