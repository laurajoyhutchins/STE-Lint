---
name: maintaining-ste-technical-glossaries
description: Use when STE-Lint reports unknown project terminology or when editing a repository's .ste/terms.json technical glossary.
---

# Maintaining STE Technical Glossaries

## Overview

The project glossary is a governed extension to the built-in runtime lexicon. An unknown token is a classification request, not permission to add a term.

## Workflow

1. Inspect the `STE-TERM-001` diagnostic and the source context.
2. Find repository evidence that establishes the term's identity, meaning, and domain use.
3. Classify it as `technical_noun`, `technical_verb`, or not a legitimate technical term.
4. If it is legitimate, update `.ste/terms.json` with definition, domain, preferred status, aliases, examples, provenance, and lifecycle status.
5. Run `ste glossary check .ste/terms.json --format json`.
6. Run `ste lint` on the original text again.

## Constraints

Do not add a term only because STE-Lint does not know it. Do not use an alias as a second canonical identity. Preserve evidence in `provenance`. Prefer one stable term for one project concept.

`TERM-DUP-001` means two entries normalize to the same identity. Resolve the canonical term instead of suppressing the diagnostic.
