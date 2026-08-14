# Issue 9 Authority-to-Runtime Mapping Execution Record

Date: 2026-08-14
Authority: GitHub Issue #3
Gate: first authority-to-runtime dictionary mapping gate
Status: candidate implemented; Issue #3 remains open

## Objective

Prove that the verified private ASD-STE100 Issue 9 dictionary authority can be projected into STE-Lint's existing runtime dictionary contract without a runtime PDF dependency, public redistribution of the populated corpus, semantic loss, invented morphology, invented POS, or silent record loss.

## Constraints

- Keep `crates/ste-data/data/test-lexicon.json` as the public embedded dataset for this gate.
- Keep the populated Issue 9 runtime derivative private.
- Validate source-declared word counts separately from structural record counts.
- Preserve source record order and physical-page provenance.
- Preserve uninterpreted source semantics rather than guessing executable restrictions/senses.
- Never synthesize ordinary English morphology.
- Never invent a POS for source expression records.
- Keep normal `ste` execution independent of Drive, the PDF, and maintenance compilers.

## Implemented steps

### 1. Extend the runtime contract

Implemented in `crates/ste-data` and `schemas/dictionary.schema.json`:

- optional retained-source authority provenance;
- separate source-declared and structural cardinalities;
- optional POS;
- entry record/page provenance;
- retained `source_semantics` cells;
- `interpretation_state` (`structural` / `interpreted`).

Backward compatibility is preserved for the existing public test lexicon; its entries default to `interpreted` and authority metadata remains optional.

TDD evidence: the enriched-model test failed before these fields/types existed, then the workspace gate passed after implementation.

### 2. Specify the compiler with synthetic authority data

Added invented-only fixtures under `fixtures/authority-ingest/` and `tools/authority-ingest/test_build_runtime_lexicon.py`.

The tests cover:

- exact POS mapping and POS absence;
- explicit verb-form cleanup without invented morphology;
- source identity mismatch rejection;
- structural-cardinality mismatch rejection;
- technical-noun alternative preservation;
- structural versus interpreted state;
- source-semantic/page-provenance preservation;
- byte-deterministic output.

TDD evidence: the suite failed with `ModuleNotFoundError` before the compiler existed, while the pre-existing ingest tests stayed green.

### 3. Implement the private authority compiler

Added `tools/authority-ingest/build_runtime_lexicon.py`.

Current CLI:

```bash
python tools/authority-ingest/build_runtime_lexicon.py \
  --authority-dir <private-authority-directory> \
  --verified-manifest data/issue9-source.manifest.json \
  --private-bundle-sha256 <exact-parent-private-bundle-sha256> \
  --out <private-runtime-lexicon.json>
```

The explicit bundle hash is required because an extracted directory cannot reconstruct the identity of the parent ZIP.

The compiler validates:

- issue and publication date;
- retained Drive object ID;
- source SHA-256, byte size, and physical page count;
- explicit parent private-bundle SHA-256;
- private-manifest source identity;
- dictionary entry count;
- structural approved/unapproved counts;
- source-declared approved/unapproved counts.

It preserves source order, explicit forms, source pages, and the four retained source-semantic cells. It copies already-interpreted senses/alternatives/restrictions when present and otherwise emits structural records without guessing semantics.

### 4. Run against the real private Issue 9 authority

Private compilation results:

- 2,196 structural records;
- 878 approved structural records;
- 1,318 unapproved structural records;
- 875 source-declared approved words;
- 1,274 source-declared unapproved words;
- 2 expression records without source POS;
- 1,538,305-byte runtime artifact;
- SHA-256 `55251f20bb8c361d3849df1ea4797a756f7195753d715ffb6e4a74616adb3c6f`;
- two independent builds produced byte-identical output.

The populated derivative and its identity manifest are retained privately, not committed.

### 5. Fix real-corpus form ambiguity

The real compilation exposed 192 normalized spellings shared by 397 structural record instances. Some collisions cross POS or approval status.

The original runtime form index stored one record per normalized spelling and would silently overwrite earlier candidates.

TDD correction:

- add a failing collision test;
- change the form index to retain all record indices;
- add `lookup_form_candidates` to return all candidates in source-record order;
- keep `lookup_form` only for unique spellings and return `None` when ambiguous.

This preserves source distinctions until future POS/sense disambiguation can resolve them.

### 6. Verify the exact candidate

Repository verification required on every final head:

```bash
python -m py_compile \
  tools/authority-ingest/ingest_issue9.py \
  tools/authority-ingest/test_ingest_issue9.py \
  tools/authority-ingest/build_runtime_lexicon.py \
  tools/authority-ingest/test_build_runtime_lexicon.py
python -m unittest discover -s tools/authority-ingest -p 'test_*.py' -v
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

GitHub Actions is the repository-owned Rust executor for this session.

## Public/private boundary

Public Git contains only compiler/runtime contracts, tests, synthetic fixtures, docs, safe counts, and hashes. The source PDF, source-derived dictionary prose, populated runtime lexicon, and private authority databases remain private.

## Gate exit

The first gate is ready for integration when the PR head is mergeable and both the authority/compiler job and Rust job are green on the pull-request event.

Merging this gate does **not** close GitHub Issue #3.

## Successor gates

After this candidate lands, Issue #3 continues with:

1. semantic enrichment of structural dictionary records into executable senses, alternatives, help/restrictions, and technical-term references;
2. POS/sense-aware lint disambiguation over multi-candidate spellings;
3. replacement/packaging decision for the private runtime corpus, subject to redistribution authority;
4. explicit per-rule executable coverage evidence toward all 53 Issue 9 writing rules;
5. full compliance claim only after dictionary semantics and all required rule checks are executable and verified.
