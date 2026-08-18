# Real-World STE Benchmark Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the provenance-first, rights-safe real-world STE benchmark and publish seed-v1 with one deep declared-STE source, twenty broad declared-STE documents, and twenty matched claim-none controls.

**Architecture:** Add a separate `ste-benchmark` binary crate to the existing workspace. Git stores strict source manifests, suite selection, and rights-safe results; copyrighted PDFs and extracted text live only in a content-addressed local cache. Benchmark-only hydration uses exact-pinned reqwest, PDF extraction is isolated behind a versioned Poppler adapter, and linting calls the existing verified runtime and `ste-lint` APIs directly.

**Tech Stack:** Rust 1.97.1, existing `serde`, `serde_json`, `sha2`, `clap`, `ste-core`, `ste-data`, and `ste-lint`; exact-pinned `reqwest = 0.13.4` with blocking rustls; external Poppler `pdftotext` for real PDF extraction; JSON Schema contracts.

**Spec:** `docs/superpowers/specs/2026-08-18-real-world-ste-benchmark-design.md`

## Global Constraints

- Preserve `fixtures/corpus` as synthetic deterministic regression material; do not put real-world source text there.
- ASD-STE100 and the verified Issue 9 runtime remain normative authority; publisher STE claims are corpus metadata only.
- Keep publisher claim and compliance/adjudication state independent.
- Do not commit full third-party PDFs or full extracted text under the default rights policy.
- Real baseline runs require `RuntimeLexicon::verified_issue9_from_bytes`; test lexicon output cannot become the committed seed baseline.
- Hydration is HTTPS-only, content-addressed, limited to 512 MiB per source, and fails closed on byte-size or SHA-256 mismatch.
- `page-text-v1` performs only UTF-8 validation, newline normalization, and terminal form-feed removal; no cleanup, OCR, header deletion, hyphenation repair, or line joining.
- Poppler extractor identity and arguments are recorded; there is no silent extraction fallback.
- Page selections are fixed before inspecting lint results and cannot exclude individual findings.
- Suite selections explicitly carry cohort identity, and matched broad/control selections record a `match_group` so matching is not reconstructed after evaluation.
- Committed benchmark results contain no source prose, diagnostic messages, diagnostic evidence, autofix replacement text, or source excerpts.
- Normal CI is hermetic and requires no network, Poppler installation, private runtime, or real source file.
- Keep the repository Rust toolchain pinned to 1.97.1 and verify with `--locked`.

---

### Task 1: Define strict benchmark contracts and crate boundary

**Files:**
- Modify: `Cargo.toml`
- Modify: `.gitignore`
- Create: `crates/ste-benchmark/Cargo.toml`
- Create: `crates/ste-benchmark/src/lib.rs`
- Create: `crates/ste-benchmark/src/model.rs`
- Create: `crates/ste-benchmark/src/main.rs`
- Create: `schemas/benchmark-source.schema.json`
- Create: `schemas/benchmark-suite.schema.json`
- Create: `schemas/benchmark-result.schema.json`
- Create: `crates/ste-benchmark/tests/model_contract.rs`
- Create: `crates/ste-benchmark/tests/fixtures/source-valid.json`
- Create: `crates/ste-benchmark/tests/fixtures/suite-valid.json`

**Interfaces:**
- Produces: `SourceManifest`, `SuiteManifest`, `SuiteSelection`, `BenchmarkResult`, `PageObservation`, `BenchmarkDiagnostic`, `SteClaimKind`, `VerificationState`, `Cohort`, `RightsPolicy`, and `BenchmarkError`.
- Consumes: existing `ste_core::{Outcome, Severity, Span}` and `ste_lint::LintMode`.

- [ ] **Step 1: Add failing model contract tests**

Create tests that require strict parsing, exact enums, retrieval date, explicit verification state, one-based page ranges, required suite cohort, optional `match_group`, and HTTPS-only URLs. Include a rejection test for an unknown field and a test proving `SteClaimKind::None` serializes as `"none"` without any `non_ste` semantic.

