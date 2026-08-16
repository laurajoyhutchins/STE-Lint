use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{
    AnalysisDocument, DocumentNodeId, DocumentNodeKind, DocumentRelationKind, LintContext,
    LintMode, Resolution,
};

#[test]
fn graph_reuses_sentence_paragraph_topic_and_entity_evidence() {
    let text = "Inspect the busway. It is ready.\n\nThe pump is ready.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = glossary();
    let context = LintContext::from_json(
        r#"{
          "topics": [{
            "start": 0,
            "end": 19,
            "topic": "inspection",
            "source": "project-topic-map"
          }],
          "semantic_orderings": [{
            "before": {"kind":"topic","start":0,"end":19},
            "after": {"kind":"paragraph","start":34,"end":52},
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
    let graph = analysis.document_graph();

    let paragraphs = graph
        .nodes
        .iter()
        .filter(|node| node.id.kind == DocumentNodeKind::Paragraph)
        .collect::<Vec<_>>();
    assert_eq!(paragraphs.len(), 2);
    assert_eq!((paragraphs[0].span.start, paragraphs[0].span.end), (0, 33));
    assert_eq!((paragraphs[1].span.start, paragraphs[1].span.end), (34, 52));

    let sentences = graph
        .nodes
        .iter()
        .filter(|node| node.id.kind == DocumentNodeKind::Sentence)
        .collect::<Vec<_>>();
    assert_eq!(sentences.len(), 3);

    let topics = graph
        .nodes
        .iter()
        .filter(|node| node.id.kind == DocumentNodeKind::Topic)
        .collect::<Vec<_>>();
    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].label.as_deref(), Some("inspection"));
    assert_eq!(topics[0].provenance.as_deref(), Some("project-topic-map"));

    let entities = graph
        .nodes
        .iter()
        .filter(|node| node.id.kind == DocumentNodeKind::EntityMention)
        .collect::<Vec<_>>();
    assert_eq!(entities.len(), 1);
    assert_eq!((entities[0].span.start, entities[0].span.end), (12, 18));

    assert!(graph.relations.iter().any(|relation| {
        relation.kind == DocumentRelationKind::Contains
            && relation.from == paragraphs[0].id
            && relation.to == sentences[0].id
    }));
    assert!(graph.relations.iter().any(|relation| {
        relation.kind == DocumentRelationKind::Precedes
            && relation.from == sentences[0].id
            && relation.to == sentences[1].id
    }));
    assert!(graph.relations.iter().any(|relation| {
        relation.kind == DocumentRelationKind::Precedes
            && relation.from == paragraphs[0].id
            && relation.to == paragraphs[1].id
    }));

    assert_eq!(graph.references.len(), 1);
    let Resolution::Resolved(target) = graph.references[0].target else {
        panic!("bounded pronoun reference should remain resolved in the graph");
    };
    assert_eq!(target.kind, DocumentNodeKind::EntityMention);
    assert_eq!(graph.references[0].source_sentence, sentences[1].id);

    assert_eq!(graph.semantic_orderings.len(), 1);
    assert_eq!(
        graph.semantic_orderings[0].before,
        Resolution::Resolved(topics[0].id)
    );
    assert_eq!(
        graph.semantic_orderings[0].after,
        Resolution::Resolved(paragraphs[1].id)
    );
    assert_eq!(
        graph.semantic_orderings[0].source,
        "project-information-order"
    );
}

#[test]
fn graph_preserves_ambiguous_reference_targets() {
    let text = "Inspect the busway and the pump. It is ready.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = two_entity_glossary();
    let analysis =
        AnalysisDocument::new(text, &lexicon, Some(&glossary), None, LintMode::Descriptive);

    let graph = analysis.document_graph();
    assert_eq!(graph.references.len(), 1);
    let Resolution::Ambiguous(targets) = &graph.references[0].target else {
        panic!("competing entity antecedents must remain ambiguous");
    };
    assert_eq!(targets.len(), 2);
    assert!(
        targets
            .iter()
            .all(|target| target.kind == DocumentNodeKind::EntityMention)
    );
}

#[test]
fn semantic_ordering_does_not_infer_missing_graph_identity() {
    let text = "The pump is ready.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "semantic_orderings": [{
            "before": {"kind":"entity_mention","start":0,"end":8},
            "after": {"kind":"sentence","start":0,"end":18},
            "source": "project-information-order"
          }]
        }"#,
    )
    .unwrap();
    context.validate(text.len()).unwrap();
    let analysis =
        AnalysisDocument::new(text, &lexicon, None, Some(&context), LintMode::Descriptive);

    let graph = analysis.document_graph();
    assert_eq!(graph.semantic_orderings.len(), 1);
    assert!(matches!(
        graph.semantic_orderings[0].before,
        Resolution::Unknown
    ));
    assert_eq!(
        graph.semantic_orderings[0].after,
        Resolution::Resolved(DocumentNodeId {
            kind: DocumentNodeKind::Sentence,
            index: 0,
        })
    );
}

fn glossary() -> Glossary {
    Glossary::from_json(
        r#"{
          "terms": [{
            "term":"busway",
            "kind":"technical_noun",
            "definition":"Synthetic governed term.",
            "domain":"electrical",
            "preferred":true,
            "aliases":[],
            "examples":[],
            "provenance":["fixture:busway"],
            "status":"approved"
          }]
        }"#,
    )
    .unwrap()
}

fn two_entity_glossary() -> Glossary {
    Glossary::from_json(
        r#"{
          "terms": [
            {
              "term":"busway",
              "kind":"technical_noun",
              "definition":"Synthetic governed electrical term.",
              "domain":"electrical",
              "preferred":true,
              "aliases":[],
              "examples":[],
              "provenance":["fixture:busway"],
              "status":"approved"
            },
            {
              "term":"pump",
              "kind":"technical_noun",
              "definition":"Synthetic governed mechanical term.",
              "domain":"mechanical",
              "preferred":true,
              "aliases":[],
              "examples":[],
              "provenance":["fixture:pump"],
              "status":"approved"
            }
          ]
        }"#,
    )
    .unwrap()
}
