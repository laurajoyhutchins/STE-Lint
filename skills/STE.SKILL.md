---
name: using-ste-lint
description: Use when writing or revising machine-consumed technical English in a repository that uses STE-Lint, especially after STE-* or SEM-* diagnostics.
---

# Using STE-Lint

## Overview

Treat `ste` as the authority for checks it implements. Use the model to repair unresolved diagnostics, not to grade its own prose.

## Workflow

1. Run `ste lint <file> --format json` with the correct `--mode`.
2. Run again with `--fix` when deterministic fixes are available.
3. Read each remaining diagnostic and its evidence.
4. Make the smallest repair that resolves the diagnostic while preserving meaning.
5. Compare the original and proposed files with `ste check-rewrite <before> <after> --format json`.
6. Run `ste lint` on the proposal again.
7. Stop only when the implemented checks are clean or a diagnostic is explicitly blocked.

For `STE-TERM-001`, use the technical-glossary workflow instead of inventing a replacement.

## Repair contract

Preserve modality, epistemic strength, negation, quantities, conditions, actors, object identity, ordering, causal claims, and literal machine identifiers. Do not add remediation, causes, facts, or steps that the source does not state.

A clean destination lint is not sufficient evidence that a rewrite is safe. `ste check-rewrite` checks the change itself.

## Exit codes

- `0`: clean or accepted
- `1`: error or rejected rewrite
- `2`: unresolved terminology blocks classification
- `3`: invalid runtime/glossary data
- `4`: I/O or internal failure

Do not suppress a diagnostic merely to make the command pass.
