# STE-Lint Initial Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a self-contained Rust CLI that proves STE-Lint's architecture with structured diagnostics, a small embedded test lexicon, repo-local technical terminology, safe mechanical autofix, and semantic rewrite checks.

**Architecture:** A Rust workspace separates shared contracts, runtime language data, glossary handling, lint passes, rewrite validation, and the CLI. Runtime checks never depend on the ASD PDF; the initial committed lexicon is intentionally small and exists only to prove the data model and execution flow. JSON diagnostics are the stable interface, while human-readable output is a presentation layer.

**Tech Stack:** Rust 2024 edition, `serde`, `serde_json`, `thiserror`, `clap`, `regex`, `tempfile`, `assert_cmd`, `predicates`.

## Global Constraints

- A released STE-Lint package is self-contained; normal linting has no PDF dependency.
- Mechanical rules belong in code, not prompts.
- Diagnostics are a stable API and do not encode ASD rule numbers into their stable diagnostic code.
- Autofix is whitelist-only and must not change intended propositions.
- ASD rules, general recommendations, and STE-Lint semantic safety rules keep separate provenance.
- Project technical terminology is repo-local in `.ste/terms.json`.
- The LLM can propose repairs but cannot declare its own output compliant.
- The first slice does not claim full ASD-STE100 coverage.
- The first slice does not automatically mutate the technical glossary.
- A fresh checkout must build and test without the ASD PDF.

---

## File map

- `Cargo.toml`: workspace members and shared dependency versions.
- `crates/ste-core/src/lib.rs`: public shared contracts for spans, severities, diagnostics, fixes, and lint status.
- `crates/ste-data/src/lib.rs`: typed embedded runtime lexicon and rule metadata.
- `crates/ste-data/data/test-lexicon.json`: intentionally small lawful runtime dataset used by the first slice.
- `crates/ste-glossary/src/lib.rs`: `.ste/terms.json` parsing and glossary integrity checks.
- `crates/ste-lint/src/lib.rs`: orchestration and lint result model.
- `crates/ste-lint/src/passes/*.rs`: independent mechanical lint passes.
- `crates/ste-rewrite-check/src/lib.rs`: deterministic semantic-diff checks.
- `crates/ste-cli/src/main.rs`: `ste` command surface and exit-code mapping.
- `schemas/*.schema.json`: language-neutral public contracts.
- `fixtures/**`: regression inputs and expected behavior.
- `skills/*.SKILL.md`: agent operating instructions for lint and glossary workflows.
- `README.md`: product contract and getting-started workflow.

### Task 1: Workspace and stable diagnostic contracts

**Files:**
- Create: `Cargo.toml`
- Create: `crates/ste-core/Cargo.toml`
- Create: `crates/ste-core/src/lib.rs`
- Create: `schemas/diagnostic.schema.json`
- Test: `crates/ste-core/src/lib.rs`

**Interfaces:**
- Produces:
  - `Span { start: usize, end: usize }`
  - `Severity::{Error, Warning, Blocked}`
  - `Fix { replacement: String, span: Span }`
  - `Diagnostic { code, severity, message, span, rules, evidence, autofix }`
  - `Outcome::{Clean, Fixed, Error, Blocked}`

- [ ] **Step 1: Write the failing serialization test**

