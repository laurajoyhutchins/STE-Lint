# Semantic NLP rule-gap and ontology audit

## Purpose

This document is the Gate 1 artifact for GitHub #68. It inventories every unresolved ASD-STE100 Issue 9 rule requirement represented by `data/rules.json`, maps those gaps to reusable evidence classes, and defines the smallest shared semantic vocabulary needed before any richer NLP runtime is selected.

Gate 1 is an architecture and evidence audit only. It does not change rule status, claim scope, diagnostic behavior, runtime authority, or the normal lint dependency surface.

## Authority snapshot

The audit is against repository `main` at `eed24e0f6b9a43b87b0d93ad72a3fd4f9555ed2b`.

- `data/rules.json` represents 53 Issue 9 rules: 40 `partial`, 11 `context_required`, and 2 `implemented` (`8.5`, `8.7`). Therefore 51 rules have at least one unresolved requirement.
- The retained Issue 9 source is identified by SHA-256 `d1f4ea9e7cd6e46b47aa9057209f99e78c0e9cfc4e27a5b07895b05c1a166431`; the verified runtime lexicon is identified by SHA-256 `34363ea2c8dc855edb180bb61b180d2dda4556b4bf93bdc89056c1b68639e157`.
- The public repository intentionally does not contain the protected full source-derived text. This audit uses the repository's verified rule/runtime projections and does not reproduce protected standard prose.
- ASD-STE100 Issue 9 authority, verified runtime dictionary data, and governed project/domain facts remain authoritative. Statistical models are evidence providers, never compliance authorities.

## Evidence classes

The classes come from GitHub #68. A rule can require more than one class; counts therefore overlap.

| Class | Meaning | Rules mapped |
| --- | --- | ---: |
| A | Deterministic compiler or document-structure fact | 9 |
| B | Harper lexical or morphological fact | 11 |
| C | Mature generic NLP syntax, entity, or reference fact | 37 |
| D | STE-specific supervised semantic fact | 27 |
| E | Governed external, standard, project, or domain authority | 24 |
| F | Genuinely editorial or human judgment | 10 |

Class E is intentionally common. A model can help identify a candidate technical term, sense, entity, or risk expression, but it cannot grant technical-noun category eligibility, dictionary approval, approved meaning, official-name status, project risk truth, or other governed authority.

## Per-rule gap classification

The gap summaries below paraphrase the current `unresolved_requirements` fields. The class list describes evidence needed to resolve the whole remaining requirement, not permission to change the current status.