```rust
#[test]
fn source_manifest_rejects_unknown_fields() {
    let json = include_str!("fixtures/source-valid.json")
        .replace("\n}", ",\n  \"unexpected\": true\n}");
    assert!(serde_json::from_str::<SourceManifest>(&json).is_err());
}

#[test]
fn suite_rejects_zero_based_pages() {
    let mut suite: SuiteManifest = serde_json::from_str(include_str!("fixtures/suite-valid.json")).unwrap();
    suite.selections[0].first_page = 0;
    assert!(suite.validate(&source_index()).is_err());
}
```

- [ ] **Step 2: Run the focused tests and confirm failure**

```bash
cargo +1.97.1 test -p ste-benchmark --test model_contract
```

Expected: compilation/test failure because the crate and contract types do not exist.

- [ ] **Step 3: Add the workspace dependency and crate**

Add to `[workspace.dependencies]`:

```toml
reqwest = { version = "=0.13.4", default-features = false, features = ["blocking", "rustls"] }
```

Create `crates/ste-benchmark/Cargo.toml` with workspace-version/edition/license plus `clap`, `serde`, `serde_json`, `sha2`, `reqwest`, `ste-core`, `ste-data`, and `ste-lint`. Use existing workspace `tempfile`, `assert_cmd`, and `predicates` as dev dependencies where needed.

- [ ] **Step 4: Implement strict typed contracts**

Use `#[serde(deny_unknown_fields)]` on persisted structs. Define claim, verification, and cohort enums exactly as the spec states. Implement `SourceManifest::validate()` and `SuiteManifest::validate(&BTreeMap<String, SourceManifest>)` with checks for schema version 1, ISO `retrieval_date`, HTTPS, PDF media type, 64-character lowercase SHA-256, positive byte/page counts, unique IDs, valid range order, range within physical page count, explicit cohort, optional non-empty `match_group`, and explicit lint mode.

- [ ] **Step 5: Add JSON Schemas matching the Rust contract**

Use `additionalProperties: false` at every persisted object boundary. Keep the three schemas language-neutral and versioned by `schema_version: 1`.

- [ ] **Step 6: Gitignore local benchmark objects**

Append:

```gitignore
/.cache/
```

Do not ignore `benchmarks/real-world/baselines/` because rights-safe baseline JSON is intended repository evidence.

- [ ] **Step 7: Run tests, formatter, and Clippy**

```bash
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p ste-benchmark --all-targets --locked -- -D warnings
cargo +1.97.1 test -p ste-benchmark --locked
```

Expected: PASS.

- [ ] **Step 8: Commit the contract boundary**

```bash
git add Cargo.toml Cargo.lock .gitignore crates/ste-benchmark schemas/benchmark-*.json
git commit -m "feat: define real-world benchmark contracts"
```

---

### Task 2: Implement content-addressed source hydration

**Files:**
- Create: `crates/ste-benchmark/src/cache.rs`
- Modify: `crates/ste-benchmark/src/lib.rs`
- Test: `crates/ste-benchmark/tests/cache.rs`

**Interfaces:**
- Produces: `ByteFetcher`, `ReqwestFetcher`, `CachedSource`, and `hydrate_source(source, cache_root, fetcher)`.
- Consumes: `SourceManifest.identity`, `SourceManifest.url`, SHA-256, filesystem atomic rename.

- [ ] **Step 1: Write failing cache tests with a fake fetcher**

Use a fake implementation that writes deterministic bytes to the provided sink. Tests must cover cache hit, successful hydration, byte-size mismatch, SHA mismatch, cleanup after failure, and the size ceiling without allocating 512 MiB.

```rust
pub trait ByteFetcher {
    fn fetch_to(
        &self,
        url: &str,
        sink: &mut dyn std::io::Write,
        max_bytes: u64,
    ) -> Result<u64, BenchmarkError>;
}
```