Add this test to `crates/ste-core/src/lib.rs` before defining the types:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_serializes_with_stable_external_field_names() {
        let diagnostic = Diagnostic {
            code: "STE-PUNC-001".into(),
            severity: Severity::Error,
            message: "Semicolons are not permitted.".into(),
            span: Span { start: 4, end: 5 },
            rules: vec!["8.1".into()],
            evidence: None,
            autofix: Some(Fix {
                span: Span { start: 4, end: 5 },
                replacement: ".".into(),
            }),
        };

        let value = serde_json::to_value(diagnostic).unwrap();
        assert_eq!(value["code"], "STE-PUNC-001");
        assert_eq!(value["severity"], "error");
        assert_eq!(value["span"]["start"], 4);
        assert_eq!(value["autofix"]["replacement"], ".");
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p ste-core diagnostic_serializes_with_stable_external_field_names
```

Expected: compile failure because the shared contract types do not exist yet.

- [ ] **Step 3: Implement the minimal shared contracts**

Create `crates/ste-core/src/lib.rs` with serializable structs and enums. Use `#[serde(rename_all = "snake_case")]` on enums and `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields. Represent `evidence` as `Option<serde_json::Value>` in the first slice so diagnostic producers can attach typed-looking evidence without locking the schema too early.

- [ ] **Step 4: Add the JSON Schema**

Create `schemas/diagnostic.schema.json` with required fields `code`, `severity`, `message`, `span`, and `rules`; allow severities `error`, `warning`, and `blocked`; model `autofix` as either `null` or `{span,replacement}`.

- [ ] **Step 5: Run all core tests**

Run:

```bash
cargo test -p ste-core
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/ste-core schemas/diagnostic.schema.json
git commit -m "feat: define stable STE diagnostic contracts"
```

### Task 2: Versioned embedded runtime language data

**Files:**
- Create: `crates/ste-data/Cargo.toml`
- Create: `crates/ste-data/src/lib.rs`
- Create: `crates/ste-data/data/test-lexicon.json`
- Create: `schemas/dictionary.schema.json`
- Create: `data/rules.json`
- Create: `data/general-recommendations.json`
- Test: `crates/ste-data/src/lib.rs`

**Interfaces:**
- Consumes: `ste-core`
- Produces:
  - `PartOfSpeech`
  - `ApprovalStatus`
  - `LexiconEntry`
  - `Alternative`
  - `RuntimeLexicon::embedded()`
  - `RuntimeLexicon::lookup_form(&str)`
  - `RuntimeLexicon::lookup_lemma(&str)`

- [ ] **Step 1: Write failing lexicon lookup tests**

Add tests that assert:

```rust
let lexicon = RuntimeLexicon::embedded().unwrap();
let entry = lexicon.lookup_form("ensures").unwrap();
assert_eq!(entry.lemma, "ensure");
assert_eq!(entry.status, ApprovalStatus::Unapproved);

let permitted = lexicon.lookup_form("permitted").unwrap();
assert_eq!(permitted.lemma, "PERMITTED");
assert_eq!(permitted.status, ApprovalStatus::Approved);
```

Also assert that an approved verb form not listed in the dataset returns `None` rather than being generated heuristically.

- [ ] **Step 2: Run and verify failure**

Run:

```bash
cargo test -p ste-data
```

Expected: compile failure because the runtime lexicon API does not exist.

- [ ] **Step 3: Define the first-slice dictionary data model**

Model entries with:

```rust
pub struct LexiconEntry {
    pub lemma: String,
    pub status: ApprovalStatus,
    pub part_of_speech: PartOfSpeech,
    pub forms: Vec<String>,
    pub senses: Vec<Sense>,
    pub alternatives: Vec<Alternative>,
    pub restrictions: Vec<String>,
}
```

Use explicit `forms`; do not derive unlisted forms.

Model alternatives with `kind`, `text`, optional `part_of_speech`, and `strategy` where strategy is one of `word_replacement`, `phrase_replacement`, or `sentence_reconstruction`.

- [ ] **Step 4: Add a deliberately small embedded lexicon**

`crates/ste-data/data/test-lexicon.json` must contain enough entries to exercise the architecture, including:

- approved `PERMITTED (adj)`
- approved `MAKE SURE (v)`
- approved `CAN (v)`
- approved `MUST (v)`
- approved `USE (v)`
- approved `WITH (prep)`
- unapproved `acceptable (adj)` with `PERMITTED`
- unapproved `ensure (v)` with `MAKE SURE`
- unapproved `may (v)` with `CAN`
- unapproved `should (v)` with `MUST`
- unapproved `using (v)` with alternatives `USE` and `WITH`

State clearly in a top-level metadata object that this is a first-slice test lexicon and not the complete Issue 9 dictionary.

- [ ] **Step 5: Add versioned rule metadata**

Create `data/rules.json` containing the identifiers and short internal titles needed by implemented passes: `1.1`, `1.2`, `1.3`, `5.1`, `6.3`, and `8.1`.

Create `data/general-recommendations.json` with GR identifiers and titles only. Do not reproduce the full copyrighted prose.

- [ ] **Step 6: Add dictionary JSON Schema and validate in tests**

The schema must require metadata plus `entries`, and each entry must require `lemma`, `status`, `part_of_speech`, `forms`, `senses`, `alternatives`, and `restrictions`.

Use `serde_json` round-tripping in Rust tests to ensure the committed dataset conforms to the Rust data model.

- [ ] **Step 7: Run tests**

Run:

```bash
cargo test -p ste-data
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/ste-data data schemas/dictionary.schema.json
git commit -m "feat: add embedded STE runtime data model"
```

### Task 3: Repo-local technical glossary and integrity diagnostics

**Files:**
- Create: `crates/ste-glossary/Cargo.toml`
- Create: `crates/ste-glossary/src/lib.rs`
- Create: `schemas/glossary.schema.json`
- Create: `fixtures/glossary/valid.json`
- Create: `fixtures/glossary/duplicate.json`
- Test: `crates/ste-glossary/src/lib.rs`

**Interfaces:**
- Consumes: `ste-core`, `ste-data`
- Produces:
  - `Glossary`
  - `TechnicalTerm`
  - `TechnicalTermKind::{TechnicalNoun, TechnicalVerb}`
  - `Glossary::load(path)`
  - `Glossary::contains_term(&str)`
  - `Glossary::validate() -> Vec<Diagnostic>`

- [ ] **Step 1: Write failing glossary validation tests**

Tests must prove:

```rust
let glossary = Glossary::from_json(include_str!("../../../fixtures/glossary/valid.json")).unwrap();
assert!(glossary.contains_term("busway"));
assert!(glossary.validate().is_empty());
```

and:

```rust
let glossary = Glossary::from_json(include_str!("../../../fixtures/glossary/duplicate.json")).unwrap();
let diagnostics = glossary.validate();
assert_eq!(diagnostics[0].code, "TERM-DUP-001");
```

- [ ] **Step 2: Run and verify failure**

Run:

```bash
cargo test -p ste-glossary
```

Expected: compile failure because glossary types are absent.

- [ ] **Step 3: Implement glossary parsing and duplicate detection**

A technical term contains:

```rust
pub struct TechnicalTerm {
    pub term: String,
    pub kind: TechnicalTermKind,
    pub definition: String,
    pub domain: String,
    pub preferred: bool,
    pub aliases: Vec<String>,
    pub examples: Vec<String>,
    pub provenance: Vec<String>,
    pub status: TermStatus,
}
```

Normalize identity for duplicate detection using Unicode-preserving lowercase plus collapsed ASCII whitespace. Do not stem terms.

Emit `TERM-DUP-001` as an error with evidence containing both conflicting term strings.

- [ ] **Step 4: Add glossary schema**

Require all fields above. Restrict `kind` to `technical_noun` and `technical_verb`; restrict `status` to `approved` and `deprecated`.

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p ste-glossary
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ste-glossary schemas/glossary.schema.json fixtures/glossary
git commit -m "feat: add repo-local STE technical glossary"
```

### Task 4: Mechanical linter passes and safe autofix

**Files:**
- Create: `crates/ste-lint/Cargo.toml`
- Create: `crates/ste-lint/src/lib.rs`
- Create: `crates/ste-lint/src/passes/mod.rs`
- Create: `crates/ste-lint/src/passes/punctuation.rs`
- Create: `crates/ste-lint/src/passes/length.rs`
- Create: `crates/ste-lint/src/passes/lexical.rs`
- Create: `fixtures/lint/semicolon.txt`
- Create: `fixtures/lint/procedure-too-long.txt`
- Create: `fixtures/lint/unknown-term.txt`
- Create: `fixtures/autofix/semicolon.before.txt`
- Create: `fixtures/autofix/semicolon.after.txt`
- Test: pass-local unit tests and `crates/ste-lint/src/lib.rs`

**Interfaces:**
- Consumes: `ste-core`, `ste-data`, `ste-glossary`
- Produces:
  - `LintMode::{Procedural, Descriptive}`
  - `LintOptions { mode, fix }`
  - `LintResult { text, diagnostics, outcome }`
  - `lint_text(text, lexicon, glossary, options) -> LintResult`

- [ ] **Step 1: Write failing punctuation/autofix tests**

Assert that:

```rust
let result = lint_text(
    "Open the valve; inspect the seal.",
    &lexicon,
    None,
    LintOptions { mode: LintMode::Procedural, fix: false },
);
assert_eq!(result.diagnostics[0].code, "STE-PUNC-001");
```

and with `fix: true`, the semicolon becomes a period and the diagnostic outcome is recorded as fixed without leaving a remaining error.

- [ ] **Step 2: Run and verify failure**

Run:

```bash
cargo test -p ste-lint punctuation
```

Expected: compile failure.

- [ ] **Step 3: Implement semicolon detection and whitelisted fix**

Detect literal `;`. Emit `STE-PUNC-001`, rule `8.1`. The only first-slice autofix replaces `;` with `.` and preserves surrounding text exactly.

Autofix application must sort fixes by descending span start so offsets remain stable.

- [ ] **Step 4: Write failing sentence-length tests**

For procedural mode, a 21-word sentence emits `STE-LEN-001` referencing rule `5.1`. For descriptive mode, a 26-word sentence emits `STE-LEN-002` referencing `6.3`.

Use a deliberately simple first-slice word counter that splits Unicode whitespace and trims surrounding punctuation. Document this limitation in the diagnostic evidence as `counter: "first_slice_whitespace"`.

- [ ] **Step 5: Implement length passes**

Do not autofix length violations.

- [ ] **Step 6: Write failing lexical tests**

Assert:
- known unapproved `acceptable` emits `STE-LEX-001` with `PERMITTED` evidence;
- a term present in the supplied glossary produces no unknown-term diagnostic;
- an unknown alphabetic token emits `STE-TERM-001` with severity `blocked`.

The lexical pass must ignore numeric literals and tokens containing `_`, `/`, `\\`, `-`, or `.` in the first slice to avoid falsely treating common machine identifiers and paths as prose words.

- [ ] **Step 7: Implement lexical pass**

Do not autofix unapproved words, even when only one alternative is present. The first slice proves the diagnostic/repair architecture rather than taking semantic risk.

- [ ] **Step 8: Add autofix idempotence test**

Run `lint_text(... fix=true)` twice and assert the second run changes no text and emits no `STE-PUNC-001`.

- [ ] **Step 9: Run all linter tests**

Run:

```bash
cargo test -p ste-lint
```

Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add crates/ste-lint fixtures/lint fixtures/autofix
git commit -m "feat: add mechanical STE lint passes and safe autofix"
```

### Task 5: Deterministic rewrite safety checker

**Files:**
- Create: `crates/ste-rewrite-check/Cargo.toml`
- Create: `crates/ste-rewrite-check/src/lib.rs`
- Create: `schemas/proposed-change.schema.json`
- Create: `fixtures/bad-repairs/modality.json`
- Create: `fixtures/bad-repairs/negation.json`
- Create: `fixtures/bad-repairs/quantity.json`
- Test: `crates/ste-rewrite-check/src/lib.rs`

**Interfaces:**
- Consumes: `ste-core`
- Produces:
  - `ProposedChange { original, proposed, target_diagnostics }`
  - `RewriteCheckResult { accepted, diagnostics }`
  - `check_rewrite(&ProposedChange) -> RewriteCheckResult`

- [ ] **Step 1: Write failing semantic regression tests**

Required cases:

```rust
assert_rejected(
    "The request may fail.",
    "The request fails.",
    "SEM-MODALITY-001",
);

assert_rejected(
    "Do not open the valve.",
    "Open the valve.",
    "SEM-NEGATION-001",
);

assert_rejected(
    "Keep the pressure below 10 psi.",
    "Keep the pressure below 20 psi.",
    "SEM-QUANTITY-001",
);
```

Also include one accepted identity-preserving repair such as punctuation-only change.

- [ ] **Step 2: Run and verify failure**

Run:

```bash
cargo test -p ste-rewrite-check
```

Expected: compile failure.

- [ ] **Step 3: Implement conservative first-slice checks**

- Modality: extract case-insensitive tokens from the protected set `may`, `can`, `could`, `must`, `should`, `will`; reject if the multiset differs.
- Negation: reject if counts of `not`, `no`, `never`, and `cannot` differ.
- Quantity: extract numeric literals using a regex that recognizes signed integers and decimals; reject if the ordered list differs.

Return the corresponding exact semantic code. These checks are intentionally conservative and may block valid rewrites rather than silently accept a semantic change.

- [ ] **Step 4: Add proposed-change JSON Schema**

Require `original`, `proposed`, and `target_diagnostics`.

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p ste-rewrite-check
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ste-rewrite-check schemas/proposed-change.schema.json fixtures/bad-repairs
git commit -m "feat: reject unsafe STE rewrite proposals"
```

### Task 6: CLI and machine-readable output

**Files:**
- Create: `crates/ste-cli/Cargo.toml`
- Create: `crates/ste-cli/src/main.rs`
- Create: `crates/ste-cli/tests/cli.rs`
- Create: `.gitignore`

**Interfaces:**
- Consumes: all library crates
- Produces executable `ste` with:
  - `ste lint <path> [--fix] [--format human|json] [--mode procedural|descriptive]`
  - `ste check-rewrite <before> <after> [--format human|json]`
  - `ste dictionary lookup <word> [--format human|json]`
  - `ste glossary check [path] [--format human|json]`
  - `ste version`

- [ ] **Step 1: Write failing CLI integration tests**

Use `assert_cmd` and `tempfile` to verify:

```rust
Command::cargo_bin("ste")
    .unwrap()
    .args(["version"])
    .assert()
    .success()
    .stdout(predicate::str::contains("ASD-STE100 Issue 9"));
```

Also verify JSON lint output contains `"code":"STE-PUNC-001"` and exits `1` without `--fix`.

- [ ] **Step 2: Run and verify failure**

Run:

```bash
cargo test -p ste-cli
```

Expected: compile failure or missing binary.

- [ ] **Step 3: Implement `lint` command**

Load the built-in lexicon. Search upward from the target file's parent for `.ste/terms.json`; if absent, continue without a glossary. Do not create configuration files implicitly.

Exit:
- `0` when clean or all errors were safely fixed;
- `1` when error diagnostics remain;
- `2` when blocked diagnostics remain and no errors remain;
- `3` for invalid language/glossary data;
- `4` for I/O or internal failures.

When `--fix` is used, write the fixed text back only after lint completes successfully.

- [ ] **Step 4: Implement `check-rewrite`, `dictionary lookup`, `glossary check`, and `version`**

`dictionary lookup` reports every matching runtime entry by exact case-insensitive lemma/form. `glossary check` defaults to `.ste/terms.json` in the current directory.

- [ ] **Step 5: Run CLI tests**

Run:

```bash
cargo test -p ste-cli
```

Expected: PASS.

- [ ] **Step 6: Run workspace tests**

Run:

```bash
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/ste-cli .gitignore
git commit -m "feat: expose STE-Lint CLI"
```

### Task 7: Agent skills, README, regression surface, and final verification

**Files:**
- Create: `skills/STE.SKILL.md`
- Create: `skills/STE-GLOSSARY.SKILL.md`
- Create: `README.md`
- Create: `docs/diagnostics.md`
- Create: `docs/dictionary-model.md`
- Create: `docs/repair-protocol.md`
- Modify: `docs/superpowers/specs/2026-08-14-ste-lint-design.md` only if implementation revealed a factual mismatch.

**Interfaces:**
- Consumes: released CLI behavior from Tasks 1-6
- Produces: durable operating instructions for humans and agents.

- [ ] **Step 1: Write the agent lint skill**

`skills/STE.SKILL.md` must instruct agents to:
1. run `ste lint`;
2. apply safe autofixes;
3. inspect remaining structured diagnostics;
4. make the smallest semantic-preserving repair;
5. run `ste check-rewrite`;
6. rerun `ste lint`;
7. stop clean or explicitly blocked;
8. never suppress a diagnostic merely to pass.

Do not reproduce the full 53-rule standard in the skill.

- [ ] **Step 2: Write the glossary skill**

`skills/STE-GLOSSARY.SKILL.md` must require evidence before adding a technical noun/verb, run `ste glossary check`, and prohibit automatic addition merely because a token is unknown.

- [ ] **Step 3: Write README**

The README must state prominently:
- STE-Lint is not yet a complete ASD-STE100 Issue 9 implementation;
- the first slice uses a small embedded test lexicon;
- runtime use does not require the ASD PDF;
- the linter is compliance authority for implemented checks;
- an LLM is an optional repair backend;
- `.ste/terms.json` is repo-local technical terminology.

Include build/test commands and short CLI examples.

- [ ] **Step 4: Document diagnostic and data contracts**

`docs/diagnostics.md` lists every code implemented in the first slice and whether it is error, warning, or blocked.

`docs/dictionary-model.md` explains word+POS+sense identity, explicit forms, alternative strategies, and the separation between built-in runtime data and repo-local terminology.

`docs/repair-protocol.md` explains why destination lint alone is insufficient and defines the semantic invariant checks.

- [ ] **Step 5: Format and lint Rust**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: both PASS.

- [ ] **Step 6: Run the complete test suite**

Run:

```bash
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 7: Smoke-test the binary**

Run:

```bash
cargo run -q -p ste-cli -- version
printf 'Open the valve; inspect the seal.\n' > /tmp/ste-smoke.txt
cargo run -q -p ste-cli -- lint /tmp/ste-smoke.txt --format json
cargo run -q -p ste-cli -- lint /tmp/ste-smoke.txt --fix
cat /tmp/ste-smoke.txt
```

Expected:
- version identifies Issue 9;
- first lint emits `STE-PUNC-001`;
- fix exits successfully;
- final file contains `Open the valve. inspect the seal.` exactly for the deliberately minimal first-slice fixer.

- [ ] **Step 8: Verify repository status**

Run:

```bash
git status --short
git log --oneline --decorate -8
```

Expected: no uncommitted changes and task-sized commits visible.

- [ ] **Step 9: Commit documentation**

```bash
git add README.md skills docs
git commit -m "docs: define STE-Lint agent workflow and contracts"
```

## Plan self-review

- Spec coverage: the plan covers the entire approved first vertical slice: workspace, runtime data schema, repo-local glossary, mechanical passes, safe fix, stable diagnostics, rewrite checks, fixtures, skills, CLI, docs, and verification.
- Scope: full Issue 9 population, editor plugins, hosted services, MCP, full NLP parsing, and automatic glossary mutation remain explicitly outside this plan.
- Type consistency: all later tasks consume the exact public types defined in preceding tasks.
- Placeholder scan: the plan contains no implementation placeholders; each task names concrete files, APIs, tests, commands, and expected outcomes.