| Rule | Remaining gap | Classes |
| --- | --- | --- |
| 1.1 | Complete technical-term classification and contextual dictionary-use decisions | E, C, D, B |
| 1.2 | General sentence parsing and POS resolution outside bounded frames | C, B |
| 1.3 | Automatic approved-sense disambiguation | D, E |
| 1.4 | Forms Harper cannot link uniquely to an approved lemma, plus grammatical roles outside bounded morphology | B, C, E |
| 1.5 | Issue 9 technical-noun category classification | E, D, C |
| 1.6 | Unknown-word classification as a technical noun or component | E, C, D |
| 1.7 | Technical-noun grammatical role outside the strong imperative frame | C, E |
| 1.8 | Company, industry, or subject-field technical-noun approval not supplied by governed terminology | E |
| 1.9 | Whether a technical noun is short and easy for the intended reader | F |
| 1.10 | International versus regional, slang, or jargon classification | E, D |
| 1.11 | Document-wide item identity and technical-noun consistency | C, D, E |
| 1.12 | Issue 9 technical-verb category classification | E, D, C |
| 1.13 | Technical-verb grammatical role outside the determiner-led frame | C, E |
| 1.14 | Spelling classification plus official-directive recognition | B, E |
| 2.1 | Multi-word noun boundaries outside the bounded Grammar v1 frame, including ambiguous heads | C, B |
| 2.2 | Non-governed noun identity/alias relations, clarification relations, and broader document/reference cases | C, E, A |
| 3.1 | Verb forms generic morphology cannot link uniquely to an approved dictionary lemma | B, C, E |
| 3.2 | Complete allowed/prohibited verb-form and tense recognition | C, B, E |
| 3.3 | Broader participle-adjective recognition and ambiguous participle roles | C, B |
| 3.4 | Complex auxiliary constructions beyond the current direct pattern | C, B |
| 3.5 | Technical-noun or modifier `-ing` roles beyond bounded Grammar v1 evidence | C, B, E |
| 3.6 | Descriptive passive actor-known/unknown semantics and passive/adjectival ambiguity | C, D |
| 3.7 | General action-word role resolution | C, D, E |
| 4.1 | Sentence clarity and structural simplicity beyond length limits | C, F |
| 4.2 | Omitted-word detection and context-sensitive contraction-like forms | C, B |
| 4.3 | Nested/mixed/wrapped lists and sentence-versus-fragment list semantics | A, C |
| 4.4 | Related-topic identity and an appropriate connecting word or phrase | D, C, F |
| 4.5 | Noun/MWN recognition plus contextual applicability of an article or demonstrative | C, F |
| 5.1 | Context-dependent semantic one-word identities used by the count model | A, E |
| 5.2 | Instruction intent, broader action structures, and simultaneous-action semantics | C, D |
| 5.3 | Imperative recognition outside the bounded base-form frame | C, D |
| 5.4 | Condition semantics beyond leading `IF`/`WHEN` structure | C, D |
| 5.5 | Instruction semantics in notes beyond sentence-initial imperative candidates | C, D |
| 6.1 | Information-order inference, gradual progression, and one-subject-per-sentence semantics | D, C, F |
| 6.2 | Key words/phrases that establish logical text structure | D, F |
| 6.3 | Unannotated semantic one-word identities needed by Rule 8.6 | A, C, E |
| 6.4 | Logical grouping of related information into paragraphs | A, D, F |
| 6.5 | Topic inference and topic-sentence progression | D, F |
| 6.6 | Paragraph identity in formats not represented by blank-line prose boundaries | A |
| 7.1 | Risk-level inference, missing labels/symbols, and conflicting/insufficient project risk authority | A, D, E |
| 7.2 | Complete safety command/condition semantics and multi-action analysis | C, D |
| 7.3 | Risk, consequence, explanation, and possible-result semantics | D |
| 8.1 | Other standard-English punctuation decisions beyond the deterministic semicolon slice | C, F |
| 8.2 | Whether hyphenated words are semantically directly related | C, D |
| 8.3 | Semantic classification of parenthetical use | C, D |
| 8.4 | Complex or nested vertical-list boundary semantics | A, C |
| 8.6 | Unannotated titles, headings, placards, labels, abbreviations, and proper nouns | A, C, E |
| 9.1 | Whether replacement is insufficient and selection of a meaning-preserving alternative construction | D, F |
| 9.2 | Approved meaning/context disambiguation for approved words | D, E |
| 9.3 | Phrasal-verb construction plus non-compositional meaning recognition | C, D, E |
| 9.4 | Document-wide terminology/wording consistency for repeated work-step types and concepts | C, D, E |

### Classification consequences

The matrix separates three questions that must not be collapsed:

1. **Can language evidence identify the structure or semantic candidate?** Classes B-D can often answer this.
2. **Can the repository resolve the fact without external authority?** Class E means the answer is no unless governed authority is present.
3. **Is the remaining requirement inherently editorial?** Class F means richer NLP can provide evidence or measurements, but an automatic compliance claim still needs an explicit policy decision and defensible evaluation boundary.

A rule containing E or F must not be promoted merely because a model performs well on a proxy task.

## Existing shared analysis substrate

The current `AnalysisDocument` is already the correct home for shared facts. Gate 2 should evolve it rather than create provider-specific rule APIs.

### Already represented

- Stable UTF-8 byte spans and source-safe word tokens through `AnalysisToken`, `AnalysisSentence`, and `SourceDocument`.
- Verified dictionary and governed glossary matches through `DictionaryMatch`, `GlossaryMatch`, `VerbFormCandidate`, and `Resolution<T>`.
- Cheap Harper observations through the internal `LinguisticTokenEvidence`: lemma, broad POS/membership flags, determiner/conjunction, auxiliary/linking-verb flags, adjective degree, and generic verb morphology.
- Grammar v1 facts through `GrammarSpan`, `NounPhrase`, `SubjectPredicate`, `AuxiliaryChain`, `ParticipleUse`, `IngUse`, and `ActionStructure`.
- Governed entity/reference facts through `EntityIdentity`, `EntityMention`, and bounded `ReferenceLink` resolution.
- Source-safe dictionary sense identity through `SenseIdentity`, `SenseEvidence`, restriction tags, and source provenance.
- Document relationships through `DocumentGraph`, sentence/paragraph/topic/entity nodes, containment/precedence relations, bounded references, and supplied semantic ordering facts.
- Safety semantics through `SafetySemantics`: level, actor, command, hazard, and consequence, each with explicit `Resolved | Ambiguous | Unknown` state.

