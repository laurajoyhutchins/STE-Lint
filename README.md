# STE-Lint

STE-Lint is a compiler-style language tool for writing controlled technical English with agent assistance.

**Current status:** this repository proves the architecture with a small embedded test lexicon. It is **not yet a complete ASD-STE100 Issue 9 implementation** and must not be presented as full STE compliance.

The linter owns the checks it implements. An LLM can propose repairs, but it does not declare its own output compliant.

## Why this exists

Prompting an agent to "write ASD-STE100" makes compliance depend on model memory and judgment. STE-Lint moves mechanical constraints into code:

```text
source text
    |
    v
ste lint
    |-- safe deterministic fixes
    `-- structured diagnostics
              |
              v
         LLM repair
              |
              v
      ste check-rewrite
              |
              v
          ste lint
```

A released STE-Lint package is self-contained. Normal use does **not** fetch or read the ASD-STE100 PDF. Source documents are maintenance inputs for the versioned language data, not runtime dependencies.

## What the first slice implements

- stable JSON diagnostics;
- an embedded, versioned runtime lexicon model;
- repo-local technical terminology in `.ste/terms.json`;
- semicolon detection with a whitelisted deterministic autofix;
- procedural sentence length diagnostics over 20 words;
- descriptive sentence length diagnostics over 25 words;
- unapproved-word diagnostics against the embedded test lexicon;
- blocked diagnostics for unknown prose terms that may need technical-term classification;
- semantic rewrite checks for modality, negation, and numeric-literal changes;
- human-readable and JSON CLI output.

The current word counter and tokenizer are intentionally conservative first-slice implementations. See `docs/diagnostics.md` for exact behavior.

## Build and test

Requires stable Rust.

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Run the CLI from the workspace:

```bash
cargo run -q -p ste-cli -- version
```

## CLI

Lint a file:

```bash
ste lint instructions.md --mode procedural
ste lint report.md --mode descriptive --format json
```

Apply only whitelisted deterministic fixes:

```bash
ste lint instructions.md --mode procedural --fix
```

Check whether a proposed rewrite changes protected semantics:

```bash
ste check-rewrite before.txt after.txt --format json
```

Inspect the runtime lexicon:

```bash
ste dictionary lookup acceptable --format json
```

Validate a project technical glossary:

```bash
ste glossary check .ste/terms.json --format json
```

Exit codes:

| Code | Meaning |
| --- | --- |
| `0` | Clean, accepted, or all applicable errors safely fixed |
| `1` | Error diagnostics remain or a rewrite was rejected |
| `2` | Lint result is blocked by unresolved terminology |
| `3` | Runtime language or glossary data is invalid |
| `4` | I/O or internal failure |

## Repo-local technical terminology

A repository can extend the effective lexicon with `.ste/terms.json` without changing built-in language data.

```json
{
  "terms": [
    {
      "term": "busway",
      "kind": "technical_noun",
      "definition": "A project term for an electrical distribution assembly.",
      "domain": "electrical",
      "preferred": true,
      "aliases": [],
      "examples": ["Inspect the busway."],
      "provenance": ["project authority"],
      "status": "approved"
    }
  ]
}
```

Unknown words are not added automatically. `STE-TERM-001` means the term needs classification, not that it is necessarily wrong.

## Agent use

`skills/STE.SKILL.md` describes the lint-repair-verify loop. `skills/STE-GLOSSARY.SKILL.md` describes technical-term maintenance. The skills intentionally do not reproduce the 53 ASD-STE100 rules; mechanical policy belongs in the executable.

## Runtime data and ASD-STE100

`crates/ste-data/data/test-lexicon.json` is a small test dataset used to prove the runtime model. It is not the complete ASD dictionary.

`tools/authority-ingest/` is reserved for maintenance tooling that can build and verify versioned runtime data from authorized source material. Populated source-derived datasets should not be committed to this public repository without an explicit redistribution basis.

## Design

The approved architecture is documented in `docs/superpowers/specs/2026-08-14-ste-lint-design.md`. The initial implementation plan is in `docs/superpowers/plans/2026-08-14-initial-vertical-slice.md`.
