# Verb paradigm design contract

Approved verb morphology is an authority projection, not generated English morphology.

- `forms` is the deduplicated lookup surface.
- `verb_paradigm.source_sequence` preserves source order and meaningful duplicate positions.
- `verb_paradigm.classification` distinguishes ordinary lexical verbs, the irregular auxiliary `BE`, and source-described defective modal auxiliaries.
- Role fields are populated only when supported by that source class and sequence.
- Unapproved verb records do not receive approved executable paradigms.
- Sentence grammar is not inferred from dictionary morphology alone.
- Rule checks that consume paradigms must block or abstain when a competing approved identity prevents a safe grammatical assertion.

This design keeps the private source-derived prose outside Git while retaining enough source-backed structure to implement bounded Rule 3 checks.