- [ ] **Step 2: Confirm tests fail**

```bash
cargo +1.97.1 test -p ste-benchmark --test cache
```

Expected: FAIL because `cache` interfaces do not exist.

- [ ] **Step 3: Implement cache path and identity verification**

Use:

```text
<cache>/sha256/<manifest.identity.sha256>.pdf
```

A cache hit is accepted only after rechecking its byte size and SHA-256. Corrupt cached bytes fail closed rather than being overwritten implicitly.

- [ ] **Step 4: Implement `ReqwestFetcher`**

Use a blocking client. Require the initial and final URL schemes to be HTTPS. Stream in bounded chunks, stop once the observed byte count exceeds `512 * 1024 * 1024`, and never buffer the entire response.

- [ ] **Step 5: Implement temporary-file cleanup and atomic promotion**

Write beside the destination with a unique temporary name. On verified size/hash match, rename atomically to the content-addressed path. On every error path, remove the temporary object.

- [ ] **Step 6: Run focused and workspace tests**

```bash
cargo +1.97.1 test -p ste-benchmark --test cache --locked
cargo +1.97.1 test --workspace --locked
```

Expected: PASS with no network use in tests.

- [ ] **Step 7: Commit hydration**

```bash
git add crates/ste-benchmark Cargo.lock
git commit -m "feat: hydrate benchmark sources by content identity"
```

---

### Task 3: Isolate and version PDF extraction

**Files:**
- Create: `crates/ste-benchmark/src/extractor.rs`
- Modify: `crates/ste-benchmark/src/lib.rs`
- Test: `crates/ste-benchmark/tests/extractor.rs`

**Interfaces:**
- Produces: `TextExtractor`, `ExtractorIdentity`, `PopplerPdftotextExtractor`, `normalize_page_text_v1`.
- Consumes: verified local PDF path and one-based physical page.

- [ ] **Step 1: Write normalization and fake-extractor tests**

Tests must establish exact normalization:

```rust
assert_eq!(normalize_page_text_v1(b"A\r\nB\rC\n\x0c").unwrap(), "A\nB\nC\n");
assert_eq!(normalize_page_text_v1(b"A  B\n").unwrap(), "A  B\n");
```

Also verify invalid UTF-8 fails and ordinary internal form-feed/whitespace is not removed.

- [ ] **Step 2: Confirm tests fail**

```bash
cargo +1.97.1 test -p ste-benchmark --test extractor
```

- [ ] **Step 3: Implement the adapter trait and identity**

Use the exact trait from the design. `PopplerPdftotextExtractor::identity()` executes `pdftotext -v`, captures the version text, and records the configured executable path plus argument template.

- [ ] **Step 4: Implement one-page extraction**

Invoke:

```text
pdftotext -f PAGE -l PAGE -enc UTF-8 PDF -
```

Require successful exit status and apply only `page-text-v1`. Return `ExtractorUnavailable` for missing executable and `ExtractionFailed` for non-zero status or invalid output.

- [ ] **Step 5: Prove CI does not require Poppler**

All automated tests use a `FakeExtractor`; production Poppler tests are ignored/manual characterization tests if retained at all.

- [ ] **Step 6: Verify and commit**

```bash
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p ste-benchmark --all-targets --locked -- -D warnings
cargo +1.97.1 test -p ste-benchmark --locked
git add crates/ste-benchmark
git commit -m "feat: add versioned PDF extraction adapter"
```

---

### Task 4: Evaluate pages through the verified STE-Lint runtime

**Files:**
- Create: `crates/ste-benchmark/src/evaluate.rs`
- Create: `crates/ste-benchmark/src/report.rs`
- Modify: `crates/ste-benchmark/src/model.rs`
- Modify: `crates/ste-benchmark/src/lib.rs`
- Test: `crates/ste-benchmark/tests/evaluate.rs`
- Test: `crates/ste-benchmark/tests/report.rs`

