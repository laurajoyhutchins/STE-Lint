---
name: maintaining-ste-technical-glossaries
description: Use when STE-Lint reports unknown technical terminology, when selecting reusable terminology profiles, or when editing a repository's .ste/terms.json technical glossary.
---

# Maintaining STE Technical Glossaries

## Overview

STE-Lint has layered terminology authority. The ASD-STE100 runtime remains the general controlled-language authority. Reusable profiles provide narrow, source-backed subject-field terminology for common domains, and `.ste/terms.json` remains the repository-specific terminology authority.

An unknown token is a classification request, not permission to add a term. A governed technical term can be valid even when the same spelling is unapproved for general dictionary use, but every reusable or repo-local admission needs an explicit technical meaning and source support.

Terminology v2 separates authored evidence from runtime lookup. `ste-terminology/v2` documents are validated and compiled into one normalized glossary index before lint analysis. Downstream passes consume the compiled identity, grammatical-role evidence, domain, lifecycle state, replacement, and source references rather than reinterpreting JSON fields independently.

## Classification hierarchy

Classify an unresolved token in this order:

1. If it is code, a path, an identifier, verbatim syntax, or another literal software entity, treat it as a structural parsing concern rather than glossary vocabulary.
2. If the verified ASD-STE100 runtime resolves the intended ordinary-language meaning, use that authority.
3. If it names a specified software subject-field concept or process, determine whether `software-core` governs that bounded technical meaning.
4. If it is a Git version-control concept, determine whether `git` governs it.
5. If it is a GitHub work-surface or GitHub Actions concept, determine whether `github` governs it.
6. If it is specific to the repository, product, organization, industry, or subject field, govern it in `.ste/terms.json` only when repository or subject-field evidence supports the classification.
7. Otherwise leave the diagnostic blocked. Do not invent authority to make the lint pass.

## software-core admission policy

Treat `software-core` as a shared software subject-field termbase, not as an extension of the general STE dictionary.

Admit a term only when all of these are true:

1. It names a specified software concept or irreducible software process.
2. The concept is stable across unrelated codebases.
3. Ordinary approved STE vocabulary would lose material technical precision or require unnatural circumlocution.
4. The admitted meaning, grammatical role, forms, aliases, and lifecycle status can be bounded explicitly.

Technical verbs receive the stricter test. If an approved general verb communicates the same operation accurately, use the approved verb rather than adding a technical verb.

Corpus frequency can trigger terminology review. Frequency never establishes terminology authority. Do not add a word merely because software engineers use it often or because adding it would reduce `STE-TERM-001` findings.

Prefer one canonical technical term for one concept. Do not admit casual shortening, regional wording, slang, jargon, fashionable developer metaphor, or redundant synonyms merely for convenience.

The durable admission and exclusion decisions for `software-core` are recorded in `docs/software-core-term-decisions.md`. Check that ledger before proposing a new shared term. An explicit exclusion is a decision to preserve, not a backlog of words to re-add.

## Reusable profiles

Repositories opt into built-in profiles through the nearest ancestor `.ste/config.json`:

```json
{
  "profiles": ["software-core", "git", "github"]
}
```

Inspect the available and effective authority before editing terminology:

```bash
ste profile list
ste profile show software-core --format json
ste profile show git --format json
ste profile show github --format json
ste glossary effective path/to/document.md --format json
```

Profiles are explicit opt-ins. No `.ste/config.json` means no reusable profiles. Do not add a profile merely to suppress one diagnostic if its domain is not genuinely applicable to the repository.

The `github` profile includes stable GitHub Actions-specific concepts such as workflows, workflow runs, runners, contexts, expressions, matrix strategies, and dispatch events. Generic software concepts used by Actions remain in `software-core` only when they satisfy the software subject-field admission policy. Do not duplicate identities across profiles.

## Terminology metalanguage

Terminology documentation is metalanguage, not governed target technical prose.

Definitions, admission rationales, source notes, rejected alternatives, grammatical explanations, maintenance instructions, and other material used to define or maintain technical terminology do not have to use only the terminology that they govern or otherwise conform to STE-Lint's target-language restrictions. The documentation needs enough language to explain why a spelling is admitted, rejected, narrowed, or replaced.

This boundary is narrow:

- ordinary target technical prose remains governed normally;
- terminology-maintenance documentation is explanatory metalanguage;
- a compliant example must satisfy the constraints that it claims to demonstrate;
- a non-STE example or counterexample is intentionally outside those constraints and must be identified as such.

