# Diagnostics

Diagnostic codes are the stable external API. ASD-STE100 rule numbers are attached as provenance where applicable, but the rule number is not part of diagnostic identity. `data/rules.json` is the authority for each source-audited rule identity, per-rule coverage, evidence paths, unresolved requirements, and claim scope.

## Current diagnostic codes

| Code | Severity | Meaning | Rule provenance | Autofix |
| --- | --- | --- | --- | --- |
| `STE-PUNC-001` | error | A semicolon is present. | 8.1 | `;` to `.` |
| `STE-PUNC-002` | error | Project evidence explicitly says the words joined by a hyphen are not directly related. | 8.2 | none |
| `STE-PUNC-003` | error | Project evidence explicitly classifies a parenthetical use outside the allowed Rule 8.3 categories. | 8.3 | none |
| `STE-SYN-001` | error | A deterministic contraction form is present. | 4.2 | none |
| `STE-GRAM-001` | error | An approved dictionary word is used in a bounded grammatical role incompatible with its approved part of speech. | 1.2; verbal cases also 3.7 | none |
| `STE-GRAM-002` | error | Shared Grammar v1 uniquely resolves a progressive `-ing` verb use. | 3.5 | none |
| `STE-GRAM-003` | error | Shared Grammar v1 uniquely resolves a passive-verb participle in procedural mode. | 3.3, 3.6 | none |
| `STE-NOUN-001` | error | Shared Grammar v1 uniquely resolves a determiner-led multi-word noun with more than three content words after the determiner. | 2.1 | none |
| `STE-NOUN-002` | error | An approved governed technical noun has a canonical form longer than three words, and an explicit no-more-than-three-word shorter glossary alias appears before the first canonical full-form mention. | 2.2 | none |
| `STE-VERB-001` | error | Direct `HAVE`/`HAS`/`HAD` plus an unambiguous approved past participle forms the bounded prohibited perfect-tense pattern. | 3.2, 3.3, 3.4 | none |
| `STE-VERB-002` | blocked | A participle-looking form in that direct pattern has competing approved dictionary identity, so grammatical use is unresolved. | 3.2, 3.4 | none |
| `STE-PROC-001` | error | A procedural instruction begins with a source-backed lexical verb form that is not its base form. | 5.3 | none |
| `STE-PROC-002` | error | A leading procedural condition lacks the comma that separates it from the command. | 5.4 | none |
| `STE-PROC-003` | error | Shared Grammar v1 uniquely resolves more than one action in a procedural instruction. | 5.2 | none |
| `STE-NOTE-001` | error | A procedural `NOTE:` sentence begins with an unambiguous approved imperative base form. | 5.5 | none |
| `STE-NOTE-002` | blocked | A `NOTE:` opening can be an imperative base form but has a competing approved identity. | 5.5 | none |
| `STE-SAFE-001` | error | A `WARNING:` or `CAUTION:` does not begin with a clear command or condition in the bounded safety-opening model. | 7.2 | none |
| `STE-SAFE-002` | blocked | A safety opening has both an approved base-verb reading and competing approved identity. | 7.2 | none |
| `STE-SAFE-003` | error | A visible `WARNING:`/`CAUTION:` level disagrees with one unambiguous supplied project risk level for the same validated safety block. | 7.1 | none |
| `STE-LIST-001` | error | A recognized simple vertical list is not introduced by a colon. | 4.3 | none |
| `STE-LIST-002` | error | A recognized simple vertical-list item starts with lowercase alphabetic text. | 4.3 | none |
| `STE-LIST-003` | error | A recognized simple vertical-list item ends with a comma or semicolon. | 4.3; semicolon endings also 8.1 | none |
| `STE-LIST-004` | error | The final item in a recognized simple vertical list does not end with a period. | 4.3 | none |
| `STE-LEN-001` | error | A procedural sentence unit exceeds 20 words under the active Issue 9 counting model. | 4.1, 5.1; 8.4–8.7 counting | none |
| `STE-LEN-002` | error | A descriptive sentence unit exceeds 25 words, including recognized note sentences. | 4.1, 6.3; 8.4–8.7 counting | none |
| `STE-PARA-001` | error | A descriptive blank-line paragraph contains more than six prose sentences. | 6.6 | none |
| `STE-PARA-002` | error | Project topic evidence resolves more than one distinct topic inside one descriptive paragraph. | 6.5 | none |
| `STE-LEX-001` | error | Every runtime dictionary record for a matched word or phrase is unapproved. | 1.1 | none |
| `STE-LEX-002` | blocked | A matched word or phrase has both approved and unapproved runtime records, so grammar or sense must be resolved. | 1.1 | none |
| `STE-TERM-001` | blocked | A prose token is absent from both the runtime lexicon and governed project glossary and needs classification. | 1.1 | none |
| `STE-TERM-002` | error | A matched technical term is deprecated by the project glossary. | 1.8 for governed technical nouns; project terminology governance otherwise | none |
| `STE-TERM-003` | error | A governed technical noun is used in the bounded imperative verb position. | 1.7 | none |
| `STE-TERM-004` | error | A governed technical verb is used in the bounded noun position after a determiner. | 1.13 | none |
| `STE-CTX-000` | blocked | Supplied project context is structurally invalid, out of range, not UTF-8 aligned, overlapping where prohibited, or inconsistent with the asserted evidence shape. | context integrity | none |
| `STE-CTX-001` | error | Project authority explicitly resolves a dictionary occurrence to a meaning that is not approved. | 1.3, 9.2 | none |
| `STE-CTX-002` | error | Project authority explicitly classifies a technical noun as regional, slang, or jargon. | 1.10 | none |
| `STE-CTX-003` | error | Project authority explicitly classifies spelling as non-American and the occurrence is not an official technical name. | 1.14 | none |
| `TERM-DUP-001` | error | Two glossary entries normalize to the same term identity. | glossary integrity | none |
| `SEM-MODALITY-001` | error | A proposed rewrite changes protected modality tokens. | rewrite safety | none |
| `SEM-NEGATION-001` | error | A proposed rewrite changes protected negation tokens. | rewrite safety | none |
| `SEM-QUANTITY-001` | error | A proposed rewrite changes the ordered numeric-literal sequence. | rewrite safety | none |

