# Real-World STE Benchmark Design

Date: 2026-08-18
Status: approved for implementation
Authority: GitHub #64

## Purpose

Build a reproducible empirical benchmark that measures STE-Lint against authentic operational technical writing that publishers identify as Simplified Technical English or ASD-STE100, plus matched technical-writing controls.

The benchmark is evidence about production writing and STE-Lint behavior. It is not ASD-STE100 authority and it must never redefine rule semantics.

## Architecture decision

Use the existing STE-Lint repository with a separate `ste-benchmark` binary crate and a separate `benchmarks/real-world/` data surface.

This is preferred over a separate corpus repository because benchmark result meaning depends on the exact STE-Lint implementation, runtime identity, and schemas. Keeping them in one repository makes a candidate reproducible from one commit and avoids cross-repository version coordination.

This is preferred over scripts under `tools/` because the benchmark must consume the same typed runtime and lint APIs as the product. A first-class Rust crate can reuse `ste-data`, `ste-lint`, `ste-core`, and the pinned workspace toolchain instead of reimplementing result semantics.

This is preferred over a hosted corpus service because the seed corpus is bounded and source documents can be hydrated from authoritative publisher or government URLs. A service would add identity, availability, privacy, and operational surfaces before they are necessary.

The benchmark crate is not part of normal `ste` lint execution. Network access, PDF extraction, and corpus hydration remain benchmark-only concerns.

## Authority and trust boundaries

The authority stack is:

```text
ASD-STE100 source authority
        |
        v
verified STE-Lint runtime + repository-owned rule semantics
        |
        v
STE-Lint
        |
        +-- fixtures/corpus
        |      synthetic deterministic rule regression
        |
        +-- benchmarks/real-world
               empirical production-writing evaluation
```

`fixtures/corpus` remains synthetic and deterministic. Real-world material must not be mixed into it.

A publisher statement that a document uses STE is a source claim, not a compliance verdict. The benchmark stores publisher claim and verification/adjudication as independent axes.

## Repository layout

```text
benchmarks/
  real-world/
    README.md
    sources/
      lycoming/
      nhtsa/
    suites/
      seed-v1.json
    baselines/
      seed-v1.json

schemas/
  benchmark-source.schema.json
  benchmark-suite.schema.json
  benchmark-result.schema.json

crates/
  ste-benchmark/
    Cargo.toml
    src/
      main.rs
      model.rs
      cache.rs
      extractor.rs
      evaluate.rs
      report.rs
    tests/
```

Future source-family adapters may add `regulations-gov/` and `ntsb/` manifests without changing the core benchmark model.

## Source manifest contract

Each source is one JSON file. Unknown fields fail validation.

Required conceptual fields are:

```json
{
  "schema_version": 1,
  "id": "lycoming-example",
  "source_family": "lycoming",
  "publisher": "Lycoming Engines",
  "title": "...",
  "document_type": "maintenance_manual",
  "url": "https://...",
  "media_type": "application/pdf",
  "identity": {
    "sha256": "...",
    "byte_size": 123,
    "physical_pages": 42
  },
  "ste_claim": {
    "kind": "explicit_ste",
    "evidence": {
      "physical_page": 10,
      "method": "publisher_statement",
      "note": "Curator paraphrase of the declaration location."
    }
  },
  "rights": {
    "redistribution": "manifest_only",
    "source_cache": "local_only",
    "derived_text": "local_only",
    "committed_excerpt": "none"
  }
}
```

STE claim kinds are exactly `explicit_asd_ste100`, `explicit_ste`, `qualified_asd_ste100`, `inferred`, and `none`.

`none` means that no explicit STE declaration was established for the source. It must not be described as proven non-STE.

Verification/adjudication kinds are exactly `unknown`, `rule_verified`, `known_violation`, and `manually_adjudicated`.

The first seed uses `unknown` unless independent adjudication exists. Lint output does not automatically promote a document into another verification state.

## Rights policy

Public availability is not redistribution permission.

The default real-world source policy is:

```text
redistribution = manifest_only
source_cache = local_only
derived_text = local_only
committed_excerpt = none
```

Full source PDFs and full extracted text are never committed under this default. A more permissive value requires an affirmative source-specific rights basis.

The STE declaration evidence note is a curator paraphrase. It must not copy a substantial source passage merely to prove the declaration.