### Gate 2 deficiency

`LinguisticDocument` currently asks Harper to create the generic word-token stream and then converts Harper character spans back to repository byte spans. That preserves byte-correct diagnostics today, but Harper still owns the first generic linguistic tokenization decision. Gate 2 must invert this dependency: repository-owned canonical source/span identity comes first; Harper and every later provider map evidence onto it.

## Smallest shared semantic ontology

Do not create one model or one bespoke type per rule. Extend the existing substrate with the following reusable facts only when a downstream rule or evaluation task justifies them.

1. **CanonicalSpan / CanonicalToken / Sentence / Paragraph**: repository-owned byte identity and structural containment. These are the coordinate system for every provider.
2. **LexicalObservation**: provider-neutral lemma, broad POS/morphology, and local lexical properties. Harper remains the default cheap provider.
3. **SyntacticRelation**: dependency/constituency-derived relation or span with subject, predicate, modifier, attachment, auxiliary, voice, and clause-role evidence. Existing Grammar v1 facts are deterministic producers of this layer where they resolve.
4. **EntityMention / EntityLink**: entity mention, reference/coreference relation, and candidate identity. Governed identity remains separate from generic NER/coreference evidence.
5. **SenseCandidate**: source-safe lexical sense candidate with evidence and score. Approved meaning remains a runtime-authority comparison, not a model label.
6. **Action / ClauseKind**: reusable command/instruction/descriptive/action structure, including action heads and explicit uncertainty.
7. **Condition / Requirement / LimitOrTolerance / WorkStepResult**: STE-specific proposition roles and relations between their spans/events.
8. **SafetyHazard / SafetyPreventiveAction / SafetyConsequence**: shared safety roles layered onto the existing `SafetySemantics` container.
9. **Topic / DiscourseRelation / TemporalRelation**: document-level topic, relatedness, ordering, explanation, progression, and event-time relationships layered onto `DocumentGraph`.
10. **EvidenceProvenance**: provider/model identity, exact model/config artifact identity, source span(s), confidence/score when meaningful, alternatives, and reproducibility metadata. This is metadata on evidence, not a new source of truth.

The existing `Resolution<T>` contract remains the semantic firewall: `Resolved`, `Ambiguous`, and `Unknown` are outcome states for repository-owned resolvers. Confidence alone must never be coerced into `Resolved`.

## Evidence and authority hierarchy

When facts conflict, use this order rather than provider confidence:

1. **ASD-STE100 Issue 9 and its verified runtime projection** for standard/dictionary authority.
2. **Governed project/domain authority** for project terminology, official technical names, explicit occurrence facts, risk classification, semantic ordering, and other externally supplied facts.
3. **Deterministic repository structure/compiler evidence** for exact spans, markup/document structure, punctuation, counting mechanics, and facts proven by deterministic analyzers.
4. **Harper lexical/morphological evidence** for cheap generic English observations it can support.
5. **Measured generic NLP evidence** for syntax, constituency, NER, coreference, and related mature tasks.
6. **Measured STE-specific model evidence** for shared semantic task candidates.
7. **Editorial/human judgment** where the requirement is not safely reducible to a validated evidence task.

Lower levels can identify candidates or expose conflicts; they cannot override a contradictory higher-level authoritative fact. A model prediction cannot approve a word, meaning, technical term, form, official name, or risk level.

## Ambiguity policy

- Every provider maps its native coordinates to canonical UTF-8 byte spans before its evidence is visible to rule logic.
- Provider token IDs, Python character offsets, transformer subwords, and model-internal coordinates do not escape the provider adapter.
- Evidence retains provenance and alternatives where the provider exposes them.
- A repository resolver may return `Resolved` only when its task-specific acceptance policy is met and there is no materially conflicting higher-authority evidence.
- Multiple plausible analyses become `Ambiguous`; absent or unusable evidence becomes `Unknown`.
- `Ambiguous` and `Unknown` fail closed. They do not silently fall back to a weaker parser if that would change compliance meaning.
- Rule passes consume resolved shared facts, not provider APIs or raw probabilities.
- Model absence, invalid model identity, span-alignment failure, or unsupported language structure is visible and non-authoritative.