## Behavior boundaries

The linter intentionally blocks or requires explicit project evidence where raw text does not safely establish grammar, sense, document identity, terminology scope, discourse topic, or domain semantics. A diagnostic-free result therefore means only that the executable checks for the active runtime, glossary, and supplied context found no remaining error or blocker.

Issue 9 sentence-length diagnostics use the `issue9_mechanical_v1` analyzer. It handles recognized list boundaries, parenthetical groups, quoted text, identifiers, number-plus-unit groups, decimals, and hyphenated groups mechanically. Explicit Rule 8.6 `count_group` evidence can additionally identify abbreviations, titles, headings, placards, labels, and proper nouns without guessing from typography or capitalization.

`NOTE:`, simple-list, paragraph, procedural, safety, grammar, and entity-semantic diagnostics all operate on bounded structural or governed-identity models. Rule 2.2 currently evaluates only explicit approved glossary aliases for governed long technical nouns; it does not infer aliases or generate hyphen-based clarifications. Rule 3.3 attribution is likewise fail-closed: it is attached only when the existing perfect/passive analyzers prove a past-participle use is non-adjectival. Competing approved adjective identity and other unresolved participle roles do not claim Rule 3.3. Nested or wrapped list grammar, general sentence parsing, topic progression, risk semantics, and other unresolved areas remain visible in `data/rules.json` rather than being inferred heuristically.

The runtime dictionary preserves ambiguity instead of selecting an arbitrary record. Mixed approved/unapproved identity produces `STE-LEX-002`; context-backed or grammar-backed rules likewise block when their required identity cannot be selected safely. An unapproved multi-word dictionary entry is not by itself treated as proof of a Rule 9.3 phrasal verb.

## Autofix boundary

Only diagnostics carrying explicit `autofix` metadata can be changed mechanically. The current implementation only autofixes `STE-PUNC-001` semicolons. Grammar, terminology, verb, note, list, context, safety, entity-semantic, and semantic-rewrite diagnostics do not receive automatic edits when a safe repair can depend on meaning or document context.

## Coverage relationship

A diagnostic code can support more than one rule, and a rule can have more than one diagnostic. The authoritative mapping is `data/rules.json`. Each rule entry carries a non-verbatim `semantic_key` audited against the canonical retained Issue 9 source, plus `evidence_artifacts`, `unresolved_requirements`, and `claim_scope`. This lets the repository detect rule-number drift and distinguish executable proof from unresolved standards work without embedding protected source prose or claiming full Issue 9 compliance.
