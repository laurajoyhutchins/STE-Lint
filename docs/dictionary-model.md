# Runtime dictionary model

STE-Lint treats the controlled lexicon as structured language data, not as a synonym map.

## Entry identity

The intended long-term identity is a combination of:

- lemma;
- part of speech when the source supplies one;
- approved sense;
- explicit permitted forms;
- context restrictions where applicable.

The runtime model can represent source expression records that have no part-of-speech marker. It preserves that absence instead of inventing a grammatical class.

The current linter does not yet perform full POS or sense disambiguation.

## Explicit forms

`RuntimeLexicon` only indexes forms listed in the dataset. It does not invent morphology. This matters for controlled-language verbs where an otherwise normal English form can be disallowed.

A spelling is not necessarily a unique dictionary identity. The full Issue 9 structural projection contains spellings shared by multiple records, including records with different parts of speech or approval states. `lookup_form_candidates` therefore returns every structural candidate for an explicit form in source-record order. The compatibility `lookup_form` API returns one entry only when the spelling has exactly one candidate; it returns `None` for an ambiguous spelling instead of silently selecting an arbitrary record.

The private Issue 9 compiler recovers source-listed verb forms from the source word-cell line structure. It removes exact duplicates only for the general lookup `forms` array. It does not synthesize an English conjugation that is absent from the authority record.

## Approved verb paradigms

Approved verb entries can additionally carry a `verb_paradigm`. This is executable morphology derived only from source-listed forms. Unapproved verb entries do not receive an approved paradigm.

The paradigm preserves both interpretation and evidence:

- `classification`: `lexical`, `irregular_auxiliary`, or `defective_modal`;
- `source_sequence`: the ordered source-listed forms, including meaningful duplicate positions;
- `base_form`;
- `simple_present_variants`;
- `simple_past_variants`;
- `past_participle` when the source supplies one.

For ordinary lexical verbs, the source-defined form positions map to the base/imperative, simple-present, simple-past, and past-participle roles. The compiler does not force that positional interpretation onto exceptional auxiliaries. `BE` is represented as an irregular auxiliary with its listed present and past variants. Source-described auxiliary modal verbs are represented as defective modals and retain their source sequence without inventing missing participles or other forms.

This distinction matters for Rule 3 checks. A flat `forms` list can establish that a spelling is listed; the paradigm can additionally establish which listed spelling is an approved past participle or tense form. Sentence-level grammatical use is still a separate problem and is not inferred solely from dictionary morphology.

## Approval states

Entries are either `approved` or `unapproved`.

An approved entry can have one or more senses. An unapproved entry can provide alternatives. Alternatives preserve more structure than `bad_word -> good_word`:

- `approved_word`;
- `approved_phrase`;
- `technical_noun`;
- `technical_verb`;
- `no_direct_alternative`.

Each alternative also records a repair strategy:

- `word_replacement`;
- `phrase_replacement`;
- `sentence_reconstruction`.

This prevents the runtime from pretending that every dictionary suggestion is a semantics-free token substitution.

## Provenance and cardinality

A compiled authority-backed runtime document can carry exact retained-source provenance:

- Drive object ID;
- retained-source SHA-256 and byte size;
- physical page count;
- parent private-authority bundle SHA-256.

Each entry can carry its zero-based structural record index and physical source pages.

The document also carries two independent dictionary count bases when built from Issue 9 authority:

- source-declared approved and unapproved word counts;
- structural approved and unapproved headword-record counts.

These measures are not expected to be equal. They are verified independently.

The small public test lexicon omits this optional authority metadata and remains backward compatible.

## Structural versus interpreted semantics

The private authority ingest retains four source-oriented dictionary cells for every record: the word cell, approved-meaning/alternatives cell, STE example cell, and non-STE example cell.

The runtime contract can preserve those cells as `source_semantics` alongside structured executable fields. This prevents a maintenance compiler from throwing away evidence merely because a source help paragraph is not yet fully interpreted.

`interpretation_state` distinguishes two states:

- `structural`: source semantics and provenance are preserved, but they are not yet fully classified into executable senses, alternatives, or restrictions;
- `interpreted`: the structured fields are intended to represent the source semantics required by runtime checks that consume them.

The private Issue 9 compilation remains structural for general dictionary meaning/help semantics. The approved `verb_paradigm` is a narrower interpreted projection of source-listed morphology and does not imply that every sense, restriction, or help paragraph is executable.

## Built-in data and project terminology

The runtime dictionary and `.ste/terms.json` have separate authority.

The verified private runtime represents the selected STE language release. Project terminology represents domain-specific technical nouns and verbs established by a repository.

A `technical_noun` or `technical_verb` reference in dictionary alternatives does not automatically add that term to the built-in general dictionary or a project's glossary.

Unknown text does not mutate either source automatically.

## Current dataset

`crates/ste-data/data/test-lexicon.json` is deliberately small. Its metadata declares `scope: first_slice_test_lexicon` so no consumer can reasonably mistake it for the complete Issue 9 dictionary. Lint and dictionary commands require a verified private runtime unless the caller explicitly opts into this test fixture.

`tools/authority-ingest/build_runtime_lexicon.py` compiles the verified private Issue 9 authority into the runtime document. The populated source-derived artifact remains private; Git stores only its identity contract and non-restricted executable/schema material.

Normal runtime use does not require or read the ASD-STE100 PDF.

## Maintenance boundary

Source ingestion, authority-to-runtime compilation, and normal runtime use are separate operations.

The public repository may contain the compiler, schema, synthetic fixtures, provenance contracts, counts, hashes, and tests. The populated Issue 9 runtime artifact must not be committed publicly without a separate redistribution basis.

A verified compiled artifact is evidence that the selected authority projection matches the runtime contract. It is not evidence that all dictionary senses or all 53 writing rules are executable, and STE-Lint must not claim full Issue 9 compliance on that basis alone.
