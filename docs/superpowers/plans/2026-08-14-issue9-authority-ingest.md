# ASD-STE100 Issue 9 Authority Ingest Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reproducibly ingest the retained ASD-STE100 Issue 9 PDF into a private structured authority bundle with verifiable provenance.

**Architecture:** Drive remains the retained-byte authority. A deterministic Python maintenance tool extracts complete page text, structured writing rules, general recommendations, dictionary rows, normalized dictionary entries, and SQLite projections. Git stores only safe identity, code, hashes, counts, tests, and verification evidence.

**Tech Stack:** Python 3, Poppler `pdftotext`, `pdfplumber==0.11.9`, `pypdf==5.9.0`, SQLite.

## Global Constraints

- Normal STE-Lint runtime does not fetch or read the ASD PDF.
- Private source-derived content is not committed to the public repository.
- Generated output is not treated as verified until source identity and structural invariants pass.
- Dictionary normalization retains raw row fragments and physical-page provenance.
- Source-declared word counts and structural record counts are separate measures.

---

### Task 1: Verify the retained source

**Files:**
- Create: `data/issue9-source.manifest.json`
- Create: `tools/authority-ingest/ingest_issue9.py`

- [x] Verify the Drive object coordinate and file title.
- [x] Compute SHA-256 and byte size from the exact retained bytes.
- [x] Verify PDF page count, encryption state, publication identity, and metadata.
- [x] Record only safe identity and verification evidence in Git.

### Task 2: Extract complete page-level evidence

**Files:**
- Private output: `pages.jsonl`
- Private output: `issue9-layout.txt`

- [x] Use `pdftotext -layout` against the verified source.
- [x] Preserve all 434 physical pages.
- [x] Record logical page labels when present.
- [x] Compute one SHA-256 for each page-level text record.

### Task 3: Structure Part 1

**Files:**
- Private output: `rules.json`
- Private output: `general-recommendations.json`

- [x] Identify the actual rule heading occurrence after each section summary.
- [x] Extract rule text through the next rule boundary.
- [x] Verify exactly 53 writing rules.
- [x] Extract GR-1 through GR-8 and verify exactly eight recommendations.

### Task 4: Structure and verify Part 2 dictionary

**Files:**
- Private output: `dictionary-rows.jsonl`
- Private output: `dictionary.json`
- Create: `tools/authority-ingest/test_ingest_issue9.py`

- [x] Use parity-aware explicit dictionary table geometry.
- [x] Preserve every extracted row with physical page and row index.
- [x] Reconstruct entries from headword rows plus continuation rows.
- [x] Preserve raw fragments alongside normalized fields.
- [x] Fix and regression-test approval classification for mixed-case connector text such as `MATT (or MATTE)`.
- [x] Parse the source-declared 875 approved and 1,274 unapproved word counts from source evidence.
- [x] Record the 878 approved and 1,318 unapproved structural headword-record counts separately from source word counts.
- [x] Verify that every dictionary page without a table is a literal blank page.

### Task 5: Produce queryable projection and verification manifest

**Files:**
- Private output: `issue9-authority.sqlite3`
- Private output: `manifest.json`

- [x] Project source, pages, rules, recommendations, dictionary entries, and raw rows into SQLite.
- [x] Hash every generated artifact.
- [x] Run two fresh source ingests and verify identical artifact hashes.
- [x] Record counts, cardinality bases, and validation results in `manifest.json`.
- [x] Package the private authority bundle separately from the public repository.

### Task 6: Document and continuously test the maintenance boundary

**Files:**
- Modify: `tools/authority-ingest/README.md`
- Modify: `.github/workflows/ci.yml`
- Modify: `.gitignore`
- Create: `docs/superpowers/specs/2026-08-14-issue9-authority-ingest-design.md`

- [x] Document exact dependencies and invocation.
- [x] Document the Drive/Git authority split and copyright boundary.
- [x] Ignore local private authority outputs.
- [x] Run maintenance parser unit tests in CI without requiring the private PDF.
- [x] Keep normal lint execution independent of the PDF.
