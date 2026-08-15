# STE-Lint

STE-Lint is a compiler-style language tool for writing controlled technical English with agent assistance.

**Current status:** STE-Lint can lint real technical prose with the verified private ASD-STE100 Issue 9 runtime dictionary, while retaining a small embedded lexicon for public tests and development. It is **not yet a complete ASD-STE100 Issue 9 implementation** and must not be presented as full STE compliance because only a subset of the 53 writing rules is executable.

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

Normal runtime use does **not** fetch or read the ASD-STE100 PDF. Source documents are maintenance inputs for versioned language data, not runtime dependencies.

## What is implemented

- stable JSON diagnostics;
- a versioned runtime lexicon model;
- exact identity verification for the authorized private Issue 9 runtime corpus;
- explicit runtime selection with `--lexicon` or `STE_LINT_LEXICON`;
- ambiguity-preserving dictionary and lexical lookup;
- repo-local technical terminology in `.ste/terms.json`;
- semicolon detection with a whitelisted deterministic autofix;
- procedural sentence length diagnostics over 20 words;
- descriptive sentence length diagnostics over 25 words;
- unapproved-word diagnostics against the active runtime lexicon;
- blocked diagnostics for dictionary spellings whose approved status depends on unresolved grammar or sense;
- blocked diagnostics for unknown prose terms that may need technical-term classification;
- semantic rewrite checks for modality, negation, and numeric-literal changes;
- human-readable and JSON CLI output.

The current word counter and tokenizer are intentionally conservative. See `docs/diagnostics.md` for exact behavior and the remaining rule-family boundary.

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

## Runtime dictionary

The public repository does not contain the populated Issue 9 dictionary. Authorized users can point STE-Lint at the verified private runtime artifact:

```bash
ste --lexicon /private/path/runtime-lexicon.json version
ste --lexicon /private/path/runtime-lexicon.json lint instructions.md --mode procedural
ste --lexicon /private/path/runtime-lexicon.json dictionary lookup check --format json
```

For persistent local or agent use, set:

```bash
export STE_LINT_LEXICON=/private/path/runtime-lexicon.json
```

On PowerShell:

```powershell
$env:STE_LINT_LEXICON = "C:\private\runtime-lexicon.json"
```

An explicit `--lexicon` path takes precedence over `STE_LINT_LEXICON`. If a configured file is missing or does not exactly match `data/issue9-runtime.manifest.json`, STE-Lint exits with code `3`. It does **not** silently fall back to test data.

If neither selector is present, STE-Lint uses `crates/ste-data/data/test-lexicon.json`. `ste version` reports the active runtime source so that embedded test data cannot be mistaken for the verified Issue 9 dictionary.

The verified private runtime identity is metadata-only in Git: 2,196 structural dictionary records, 1,538,305 bytes, SHA-256 `55251f20bb8c361d3849df1ea4797a756f7195753d715ffb6e4a74616adb3c6f`. Populated source-derived prose remains outside the public repository.

A verified full dictionary does **not** imply full ASD-STE100 compliance. A clean result means only that the diagnostic families implemented by this version found no applicable errors or unresolved blockers.

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

Inspect the active runtime lexicon:

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
| `2` | Lint result is blocked by unresolved terminology, grammar, or sense |
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

`data/issue9-runtime.manifest.json` is the public identity contract for the authorized private runtime dictionary. It contains hashes and cardinalities, not dictionary prose.

`crates/ste-data/data/test-lexicon.json` remains a small public test dataset. It is not the complete ASD dictionary.

`tools/authority-ingest/` contains maintenance tooling that builds and verifies versioned runtime data from authorized source material. Populated source-derived datasets must not be committed to this public repository without an explicit redistribution basis.

## Design

The approved architecture is documented in `docs/superpowers/specs/2026-08-14-ste-lint-design.md`. The runtime-usability execution plan is in `docs/superpowers/plans/2026-08-15-private-runtime-usability.md`.
