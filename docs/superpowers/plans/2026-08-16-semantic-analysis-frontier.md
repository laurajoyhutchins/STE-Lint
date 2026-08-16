# Semantic Analysis Frontier Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace independent grammar-sensitive mini-parsers with a deterministic shared semantic-analysis core, then use that core to reduce the remaining context-dependent ASD-STE100 Issue 9 coverage without guessing.

**Architecture:** Each lint invocation constructs one `AnalysisDocument` from source text, the verified runtime lexicon, governed glossary, optional project context, and lint mode. The document owns stable byte spans plus token, sentence, dictionary, glossary, POS, verb-form, and resolution evidence; rule passes consume the shared analysis rather than re-tokenizing text. Later gates extend the same IR with grammar, entity/reference, sense, discourse, and safety semantics.

**Tech Stack:** Rust workspace (`ste-lint`, `ste-data`, `ste-glossary`, `ste-core`), serde/serde_json, existing GitHub Actions CI, synthetic engineering corpus in `fixtures/corpus/`.

## Global Constraints

- Deterministic and fail-closed; ambiguity is explicit rather than guessed.
- No external statistical NLP dependency.
- Preserve exact original byte spans.
- Do not publish protected ASD-STE100 source-derived prose.
- Reuse the verified runtime dictionary, project glossary, `.ste/context.json`, and current structural analyzers.
- `data/rules.json` remains the coverage authority.
- `full_compliance_claimed` remains `false` until Issue #3 success criteria are actually met.
- Gate 1 changes architecture only; it must not promote any rule status.

---

### Task 1: Shared analysis IR foundation

**Files:**
- Create: `crates/ste-lint/src/analysis/mod.rs`
- Create: `crates/ste-lint/src/analysis/token.rs`
- Create: `crates/ste-lint/src/analysis/sentence.rs`
- Create: `crates/ste-lint/src/analysis/grammar.rs`
- Create: `crates/ste-lint/src/analysis/document.rs`
- Modify: `crates/ste-lint/src/lib.rs`
- Test: `crates/ste-lint/tests/analysis_ir.rs`

**Interfaces:**
- Consumes: `&str`, `&RuntimeLexicon`, `Option<&Glossary>`, `Option<&LintContext>`, `LintMode`.
- Produces: `AnalysisDocument<'a>`, `AnalysisToken<'a>`, `AnalysisSentence`, `DictionaryMatch<'a>`, `GlossaryMatch<'a>`, `VerbFormCandidate<'a>`, `ObservedRole`, and `Resolution<T>`.

- [ ] Write failing integration tests that require stable token spans, sentence identity, dictionary/POS candidates, verb-form role evidence, governed glossary identity, and explicit resolved/ambiguous/unknown states.
- [ ] Run repository CI on the test-only head and verify failure is caused by the missing IR API.
- [ ] Implement the minimal analysis builder and public read-only interfaces.
- [ ] Run rustfmt, Clippy, unit/integration tests, and the synthetic corpus until green.
- [ ] Keep `data/rules.json` byte-for-byte unchanged.

### Task 2: Migrate dictionary role analysis

**Files:**
- Modify: `crates/ste-lint/src/passes/dictionary_roles.rs`
- Modify: `crates/ste-lint/src/lib.rs`
- Test: existing dictionary-role tests plus `crates/ste-lint/tests/analysis_ir.rs`

**Interfaces:**
- Consumes: `&AnalysisDocument` longest dictionary matches and bounded `ObservedRole` evidence.
- Produces: unchanged `STE-GRAM-001` diagnostics.

- [ ] Pin current positive and negative role cases before migration.
- [ ] Replace local tokenization, longest-window lookup, determiner/copula checks, and sentence-start logic with shared analysis queries.
- [ ] Delete superseded local helpers only after exact behavior is green.

### Task 3: Migrate governed technical-term roles

**Files:**
- Modify: `crates/ste-lint/src/passes/technical_roles.rs`
- Modify: `crates/ste-lint/src/lib.rs`
- Test: existing technical-role regressions.

**Interfaces:**
- Consumes: `&AnalysisDocument` glossary matches and bounded `ObservedRole` evidence.
- Produces: unchanged `STE-TERM-003` / `STE-TERM-004` diagnostics.

- [ ] Replace glossary-specific token scanning with shared longest glossary matches.
- [ ] Preserve governed term provenance, aliases, kind, and exact spans.
- [ ] Remove duplicated sentence-start/determiner/token helpers.

### Task 4: Migrate direct perfect-tense analysis

**Files:**
- Modify: `crates/ste-lint/src/passes/perfect.rs`
- Modify: `crates/ste-lint/src/lib.rs`
- Test: existing `perfect.rs` tests.

**Interfaces:**
- Consumes: analysis word tokens plus source-backed `VerbFormCandidate::PastParticiple` evidence and dictionary resolution state.
- Produces: unchanged `STE-VERB-001` / `STE-VERB-002` diagnostics.

- [ ] Preserve multiword participles and punctuation barriers.
- [ ] Preserve blocker behavior when a participle spelling has competing approved identity.
- [ ] Delete local word-span and participle-candidate parsing after parity is verified.

### Task 5: Migrate procedural and safety-opening syntax

**Files:**
- Modify: `crates/ste-lint/src/passes/procedural.rs`
- Modify: `crates/ste-lint/src/lib.rs`
- Test: existing procedural/safety tests plus corpus cases.

**Interfaces:**
- Consumes: analysis sentences, leading token/dictionary matches, source-backed lexical base-form evidence, and explicit ambiguity state.
- Produces: unchanged `STE-PROC-001`, `STE-PROC-002`, `STE-SAFE-001`, and `STE-SAFE-002` diagnostics.

- [ ] Preserve label and leading-condition exclusions.
- [ ] Preserve exact safety ambiguity blockers.
- [ ] Remove local `first_word` and `leading_dictionary_match` token logic after parity is green.

### Task 6: Gate 1 parity and integration

**Files:**
- Modify only if evidence demands it: `docs/diagnostics.md`, `docs/rule-coverage.md`.
- Do not change rule statuses.

- [ ] Assert the 14-case engineering corpus produces the exact pre-migration outcomes and code sets.
- [ ] Run exact-head authority-ingest, rustfmt, Clippy, and workspace tests.
- [ ] Confirm `data/rules.json` has no status or claim-boundary change.
- [ ] Land the coherent Gate 1 PR only after exact-head and PR-triggered CI are green.

### Task 7: Successor gates after Gate 1 lands

**Files:** Later tasks extend `crates/ste-lint/src/analysis/` rather than adding new pass-local parsers.

- [ ] Grammar v1: noun phrases, subject/predicate, auxiliary chains, participles, `-ing`, action structure.
- [ ] Entity/reference model: technical names, definitions, stable entities, pronoun/reference resolution.
- [ ] Structured sense identity: source-safe sense IDs and restriction tags, then bounded sense resolution.
- [ ] Document relationship graph: sentence/paragraph/topic/entity relationships and supplied semantic ordering facts.
- [ ] Safety semantics: safety level, actor, command, hazard, consequence with explicit resolution state.
- [ ] Hardening: positive/negative/ambiguity corpus expansion plus private-runtime aggregate audits before coverage promotion.
