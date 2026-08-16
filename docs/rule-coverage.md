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

## Project context evidence

Some Issue 9 decisions cannot be inferred safely from raw text. A project can provide explicit evidence in the nearest ancestor `.ste/context.json`. The CLI discovers that file using the same nearest-project-file rule as `.ste/terms.json`. A present but malformed context file is invalid data and causes exit code 3; STE-Lint never silently ignores it.

Occurrence facts identify a byte span, a non-empty provenance source, and one or more explicit facts. The current occurrence vocabulary supports:

- `dictionary_meaning`: `approved` or `not_approved`, used by the bounded Rule 1.3 check;
- `technical_noun_scope`: `international`, `regional`, `slang`, or `jargon`, used by the bounded Rule 1.10 check;
- `spelling`: `american` or `non_american`, together with `official_technical_name`, used by the bounded Rule 1.14 check.

Paragraph-topic facts are separate because the linter must not infer discourse topics from wording. Each `topics` item identifies a byte span, a non-empty `topic` identity, and a provenance `source`. In descriptive mode, `STE-PARA-002` reports a paragraph only when project-supplied topic evidence resolves more than one distinct topic inside the same blank-line-delimited paragraph. Repeated evidence for the same topic is allowed, and different topics in different paragraphs are allowed.

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
    }
  ],
  "topics": [
    {
      "start": 20,
      "end": 45,
      "topic": "pump condition",
      "source": "document topic review 2026-08-16"
    }
  ]
}
```

Spans must be valid byte ranges for the file being linted and must land on UTF-8 character boundaries. Topic spans must be contained in one paragraph. Context-backed diagnostics retain the supplied provenance. Providing a fact makes only that bounded rule slice executable; it does not imply that STE-Lint inferred the fact or evaluated every semantic use of the rule.

## Current conservative inventory

At this gate the 53 rules classify as:

- 2 `implemented`;
- 29 `partial`;
- 22 `context_required`;
- 0 `not_implemented`.

Only Rules 8.5 and 8.7 are marked `implemented`. This is intentionally strict. A zero `not_implemented` count means every Issue 9 rule has either an executable slice or an explicit context-required boundary. It does **not** mean every rule is fully implemented. Rules 1.3, 1.10, and 1.14 are `partial` because supplied occurrence evidence can drive bounded checks; automatic sense, terminology-scope, and spelling classification remain unresolved. Rule 6.5 is now `partial` because supplied topic evidence can prove multiple distinct topics inside one paragraph; STE-Lint does not infer topics, topic progression, or logical discourse structure. Rules 6.1, 6.2, and 6.4 therefore remain `context_required`. Rules 4.3 and 5.5 remain partial for similarly bounded structural behavior. Sentence-length enforcement remains partial for Rules 5.1 and 6.3 because some Issue 9 one-word categories need document or identity context. Rule 3.4 remains partial because direct perfect-tense constructions are checked while other auxiliary constructions need deeper grammar.

## Claim boundary

A zero-exit lint means that the diagnostics implemented by that version found no remaining errors or blockers. It does not mean that all 53 rules were evaluated.

Any workflow that needs an ASD-STE100 claim must pair the lint result with the coverage manifest and state unresolved rule coverage explicitly. The manifest field `full_compliance_claimed` remains `false` until the repository has evidence sufficient to change that statement.
