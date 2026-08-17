use ste_glossary::Glossary;

#[test]
fn legacy_glossary_domain_is_preserved_in_compiled_terms() {
    let glossary = Glossary::from_json(
        r#"{
          "terms": [{
            "term": "busway",
            "kind": "technical_noun",
            "definition": "An electrical distribution assembly.",
            "domain": "electrical",
            "preferred": true,
            "forms": ["busways"],
            "aliases": [],
            "examples": [],
            "provenance": ["project authority"],
            "status": "approved"
          }]
        }"#,
    )
    .unwrap();

    let term = glossary.lookup_term("busways").unwrap();
    assert_eq!(term.domain, "electrical");
    assert_eq!(term.canonical, "busway");
}
