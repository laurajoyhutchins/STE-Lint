use ste_glossary::Glossary;

#[test]
fn explicit_forms_resolve_to_the_governed_term() {
    let glossary = Glossary::from_json(
        r#"{
          "terms": [{
            "term": "repository",
            "kind": "technical_noun",
            "definition": "A governed software repository.",
            "domain": "software-core",
            "preferred": true,
            "forms": ["repositories"],
            "aliases": ["repo"],
            "examples": [],
            "provenance": ["test authority"],
            "status": "approved"
          }]
        }"#,
    )
    .unwrap();

    assert_eq!(
        glossary.lookup_term("repositories").unwrap().term,
        "repository"
    );
}

#[test]
fn alias_collision_with_another_canonical_term_is_rejected() {
    let glossary = Glossary::from_json(
        r#"{
          "terms": [
            {
              "term": "repository",
              "kind": "technical_noun",
              "definition": "A repository.",
              "domain": "software-core",
              "preferred": true,
              "aliases": ["repo"],
              "examples": [],
              "provenance": ["test authority"],
              "status": "approved"
            },
            {
              "term": "repo",
              "kind": "technical_noun",
              "definition": "A conflicting canonical identity.",
              "domain": "project",
              "preferred": true,
              "aliases": [],
              "examples": [],
              "provenance": ["test authority"],
              "status": "approved"
            }
          ]
        }"#,
    )
    .unwrap();

    let diagnostics = glossary.validate();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "TERM-ID-CONFLICT-001")
    );
}

#[test]
fn form_collision_with_another_alias_is_rejected() {
    let glossary = Glossary::from_json(
        r#"{
          "terms": [
            {
              "term": "repository",
              "kind": "technical_noun",
              "definition": "A repository.",
              "domain": "software-core",
              "preferred": true,
              "forms": ["repositories"],
              "aliases": [],
              "examples": [],
              "provenance": ["test authority"],
              "status": "approved"
            },
            {
              "term": "repository collection",
              "kind": "technical_noun",
              "definition": "A synthetic conflicting term.",
              "domain": "project",
              "preferred": true,
              "aliases": ["repositories"],
              "examples": [],
              "provenance": ["test authority"],
              "status": "approved"
            }
          ]
        }"#,
    )
    .unwrap();

    let diagnostics = glossary.validate();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "TERM-ID-CONFLICT-001")
    );
}
