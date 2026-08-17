# STE-Lint

STE-Lint is a compiler-style language tool for writing controlled technical English with agent assistance.

**Current status:** STE-Lint can lint real technical prose with the verified private ASD-STE100 Issue 9 runtime dictionary. It is **not yet a complete ASD-STE100 Issue 9 implementation**. Use `ste coverage` to inspect exactly which of the 53 writing rules are implemented, partial, context-dependent, or not implemented.

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

- stable JSON diagnostics and machine-readable 53-rule coverage reporting;
- a versioned runtime lexicon model with exact private-runtime identity verification;
- explicit runtime selection with `--lexicon` or `STE_LINT_LEXICON`, with fail-closed lint/dictionary operation;
- ambiguity-preserving, longest-match dictionary and glossary phrase lookup;
- source-backed approved verb paradigms that preserve lexical, irregular-auxiliary, and defective-modal distinctions;
- governed reusable and repo-local technical terminology, including technical nouns, technical verbs, dual-use terms, explicit forms, aliases, provenance, and lifecycle status;
- deterministic composition of opt-in `software-core`, `git`, and `github` terminology profiles with repo-local `.ste/terms.json`;
- repo-local `.ste/context.json` evidence for bounded sense, terminology-scope, and spelling decisions that raw text cannot safely establish;
- semicolon detection with a whitelisted deterministic autofix;
- contraction detection for the deterministic portion of Rule 4.2;
- direct `HAVE`/`HAS`/`HAD` plus approved-past-participle detection for the deterministic portion of Rule 3.4;
- Issue 9-aware procedural/descriptive sentence-length counting and descriptive paragraph limits;
- deterministic text-level handling for vertical-list boundaries, parentheticals, quoted text, identifiers, number+unit groups, decimals, and hyphenated groups;
- procedural `NOTE:` recognition: note sentences use the descriptive 25-word limit, and source-backed sentence-initial imperative candidates are diagnosed or blocked when ambiguous;
- bounded simple vertical-list mechanics: colon introduction, uppercase item starts, comma/semicolon prohibition, and final-item period;
- unapproved-word/phrase diagnostics and blockers for unresolved dictionary ambiguity or unknown project terminology;
- semantic rewrite checks for modality, negation, and numeric-literal changes;
- human-readable and JSON CLI output.

The executable deliberately does not guess document semantics that raw prose cannot establish. Nested/wrapped list grammar, sentence-versus-fragment classification, article choice, non-imperative requirements in notes, general POS/sense resolution, and many other Issue 9 rules still require deeper grammar, document structure, or domain context. `ste coverage --format json` is the authority for that boundary.

## Build and test

Requires stable Rust.

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Runtime dictionary

The public repository does not contain the populated Issue 9 dictionary. Authorized users can point STE-Lint at the verified private runtime artifact:

```bash
ste --lexicon /private/path/runtime-lexicon.json version
ste --lexicon /private/path/runtime-lexicon.json lint instructions.md --mode procedural
ste --lexicon /private/path/runtime-lexicon.json dictionary lookup check --format json
```

For persistent local or agent use:

```bash
export STE_LINT_LEXICON=/private/path/runtime-lexicon.json
```

PowerShell:

```powershell
$env:STE_LINT_LEXICON = "C:\private\runtime-lexicon.json"
```

An explicit `--lexicon` takes precedence over `STE_LINT_LEXICON`. Missing or identity-mismatched runtime data exits with code `3` and never silently falls back to test data. Development/public-fixture use can explicitly opt in with `--allow-test-lexicon`.

The verified private runtime identity is metadata-only in Git: 2,196 structural dictionary records, 1,619,057 bytes, SHA-256 `34363ea2c8dc855edb180bb61b180d2dda4556b4bf93bdc89056c1b68639e157`. It contains 208 source-backed approved verb paradigms: 203 lexical verbs, 4 defective modal verbs, and 1 irregular auxiliary. Populated source-derived prose remains outside the public repository.

A verified full dictionary does **not** imply full ASD-STE100 compliance.

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

Inspect executable Issue 9 coverage without requiring the private runtime:

```bash
ste coverage
ste coverage --format json
```

Check whether a proposed rewrite changes protected semantics:

```bash
ste check-rewrite before.txt after.txt --format json
```

Inspect the active runtime lexicon:

```bash
ste dictionary lookup acceptable --format json
```

Inspect built-in terminology profiles and the effective terminology environment for a document:

```bash
ste profile list
ste profile show github --format json
ste glossary effective docs/example.md --format json
```

Validate a project technical glossary:

```bash
ste glossary check .ste/terms.json --format json
```

Exit codes:

| Code | Meaning |
| --- | --- |
| `0` | Clean, accepted, safely fixed, or coverage reported |
| `1` | Error diagnostics remain or a rewrite was rejected |
| `2` | Lint result is blocked by unresolved terminology, grammar, or sense |
| `3` | Required runtime language, glossary, profile configuration, or project-context data is missing or invalid |
| `4` | I/O or internal failure |

