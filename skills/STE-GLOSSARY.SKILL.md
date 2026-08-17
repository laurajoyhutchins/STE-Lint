---
name: maintaining-ste-technical-glossaries
description: Use when STE-Lint reports unknown technical terminology, when selecting reusable terminology profiles, or when editing a repository's .ste/terms.json technical glossary.
---

# Maintaining STE Technical Glossaries

## Overview

STE-Lint has layered terminology authority. The ASD-STE100 runtime remains the general controlled-language authority. Reusable profiles provide narrow, source-backed technical vocabulary for common domains, and `.ste/terms.json` remains the repository-specific terminology authority.

An unknown token is a classification request, not permission to add a term. A governed technical term can be valid even when the same spelling is unapproved for general dictionary use, but every reusable or repo-local admission needs an explicit technical meaning and provenance.

## Classification hierarchy

Classify an unresolved token in this order:

1. If it is code, a path, an identifier, verbatim syntax, or another literal software entity, treat it as a structural parsing concern rather than glossary vocabulary.
2. If the verified ASD-STE100 runtime resolves the intended ordinary-language meaning, use that authority.
3. If it is an established generic software concept, determine whether `software-core` governs the intended technical meaning.
4. If it is a Git version-control concept, determine whether `git` governs it.
5. If it is a GitHub work-surface concept, determine whether `github` governs it.
6. If it is specific to the repository, product, organization, industry, or subject field, govern it in `.ste/terms.json` only when repository or subject-field evidence supports the classification.
7. Otherwise leave the diagnostic blocked. Do not invent authority to make the lint pass.

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

The initial `github` profile does not include GitHub Actions terminology. Treat Actions-specific terms as unresolved unless another applicable authority or repo-local glossary governs them.

## Repo-local workflow

1. Inspect the `STE-TERM-001` diagnostic and the source context.
2. Inspect `ste glossary effective` so you do not duplicate terminology already governed by an inherited profile.
3. Find repository or subject-field evidence that establishes the term's identity, meaning, domain use, and lifecycle status.
4. Classify it as `technical_noun`, `technical_verb`, `technical_noun_and_verb`, or not a legitimate technical term.
5. If it is legitimate and repo-specific, update `.ste/terms.json` with definition, domain, preferred status, explicit forms where established, aliases, examples, provenance, and lifecycle status.
6. Run `ste glossary check .ste/terms.json --format json`.
7. Run `ste glossary effective` again to verify that profile and project terminology compose without identity conflicts.
8. Run `ste lint` on the original text again with the verified Issue 9 runtime configured.

## Forms and aliases

`forms` is an explicit list of governed grammatical spellings. Do not infer or generate plurals, participles, conjugations, or other morphology merely because the canonical term is governed.

`aliases` are alternate identities for the same technical concept. Do not use an alias as a second canonical identity, and do not use aliases to smuggle unrelated meanings into one entry.

Recognition as a technical term does not exempt the text from applicable ASD-STE100 grammar rules. Vocabulary identity and grammatical legality are separate questions.

## Conflict handling

Composition is fail closed. A repo-local glossary cannot silently override a reusable profile, and one profile cannot silently override another.

`TERM-DUP-001` means two entries normalize to the same canonical identity. Resolve the canonical term instead of suppressing the diagnostic.

`TERM-ID-CONFLICT-001` means a canonical term, alias, or explicit form collides with another governed identity. Determine which authority and concept are correct, then remove the conflicting identity. Do not create precedence rules as a local workaround.

## Constraints

Do not add a term only because STE-Lint does not know it. Preserve evidence in `provenance`. Prefer one stable term for one technical concept. Do not convert common developer slang into `software-core` or a project glossary without a precise technical meaning and authority.

Use `technical_noun_and_verb` only when reusable-profile, project, or subject-field authority establishes both grammatical uses. The current linter does not yet parse sentence grammar deeply enough to prove noun-versus-verb use automatically.

A glossary entry with `status: deprecated` produces `STE-TERM-002` when used. Remove or replace deprecated terminology rather than relying on a coincidentally approved general-dictionary spelling.
