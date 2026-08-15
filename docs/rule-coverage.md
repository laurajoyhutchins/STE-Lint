# Issue 9 rule coverage

STE-Lint tracks all 53 ASD-STE100 Issue 9 writing-rule identifiers in `data/rules.json`.

This manifest is a capability contract, not a copy of the source rules. It stores rule IDs, conservative execution status, and the diagnostic codes that currently provide executable evidence. It does not reproduce protected rule prose.

## Inspect coverage

```bash
ste coverage
ste coverage --format json
```

Coverage inspection does not require `--lexicon` or `STE_LINT_LEXICON`. This lets CI and agents inspect verifier capability before a protected runtime artifact is available.

The human summary reports the current status counts and always states that full Issue 9 compliance is not claimed. JSON output exposes every rule entry for automated policy checks.

## Status semantics

- `implemented`: complete for the current executable text-level scope of that rule.
- `partial`: a bounded source-backed slice is executable, but other applications of the rule remain unresolved.
- `context_required`: safe evaluation requires grammar, document structure, item identity, terminology authority, discourse, or domain semantics that current runtime evidence does not establish.
- `not_implemented`: no executable check currently exists.

The status vocabulary deliberately separates “not implemented” from “cannot be decided safely with current context.” That distinction prevents missing parser capability from being confused with a negative result.

## Current conservative inventory

At this gate the 53 rules classify as:

- 2 `implemented`;
- 17 `partial`;
- 29 `context_required`;
- 5 `not_implemented`.

Only Rules 8.5 and 8.7 are marked `implemented`. This is intentionally strict. Rules 4.3 and 5.5 are now `partial`: simple vertical-list mechanics and bounded note behavior are executable, but nested/wrapped lists, sentence-versus-fragment classification, article choice, and non-imperative ways of expressing note requirements still need deeper structure or grammar. Sentence-length enforcement remains partial for Rules 5.1 and 6.3 because some Issue 9 one-word categories need document or identity context. Rule 3.4 remains partial because direct perfect-tense constructions are checked while other auxiliary constructions need deeper grammar.

## Claim boundary

A zero-exit lint means that the diagnostics implemented by that version found no remaining errors or blockers. It does not mean that all 53 rules were evaluated.

Any workflow that needs an ASD-STE100 claim must pair the lint result with the coverage manifest and state unresolved rule coverage explicitly. The manifest field `full_compliance_claimed` remains `false` until the repository has evidence sufficient to change that statement.