## Reusable terminology profiles

A repository can opt into source-backed built-in terminology profiles through the nearest ancestor `.ste/config.json`:

```json
{
  "profiles": ["software-core", "git", "github"]
}
```

The initial built-in profiles have deliberately narrow ownership:

- `software-core` contains common software-engineering concepts intended to be portable across unrelated codebases;
- `git` contains Git version-control concepts;
- `github` contains GitHub work-surface concepts that are not owned by the generic Git profile.

GitHub Actions terminology is not part of the initial `github` profile. A narrower profile can be added later if the corpus demonstrates that need.

Profiles are explicit opt-ins. If `.ste/config.json` is absent, STE-Lint enables no reusable profiles and preserves the existing runtime-plus-project-glossary behavior. Unknown profile IDs, duplicate profile selections, malformed configuration, malformed profile data, or identity conflicts fail closed as invalid data.

Effective terminology is composed in this order:

```text
ASD-STE100 runtime dictionary
        +
selected reusable profiles
        +
repo-local .ste/terms.json
```

The order does not create override precedence. Two sources cannot silently define the same normalized canonical term, alias, or explicit form differently. Canonical duplicates retain `TERM-DUP-001`; conflicting canonical/alias/form identities produce `TERM-ID-CONFLICT-001`.

Profiles extend terminology authority only. They do not exempt profile terms from applicable ASD-STE100 grammar rules, and they do not authorize unknown words automatically. Code and verbatim identifiers are a structural parsing concern rather than vocabulary that should be admitted through a glossary merely to suppress diagnostics.

## Repo-local technical terminology

A repository can extend the effective lexicon with `.ste/terms.json` without changing built-in language data. Exact governed terminology is evaluated before general dictionary approval status because a valid technical noun or technical verb can use a spelling that is not approved for general dictionary use.

```json
{
  "terms": [
    {
      "term": "busway",
      "kind": "technical_noun",
      "definition": "A project term for an electrical distribution assembly.",
      "domain": "electrical",
      "preferred": true,
      "forms": ["busways"],
      "aliases": [],
      "examples": ["Inspect the busway."],
      "provenance": ["project authority"],
      "status": "approved"
    }
  ]
}
```

`kind` can be `technical_noun`, `technical_verb`, or `technical_noun_and_verb` when project or subject-field authority establishes both uses. The current linter does not yet prove noun-versus-verb use from sentence grammar.

`forms` is optional for backward compatibility. When supplied, it is an explicit list of governed grammatical spellings. STE-Lint does not stem terms or generate noun plurals or verb conjugations from a glossary entry. `aliases` are alternate identities for the same governed concept and are not a substitute for forms.

Unknown words are not added automatically. `STE-TERM-001` means the term needs classification, not that it is necessarily wrong. A governed term with `status: deprecated` produces `STE-TERM-002`.

## Repo-local context evidence

When a rule needs a fact that cannot be established safely from raw text, a repository can provide explicit occurrence evidence in the nearest ancestor `.ste/context.json`. A present but malformed context file is an error; STE-Lint does not silently lint without it.

```json
{
  "occurrences": [
    {
      "start": 0,
      "end": 6,
      "source": "terminology review 2026-08-16",
      "spelling": "non_american",
      "official_technical_name": false
    }
  ]
}
```

The current vocabulary supports `dictionary_meaning`, `technical_noun_scope`, and `spelling` facts. Those facts drive bounded checks for Rules 1.3, 1.10, and 1.14 and retain the supplied provenance in diagnostic evidence. They are assertions supplied by project authority, not classifications silently invented by the linter. See `docs/rule-coverage.md` for the exact fields and claim boundary.

## Agent use

`skills/STE.SKILL.md` describes the lint-repair-verify loop and requires coverage inspection before broader compliance claims. `skills/STE-GLOSSARY.SKILL.md` describes reusable-profile and repo-local technical-term maintenance. Mechanical policy belongs in the executable rather than copied into agent prompts.

## Runtime data and ASD-STE100

`data/issue9-runtime.manifest.json` is the public identity contract for the authorized private runtime dictionary. `data/rules.json` is the machine-readable capability contract for all 53 Issue 9 rule IDs. Neither file contains the protected full source prose.

`crates/ste-data/data/test-lexicon.json` remains a small public test dataset. It is not the complete ASD dictionary and requires explicit `--allow-test-lexicon` opt-in for lint or dictionary commands.

`tools/authority-ingest/` contains maintenance tooling that builds and verifies versioned runtime data from authorized source material. Populated source-derived datasets must not be committed to this public repository without an explicit redistribution basis.

## Design

The approved architecture is documented in `docs/superpowers/specs/2026-08-14-ste-lint-design.md`. Execution records for private runtime and executable-rule gates live under `docs/superpowers/plans/`.