**Interfaces:**
- Produces: `evaluate_suite`, `evaluate_page`, `aggregate_result`, rights-safe persisted results.
- Consumes: verified `RuntimeLexicon`, `TextExtractor`, source cache, suite selections, `ste_lint::lint_text_with_context`.

- [ ] **Step 1: Write a test proving result serialization cannot leak source text**

Use unique synthetic text and a fake diagnostic-bearing lint case. Serialize the benchmark result and assert it does not contain the source text, diagnostic message text, `evidence`, `autofix`, or replacement strings.

- [ ] **Step 2: Write coordinate and aggregation tests**

Require page-local UTF-8 byte spans, normalized text SHA-256, byte/word counts, outcomes, cohort and match-group identity, diagnostic counts by code/rule/cohort/mode, and diagnostics-per-1,000-words.

- [ ] **Step 3: Confirm tests fail**

```bash
cargo +1.97.1 test -p ste-benchmark --test evaluate --test report
```

- [ ] **Step 4: Load production runtime only through verified identity**

Implement:

```rust
pub fn load_verified_runtime(path: &Path) -> Result<RuntimeLexicon, BenchmarkError> {
    let bytes = std::fs::read(path)?;
    RuntimeLexicon::verified_issue9_from_bytes(&bytes).map_err(BenchmarkError::from)
}
```

Keep a separately named test-only path for `RuntimeLexicon::embedded()` and mark results created with it as `authoritative_runtime: false`.

- [ ] **Step 5: Evaluate each selected page without fixes or source-specific context**

Call:

```rust
lint_text_with_context(
    &text,
    lexicon,
    None,
    None,
    LintOptions { mode, fix: false },
)
```

Map each `Diagnostic` to only code, severity, rule IDs, and span. Never clone message/evidence/autofix into persisted benchmark state.

- [ ] **Step 6: Add deterministic run identity**

Hash the suite bytes, every source-manifest file, the hydrated source identities, and the current `ste-benchmark` executable. Record runtime metadata, extractor identity, `page-text-v1`, `whitespace-v1`, timestamp, and optional Git SHA.

- [ ] **Step 7: Fail the whole run when a selected page cannot be evaluated**

Do not produce a successful seed result with omitted pages. Return the specific infrastructure/source error before writing the requested output file.

- [ ] **Step 8: Run all tests and commit**

```bash
cargo +1.97.1 test -p ste-benchmark --locked
cargo +1.97.1 test --workspace --locked
git add crates/ste-benchmark schemas/benchmark-result.schema.json
git commit -m "feat: evaluate real-world benchmark pages"
```

---

### Task 5: Expose the benchmark CLI and hermetic integration tests

**Files:**
- Modify: `crates/ste-benchmark/src/main.rs`
- Create: `crates/ste-benchmark/tests/cli.rs`
- Create: `benchmarks/real-world/README.md`

**Interfaces:**
- Produces: `validate`, `hydrate`, `run`, and `summarize` commands.
- Consumes: Tasks 1-4 library interfaces.

- [ ] **Step 1: Add CLI tests for `validate` and `summarize`**

Use `assert_cmd` with synthetic fixtures only. Verify JSON output stability and non-zero exits for invalid manifest/suite/result inputs.

- [ ] **Step 2: Implement clap command shape**

Use:

```text
ste-benchmark validate --suite PATH
ste-benchmark hydrate --suite PATH [--cache-dir PATH]
ste-benchmark run --suite PATH --lexicon PATH --output PATH [--cache-dir PATH] [--pdftotext PATH]
ste-benchmark summarize RESULT [--format human|json]
```

`run` requires already verified cache objects and returns `source_not_cached`; it does not silently perform network hydration. This keeps acquisition and evaluation separable.

- [ ] **Step 3: Document maintainer workflow and rights boundary**

