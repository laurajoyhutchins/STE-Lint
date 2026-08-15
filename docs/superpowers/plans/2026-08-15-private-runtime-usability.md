# Verified Private Runtime Usability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make STE-Lint usable on real technical prose with the verified private ASD-STE100 Issue 9 runtime corpus without publishing that corpus.

**Architecture:** `ste-data` owns the exact runtime identity contract and verification because that contract applies to every consumer. `ste-cli` owns runtime selection: an explicitly supplied path or `STE_LINT_LEXICON` may select the verified private corpus, while the embedded test lexicon remains an explicit development fallback. Missing or invalid configured data is an error, never a silent fallback.

**Tech Stack:** Rust 2024 workspace, serde/serde_json, SHA-256, clap, assert_cmd.

## Global Constraints

- The public repository must not contain populated source-derived Issue 9 dictionary prose.
- Runtime use must not require the source PDF.
- Source-declared word counts remain distinct from structural record counts.
- Ambiguous normalized forms remain multi-candidate; no last-record-wins behavior may return.
- A clean lint result is authoritative only for implemented diagnostic families, not full ASD-STE100 compliance.

---

### Task 1: Verified runtime identity

**Files:**
- Create: `data/issue9-runtime.manifest.json`
- Modify: `Cargo.toml`
- Modify: `crates/ste-data/Cargo.toml`
- Modify: `crates/ste-data/src/lib.rs`

**Interfaces:**
- Produces `RuntimeLexicon::verified_issue9_from_bytes(bytes: &[u8])` and `RuntimeLexicon::verified_issue9_from_json(json: &str)`.
- Verification checks exact byte size/SHA-256, Issue 9 metadata, structural cardinalities, and entry count against the public identity-only manifest before returning a runtime.

- [ ] Add tests that valid synthetic bytes are accepted only when their manifest identity matches.
- [ ] Add tests that same-size byte tampering is rejected by SHA-256.
- [ ] Add tests that metadata/cardinality mismatches are rejected even when JSON is syntactically valid.
- [ ] Implement the minimal verifier and keep `RuntimeLexicon::embedded()` unchanged as a test fixture path.
- [ ] Run format, Clippy, and workspace tests.

### Task 2: Deterministic CLI runtime selection

**Files:**
- Modify: `crates/ste-cli/src/main.rs`
- Modify: `crates/ste-cli/tests/cli.rs`

**Interfaces:**
- Global `--lexicon <PATH>` selects the verified private runtime.
- `STE_LINT_LEXICON` provides the same selection when the flag is absent.
- If neither is set, commands that need a lexicon use the embedded test lexicon and identify it truthfully as test data.
- If a configured path is missing or invalid, exit code 3 is returned; no fallback occurs.

- [ ] Add RED CLI tests for explicit path selection, environment selection, and invalid configured path behavior.
- [ ] Wire one runtime resolver through lint, dictionary, and version commands.
- [ ] Make `version` identify whether runtime data is verified Issue 9 or embedded test data.
- [ ] Run the CLI test suite and workspace verification.

### Task 3: Real-corpus acceptance and truthful operator docs

**Files:**
- Modify: `README.md`
- Modify: `docs/diagnostics.md` only if runtime semantics need clarification.

**Interfaces:**
- The private real corpus remains outside Git.
- Acceptance evidence records the verified runtime identity: 2,196 structural records, 1,538,305 bytes, SHA-256 `55251f20bb8c361d3849df1ea4797a756f7195753d715ffb6e4a74616adb3c6f`.

- [ ] Verify the private artifact against the landed verifier contract.
- [ ] Document `--lexicon` and `STE_LINT_LEXICON` for authorized local use.
- [ ] State explicitly that full dictionary availability does not imply all 53 rules are executable.
- [ ] Open a PR with exact-head CI and private acceptance evidence.
