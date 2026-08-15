# Issue 9 authority-to-runtime mapping design

Date: 2026-08-14
Status: implemented candidate for GitHub Issue #3 first gate

## Goal

Compile the verified private ASD-STE100 Issue 9 dictionary authority into the existing STE-Lint runtime dictionary contract without introducing a runtime PDF dependency, losing source semantics, inventing linguistic facts, or publishing the populated source-derived corpus.

This gate does not replace `crates/ste-data/data/test-lexicon.json` and does not claim full Issue 9 compliance.

## Authority boundary

The retained PDF in Google Drive remains the canonical source object. `data/issue9-source.manifest.json` records the expected retained-source identity, structural counts, source-declared counts, and private authority-bundle digest.

The private authority bundle contains source-derived text and remains private. The public repository may contain the runtime types/schema, compiler, synthetic fixtures, validation logic, counts, hashes, provenance contracts, and tests. It must not contain the populated Issue 9 runtime lexicon without a separate redistribution basis.

Normal `ste` execution does not read the PDF and does not run the authority compiler.

## Architecture

Extend `ste-data`; do not add a parallel runtime model or public intermediate dataset.

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

`tools/authority-ingest/build_runtime_lexicon.py` consumes:

```text
--authority-dir <private directory containing source.json, manifest.json, dictionary.json>
--verified-manifest data/issue9-source.manifest.json
--private-bundle-sha256 <exact parent private bundle SHA-256>
--out <private runtime-lexicon.json>
```

The bundle hash is explicit because an extracted directory cannot reconstruct the identity of the ZIP that contained it. The compiler verifies that supplied coordinate against Git's retained-source manifest.

## Runtime document contract

`LexiconMetadata` can carry optional `AuthorityProvenance`:

- Drive object ID;
- retained-source SHA-256;
- retained-source byte size;
- physical page count;
- parent private-authority bundle SHA-256.

It can also carry `DictionaryCardinalities` with four independent values:

- source-declared approved words;
- source-declared unapproved words;
- structural approved headword records;
- structural unapproved headword records.

The source-declared and structural measures are verified independently and are never asserted equal.

Each compiled entry can carry `EntryProvenance` with its zero-based structural record index and physical source pages.

## Part of speech and expressions

`LexiconEntry.part_of_speech` is optional. Issue 9 contains expression records whose source cells do not supply a POS marker. STE-Lint preserves that absence instead of inventing one.

## Explicit forms

The compiler emits explicit forms only. It cleans source formatting/help residue such as `No other verb forms.`, parenthetical `also` notation, and exact duplicates while preserving source order. It never synthesizes ordinary English morphology.

If a record has no separate source-listed forms, its headword is its explicit form.

## Ambiguous spellings

A normalized spelling is not a dictionary identity. The real Issue 9 structural projection contains many spellings shared by multiple records, including records with different POS or approval status.

`RuntimeLexicon` therefore indexes each form to all structural candidate records. `lookup_form_candidates` returns every candidate in source-record order. The compatibility `lookup_form` API returns an entry only when exactly one candidate exists; it returns `None` when the spelling is ambiguous instead of silently selecting one record.

This preserves structural distinctions until later POS/sense disambiguation can resolve them.

## Source semantics and interpretation state

The authority ingest retains four source-oriented dictionary cells for every record:

- word cell;
- approved-meaning/alternatives cell;
- STE example cell;
- non-STE example cell.

The compiler preserves those cells in private `source_semantics` and preserves source-page provenance. It also carries structured `senses`, `alternatives`, and `restrictions` when they are already explicitly interpreted.

Each entry has an `interpretation_state`:

- `structural`: source semantics are preserved but not fully classified into executable semantics;
- `interpreted`: structured fields are intended to represent the source semantics required by runtime checks that consume them.

The first full private Issue 9 compilation is structural. Raw help prose is not guessed into an executable restriction merely to increase apparent coverage.

Existing public test-lexicon records default to `interpreted` for backward compatibility.

## Compiler verification

Before emitting output, the compiler verifies:

- issue and publication date;
- retained Drive file ID;
- source SHA-256 and byte size;
- physical page count;
- explicit parent private-bundle SHA-256;
- private-manifest source identity;
- dictionary entry count;
- structural approved and unapproved record counts;
- source-declared approved and unapproved word counts.

It derives structural approved/unapproved counts from `dictionary.json`. It does not compare source-declared word counts to structural record counts.

Output preserves input record order and is deterministic UTF-8 JSON with a terminal newline, written atomically.

## Public test boundary

Public tests use invented synthetic authority records. They cover:

- source identity mismatch rejection;
- cardinality mismatch rejection;
- exact POS mapping and POS absence;
- explicit verb-form cleanup and order-preserving deduplication;
- technical-noun alternative preservation;
- structural versus interpreted state;
- source-semantic/page-provenance preservation;
- deterministic output;
- backward-compatible public test-lexicon parsing;
- ambiguous form candidate preservation.

CI does not require the private PDF or private authority bundle.

## Non-goals of this gate

- publishing or embedding the populated Issue 9 runtime lexicon;
- replacing the current test lexicon;
- full dictionary sense/help/restriction interpretation;
- POS or sense disambiguation in the linter;
- implementing all 53 writing rules;
- changing agent prompts to carry the dictionary;
- making runtime use depend on Drive or the PDF.

## Exit condition

This gate is complete when the public candidate has green Python authority/compiler tests and green Rust format/Clippy/workspace tests, and the exact compiler produces a deterministic private runtime artifact from the verified 2,196-record authority corpus with retained provenance and no silent loss of ambiguous structural records.

GitHub Issue #3 remains open after this gate for semantic enrichment and executable rule coverage.
