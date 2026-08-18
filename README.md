# STE-Lint

STE-Lint is a compiler-style language tool for writing controlled technical English with agent assistance.

**Software status:** STE-Lint is a release-grade, installable CLI for the ASD-STE100 Issue 9 coverage it claims. All 53 Issue 9 rules are represented in the executable coverage contract, and no rule is left as `not_implemented`. Mechanically provable cases are enforced in code; rules that require document, domain, or human judgment are explicitly partial or context-required instead of being guessed. STE-Lint does **not** claim complete automatic ASD-STE100 Issue 9 compliance. Use `ste coverage` to inspect the exact boundary.

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
- exact-pinned CommonMark structure parsing with `pulldown-cmark`, projected back to canonical UTF-8 byte spans;
- exact-pinned, in-process generic English evidence from `harper-core`, behind an authority firewall that prevents parser vocabulary or morphology from granting STE approval;
- ambiguity-preserving, longest-match dictionary and glossary phrase lookup;
- source-backed approved verb paradigms that preserve lexical, irregular-auxiliary, and defective-modal distinctions;
- bounded rejection of source-linked out-of-inventory verb/adjective forms for Rules 1.4 and 3.1;
- explicit project-authority phrasal-verb evidence for the safely enforceable Rule 9.3 slice;
- governed reusable and repo-local technical terminology, including technical nouns, technical verbs, dual-use terms, explicit forms, typed aliases, provenance, and lifecycle status;
- deterministic composition of opt-in `software-core`, `git`, and `github` terminology profiles with repo-local `.ste/terms.json`;
- repo-local `.ste/context.json` authority for governed named entities, measurement units, protected text, document structure, and other bounded decisions that raw text cannot safely establish;
- Rule 2.2 long-technical-noun ordering through governed identity: full form first, then an authorized short form, abbreviation, or acronym;
- Rule 8.1 semicolon detection only in STE-authored text, with protected/code/verbatim/immutable external boundaries and no meaning-changing automatic replacement;
- contraction detection for the deterministic portion of Rule 4.2;
- direct `HAVE`/`HAS`/`HAD` plus approved-past-participle detection for the deterministic portion of Rule 3.4;
- one canonical Issue 9 count-group projection shared by Rules 5.1, 6.3, 8.4, 8.5, 8.6, and 8.7;
- deterministic counting for numeric expressions, governed number+unit groups, abbreviations/acronyms, alphanumeric identifiers, quoted text, headings, governed titles/placards/labels, governed proper nouns, parentheticals, and hyphenated groups;
- procedural `NOTE:` recognition: note sentences use the descriptive 25-word limit, and source-backed sentence-initial imperative candidates are diagnosed or blocked when ambiguous;
- bounded vertical-list mechanics with parser-backed Markdown structure plus STE-specific list forms and punctuation/counting semantics;
- unapproved-word/phrase diagnostics and blockers for unresolved dictionary ambiguity or unknown project terminology;
- semantic rewrite checks for modality, negation, and numeric-literal changes;
- human-readable and JSON CLI output.

The executable deliberately does not guess document semantics that raw prose cannot establish. Article choice, terminology-category approval, discourse quality, safety consequences, document-wide consistency, and other judgment-heavy Issue 9 requirements remain partial or context-required. `ste coverage --format json` is the authority for that boundary.

## Install

STE-Lint is pinned to Rust `1.97.1`. From a repository checkout:

```bash
rustup toolchain install 1.97.1 --profile minimal
cargo +1.97.1 install --path crates/ste-cli --locked
ste --help
```

The installed binary is named `ste`.

For a public-data first run that does not require the private production dictionary:

```bash
ste coverage
ste profile list
ste --allow-test-lexicon version
ste --allow-test-lexicon dictionary lookup USE
```

`--allow-test-lexicon` is an explicit development/demo opt-in. Do not use the embedded test lexicon as production ASD-STE100 authority.

## Build and test

Use the pinned toolchain and committed lockfile:

```bash
cargo +1.97.1 build --workspace --locked
cargo +1.97.1 test --workspace --locked
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --workspace --all-targets --locked -- -D warnings
python -m unittest discover -s tools/authority-ingest -p 'test_*.py' -v
```

CI runs the same pinned Rust verification plus the authority-ingest suite.

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

The built-in profiles have deliberately narrow ownership:

- `software-core` contains common software-engineering and generic-codebase concepts intended to be portable across unrelated codebases;
- `git` contains Git version-control concepts;
- `github` contains GitHub work-surface and GitHub Actions concepts that are not owned by the generic Git or software-core profiles.

The `github` profile includes stable Actions-specific concepts such as workflows, workflow runs, runners, contexts, expressions, matrix strategies, and dispatch events. Generic software concepts used by Actions, such as jobs, artifacts, caches, environments, variables, and secrets, remain owned by `software-core` so profile composition does not duplicate identities.

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

A repository can extend the effective lexicon with `.ste/terms.json` without changing built-in language data. New glossaries use the explicit `ste-terminology/v2` schema. The document declares its domain and source catalog once; each term carries a stable concept ID, canonical spelling, governed grammatical roles, explicit forms or aliases, source support, and lifecycle status.

