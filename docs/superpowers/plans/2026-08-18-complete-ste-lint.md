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

- [x] Add failing CommonMark characterization for multiline code spans.
- [x] Implement `SourceDocument` with byte ranges from offset events.
- [x] Route protected code, paragraph, and CommonMark list recognition through `SourceDocument`.
- [x] Delete superseded backtick and generic Markdown-list recognition while retaining only STE-specific non-CommonMark list forms.
- [x] Run focused structure tests and full Rust tests on the release candidate.

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

- [x] Add token/span and generic evidence characterization tests.
- [x] Build deterministic char-index-to-byte-index mapping.
- [x] Make Harper the canonical generic word token stream.
- [x] Replace generic determiner/linking/morphology evidence where compatibility is proven.
- [x] Keep runtime dictionary/glossary authority decisive for approval and permitted forms.
- [x] Delete the superseded generic lexical scanner and comparison scaffolding.
- [x] Run analysis/grammar tests, curated goldens, profile regressions, and the representative corpus through the workspace suite.

### Task 3: Close remaining executable rule gaps honestly

**Files:**
- Modify/create passes under `crates/ste-lint/src/passes/`
- Modify: `data/rules.json`
- Test: focused rule tests under `crates/ste-lint/tests/`
- Modify: `docs/rule-coverage.md`

**Interfaces:**
- Consumes: source-backed runtime dictionary forms plus generic role evidence.
- Produces: bounded diagnostics for mechanically provable cases; context-required classification when semantic evidence cannot be inferred safely.

- [x] Implement source-linked out-of-inventory verb/adjective form detection for Rules 1.4/3.1 without generated STE morphology authority.
- [x] Implement a bounded Rule 9.3 phrasal-verb slice only where lexical composition evidence is explicit; otherwise require context rather than inventing semantics.
- [x] Ensure zero rules remain `not_implemented`; mechanically unsafe rules are represented as partial or context-required with an explicit unresolved boundary.
- [x] Update coverage truth and tests atomically.

### Task 4: Release usability and exact-head verification

**Files:**
- Modify: `README.md`
- Modify: CLI/docs only where install/use gaps are found.
- Modify: `Cargo.lock`

**Interfaces:**
- Produces: installable `ste` CLI with documented verified-runtime and terminology/context workflows.

- [x] Generate and commit Cargo.lock on Rust 1.97.1.
- [x] Verify `cargo +1.97.1 fmt --all -- --check`.
- [x] Verify `cargo +1.97.1 clippy --workspace --all-targets --locked -- -D warnings`.
- [x] Verify `cargo +1.97.1 test --workspace --locked` plus authority-ingest tests, goldens, profiles, and representative engineering regressions.
- [x] Exercise the CLI through integration tests covering runtime selection, coverage, profiles/glossary behavior, procedural linting, JSON output, deterministic fix mode, rewrite checking, and dictionary inspection; document `ste --help` as the installed entrypoint.
- [x] Update README with exact Rust 1.97.1 installation, public-data first-run, private-runtime production use, and pinned verification commands.
- [x] Require release integration through PR #63 only after both permanent required checks pass on the exact final head; close temporary characterization/recovery PRs without merging them.

## Final Result

The release candidate has one generic source-structure authority (`pulldown-cmark`) and one generic linguistic evidence stream (`harper-core`). Both are isolated behind adapters that project back to original UTF-8 byte coordinates. The ASD-STE100 runtime, governed terminology, explicit project context, and repository-owned rule semantics remain the only sources that can grant STE authority.

The remaining executable gaps were closed without inflating compliance claims: Rules 1.4 and 3.1 now reject source-linked out-of-inventory forms only when generic morphology can identify one approved runtime lemma, and Rule 9.3 consumes explicit project phrasal-verb evidence rather than inferring compositional meaning. `data/rules.json` accounts for all 53 Issue 9 rules with zero `not_implemented` statuses while retaining `full_compliance_claimed: false` and explicit partial/context-required boundaries.

Release verification is pinned to Rust 1.97.1 and the committed lockfile. Permanent CI verifies formatting, Clippy with warnings denied, the full locked workspace test suite, and the Python authority-ingest suite. README installation uses `cargo +1.97.1 install --path crates/ste-cli --locked`, so the shipped binary is directly installable as `ste` from a clean repository checkout.
