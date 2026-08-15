---
name: using-ste-lint
description: Use when writing or revising machine-consumed technical English in a repository that uses STE-Lint, especially after STE-* or SEM-* diagnostics.
---

# Using STE-Lint

## Overview

Treat `ste` as the authority for checks it implements. Use the model to repair unresolved diagnostics, not to grade its own prose.

A clean lint result is bounded evidence. Before describing what STE-Lint verified, inspect `ste coverage` or `ste coverage --format json`. The coverage command does not require the private runtime lexicon and tracks all 53 Issue 9 rules as `implemented`, `partial`, `context_required`, or `not_implemented`.

Do not convert “no diagnostics from implemented checks” into “ASD-STE100 compliant.” Full Issue 9 compliance is not claimed while any rule is partial, context-dependent, or unimplemented.

## Workflow

1. Run `ste coverage --format json` when the verification scope matters or may have changed.
2. Run `ste lint <file> --format json` with the correct `--mode`.
3. Run again with `--fix` when deterministic fixes are available.
4. Read each remaining diagnostic and its evidence.
5. Make the smallest repair that resolves the diagnostic while preserving meaning.
6. Compare the original and proposed files with `ste check-rewrite <before> <after> --format json`.
7. Run `ste lint` on the proposal again.
8. Stop only when the implemented checks are clean or a diagnostic is explicitly blocked.
9. Report unresolved coverage separately when the task requires a broader STE judgment than the executable manifest supports.

For `STE-TERM-001`, use the technical-glossary workflow instead of inventing a replacement.

## Coverage contract

- `implemented`: the current executable text-level rule scope is covered.
- `partial`: a bounded, source-backed slice is executable; do not generalize beyond it.
- `context_required`: safe evaluation needs grammar, document structure, identity, or domain authority that the current linter does not establish.
- `not_implemented`: no executable check currently exists.

`data/rules.json` is the machine-readable capability authority. Diagnostic-code mappings in that file show which current checks contribute evidence to each rule.

## Repair contract

Preserve modality, epistemic strength, negation, quantities, conditions, actors, object identity, ordering, causal claims, and literal machine identifiers. Do not add remediation, causes, facts, or steps that the source does not state.

A clean destination lint is not sufficient evidence that a rewrite is safe. `ste check-rewrite` checks the change itself.

## Exit codes

- `0`: clean, accepted, or coverage reported successfully
- `1`: error or rejected rewrite
- `2`: unresolved terminology, grammar, or sense blocks classification
- `3`: invalid runtime/glossary data
- `4`: I/O or internal failure

Do not suppress a diagnostic merely to make the command pass.