```json
{
  "schema": "ste-terminology/v2",
  "domain": "electrical",
  "sources": {
    "project-spec": {
      "title": "Project terminology specification",
      "reviewed_on": "2026-08-17"
    }
  },
  "terms": [
    {
      "id": "busway",
      "canonical": "busway",
      "roles": ["noun"],
      "definition": "A project term for an electrical distribution assembly.",
      "forms": [
        {"text": "busways", "roles": ["noun"]}
      ],
      "aliases": [],
      "sources": [
        {
          "source": "project-spec",
          "supports": ["admission", "definition", "role", "forms", "status"]
        }
      ],
      "status": "approved"
    }
  ]
}
```

`id` is the stable concept identity and `canonical` is its preferred display spelling. `roles` is a set containing `noun`, `verb`, or both. `forms` are explicit governed spellings and retain the grammatical roles that each spelling can represent. STE-Lint does not stem terms or generate plurals, participles, conjugations, or other morphology as STE authority.

Aliases are structured alternate identities with one of `abbreviation`, `acronym`, `short_form`, `synonym`, or `legacy`. Sources are structured references. A term source can explicitly support `admission`, `definition`, `role`, `forms`, `alias`, and `status`; do not claim support that the source does not establish.

`status` is `approved` or `deprecated`. There is no separate preferred flag. A deprecated term can name a stable `replacement` term ID when authority establishes the relationship. Examples are optional.

Reusable profiles use the same term schema. `schema: ste-terminology/v2` identifies the serialization contract, while `profile.version` identifies the vocabulary revision. Those versions are independent.

Terminology documents compile into one normalized runtime glossary index before linting. The compiled index owns canonical, alias, and form lookup plus maximum phrase width, and glossary matches retain how a spelling matched and which grammatical roles that spelling can represent. Composition remains fail closed: canonical duplicates retain `TERM-DUP-001`, and canonical/alias/form identity collisions produce `TERM-ID-CONFLICT-001`.

A bounded legacy `.ste/terms.json` reader remains available so existing repositories do not need an immediate migration, but legacy input is compiled into the same runtime model. Do not create new legacy-format glossaries.

Unknown words are not added automatically. `STE-TERM-001` means the term needs classification, not that it is necessarily wrong. A governed term with `status: deprecated` produces `STE-TERM-002`.

## Repo-local context evidence

When a rule needs a fact that cannot be established safely from raw text, a repository can provide governed authority in the nearest ancestor `.ste/context.json`. A present but malformed context file is an error; STE-Lint does not silently lint without it.

Stable named entities and measurement units belong in the document-level registries instead of being repeated as per-occurrence guesses. Named entities carry an explicit class (`person`, `group`, `organization`, or `geopolitical_entity`) plus canonical and alternate forms. Measurement units carry canonical and alternate forms. Identity collisions fail closed.

Occurrence facts remain available for genuinely local evidence, including protected/external text boundaries and explicit structural identities such as titles, placards, and labels. Counting authority and authored-text authority are separate: a title can count as one word while remaining STE-authored and therefore still subject to Rule 8.1.

```json
{
  "named_entities": [
    {
      "id": "example-standards-council",
      "canonical": "Example Aerospace Standards Council",
      "class": "organization",
      "forms": ["EASC"],
      "source": "project terminology authority"
    }
  ],
  "measurement_units": [
    {
      "id": "widget-flux",
      "canonical": "widget flux",
      "forms": ["wf"],
      "source": "project measurement authority"
    }
  ],
  "occurrences": [
    {
      "start": 0,
      "end": 12,
      "source": "document structure authority",
      "text_authority": "title"
    }
  ]
}
```

The context schema carries explicit project facts rather than classifications invented by the linter. STE-Lint does not run probabilistic NER for Rule 8.6 and does not infer that an arbitrary word following a number is a unit. Diagnostic evidence retains supplied provenance. See `docs/rule-coverage.md` for the exact fields and claim boundary.

## Agent use

`skills/STE.SKILL.md` describes the lint-repair-verify loop and requires coverage inspection before broader compliance claims. `skills/STE-GLOSSARY.SKILL.md` describes reusable-profile and repo-local technical-term maintenance. Mechanical policy belongs in the executable rather than copied into agent prompts.

## Runtime data and ASD-STE100

`data/issue9-runtime.manifest.json` is the public identity contract for the authorized private runtime dictionary. `data/rules.json` is the machine-readable capability contract for all 53 Issue 9 rule IDs. Neither file contains the protected full source prose.

`crates/ste-data/data/test-lexicon.json` remains a small public test dataset. It is not the complete ASD dictionary and requires explicit `--allow-test-lexicon` opt-in for lint or dictionary commands.

`tools/authority-ingest/` contains maintenance tooling that builds and verifies versioned runtime data from authorized source material. Populated source-derived datasets must not be committed to this public repository without an explicit redistribution basis.

## Design

The approved architecture is documented in `docs/superpowers/specs/2026-08-14-ste-lint-design.md`. Execution records for private runtime and executable-rule gates live under `docs/superpowers/plans/`.
