# Issue 9 rule coverage

STE-Lint tracks all 53 ASD-STE100 Issue 9 writing-rule identifiers in `data/rules.json`.

This manifest is the machine-readable capability and evidence contract. It does not copy protected rule prose. Each rule records its conservative execution status, stable diagnostic codes, repository evidence artifacts, unresolved requirements, and the exact scope of the claim that STE-Lint can currently support.

## Inspect coverage

```bash
ste coverage
ste coverage --format json
```

Coverage inspection does not require `--lexicon` or `STE_LINT_LEXICON`. This lets CI, agents, and reviewers inspect verifier capability before a protected runtime artifact is available.

The human summary reports status counts and always states that full Issue 9 compliance is not claimed. JSON output exposes every rule entry, including its evidence and remaining gaps.

## Per-rule evidence fields

Every rule entry contains:

- `status`: `implemented`, `partial`, `context_required`, or `not_implemented`;
- `diagnostic_codes`: stable executable diagnostics currently associated with the rule;
- `evidence_artifacts`: repository paths that demonstrate an executable slice; `partial` and `implemented` rules must cite at least one existing path;
- `unresolved_requirements`: what still prevents a broader claim; this must be non-empty for every rule that is not `implemented`;
- `claim_scope`: the bounded statement that the current status actually supports.

CI verifies that the manifest contains all 53 unique rule IDs, that only Rules 8.5 and 8.7 are currently marked `implemented`, that executable rules cite real repository paths, and that incomplete rules explicitly state what remains unresolved. Runtime validation also rejects structurally incomplete evidence metadata.

An evidence path is proof of implementation or regression coverage, not proof that every semantic application of the source rule is decidable. The `claim_scope` and `unresolved_requirements` fields define that boundary.

## Status semantics

- `implemented`: complete for the rule's stated `claim_scope`, with executable evidence and no unresolved requirements inside that scope.
- `partial`: a bounded source-backed slice is executable and evidenced, but additional applications remain explicitly unresolved.
- `context_required`: no safe automatic verdict is available without grammar, document structure, identity, terminology authority, discourse, or domain semantics identified in `unresolved_requirements`.
- `not_implemented`: no executable check currently establishes the source-audited rule requirement; the remaining evidence or implementation condition stays explicit in `unresolved_requirements`.

A `not_implemented` entry is an explicit capability gap, not a silent pass. Reducing that count does not mean full standards implementation: `partial` and `context_required` entries also preserve unresolved source-defined applications.

## Project context evidence

Some Issue 9 decisions cannot be inferred safely from raw text. A project can provide explicit evidence in the nearest ancestor `.ste/context.json`. The CLI discovers that file using the same nearest-project-file rule as `.ste/terms.json`. A present but malformed context file is invalid data and causes exit code 3; STE-Lint never silently ignores it.

Occurrence evidence can supply bounded facts for dictionary meaning, technical-noun scope, spelling, Rule 8.6 count-group identity, Rule 8.2 hyphen relation, and Rule 8.3 parenthesis use. Paragraph-topic evidence can identify topic spans for the bounded Rule 6.5 check. All supplied facts retain provenance and are validated for byte-range and UTF-8 safety before use.

Example:

```json
{
  "occurrences": [
    {
      "start": 0,
      "end": 6,
      "source": "terminology review 2026-08-16",
      "spelling": "non_american",
      "official_technical_name": false
    },
    {
      "start": 20,
      "end": 54,
      "source": "document identity review 2026-08-16",
      "count_group": "proper_noun"
    }
  ],
  "topics": [
    {
      "start": 60,
      "end": 85,
      "topic": "pump condition",
      "source": "document topic review 2026-08-16"
    }
  ]
}
```

Context facts are assertions supplied by project authority. They are not classifications silently invented by the linter.

## Representative regression evidence

`fixtures/corpus/manifest.json` defines a public synthetic engineering regression corpus. Its cases are written specifically for STE-Lint and are not copied from ASD-STE100 or third-party manuals. The corpus test requires exact outcomes and exact diagnostic-code sets across clean controls, punctuation, lexical errors, blockers, notes, lists, context evidence, word counting, paragraphs, safety, procedures, contractions, and verb morphology.

This corpus improves regression breadth. It does not change a rule's status by itself.

## Current conservative inventory

The 53 rules classify as:

- 2 `implemented`;
- 36 `partial`;
- 12 `context_required`;
- 3 `not_implemented`.

Only Rules 8.5 and 8.7 are marked `implemented`. All other rules carry at least one explicit unresolved requirement. `data/rules.json` is the authority for the exact per-rule claim boundary.

## Claim boundary

A zero-exit lint means that the diagnostics executable for that version, runtime, glossary, and supplied project context found no remaining errors or blockers. It does not mean that all semantic applications of all 53 rules were automatically evaluated.

Any workflow that makes an ASD-STE100 statement must pair the lint result with the coverage manifest and preserve unresolved rule coverage. The manifest field `full_compliance_claimed` remains `false`.