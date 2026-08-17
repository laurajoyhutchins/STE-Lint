from pathlib import Path

lib = Path("crates/ste-glossary/src/lib.rs")
text = lib.read_text()
text = text.replace(
    "    fn from_legacy(legacy: LegacyGlossary) -> Self {\n        let mut sources = BTreeMap::new();",
    "    fn from_legacy(legacy: LegacyGlossary) -> Self {\n        let domain = legacy\n            .terms\n            .first()\n            .map(|term| term.domain.clone())\n            .unwrap_or_else(|| \"legacy\".to_owned());\n        let mut sources = BTreeMap::new();",
)
text = text.replace(
    '        Self::compile(None, "legacy", sources, terms)\n',
    '        Self::compile(None, &domain, sources, terms)\n',
)
text = text.replace(
    '    #[allow(dead_code)]\n    domain: String,\n',
    '    domain: String,\n',
)
lib.write_text(text)

readme = Path("README.md")
text = readme.read_text()
start = text.index("## Repo-local technical terminology")
end = text.index("## Repo-local context evidence")
section = '''## Repo-local technical terminology

A repository can extend the effective lexicon with `.ste/terms.json` without changing built-in language data. New glossaries use the explicit `ste-terminology/v2` schema. The document declares its domain and source catalog once; each term carries a stable concept ID, canonical spelling, governed grammatical roles, explicit forms or aliases, source support, and lifecycle status.

```json
{
  "schema": "ste-terminology/v2",
  "domain": "electrical",
  "sources": {
    "project-spec": {
      "title": "Project terminology specification",
      "reviewed_on": "2026-08-17"
    }
  },
  "terms": [
    {
      "id": "busway",
      "canonical": "busway",
      "roles": ["noun"],
      "definition": "A project term for an electrical distribution assembly.",
      "forms": [
        {"text": "busways", "roles": ["noun"]}
      ],
      "aliases": [],
      "sources": [
        {
          "source": "project-spec",
          "supports": ["admission", "definition", "role", "forms", "status"]
        }
      ],
      "status": "approved"
    }
  ]
}
```

`id` is the stable concept identity and `canonical` is its preferred display spelling. `roles` is a set containing `noun`, `verb`, or both. `forms` are explicit governed spellings and retain the grammatical roles that each spelling can represent. STE-Lint does not stem terms or generate plurals, participles, conjugations, or other morphology.

Aliases are structured alternate identities with one of `abbreviation`, `acronym`, `short_form`, `synonym`, or `legacy`. Sources are structured references. A term source can explicitly support `admission`, `definition`, `role`, `forms`, `alias`, and `status`; do not claim support that the source does not establish.

`status` is `approved` or `deprecated`. There is no separate preferred flag. A deprecated term can name a stable `replacement` term ID when authority establishes the relationship. Examples are optional.

Reusable profiles use the same term schema. `schema: ste-terminology/v2` identifies the serialization contract, while `profile.version` identifies the vocabulary revision. Those versions are independent.

Terminology documents compile into one normalized runtime glossary index before linting. The compiled index owns canonical, alias, and form lookup plus maximum phrase width, and glossary matches retain how a spelling matched and which grammatical roles that spelling can represent. Composition remains fail closed: canonical duplicates retain `TERM-DUP-001`, and canonical/alias/form identity collisions produce `TERM-ID-CONFLICT-001`.

A bounded legacy `.ste/terms.json` reader remains available so existing repositories do not need an immediate migration, but legacy input is compiled into the same runtime model. Do not create new legacy-format glossaries.

Unknown words are not added automatically. `STE-TERM-001` means the term needs classification, not that it is necessarily wrong. A governed term with `status: deprecated` produces `STE-TERM-002`.

'''
readme.write_text(text[:start] + section + text[end:])
