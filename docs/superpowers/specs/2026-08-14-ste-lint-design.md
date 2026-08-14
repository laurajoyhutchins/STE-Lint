# STE-Lint design

Date: 2026-08-14
Status: approved conversational design, pending implementation

## Purpose

STE-Lint is a compiler-style language tool for writing ASD-STE100-compatible technical English with agent assistance.

The linter owns mechanically checkable compliance. An LLM can propose repairs when a violation cannot be corrected safely by deterministic code. The linter and rewrite validator make the final decision about whether a proposed repair is acceptable.

A released STE-Lint package is self-contained. Agents do not fetch or read the ASD-STE100 PDF during normal use. Source documents are maintenance inputs used to build and verify the versioned runtime language data.

## Product model

Normal use is:

```text
source text
    |
    v
STE linter
    |
    +-- deterministic safe fixes
    |
    +-- structured diagnostics
              |
              v
        LLM repair proposal
              |
              v
        rewrite validator
        - target error resolved
        - no new errors
        - semantic invariants preserved
              |
              v
            lint again
```

The LLM never declares its own output compliant.

## Core principles

1. Mechanical rules belong in code, not in prompts.
2. Diagnostics are a stable API.
3. Autofixes are allowed only when semantics can be preserved mechanically.
4. Semantic repair safety is separate from ASD-STE100 language compliance.
5. ASD rules, general recommendations, and STE-Lint agent-safety rules keep separate provenance.
6. Project technical terminology is repo-local and separately governed.
7. Every discovered bad repair becomes a regression fixture.
8. A released binary or package contains the runtime language data it needs. Runtime use has no PDF dependency.

## Repository architecture

The initial implementation uses a Rust workspace.

```text
STE-Lint/
├── Cargo.toml
├── README.md
├── LICENSE
├── crates/
│   ├── ste-core/
│   ├── ste-data/
│   ├── ste-lint/
│   ├── ste-rewrite-check/
│   ├── ste-glossary/
│   └── ste-cli/
├── data/
│   ├── rules.json
│   ├── general-recommendations.json
│   └── issue9/
├── schemas/
│   ├── dictionary.schema.json
│   ├── glossary.schema.json
│   ├── diagnostic.schema.json
│   └── proposed-change.schema.json
├── fixtures/
│   ├── lint/
│   ├── autofix/
│   ├── bad-repairs/
│   └── glossary/
├── skills/
│   ├── STE.SKILL.md
│   └── STE-GLOSSARY.SKILL.md
├── tools/
│   └── authority-ingest/
└── docs/
    ├── diagnostics.md
    ├── dictionary-model.md
    └── repair-protocol.md
```

The exact crate split can collapse if implementation shows that a boundary has no value. Public JSON contracts must remain language-neutral.

## Runtime language data

`ste-data` owns the versioned structured representation used by the linter.

The runtime dictionary is not a simple word-to-replacement map. It must preserve:

- lemma
- approval status
- part of speech
- permitted forms
- verb class where applicable
- approved senses
- alternative words
- alternative phrases
- sentence-reconstruction alternatives
- technical noun and technical verb references
- context restrictions
- help categories
- paired STE and non-STE examples where useful to a diagnostic
- provenance sufficient to maintain the dataset

Lookup is based on word plus part of speech plus allowed sense, not spelling alone.

The linter must distinguish at least:

- approved use
- unapproved word
- wrong part of speech
- forbidden form
- unapproved sense
- context-restricted use
- unknown term that may be a technical noun or technical verb

Source ingestion and runtime use are intentionally decoupled. `tools/authority-ingest` is a maintenance tool. It is not part of normal lint execution.

Because this repository is public, populated source-derived datasets must have an explicit redistribution basis before they are committed. This constraint must not create a runtime PDF dependency in the product architecture.

## Project technical terminology

Repositories can provide a local technical glossary at:

```text
.ste/terms.json
```

The built-in STE data and the repo-local technical glossary form the effective lexicon for that repository.

A technical term entry includes at least:

- term
- kind: `technical_noun` or `technical_verb`
- definition
- domain
- preferred status
- aliases
- examples
- provenance
- lifecycle status

Unknown words are not automatically added to the glossary. They produce a structured diagnostic that requires classification.

Glossary changes are linted before acceptance. Initial glossary integrity diagnostics include duplicate identity, part-of-speech conflicts, alias conflicts, replacement cycles, invalid examples, undefined dependent terminology, and missing provenance.

## Diagnostics

Diagnostics are structured objects and a stable external contract.

Diagnostic codes use semantic families rather than ASD rule numbers so that the API can survive future standard revisions.

Initial families:

```text
STE-LEX-*     lexical approval
STE-POS-*     part of speech
STE-FORM-*    morphology and permitted forms
STE-SENSE-*   approved sense
STE-TERM-*    technical terminology
STE-SYN-*     sentence construction
STE-REF-*     reference and pronoun ambiguity
STE-REL-*     ambiguous relationships
STE-CTX-*     context restrictions
STE-PUNC-*    punctuation
STE-LEN-*     sentence and paragraph limits
STE-STYLE-*   consistency
TERM-*        technical glossary integrity
SEM-*         rewrite semantic safety
```

Each diagnostic can link to one or more ASD rule identifiers or general recommendations without making those identifiers part of the stable code.

A diagnostic includes:

- code
- severity
- source span
- concise message
- applicable standard rule or recommendation references
- dictionary or glossary evidence when relevant
- candidate alternatives when known
- autofix metadata or `null`

The CLI supports machine-readable JSON output as a first-class interface.