## Evaluation requirements before provider selection

Gate 3 must select richer NLP by measured STE-relevant evidence, not reputation. The harness must evaluate at least:

- canonical byte-span alignment correctness, including Unicode and punctuation boundaries;
- POS/morphology parity and disagreement against Harper where both provide evidence;
- dependency and constituency accuracy on the rule-relevant constructions in this matrix;
- NER and coreference/reference accuracy on technical-English entities and repeated mentions;
- ambiguity and failure behavior, including hard negatives and deliberately underdetermined cases;
- technical-English robustness on synthetic/adjudicated cases and the representative real-world benchmark from GitHub #64 before final production selection;
- task-specific precision/recall/F1 or exact-match metrics for STE semantic facts, plus calibration where confidence thresholds are used;
- CPU latency, peak memory, model artifact size, deterministic/reproducible settings, and offline behavior;
- licensing, redistribution, model-card provenance, exact artifact hashing, and feasible hydration from Rust;
- behavior when model artifacts are absent, corrupt, incompatible, or unavailable.

No rule status changes are justified by provider-selection metrics alone. Rule promotion remains a later Gate 6 decision with rule-level evidence and an explicit claim policy.

## Future heuristic cutover candidates

These are deletion candidates only after a richer provider is measured, shadowed, and proven superior for the exact task. Gate 1 does not delete them.

- Grammar v1's bounded noun-phrase, subject/predicate, auxiliary, participle, `-ing`, and coordinated-action recognizers can be replaced selectively by shared syntax evidence when parity and improvement are demonstrated.
- `perfect.rs` still contains bounded participle search/source-form scanning that can shrink after shared syntax/morphology covers the same cases.
- `procedural.rs` retains bounded imperative/condition/safety-opening logic that should migrate to shared `Action`, `ClauseKind`, and `Condition` facts rather than accumulate more patterns.
- `notes.rs` still has local imperative-prefix/word-span scanning and is a clear consumer of shared instruction semantics.
- `entity.rs` currently resolves only governed/official entities and bounded `it`/`its` references; mature NER/coreference evidence should extend the shared entity layer rather than create a second entity subsystem.
- simple list and paragraph mechanics remain deterministic; only their unresolved sentence/fragment, nesting, or relatedness semantics should consume richer evidence.

Do not delete a deterministic heuristic merely because a neural provider can duplicate it. Delete only superseded machinery whose replacement is both more accurate for the required task and compatible with the authority/firewall rules above.

## Explicit non-goals

- No model that emits a final `Rule X violated` compliance judgment.
- No LLM-as-judge compliance path.
- No replacement of Harper for lexical tasks it already supplies cheaply and adequately.
- No model-granted ASD-STE100 dictionary, terminology, sense, form, technical-category, or domain authority.
- No remote inference dependency or implicit model download during normal linting.
- No protected Issue 9 prose or unsafe derivatives in public datasets, fixtures, prompts, or model artifacts.
- No rule-status or coverage-claim change in this gate.
- No one-model-per-rule ontology.
- No silent fallback when richer evidence is absent or ambiguous.

## Gate 1 conclusion and Gate 2 entry condition

All 51 unresolved rules now have an evidence-class mapping. The current shared analysis substrate already contains reusable grammar, entity/reference, sense, document-graph, and safety facts, so the next real gate is not another rule-specific parser. Gate 2 should make canonical source/span identity provider-neutral, wrap Harper as one evidence provider behind that boundary, add first-class provider/model provenance and alternatives, and preserve all existing diagnostics and rule statuses.

Gate 2 is complete only when existing Harper-backed behavior remains compatible, every exposed provider fact is mapped to repository-owned canonical byte spans, and `Resolved | Ambiguous | Unknown` remains fail-closed without any standards-coverage promotion.

## Gate 6 re-audit after the Gate 5 semantic substrate

This section refreshes the promotion boundary after Gates 2 through 5 landed. It is a current rule re-audit, not a rule-status change.

### Current authority snapshot

The re-audit is against repository `main` at `584ded197d1c9b5e581f20df37e29c7ca302a43a`, with `data/rules.json` blob `9511a28b5a1472faf51f916d2faf4ef7dafb32fc`.

