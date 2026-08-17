use ste_glossary::{AliasKind, Glossary, GlossaryIdentityKind, TermRole};

#[test]
fn compiled_index_preserves_canonical_form_and_alias_evidence() {
    let glossary = Glossary::from_json(
        r#"{
          "schema": "ste-terminology/v2",
          "domain": "software-core",
          "sources": {
            "project": {"title": "Project terminology authority"}
          },
          "terms": [{
            "id": "repository",
            "canonical": "repository",
            "roles": ["noun"],
            "definition": "A governed software repository.",
            "forms": [{"text": "repositories", "roles": ["noun"]}],
            "aliases": [{"text": "repo", "kind": "short_form"}],
            "sources": [{"source": "project", "supports": ["admission", "definition", "role", "forms", "alias", "status"]}],
            "status": "approved"
          }]
        }"#,
    )
    .unwrap();

    let canonical = glossary.lookup_identity("repository").unwrap();
    assert_eq!(canonical.identity_kind, GlossaryIdentityKind::Canonical);
    assert_eq!(canonical.roles, &[TermRole::Noun]);

    let form = glossary.lookup_identity("repositories").unwrap();
    assert_eq!(form.identity_kind, GlossaryIdentityKind::Form);
    assert_eq!(form.roles, &[TermRole::Noun]);

    let alias = glossary.lookup_identity("repo").unwrap();
    assert_eq!(alias.identity_kind, GlossaryIdentityKind::Alias);
    assert_eq!(alias.alias_kind, Some(AliasKind::ShortForm));
    assert_eq!(alias.term.id, "repository");
    assert_eq!(
        alias.term.source_catalog["project"].title,
        "Project terminology authority"
    );
}
