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

CI verifies all 53 unique rule IDs, the exact implemented-rule set, repository evidence paths, conservative unresolved requirements, and the status totals. Runtime validation also rejects structurally incomplete evidence metadata.

An evidence path is proof of implementation or regression coverage, not proof that every semantic application of the source rule is decidable. The `claim_scope` and `unresolved_requirements` fields define that boundary.

## Status semantics

- `implemented`: complete for the rule's stated `claim_scope`, with executable evidence and no unresolved requirements inside that scope.
- `partial`: a bounded source-backed slice is executable and evidenced, but additional applications remain explicitly unresolved.
- `context_required`: no safe automatic verdict is available without grammar, document structure, identity, terminology authority, discourse, or domain semantics identified in `unresolved_requirements`.
- `not_implemented`: no executable check currently establishes the source-audited rule requirement; the remaining evidence or implementation condition stays explicit in `unresolved_requirements`.

A `not_implemented` entry is an explicit capability gap, not a silent pass. The current manifest has zero entries in this state. This does not mean full standards implementation: `partial` and `context_required` entries preserve unresolved source-defined applications.

## Governed identity and context evidence

Some Issue 9 decisions cannot be inferred safely from raw text. A project can provide governed authority in the nearest ancestor `.ste/context.json`. The CLI discovers that file using the same nearest-project-file rule as `.ste/terms.json`. A present but malformed context file is invalid data and causes exit code 3; STE-Lint never silently ignores it.

The context model has three distinct kinds of authority that are intentionally not interchangeable:

- global named-entity authority defines stable IDs, canonical forms, explicit alternate forms, provenance, and the proper-noun class `person`, `group`, `organization`, or `geopolitical_entity`;
- global measurement-unit authority defines stable unit IDs, canonical forms, explicit alternate forms, and provenance;
- occurrence authority supplies bounded document facts such as dictionary meaning, technical-noun scope, spelling, Rule 8.2 hyphen relation, Rule 8.3 parenthesis use, explicit Rule 9.3 phrasal-verb classification, count-group structure, or text-authority boundaries.

Named entities are not inferred by probabilistic NER. Unknown words after numbers are not guessed to be measurement units. Ambiguous governed surfaces fail closed instead of being assigned arbitrary identity.

`text_authority` distinguishes text that is not STE-authored from structural text that merely has special counting semantics. Protected text, immutable quoted external text, code/verbatim text, formulas, and document numbering can be excluded from authored-text rules when explicitly governed. Titles, placards, and labels can count as one word without thereby becoming exempt from Rule 8.1. Authored quotation remains STE-authored text.

All supplied facts retain provenance and are validated for byte-range and UTF-8 safety before use. Context facts are assertions supplied by project authority. They are not classifications silently invented by the linter.

Example:

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

Paragraph-topic evidence can additionally identify topic spans for the bounded Rule 6.5 check. Semantic-ordering evidence can identify an explicit before/after relationship between exact sentence, paragraph, topic, or entity-mention graph targets for the bounded Rule 6.1 check; missing, ambiguous, overlapping, or self-referential targets do not produce an automatic verdict.

## Shared Issue 9 counting model

Rules 5.1, 6.3, 8.4, 8.5, 8.6, and 8.7 consume one canonical count-group projection rather than maintaining parallel counters.

The projection deterministically handles numeric expressions, governed number-plus-unit groups, governed abbreviations and acronyms, alphanumeric identifiers, quoted text, structural headings, explicitly governed titles/placards/labels, governed proper nouns, parenthetical groups, and hyphenated groups. Project authority extends named-entity and measurement-unit identity without changing the counting algorithm.

Rule 8.4 is implemented for the supported source model through the parser-backed semantic list tree: the introducing colon terminates its count unit, and wrapped or nested items become independent count units. This does not decide the separate Rule 4.3 editorial question of when prose is complex enough to require a vertical list.

## Deterministic grammar evidence

Generic syntax and morphology are evidence layers, not STE authority. CommonMark supplies document structure, and the pinned Harper/Brill stack supplies deterministic generic token, part-of-speech, chunk, and morphology evidence. Approval, grammatical identity, allowed forms, and terminology identity still come from the verified ASD-STE100 runtime or governed project terminology. If competing authoritative interpretations would produce different rule verdicts, the linter blocks or leaves the application unresolved instead of selecting a heuristic answer.

The deterministic-rule audit deliberately leaves source requirements partial when meaning or editorial judgment remains necessary. Rule 3.7 still requires identifying an action-denoting non-verb and the applicable approved verb. Rule 4.3 still requires deciding that prose is sufficiently complex to require a vertical list. Rule 1.14 still requires a versioned offline American-English spelling authority plus governed external-directive exceptions for a complete claim.

## Rule 8.1 authored-text boundary

Rule 8.1 is enforced as a semicolon prohibition in STE-authored text. The implementation does not treat general English punctuation correctness as an unresolved Rule 8.1 requirement.

The semicolon diagnostic has no automatic replacement. Replacing a semicolon with a period can alter sentence structure or meaning, so the linter reports the occurrence and leaves the repair to an explicit rewrite. Protected immutable external text and code/verbatim boundaries do not leak into adjacent authored text.

## Representative regression evidence

`fixtures/corpus/manifest.json` defines a public synthetic engineering regression corpus. Its cases are written specifically for STE-Lint and are not copied from ASD-STE100 or third-party manuals. The corpus test requires exact outcomes and exact diagnostic-code sets across clean controls, punctuation, lexical errors, blockers, notes, lists, context evidence, word counting, paragraphs, safety, procedures, contractions, and verb morphology.

Rule-specific fixtures for the implemented 2.2, 8.1, and 8.6 work are also independently authored. They exercise identity ordering, alias collisions, protected/authored boundaries, Unicode spans, proper-noun classes, project units, numeric syntax, identifiers, headings, labels, and cross-rule count behavior without embedding protected source text.

## Current conservative inventory

The 53 rules classify as:

- 9 `implemented`;
- 33 `partial`;
- 11 `context_required`;
- 0 `not_implemented`.

The implemented rules are 2.2, 5.1, 6.3, 6.6, 8.1, 8.4, 8.5, 8.6, and 8.7. Rules 5.1 and 6.3 use the completed shared Issue 9 count-group model. Rule 6.6 uses parser-backed paragraph identity, and Rule 8.4 uses the shared semantic list tree and canonical count projection. `data/rules.json` is the authority for every exact per-rule claim boundary.

## Claim boundary

A zero-exit lint means that the diagnostics executable for that version, runtime, glossary, and supplied project context found no remaining errors or blockers. It does not mean that all semantic applications of all 53 rules were automatically evaluated.

Any workflow that makes an ASD-STE100 statement must pair the lint result with the coverage manifest and preserve unresolved rule coverage. The manifest field `full_compliance_claimed` remains `false`.