## Hydration and source identity

The source URL is a locator. SHA-256 plus byte size is the source identity.

Default cache location:

```text
.cache/ste-lint/benchmark/sha256/<sha256>.pdf
```

`/.cache/` is gitignored.

Hydration requirements:

1. HTTPS only.
2. Stream to a temporary file rather than buffering a complete manual in memory.
3. Refuse more than 512 MiB for one source.
4. Compute byte count and SHA-256 while receiving bytes.
5. Require both to equal the manifest.
6. Atomically rename the verified object into the content-addressed cache.
7. Delete temporary material after any failure.
8. Never update a manifest automatically when a URL starts serving different bytes.

A URL serving a different document produces `source_identity_mismatch`. It does not silently create a new benchmark source.

The benchmark hydration dependency is exact-pinned `reqwest = 0.13.4` with `default-features = false` and features `blocking` and `rustls`. It is benchmark-only and does not become a normal STE-Lint runtime dependency.

## PDF extraction boundary

PDF extraction is representation evidence, not STE authority.

Define an internal adapter:

```rust
pub trait TextExtractor {
    fn identity(&self) -> Result<ExtractorIdentity, BenchmarkError>;
    fn extract_page(
        &self,
        pdf: &std::path::Path,
        physical_page: u32,
    ) -> Result<String, BenchmarkError>;
}
```

Seed v1 uses Poppler `pdftotext` as an external adapter rather than making one Rust PDF parser part of STE-Lint's language semantics.

The production adapter invokes one physical page at a time:

```text
pdftotext -f <page> -l <page> -enc UTF-8 <pdf> -
```

The executable path, reported version string, and exact arguments are part of extractor identity. Missing Poppler is `extractor_unavailable`. There is no silent fallback to another parser or OCR engine.

### page-text-v1 normalization

The only allowed normalization is:

1. Require valid UTF-8.
2. Convert CRLF and bare CR to LF.
3. Remove one terminal PDF form-feed marker while preserving an adjacent terminal LF.
4. Preserve all other whitespace and characters.

Do not join lines, remove headers/footers, repair hyphenation, repair tables, infer columns, or OCR in seed v1. Those transformations would make the benchmark harder to reproduce and could change lint meaning.

Each physical page is evaluated independently. This intentionally makes the extraction unit explicit. Page-boundary artifacts are a limitation of seed v1 and are not manually edited away.

## Suite contract

A suite selects immutable source identities and bounded physical-page ranges:

```json
{
  "schema_version": 1,
  "id": "seed-v1",
  "selections": [
    {
      "id": "source:procedures",
      "source_id": "source",
      "first_page": 20,
      "last_page": 29,
      "mode": "procedural"
    }
  ]
}
```

Pages are one-based physical PDF pages. `first_page <= last_page` and both must fit the source identity page count.

A selection cannot exclude arbitrary lines, sentences, findings, or byte ranges. Page ranges are selected before reviewing lint findings so the suite cannot be tuned around errors.

## Runtime and lint execution

Real-source benchmark runs require a verified ASD-STE100 Issue 9 runtime supplied explicitly with `--lexicon` and loaded through `RuntimeLexicon::verified_issue9_from_bytes`.

The embedded test lexicon may be enabled only for hermetic development/tests. A result produced from test runtime data is marked non-authoritative and cannot be committed as the seed baseline.

The benchmark calls `ste_lint::lint_text_with_context` directly with `fix: false`, no source-specific glossary, no source-specific semantic context, and mode from the suite selection.

Blocked technical terminology remains a meaningful benchmark measurement. The corpus must not acquire a special glossary merely to make scores look better.

## Rights-safe result contract

Do not serialize `LintResult` directly. It contains source text and diagnostic fields that can contain source-derived evidence or replacement text.

The benchmark result owns a narrower type:

```rust
pub struct BenchmarkDiagnostic {
    pub code: String,
    pub severity: Severity,
    pub rules: Vec<String>,
    pub span: Span,
}
```

Each page observation contains source ID, selection ID, physical page, lint mode, normalized text SHA-256, normalized byte count, word count, lint outcome, and rights-safe diagnostics.

It must contain no extracted prose, diagnostic message, evidence object, autofix replacement, or source excerpt.

Coordinates are page-local UTF-8 byte offsets into the exact normalized page text identified by `normalized_text_sha256`.