`benchmarks/real-world/README.md` must explain that manifests and rights-safe measurements are distributable repository data, while PDFs/extracted text remain local under source-specific rights.

- [ ] **Step 4: Verify normal CI-equivalent commands without Poppler/network**

```bash
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --workspace --all-targets --locked -- -D warnings
cargo +1.97.1 test --workspace --locked
python -m unittest discover -s tools/authority-ingest -p 'test_*.py' -v
```

Expected: PASS on the existing CI environment without installing Poppler.

- [ ] **Step 5: Commit CLI/documentation**

```bash
git add crates/ste-benchmark benchmarks/real-world/README.md
git commit -m "feat: expose real-world benchmark workflow"
```

---

### Task 6: Curate seed-v1 before observing lint findings

**Files:**
- Create: source manifests under `benchmarks/real-world/sources/lycoming/`
- Create: source manifests under `benchmarks/real-world/sources/nhtsa/`
- Create: `benchmarks/real-world/suites/seed-v1.json`

**Interfaces:**
- Produces: exactly 41 source manifests and one frozen suite selection.
- Consumes: authoritative publisher/government URLs and the strict manifest contract.

- [ ] **Step 1: Admit one deep Lycoming manual**

Use an official Lycoming-hosted maintenance manual with an explicit STE declaration. Retrieve the current candidate outside the benchmark cache and record exact identity:

```bash
curl -L --fail --output /tmp/ste-source.pdf 'HTTPS_SOURCE_URL'
sha256sum /tmp/ste-source.pdf
stat -c '%s' /tmp/ste-source.pdf
pdfinfo /tmp/ste-source.pdf | grep '^Pages:'
```

Record the admission date as `retrieval_date`. Inspect the declaration location and record a curator paraphrase plus physical page. Do not paste source prose into the manifest.

- [ ] **Step 2: Select the two Lycoming windows before linting**

Choose one clearly procedural contiguous 10-page range and one clearly descriptive contiguous 10-page range from section boundaries/table-of-contents evidence. Commit those physical page coordinates with cohort `declared_ste_deep` before any `ste-benchmark run` is performed against the manual.

- [ ] **Step 3: Admit 20 Aston Martin/NHTSA declared-STE communications**

For every document, verify the authoritative NHTSA URL, exact bytes/hash/page count, `retrieval_date`, explicit declaration location, document class, publication date when available, publisher, verification state `unknown`, and default rights policy. Reject duplicates by source SHA-256 even when URLs differ. Assign suite cohort `declared_ste_broad`.

- [ ] **Step 4: Admit 20 claim-none NHTSA controls**

Select manufacturer communications with no established explicit STE declaration, matched as closely as practical to the broad cohort by era, document class/purpose, and length. For each candidate, search the document and available metadata for `Simplified Technical English`, `ASD-STE100`, and `STE100` before assigning claim kind `none`; `none` still means no explicit declaration established, not proven non-STE. Assign suite cohort `claim_none_control` and a `match_group` that records the corresponding broad/control matching decision.

- [ ] **Step 5: Validate the frozen suite**

```bash
cargo +1.97.1 run -p ste-benchmark --locked -- validate \
  --suite benchmarks/real-world/suites/seed-v1.json
```

Expected: exactly 41 unique source identities, valid immutable page selections, all required cohort values, and explicit match groups for matched broad/control selections.

- [ ] **Step 6: Review the commit for accidental copyrighted text before landing it**

Inspect every new benchmark JSON file. The only source-derived prose permitted under default policy is titles/ordinary bibliographic metadata and curator-written paraphrases. No copied procedure sentences, manual pages, PDF blobs, or extracted-text files may be present.

- [ ] **Step 7: Commit the frozen seed manifests**

```bash
git add benchmarks/real-world/sources benchmarks/real-world/suites/seed-v1.json
git commit -m "data: curate seed-v1 STE benchmark sources"
```

---

