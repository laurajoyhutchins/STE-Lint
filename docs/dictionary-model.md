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

The first slice does not yet perform full POS or sense disambiguation during linting.

## Explicit forms

`RuntimeLexicon::lookup_form` only resolves forms listed in the dataset. It does not invent morphology. This matters for controlled-language verbs where an otherwise normal English form can be disallowed.

The private Issue 9 compiler cleans formatting/help text out of source-listed verb forms and removes exact duplicates while preserving order. It does not synthesize an English conjugation that is absent from the authority record.

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

The first full private Issue 9 compilation is structural. It does not guess that raw source prose is already a machine-enforceable restriction. Semantic enrichment is a later gate and must retain the source-oriented evidence.

## Built-in data and project terminology

The built-in runtime data and `.ste/terms.json` have separate authority.

Built-in data represents the selected STE language release packaged with the tool. Project terminology represents domain-specific technical nouns and verbs established by a repository.

A `technical_noun` or `technical_verb` reference in dictionary alternatives does not automatically add that term to the built-in general dictionary or a project's glossary.

Unknown text does not mutate either source automatically.

## Current dataset

`crates/ste-data/data/test-lexicon.json` is deliberately small. Its metadata declares `scope: first_slice_test_lexicon` so no consumer can reasonably mistake it for the complete Issue 9 dictionary.

Normal lint execution reads this packaged data and does not require the ASD-STE100 PDF.

`tools/authority-ingest/build_runtime_lexicon.py` can compile the verified private Issue 9 authority into an enriched runtime document, but that populated source-derived artifact remains private during this gate and does not replace the embedded test lexicon.

## Maintenance boundary

Source ingestion, authority-to-runtime compilation, and normal runtime use are separate operations.

The public repository may contain the compiler, schema, synthetic fixtures, provenance contracts, counts, hashes, and tests. The populated Issue 9 runtime artifact should not be committed publicly without a separate redistribution basis.

A compiled structural artifact is evidence that the authority can be projected losslessly into the runtime contract. It is not evidence that all dictionary senses or all 53 writing rules are executable, and STE-Lint must not claim full Issue 9 compliance on that basis alone.
