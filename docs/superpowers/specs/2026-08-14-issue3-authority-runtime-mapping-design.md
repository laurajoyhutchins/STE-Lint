# Issue 9 authority-to-runtime mapping design

Date: 2026-08-14
Status: authorized by GitHub Issue #3 and the owner instruction to work the issue

## Goal

Define a deterministic maintenance compiler from the verified private ASD-STE100 Issue 9 authority bundle into STE-Lint runtime dictionary data without introducing a runtime PDF dependency, losing source semantics, or publishing the populated source-derived corpus in this public repository.

This design implements the first gate of GitHub Issue #3. It does not replace `crates/ste-data/data/test-lexicon.json` and does not claim full Issue 9 compliance.

## Authority boundary

The retained ASD-STE100 PDF remains the canonical private source object in Google Drive. `data/issue9-source.manifest.json` on `main` owns the expected source identity, verified structural counts, private bundle digest, and cardinality model.

The private authority bundle contains source-derived dictionary text and remains private. Git may contain:

- the runtime schema and typed model;
- the compiler that consumes an authorized private bundle;
- synthetic fixtures that contain no copied ASD dictionary prose;
- validation logic, tests, counts, hashes, and provenance contracts;
- documentation of the mapping boundary.

Git must not contain the populated Issue 9 runtime dictionary unless a separate redistribution authority permits publication.

Normal `ste` execution does not read the PDF or run the authority compiler.

## Chosen architecture

Extend the existing `ste-data` runtime model and add one maintenance compiler under `tools/authority-ingest/`.

Do not introduce a second authority crate or an intermediate public dataset. The compiler consumes the existing private `source.json`, `manifest.json`, and `dictionary.json` products and emits the JSON contract already owned by `ste-data`, enriched with the provenance and source-semantic fields required by the approved STE-Lint design.

The pipeline is:

```text
verified retained PDF
        |
        v
private authority ingest
        |
        +-- source.json
        +-- manifest.json
        `-- dictionary.json
                |
                v
build_runtime_lexicon.py
                |
                v
private runtime-lexicon.json
                |
                v