The current coverage ledger contains 53 Issue 9 rules:

- 9 `implemented`;
- 33 `partial`;
- 11 `context_required`;
- 44 rules with at least one unresolved requirement.

The implemented rules are `2.2`, `5.1`, `6.3`, `6.6`, `8.1`, `8.4`, `8.5`, `8.6`, and `8.7`. Their existing claim scopes remain unchanged by this re-audit.

Gate 5 added a source-safe `ClauseKind` evidence type for `Condition`, `Requirement`, `LimitOrTolerance`, and `WorkStepResult` plus a fail-closed semantic resolver. No rule pass consumes `ClauseKind` at this snapshot, so that substrate does not by itself justify a coverage promotion.

### Primary promotion disposition

The Gate 1 evidence classes intentionally overlap. Gate 6 needs one primary disposition per unresolved rule so later work cannot mistake model capability for authority. The following classification applies this precedence:

1. If the unresolved requirement contains a genuinely editorial or human-judgment component, classify it as `editorial/human`.
2. Otherwise, if it requires governed standard, project, domain, terminology, risk, or other external authority, classify it as `authority-dependent`.
3. Otherwise, classify it as an `evidence-only candidate`: richer deterministic, generic-NLP, or STE-specific evidence could in principle complete the language-analysis part, subject to rule-level verification and the Gate 6 claim policy.

| Primary disposition | Count | Rules |
| --- | ---: | --- |
| Evidence-only candidate | 14 | `1.2`, `2.1`, `3.3`, `3.4`, `3.6`, `4.2`, `5.2`, `5.3`, `5.4`, `5.5`, `7.2`, `7.3`, `8.2`, `8.3` |
| Authority-dependent | 20 | `1.1`, `1.3`, `1.4`, `1.5`, `1.6`, `1.7`, `1.8`, `1.10`, `1.11`, `1.12`, `1.13`, `1.14`, `3.1`, `3.2`, `3.5`, `3.7`, `7.1`, `9.2`, `9.3`, `9.4` |
| Editorial/human | 10 | `1.9`, `4.1`, `4.3`, `4.4`, `4.5`, `6.1`, `6.2`, `6.4`, `6.5`, `9.1` |

These buckets are a routing decision, not new evidence. A rule can still consume several evidence classes internally.

### Promotion decision

No rule status changes in this re-audit.

For the 14 evidence-only candidates, a future promotion requires direct rule-level positive, negative, boundary, and ambiguity tests; an explicit resolver policy; fail-closed behavior below the accepted evidence threshold; and representative technical-English benchmark evidence when model-backed facts are part of the claim.

For the 20 authority-dependent rules, language models may identify candidates or relationships, but they cannot grant the missing standard, dictionary, terminology, project, domain, official-name, risk, or other governed authority. A complete automatic claim requires the relevant authority to be represented and compared by repository-owned logic.

For the 10 editorial/human rules, richer NLP may provide measurements or candidate evidence, but the current source-defined requirement still includes judgment that is not safely reduced to an automatic verdict. Promotion requires an explicit defensible policy boundary in addition to evidence quality.

The source-safe Gate 5 characterization (`56/16/17`, macro-F1 `0.875`) remains characterization evidence only. GitHub #64 remains the required representative technical-English benchmark source before final production model selection or model-backed coverage promotion. The current Gate 5 semantic facts may therefore be shadowed or wired into repository-owned analysis without changing a rule's claim status, but they must not be treated as compliance authority.

### Gate 6 execution boundary

The next Gate 6 implementation work must preserve these rules:

- migrate rule consumers to shared semantic facts rather than provider APIs;
- keep deterministic rules deterministic;
- do not promote an authority-dependent rule merely because semantic evidence is accurate;
- do not promote an editorial/human rule without an explicit claim-policy decision;
- require representative GitHub #64 evidence before a model-backed status promotion;
- update `data/rules.json`, CLI/docs, and rule tests atomically when a promotion is eventually justified;
- keep `partial` or `context_required` when the full stated requirement is not established.

This re-audit narrows Gate 6 from 44 undifferentiated unresolved rules to 14 evidence-only promotion candidates plus 30 rules whose current completion boundary is authority or editorial judgment. It does not redefine `implemented` and does not change the repository's compliance claim.