Do not implement the metalanguage boundary by adding per-line lint suppressions to a terminology ledger.

## Terminology v2 schema

A repo-local v2 glossary declares its schema and domain once, then provides source-backed terms:

```json
{
  "schema": "ste-terminology/v2",
  "domain": "example-project",
  "sources": {
    "project-spec": {
      "title": "Project specification",
      "reviewed_on": "2026-08-17"
    }
  },
  "terms": [
    {
      "id": "execution-receipt",
      "canonical": "execution receipt",
      "roles": ["noun"],
      "definition": "A durable record of one execution result.",
      "forms": [
        {"text": "execution receipts", "roles": ["noun"]}
      ],
      "aliases": [
        {"text": "receipt record", "kind": "short_form"}
      ],
      "sources": [
        {
          "source": "project-spec",
          "supports": ["admission", "definition", "role", "forms", "alias", "status"]
        }
      ],
      "status": "approved"
    }
  ]
}
```

`id` is the stable concept identity. `canonical` is the preferred display spelling and can change without requiring consumers to invent a new concept identity. `roles` is a set containing `noun`, `verb`, or both. Do not add grammatical roles that project or subject-field evidence does not support.

`forms` contains explicit governed spellings and the grammatical roles each spelling can represent. One spelling can legitimately retain more than one role when the evidence supports the ambiguity. STE-Lint does not stem terms or generate plurals, participles, conjugations, or other morphology.

`aliases` are structured alternate identities. Allowed kinds are `abbreviation`, `acronym`, `short_form`, `synonym`, and `legacy`. An alias is not a substitute for a grammatical form.

`status` is `approved` or `deprecated`. Do not use a separate preferred flag. A deprecated entry can name an explicit `replacement` term ID when the authority establishes the replacement relationship.

Source references are structured. `supports` can contain `admission`, `definition`, `role`, `forms`, `alias`, and `status`. Do not claim a source supports evidence that it does not establish. Prior inclusion in a STE-Lint curated baseline is history, not independent admission authority.

Reusable profiles use the same term schema. Their top-level `profile.version` is the vocabulary revision, while `schema: ste-terminology/v2` identifies the serialization and interpretation contract. Do not conflate those two versions.

## Repo-local workflow

1. Inspect the `STE-TERM-001` diagnostic and the source context.
2. Inspect `ste glossary effective` so you do not duplicate terminology already governed by an inherited profile.
3. Classify the spelling against the verified ASD-STE100 runtime and applicable reusable profiles before proposing new authority.
4. Find repository or subject-field evidence that establishes the concept identity, meaning, roles, forms or aliases, and lifecycle status you intend to claim.
5. If it is legitimate and repo-specific, update `.ste/terms.json` using the v2 schema and only the evidence that is actually established.
6. Run `ste glossary check .ste/terms.json --format json`.
7. Run `ste glossary effective` again to verify that profile and project terminology compile without identity conflicts.
8. Run `ste lint` on the original text again with the verified Issue 9 runtime configured.

A bounded legacy `.ste/terms.json` input can still be read for compatibility, but it is compiled into the same runtime glossary. Do not create new legacy-format glossaries.

## Compiled identity behavior

Composition is fail closed. Canonical spellings, aliases, and forms are normalized once into the compiled glossary index. A repo-local glossary cannot silently override a reusable profile, and one profile cannot silently override another.

`TERM-DUP-001` means two entries normalize to the same canonical identity. `TERM-ID-CONFLICT-001` means a canonical spelling, alias, or explicit form collides with another governed identity. Stable term IDs, role/form evidence, source references, and replacement relationships also validate before linting proceeds.

Recognition as a technical term does not exempt the text from applicable ASD-STE100 grammar rules. Vocabulary identity and grammatical legality are separate questions. Preserve genuine ambiguity when the terminology evidence allows more than one grammatical role; do not choose the convenient interpretation merely to make lint pass.

## Constraints

Do not add a term only because STE-Lint does not know it. Prefer one stable ID for one technical concept. Do not convert common developer slang into `software-core` or a project glossary without a precise technical meaning and authority. Do not add heuristic morphology, automatic terminology admission, or local precedence rules.

A glossary entry with `status: deprecated` produces `STE-TERM-002` when used. If a governed replacement exists, record its stable term ID so diagnostics can preserve that evidence; semantic rewrite safety remains a separate decision.
