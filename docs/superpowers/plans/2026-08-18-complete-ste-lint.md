# Complete STE-Lint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish STE-Lint as release-grade software by completing the approved parser-dependency refactor, eliminating remaining mechanically implementable `not_implemented` rule gaps, and verifying an installable CLI on the pinned toolchain.

**Architecture:** Generic Markdown structure comes from exact-pinned `pulldown-cmark`; generic English linguistic evidence comes from exact-pinned `harper-core`; ASD-STE100 runtime data, project terminology, context authority, fail-closed resolution, diagnostics, and rule semantics remain repository-owned. STE byte offsets remain canonical at every public boundary.

**Tech Stack:** Rust 1.97.1, pulldown-cmark 0.13.4, harper-core 2.7.0, existing STE runtime/glossary/context crates, GitHub CI.

**Spec:** GitHub #58 plus the canonical `data/rules.json` coverage contract and GitHub #3 completion authority.

## Global Constraints

- Preserve verified private ASD-STE100 Issue 9 authority and do not publish restricted source prose.
- External parser output is evidence, never STE approval authority.
- Canonical source coordinates are UTF-8 byte offsets into the original input.
- Preserve fail-closed `Resolved | Ambiguous | Unknown` semantics.
- No network/service/model/JVM/Python dependency in normal lint execution.
- Keep Rust pinned to 1.97.1 and commit the exact generated Cargo.lock.
- Do not claim automatic coverage for rules that require human/project context; represent those honestly as context-required.

---

### Task 1: Characterize and replace generic Markdown recognition

**Files:**
- Create: `crates/ste-lint/src/analysis/source.rs`
- Modify: `crates/ste-lint/src/analysis/mod.rs`
- Modify: `crates/ste-lint/src/structure.rs`
- Modify: `crates/ste-lint/src/document_structure.rs`
- Test: `crates/ste-lint/tests/markdown_sentence_boundaries.rs`

**Interfaces:**
- Produces: `SourceDocument::new(&str)`, canonical paragraph/list/code spans.
- Consumes: `pulldown-cmark::Parser::into_offset_iter`.

- [ ] Add failing CommonMark characterization for multiline code spans.
- [ ] Implement `SourceDocument` with byte ranges from offset events.
- [ ] Route protected code, paragraph, and list recognition through `SourceDocument`.
- [ ] Delete superseded backtick and Markdown-list recognition.
- [ ] Run focused structure tests and full Rust tests.

### Task 2: Replace generic tokenization with Harper evidence

**Files:**
- Create: `crates/ste-lint/src/analysis/linguistic.rs`
- Modify: `crates/ste-lint/src/analysis/token.rs`
- Modify: `crates/ste-lint/src/analysis/document.rs`
- Modify: `crates/ste-lint/src/analysis/grammar.rs`
- Test: `crates/ste-lint/tests/analysis_ir.rs`

**Interfaces:**
- Produces: canonical byte-spanned STE tokens plus bounded generic grammar predicates.
- Consumes: `harper_core::Document`/`TokenKind`; converts Harper character spans at one adapter boundary.

- [ ] Add token/span and generic evidence characterization tests.
- [ ] Build deterministic char-index-to-byte-index mapping.
- [ ] Make Harper the canonical generic word token stream.
- [ ] Replace generic determiner/linking/morphology evidence where compatibility is proven.
- [ ] Keep runtime dictionary/glossary authority decisive for approval and permitted forms.
- [ ] Delete the superseded generic lexical scanner and comparison scaffolding.
- [ ] Run analysis/grammar tests and goldens.

### Task 3: Close remaining executable rule gaps honestly

**Files:**
- Modify/create passes under `crates/ste-lint/src/passes/`
- Modify: `data/rules.json`
- Test: focused rule tests under `crates/ste-lint/tests/`
- Modify: `docs/rule-coverage.md`

**Interfaces:**
- Consumes: source-backed runtime dictionary forms plus generic role evidence.
- Produces: bounded diagnostics for mechanically provable cases; context-required classification when semantic evidence cannot be inferred safely.

- [ ] Implement source-linked out-of-inventory verb/adjective form detection for Rules 1.4/3.1 without generated STE morphology authority.
- [ ] Implement a bounded Rule 9.3 phrasal-verb slice only where lexical composition evidence is explicit; otherwise change the rule to context-required rather than inventing semantics.
- [ ] Ensure zero rules remain `not_implemented` unless authoritative evidence proves implementation is unsafe even with explicit context.
- [ ] Update coverage truth and tests atomically.

### Task 4: Release usability and exact-head verification

**Files:**
- Modify: `README.md`
- Modify: CLI/docs only where install/use gaps are found.
- Modify: `Cargo.lock`

**Interfaces:**
- Produces: installable `ste-lint` CLI with documented verified-runtime and terminology/context workflows.

- [ ] Generate and commit Cargo.lock on Rust 1.97.1.
- [ ] Verify `cargo fmt --all -- --check`.
- [ ] Verify `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [ ] Verify `cargo test --workspace --locked` plus authority-ingest tests, goldens, profiles, and engineering corpus.
- [ ] Smoke-test CLI help, coverage output, profile inspection, procedural/descriptive linting, JSON output, and fix mode.
- [ ] Update README with concrete install and first-run examples.
- [ ] Merge only an exact verified head and close superseded temporary PRs/issues.
