# STE semantic task layer

GitHub #68 Gate 5 introduces shared STE-specific semantic facts above the provider-neutral linguistic evidence boundary. These facts are evidence, not standards or project authority.

## Small shared ontology

The first shared supervised task is `ClauseKind` with four values:

- `Condition`
- `Requirement`
- `LimitOrTolerance`
- `WorkStepResult`

The layer deliberately reuses existing repository-owned facts instead of duplicating them. Safety roles stay in the safety analysis model, entity/reference facts stay in the entity model, lexical-sense facts stay in the sense model, and document structure stays in the document graph.

No rule verdict consumes `ClauseKind` in this gate.

## Authority firewall

A semantic provider can propose a candidate fact for a canonical UTF-8 source span. The fact retains the normal provider/model provenance in `AnalysisEvidence`.

Model evidence cannot grant:

- ASD-STE100 lexical approval or permitted forms;
- an approved STE meaning;
- technical-term, abbreviation, or acronym authority;
- project/domain entity identity;
- compliance or violation status.

Repository-owned rule logic remains the only consumer that can combine semantic evidence with verified ASD-STE100 runtime and governed project authority.

## Fail-closed resolution

`resolve_clause_kind` resolves only evidence at the requested canonical source span.

The default minimum confidence is `0.80`.

- no qualifying evidence -> `Unknown`;
- one unique qualifying candidate -> `Resolved`;
- two or more distinct qualifying candidates -> `Ambiguous`;
- a qualifying alternative analysis prevents false resolution;
- invalid policy thresholds -> `Unknown`.

Relative score differences do not override a material semantic conflict. This is intentionally more conservative than selecting the highest-scoring class.

## Source-safe evaluation contract

The Gate 5 classifier corpus is independently authored/synthetic and must remain separate from protected ASD-STE100 source prose and from the real-world benchmark in GitHub #64.

The current characterization used fixed train/dev/test partitions of 56/16/17 examples. The dev-selected conservative decision policy measured test macro-F1 `0.875`. An intentionally ambiguous control exposed a zero-abstention failure for an unconstrained classifier; the conservative resolver increases abstention rather than converting conflicting evidence into truth.

Those measurements are characterization evidence, not a production model selection or coverage-promotion claim. Final provider/runtime selection and any rule promotion still require the representative technical-English benchmark evidence required by GitHub #64.

Future persisted supervised datasets must record, per example:

- stable example identity;
- split (`train`, `dev`, or `test`);
- canonical source text and labeled span;
- independently authored/safely derived provenance;
- one `ClauseKind` label or explicit ambiguity/abstention label;
- hard-negative category when applicable.

Evaluation must report per-class precision, recall, F1, macro-F1, abstention/ambiguity behavior, and technical-domain generalization. Train/dev/test identities must be disjoint and immutable within a reported evaluation.

## Next gate

After this substrate is verified, Gate 6 re-audits remaining `partial` and `context_required` rules. A rule may consume shared semantic facts only with explicit evidence/claim semantics, authority-firewall tests, representative benchmark evidence, and fail-closed behavior below the accepted evidence threshold.
