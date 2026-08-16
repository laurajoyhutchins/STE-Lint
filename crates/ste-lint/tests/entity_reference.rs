use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{
    AnalysisDocument, EntityIdentity, EntityMentionKind, LintContext, LintMode, ReferenceBasis,
    Resolution,
};

#[test]
fn governed_aliases_share_one_stable_entity_identity() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = entity_glossary();
    let analysis = AnalysisDocument::new(
        "Inspect the bus duct. It is ready.",
        &lexicon,
        Some(&glossary),
        None,
        LintMode::Descriptive,
    );

    let Resolution::Resolved(mention) = analysis.entity_mention_at(2) else {
        panic!("governed alias should resolve to its canonical entity identity");
    };
    assert_eq!(mention.kind, EntityMentionKind::GovernedTechnicalTerm);
    assert_eq!(
        mention.identity,
        EntityIdentity::GovernedTerm {
            term: "busway".into(),
            domain: "electrical".into(),
        }
    );
    assert_eq!((mention.span.start, mention.span.end), (12, 20));
    assert_eq!(mention.surface, "bus duct");
    assert!(mention.definition.is_some());
    assert_eq!(mention.provenance, vec!["fixture:busway"]);
}

#[test]
fn official_technical_name_context_is_a_valid_stable_entity_source() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 12,
            "source": "official-name-register",
            "official_technical_name": true
          }]
        }"#,
    )
    .unwrap();
    context
        .validate("RAVEN MODULE is installed. It is ready.".len())
        .unwrap();

    let analysis = AnalysisDocument::new(
        "RAVEN MODULE is installed. It is ready.",
        &lexicon,
        None,
        Some(&context),
        LintMode::Descriptive,
    );
    let Resolution::Resolved(mention) = analysis.entity_mention_at(0) else {
        panic!("official technical-name evidence should establish a stable entity identity");
    };
    assert_eq!(mention.kind, EntityMentionKind::OfficialTechnicalName);
    assert_eq!(
        mention.identity,
        EntityIdentity::OfficialTechnicalName {
            normalized: "raven module".into(),
        }
    );
    assert_eq!((mention.span.start, mention.span.end), (0, 12));
    assert_eq!(mention.provenance, vec!["official-name-register"]);
}

#[test]
fn overlapping_authorities_remain_explicitly_ambiguous() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = entity_glossary();
    let context = LintContext::from_json(
        r#"{
          "occurrences": [{
            "start": 0,
            "end": 6,
            "source": "official-name-register",
            "official_technical_name": true
          }]
        }"#,
    )
    .unwrap();
    let analysis = AnalysisDocument::new(
        "busway",
        &lexicon,
        Some(&glossary),
        Some(&context),
        LintMode::Descriptive,
    );

    let Resolution::Ambiguous(candidates) = analysis.entity_mention_at(0) else {
        panic!("distinct authority-backed identities on one span must not be silently collapsed");
    };
    assert_eq!(candidates.len(), 2);
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.kind == EntityMentionKind::GovernedTechnicalTerm)
    );
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.kind == EntityMentionKind::OfficialTechnicalName)
    );
    assert_eq!(analysis.entity_mentions().len(), 2);
}

#[test]
fn repeated_official_name_mentions_keep_distinct_spans() {
    let text = "RAVEN MODULE. RAVEN MODULE.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "occurrences": [
            {
              "start": 0,
              "end": 12,
              "source": "official-name-register",
              "official_technical_name": true
            },
            {
              "start": 14,
              "end": 26,
              "source": "official-name-register",
              "official_technical_name": true
            }
          ]
        }"#,
    )
    .unwrap();
    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        None,
        Some(&context),
        LintMode::Descriptive,
    );

    let mentions = analysis.entity_mentions();
    assert_eq!(mentions.len(), 2);
    assert_eq!(
        mentions
            .iter()
            .map(|mention| (mention.span.start, mention.span.end))
            .collect::<Vec<_>>(),
        vec![(0, 12), (14, 26)]
    );
    assert_eq!(mentions[0].identity, mentions[1].identity);
}

#[test]
fn singular_reference_resolves_to_unique_governed_entity_in_previous_sentence() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = entity_glossary();
    let analysis = AnalysisDocument::new(
        "Inspect the busway. It is ready.",
        &lexicon,
        Some(&glossary),
        None,
        LintMode::Descriptive,
    );

    let Resolution::Resolved(reference) = analysis.reference_at(3) else {
        panic!("a singular reference with one bounded antecedent should resolve");
    };
    assert_eq!(
        reference.basis,
        ReferenceBasis::PreviousSentenceUniqueEntity
    );
    assert_eq!(
        reference.antecedent.identity,
        EntityIdentity::GovernedTerm {
            term: "busway".into(),
            domain: "electrical".into(),
        }
    );
    assert_eq!(
        (reference.reference.start, reference.reference.end),
        (20, 22)
    );
}

#[test]
fn singular_reference_is_ambiguous_when_two_distinct_entities_compete() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let glossary = entity_glossary();
    let analysis = AnalysisDocument::new(
        "Inspect the busway and the pump. It is ready.",
        &lexicon,
        Some(&glossary),
        None,
        LintMode::Descriptive,
    );

    let Resolution::Ambiguous(candidates) = analysis.reference_at(6) else {
        panic!("two prior entity identities must remain ambiguous");
    };
    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| &candidate.antecedent.identity)
            .collect::<Vec<_>>(),
        vec![
            &EntityIdentity::GovernedTerm {
                term: "busway".into(),
                domain: "electrical".into(),
            },
            &EntityIdentity::GovernedTerm {
                term: "pump".into(),
                domain: "mechanical".into(),
            },
        ]
    );
}

#[test]
fn singular_reference_without_bounded_entity_evidence_is_unknown() {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let analysis =
        AnalysisDocument::new("It is ready.", &lexicon, None, None, LintMode::Descriptive);

    assert!(matches!(analysis.reference_at(0), Resolution::Unknown));
}

fn entity_glossary() -> Glossary {
    Glossary::from_json(
        r#"{
          "terms": [
            {
              "term": "busway",
              "kind": "technical_noun",
              "definition": "Synthetic governed electrical term.",
              "domain": "electrical",
              "preferred": true,
              "aliases": ["bus duct"],
              "examples": [],
              "provenance": ["fixture:busway"],
              "status": "approved"
            },
            {
              "term": "pump",
              "kind": "technical_noun",
              "definition": "Synthetic governed mechanical term.",
              "domain": "mechanical",
              "preferred": true,
              "aliases": [],
              "examples": [],
              "provenance": ["fixture:pump"],
              "status": "approved"
            }
          ]
        }"#,
    )
    .unwrap()
}