RuntimeLexicon::from_json
```

The current embedded test lexicon remains the public executable fixture until the private runtime artifact has a lawful publication/package path and the remaining semantic mappings are complete.

## Runtime model changes

### Document provenance

`LexiconMetadata` gains optional runtime provenance so the small public test lexicon remains valid while a compiled Issue 9 document carries the exact source coordinate.

The provenance model records:

- standard and issue;
- publication date;
- retained Drive object ID;
- retained-source SHA-256;
- retained-source byte size;
- physical page count;
- private authority bundle SHA-256.

### Separate cardinalities

The runtime metadata can record both count bases:

- source-declared approved-word count;
- source-declared unapproved-word count;
- structural approved-headword-record count;
- structural unapproved-headword-record count.

These measures are never asserted equal. The compiler validates each against the verified public manifest independently.

### Entry provenance

Each compiled dictionary entry records:

- zero-based structural record index from the private `dictionary.json` sequence;
- physical source pages retained by the authority ingest.

This is maintenance provenance, not an ASD rule citation.

### Part of speech

`LexiconEntry.part_of_speech` becomes optional because Issue 9 contains dictionary expression records such as `FOR EXAMPLE` and `such as` for which the source does not supply a part-of-speech marker. STE-Lint must preserve that absence instead of inventing a grammatical class.

Existing callers that only display the field continue to work. Future POS enforcement must require an explicit part of speech when the applicable rule requires one.

### Forms

The compiler emits an explicit `forms` array for every record.

- For non-verbs and source expressions, the lemma itself is an explicit form.
- For verbs, source-listed forms are normalized, source help such as `No other verb forms.` is removed from the form token, parenthetical `also` syntax is normalized, and duplicate forms are removed while preserving source order.
- The compiler never invents ordinary English morphology.

This keeps the existing `RuntimeLexicon::lookup_form` invariant: a form is usable only when it is explicitly present in runtime data.

### Source semantics and interpretation state

The current authority ingest intentionally retains the four source columns as text. Those fields can contain multiple approved senses, multiple alternatives, help, restrictions, technical-term references, and sentence-reconstruction guidance in layouts that are not yet fully semantically classified.

The compiler therefore preserves a private lossless `source_semantics` object on each compiled entry:

- source word cell;
- source approved-meaning/alternatives cell;
- STE example cell;
- non-STE example cell.

This prevents semantic loss while avoiding fabricated structure.

Each entry also carries `interpretation_state`:

- `structural` when the record is losslessly mapped but the source semantic cell still requires interpretation before sense/restriction enforcement;
- `interpreted` only when the structured `senses`, `alternatives`, `restrictions`, and help fields fully represent the source semantics required by the runtime checks that consume them.

The first compiler emits `structural` for source records unless a mapping is mechanically exact. A future semantic-enrichment gate may promote records to `interpreted`; it must not delete the source-semantic evidence.

This is the key safety boundary: source prose is preserved privately, but STE-Lint does not pretend an unparsed help paragraph is already an executable restriction.

### Existing interpreted fields

The existing fields remain authoritative for executable semantics when populated:

- `senses` for approved meanings;
- `alternatives` for approved words, approved phrases, technical nouns, technical verbs, and no-direct-alternative cases;
- `Alternative.strategy` for word replacement, phrase replacement, or sentence reconstruction;
- `restrictions` for interpreted usage restrictions.

The first gate proves that the compiler can carry these fields without loss using synthetic fixtures. It does not require guessing a complete semantic parse for every one of the 2,196 private records.

## Compiler contract

Create `tools/authority-ingest/build_runtime_lexicon.py` with a small library surface and CLI.

Inputs:

```text
--authority-dir <private directory containing source.json, manifest.json, dictionary.json>
--verified-manifest data/issue9-source.manifest.json
--out <runtime-lexicon.json>
```

The compiler must:

1. verify source SHA-256, byte size, physical page count, issue, and publication date against the public verified manifest;
2. verify both source-declared and structural count bases independently;
3. reject an authority directory whose dictionary record counts differ from the verified structural counts;
4. map POS abbreviations without inventing a POS for source expressions;
5. normalize explicit forms deterministically without inventing morphology;
6. retain source-semantic cells and source pages for every record;
7. preserve the input record order;
8. emit UTF-8 JSON deterministically;
9. write no PDF-derived output into the repository by default.

The script is maintenance tooling. It is not linked into the `ste` runtime.

## Synthetic fixture

Add a small synthetic authority fixture under `fixtures/authority-ingest/` containing invented entries that exercise the contract without copying ASD source prose:

- an approved verb with explicit irregular forms and a no-other-forms note;
- an unapproved adjective with an approved-word alternative and a word-replacement strategy;
- an unapproved noun whose alternative is a technical noun;
- a phrase/expression record with no POS;
- an entry with source help/restriction text that remains `structural`.

The fixture source identity is synthetic and is tested only through compiler unit helpers. Full end-to-end source-identity validation uses generated temporary fixture manifests, not the real private corpus.

## Testing

Use test-driven development.

Rust tests cover:

- backward-compatible parsing of the existing embedded test lexicon;
- optional POS for expression records;
- provenance and count metadata round-trip;
- lossless round-trip of source semantics and interpretation state;
- explicit-form lookup remains non-generative.

Python tests cover:

- source identity mismatch rejection;
- cardinality mismatch rejection;
- POS abbreviation mapping;
- verb-form cleanup and order-preserving deduplication;
- expression records with no POS;
- deterministic output across repeated builds;
- source-semantic and page-provenance preservation.

CI runs the Python tests without the private PDF or private bundle. Existing Rust format, Clippy, and test gates remain unchanged.

A private verification run against the retained authority bundle may additionally produce a runtime artifact and derivative hash, but that artifact is stored privately rather than committed.

## Non-goals of this gate

- replacing `test-lexicon.json`;
- implementing all 53 writing rules;
- full sense disambiguation;
- full parsing of every dictionary help paragraph;
- publishing the complete Issue 9 dictionary;
- changing agent prompts to carry the dictionary;
- making the runtime fetch Drive or the PDF.

## Exit condition

This gate is complete when a coherent pull request contains the enriched runtime contract, deterministic authority-to-runtime compiler, synthetic regression fixtures, and green CI, and a private run against the verified Issue 9 authority bundle produces a deterministic runtime artifact whose provenance points back to the exact retained source and private authority bundle.
