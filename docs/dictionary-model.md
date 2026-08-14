# Runtime dictionary model

STE-Lint treats the controlled lexicon as structured language data, not as a synonym map.

## Entry identity

The intended long-term identity is a combination of:

- lemma;
- part of speech;
- approved sense;
- explicit permitted forms;
- context restrictions where applicable.

The first slice already preserves these fields in the data model, but it does not yet perform full POS or sense disambiguation during linting.

## Explicit forms

`RuntimeLexicon::lookup_form` only resolves forms listed in the dataset. It does not invent morphology. This matters for controlled-language verbs where an otherwise normal English form can be disallowed.

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

## Built-in data and project terminology

The built-in runtime data and `.ste/terms.json` have separate authority.

Built-in data represents the selected STE language release packaged with the tool. Project terminology represents domain-specific technical nouns and verbs established by a repository.

Unknown text does not mutate either source automatically.

## Current dataset

`crates/ste-data/data/test-lexicon.json` is deliberately small. Its metadata declares `scope: first_slice_test_lexicon` so no consumer can reasonably mistake it for the complete Issue 9 dictionary.

Normal lint execution reads this packaged data and does not require the ASD-STE100 PDF.

## Maintenance boundary

Future authority-ingest tooling can construct or verify a populated release dataset from authorized source material. The public runtime artifact should have a clear redistribution basis before source-derived dictionary content is committed.