## Run identity

A benchmark run records enough identity to explain every result change:

- suite file SHA-256
- source manifest SHA-256 values
- hydrated document SHA-256 values
- workspace/package version
- benchmark executable SHA-256
- verified runtime identity
- extractor executable/version/arguments
- normalization version `page-text-v1`
- word-count version `whitespace-v1`
- run timestamp
- Git commit SHA when available as supplemental evidence

The executable hash is the decisive local software identity because it covers the linked STE-Lint candidate even when the worktree is dirty.

## Reporting

Reports provide measurements, not a synthetic compliance score.

Required aggregate dimensions are sources, selected pages, bytes, words, clean/error/blocked outcomes, diagnostics per 1,000 words, diagnostics by code, diagnostics by ASD rule reference, blocked diagnostics by code, source family, publisher claim cohort, and procedural versus descriptive mode.

There is no single `STE score` in seed v1.

## Seed v1 composition

The first seed has three cohorts:

1. `declared-ste-deep`: one official Lycoming maintenance manual that explicitly declares STE. Select one contiguous 10-page procedural window and one contiguous 10-page descriptive window from clearly labeled sections. Select these ranges before running STE-Lint.
2. `declared-ste-broad`: 20 Aston Martin communications from the NHTSA archive with explicit ASD-STE100 or STE declarations. Use the whole document only when its English technical text is homogeneous; otherwise select an objectively bounded page range before linting.
3. `claim-none-control`: 20 NHTSA manufacturer communications for which no explicit STE declaration is established, matched as closely as practical by publication era, document class, purpose, and length. These are controls for publisher claim, not proven non-STE examples.

FAA/Regulations.gov and NTSB sources are deferred until after seed v1. The source model must already be general enough to add them without a schema rewrite.

## CLI

The benchmark binary is `ste-benchmark` with commands:

```text
ste-benchmark validate --suite benchmarks/real-world/suites/seed-v1.json
ste-benchmark hydrate --suite benchmarks/real-world/suites/seed-v1.json
ste-benchmark run --suite ... --lexicon ... --output target/benchmark/seed-v1.json
ste-benchmark summarize target/benchmark/seed-v1.json
```

Global controls include `--cache-dir`, `--pdftotext`, and `--format human|json` where applicable.

## Failure model

Infrastructure/source failures are not lint outcomes. At minimum distinguish `manifest_invalid`, `source_not_cached`, `source_download_failed`, `source_identity_mismatch`, `source_too_large`, `extractor_unavailable`, `extraction_failed`, `runtime_invalid`, and `result_write_failed`.

A seed run is fail-closed. If one selected page cannot be evaluated exactly, the baseline run fails rather than publishing a partial aggregate.

## Test strategy

Normal CI remains hermetic. It does not contact Lycoming, NHTSA, require Poppler, or require the private Issue 9 runtime.

Tests use synthetic tiny files and fake fetcher/extractor implementations to cover strict manifest/suite parsing, HTTPS and page-range validation, cache identity success and mismatch, temporary-file cleanup and size ceiling, extraction identity and normalization, exact page-local byte spans, invalid runtime rejection, rights-safe serialization, aggregation/cohort dimensions, and CLI validation/summarization.

Real-source hydration and benchmark execution are explicit maintainer operations.

## Acceptance

Seed v1 is complete when:

1. `fixtures/corpus` remains unchanged in purpose and contains only synthetic regression material.
2. `ste-benchmark` is a separate workspace binary and normal `ste` execution has no network/PDF dependency.
3. Source, suite, and result schemas are strict and versioned.
4. Source hydration is content-addressed and fails closed on identity mismatch.
5. Source bytes and extracted full text remain local-only under the default rights policy.
6. Poppler extraction is versioned and has no silent fallback.
7. Real benchmark runs require verified Issue 9 runtime data.
8. Committed result records contain no source prose, diagnostic evidence, or autofix replacement text.
9. Seed v1 contains 1 deep Lycoming source, 20 declared-STE Aston Martin/NHTSA documents, and 20 claim-none NHTSA controls.
10. Selected Lycoming windows are fixed before inspecting lint findings.
11. A machine-readable baseline and aggregate summary can be regenerated from manifests plus locally hydrated source objects.
12. Hermetic workspace CI passes without network access, Poppler, or private source material.
