---
name: maintaining-ste-technical-glossaries
description: Use when STE-Lint reports unknown project terminology or when editing a repository's .ste/terms.json technical glossary.
---

# Maintaining STE Technical Glossaries

## Overview

The project glossary is a governed extension to the runtime dictionary. An unknown token is a classification request, not permission to add a term.

A governed technical term can be valid even when the same spelling is unapproved for general dictionary use. STE-Lint therefore evaluates an exact project glossary identity before general dictionary approval status. This does not authorize arbitrary exceptions: the glossary entry must have project, industry, or subject-field evidence.

## Workflow

1. Inspect the `STE-TERM-001` diagnostic and the source context.
2. Find repository evidence that establishes the term's identity, meaning, domain use, and lifecycle status.
3. Classify it as `technical_noun`, `technical_verb`, `technical_noun_and_verb`, or not a legitimate technical term.
4. If it is legitimate, update `.ste/terms.json` with definition, domain, preferred status, aliases, examples, provenance, and lifecycle status.
5. Run `ste glossary check .ste/terms.json --format json`.
6. Run `ste lint` on the original text again with the verified Issue 9 runtime configured.

## Constraints

Do not add a term only because STE-Lint does not know it. Do not use an alias as a second canonical identity. Preserve evidence in `provenance`. Prefer one stable term for one project concept.

Use `technical_noun_and_verb` only when project or subject-field authority establishes both grammatical uses. STE permits some domain terms to function in both categories, but the current linter does not yet parse sentence grammar deeply enough to prove noun-versus-verb use automatically.

A glossary entry with `status: deprecated` produces `STE-TERM-002` when used. Remove or replace deprecated terminology rather than relying on a coincidentally approved general-dictionary spelling.

`TERM-DUP-001` means two entries normalize to the same identity. Resolve the canonical term instead of suppressing the diagnostic.