## Severity and outcomes

The linter distinguishes normative rules from recommendations.

- ASD rule violation: error unless the rule implementation requires interpretation and can only be reported as an ambiguity diagnostic.
- ASD general recommendation: warning by default.
- STE-Lint semantic repair-safety violation: error.
- Technical-term uncertainty: blocked diagnostic when the linter cannot determine whether the term is legitimate without project knowledge.

Operational outcomes:

```text
FIXED    a deterministic safe repair was applied
ERROR    repair requires interpretation
BLOCKED  compliance cannot be determined without missing domain information
```

## Autofix boundary

Autofix is a whitelist, not a best-effort rewrite system.

A mechanical fix is permitted only when the linter can establish that the intended proposition does not change.

Typical safe candidates:

- formatting and whitespace normalization
- some punctuation repairs
- deterministic spelling normalization
- exact form normalization where lemma, sense, and grammatical role are already unambiguous

Typical non-autofix cases:

- changing modality
- changing negation
- choosing among multiple lexical alternatives
- resolving pronouns
- resolving ambiguous `with`
- changing part of speech
- restructuring a condition
- selecting an intended sense
- inventing remediation, causes, actors, or missing steps

## Rewrite validation

`ste-rewrite-check` validates a proposed change, not only the resulting destination text.

Input includes:

- original text
- proposed text
- diagnostics the repair intends to resolve

Acceptance requires:

1. the target diagnostics are resolved;
2. the proposal creates no new error diagnostics;
3. protected semantic invariants are preserved.

Initial semantic invariants include:

- modality and epistemic strength
- negation
- quantities and bounds
- actor identity
- object identity
- conditions and exceptions
- temporal ordering
- causal claims
- literal identifiers and machine tokens

Initial semantic diagnostics include:

```text
SEM-MODALITY-001
SEM-NEGATION-001
SEM-QUANTITY-001
SEM-CONDITION-001
SEM-ACTOR-001
SEM-IDENTITY-001
SEM-TEMPORAL-001
SEM-CAUSE-001
```

Some semantic checks can be deterministic. Others can be conservative and return `BLOCKED` rather than guessing.

## Agent workflow

The skill file teaches agents how to operate STE-Lint. It does not attempt to reproduce the STE ruleset in prompt form.

The core workflow is:

1. Run `ste lint`.
2. Apply deterministic fixes.
3. Read remaining structured diagnostics.
4. Make the smallest repair that resolves a diagnostic while preserving meaning.
5. Run `ste check-rewrite` on the proposal.
6. Run `ste lint` again.
7. Stop only when clean or explicitly blocked.
8. Use the glossary workflow for legitimate unknown technical terms.
9. Do not suppress diagnostics merely to make validation pass.

A separate glossary skill handles classification, evidence, and safe maintenance of `.ste/terms.json`.

## CLI

Initial command surface:

```text
ste lint <path> [--fix] [--format human|json]
ste check-rewrite <before> <after> [--format human|json]
ste dictionary lookup <word>
ste glossary check [path]
ste version
```

Suggested exit codes:

```text
0 clean
1 lint violations
2 blocked or unknown terminology
3 invalid language or glossary data
4 internal failure
```

The command surface should stay small until real use demonstrates a need for more commands.

## Testing strategy

The project uses test-driven development for lint rules and repair validation.

Every diagnostic gets positive and negative fixtures.

Fixture classes include:

- compliant text
- one-rule violations
- safe autofixes
- unsafe proposed fixes
- semantic regressions
- glossary mutations
- previously observed agent failures

A bad agent repair becomes a regression fixture containing:

- original input
- original diagnostic
- bad proposed repair
- expected rejection code
- one or more acceptable repairs when useful

Property tests should cover invariants such as:

- applying an autofix twice is idempotent;
- autofix never creates a new error diagnostic;
- JSON diagnostics conform to the published schema;
- glossary compilation is deterministic;
- approved form lookup round-trips to its canonical entry.

## Initial implementation slice

The first usable release does not need every STE rule or every semantic check.

The initial vertical slice should prove the architecture with:

1. Rust workspace and shared diagnostic model.
2. Versioned runtime data schema and a small lawful test lexicon.
3. Repo-local `.ste/terms.json` loading and validation.
4. Mechanical lint passes for semicolons, procedural sentence length, descriptive sentence length, lexical approval against the test lexicon, and unknown technical terms.
5. Safe autofix infrastructure with at least one trivial deterministic fix.
6. JSON and human-readable diagnostics.
7. Rewrite checking for modality, negation, and literal-number changes.
8. Regression fixtures for common agent repair failures.
9. Minimal `STE.SKILL.md` and `STE-GLOSSARY.SKILL.md` that drive the executable workflow.

After this slice works, expand rule coverage and populate the runtime Issue 9 dataset through the maintenance pipeline.

## Non-goals for the first slice

- full natural-language parsing
- claiming perfect semantic equivalence checking
- editor plugins
- hosted service
- MCP server
- automatic glossary mutation
- a prompt-only STE authoring mode

## Acceptance criteria for the first slice

A fresh checkout can build and test without the ASD PDF.

A user can create a text file and run the CLI to receive stable structured diagnostics.

The CLI can apply only whitelisted deterministic fixes.

A repo-local technical glossary affects lint results without modifying built-in language data.

A proposed rewrite that strengthens modality, drops negation, or changes a numeric literal is rejected with an exact semantic diagnostic.

All committed fixtures and tests pass locally.

The README explains that the LLM is a repair backend and the linter is the compliance authority.