### Task 7: Hydrate, characterize, run, and publish the first baseline

**Files:**
- Create: `benchmarks/real-world/baselines/seed-v1.json`
- Modify: `benchmarks/real-world/README.md`
- Modify: `README.md`

**Interfaces:**
- Produces: first reproducible rights-safe empirical baseline.
- Consumes: frozen seed-v1 suite, locally hydrated PDFs, Poppler identity, verified Issue 9 runtime, exact STE-Lint candidate.

- [ ] **Step 1: Hydrate all frozen sources and require exact identity**

```bash
cargo +1.97.1 run -p ste-benchmark --locked -- hydrate \
  --suite benchmarks/real-world/suites/seed-v1.json
```

Expected: every object is present at its content-addressed cache coordinate. Any moved URL or hash mismatch stops the run and requires source investigation, not manifest auto-update.

- [ ] **Step 2: Characterize the chosen Poppler installation before baseline execution**

Record `pdftotext -v` identity through the adapter. Manually inspect a small representative set of extracted pages for catastrophic extraction failure only, such as empty output or completely scrambled encoding. Do not edit individual extracted pages or change suite ranges based on STE-Lint findings.

- [ ] **Step 3: Run the complete seed against verified Issue 9 runtime directly into the rights-safe baseline format**

```bash
cargo +1.97.1 run -p ste-benchmark --locked -- run \
  --suite benchmarks/real-world/suites/seed-v1.json \
  --lexicon private-authority/issue9-runtime.json \
  --output benchmarks/real-world/baselines/seed-v1.json
```

Expected: authoritative runtime identity is true, all selected pages are represented, no partial-success state exists, and the output conforms to the rights-safe `BenchmarkResult` schema rather than raw `LintResult`.

- [ ] **Step 4: Summarize the committed baseline without rewriting it**

```bash
cargo +1.97.1 run -p ste-benchmark --locked -- summarize \
  benchmarks/real-world/baselines/seed-v1.json --format human
```

The baseline itself retains page-level coordinate observations and aggregate fields. `summarize` is a read-only presentation step.

- [ ] **Step 5: Prove the committed baseline contains no prohibited fields**

Add/execute a test that deserializes the committed baseline into the strict result type and recursively rejects keys named `text`, `message`, `evidence`, `autofix`, `replacement`, or `excerpt` anywhere in the persisted structure.

- [ ] **Step 6: Document the first empirical measurements without calling them compliance truth**

Update benchmark README with source/page/word counts, extractor/runtime/run identities, major diagnostic categories, match-group/cohort comparisons, and known extraction/page-boundary limitations. Do not convert the result into a single STE score and do not call publisher-declared sources gold-compliant fixtures.

- [ ] **Step 7: Run exact final verification**

```bash
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --workspace --all-targets --locked -- -D warnings
cargo +1.97.1 test --workspace --locked
python -m unittest discover -s tools/authority-ingest -p 'test_*.py' -v
```

Expected: PASS on the exact candidate. Hosted required checks must also pass on the final PR head before integration.

- [ ] **Step 8: Commit baseline and documentation**

```bash
git add benchmarks/real-world/baselines/seed-v1.json benchmarks/real-world/README.md README.md
git commit -m "data: publish seed-v1 STE benchmark baseline"
```

## Plan self-review

Spec coverage is complete: Tasks 1-2 implement identity/rights schemas and hydration; Task 3 implements the extraction boundary; Task 4 implements verified evaluation and rights-safe results; Task 5 supplies the explicit CLI and hermetic CI surface; Task 6 freezes all three seed cohorts and matching evidence before lint observation; Task 7 creates and verifies the first empirical baseline.

The plan intentionally does not implement NTSB/Regulations.gov harvesting, OCR, layout repair, a crawler, a hosted corpus service, an STE score, source-specific glossaries, or manual compliance adjudication. Those are separate future decisions after seed-v1 demonstrates the benchmark contract.
