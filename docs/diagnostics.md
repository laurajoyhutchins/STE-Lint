# Diagnostics

Diagnostic codes are the stable external API. ASD-STE100 rule numbers are attached as provenance where applicable, but they are not encoded into stable diagnostic identities.

## Implemented diagnostics

| Code | Severity | Meaning | Rule provenance | Autofix |
| --- | --- | --- | --- | --- |
| `STE-PUNC-001` | error | A semicolon is present. | 8.1 | `;` to `.` |
| `STE-LEN-001` | error | A procedural sentence exceeds 20 words. | 5.1 | none |
| `STE-LEN-002` | error | A descriptive sentence exceeds 25 words. | 6.3 | none |
| `STE-LEX-001` | error | Every runtime dictionary record for a matched word or phrase form is unapproved and no approved project technical-term identity governs the match. | 1.1, 9.2 | none |
| `STE-LEX-002` | blocked | A matched word or phrase form has both approved and unapproved runtime dictionary records, so grammar or sense must be resolved before approval can be determined. | 1.1, 9.2 | none |
| `STE-TERM-001` | blocked | A prose token is absent from both the runtime lexicon and project glossary after phrase matching. | 1.1 | none |
| `STE-TERM-002` | error | A matched technical term is deprecated by the project glossary. | project terminology governance | none |
| `TERM-DUP-001` | error | Two glossary entries normalize to the same term identity. | project glossary integrity | none |
| `SEM-MODALITY-001` | error | A proposed rewrite changes protected modality tokens. | STE-Lint repair safety | none |
| `SEM-NEGATION-001` | error | A proposed rewrite changes protected negation tokens. | STE-Lint repair safety | none |
| `SEM-QUANTITY-001` | error | A proposed rewrite changes the ordered numeric-literal sequence. | STE-Lint repair safety | none |

## Current behavior

### Sentence counting

The current sentence pass splits on `.`, `?`, and `!`. Word counting splits Unicode whitespace and trims surrounding ASCII punctuation. Diagnostics identify this implementation as `first_slice_whitespace` in their evidence.

This is not yet the complete ASD-STE100 word-count algorithm.

### Lexical token and phrase handling

The lexical pass strips surrounding sentence punctuation before it classifies prose. It ignores tokens that still contain `_`, `/`, `\\`, `-`, `.`, or a digit. This keeps normal sentence-final words such as `acceptable.` visible to the lexicon while avoiding false prose diagnostics for identifiers, paths, versions, and similar machine tokens such as `occurrence_id`, `path/to/file`, or `1.2`.

A token ignored by this pass is not therefore declared STE-compliant. It is simply outside this lexical check.

Before classifying individual tokens, STE-Lint performs longest-match lookup over adjacent alphabetic tokens separated only by whitespace. This lets approved multiword dictionary or glossary forms suppress false component diagnostics and lets unapproved multiword dictionary forms produce one diagnostic over the full phrase. Phrase matching never crosses punctuation.

### Project technical-term authority

An exact project glossary identity is evaluated before the general dictionary status for the same word or phrase. This is necessary because STE permits valid technical nouns and technical verbs that are absent from the dictionary or unapproved for general use. An approved governed technical term therefore satisfies this lexical gate even if the same spelling has an unapproved dictionary record.

This precedence is narrow. It applies only to an exact canonical term or governed alias. It does not automatically add unknown words, infer new terminology, or prove that a technical noun is used as a noun or a technical verb is used as a verb. Those grammatical checks remain successor work.

Glossary kinds are `technical_noun`, `technical_verb`, and `technical_noun_and_verb`. The combined kind exists for domain terms whose authority establishes both uses. A governed term marked `deprecated` emits `STE-TERM-002` even if the same spelling could otherwise pass a dictionary lookup.

### Dictionary ambiguity

The full Issue 9 structural dictionary contains forms shared by multiple records. STE-Lint preserves every candidate rather than selecting the last record. When all candidates are approved, the lexical approval check passes. When all candidates are unapproved, `STE-LEX-001` is emitted. When approval status differs between candidates, `STE-LEX-002` blocks rather than guessing a part of speech or approved sense.

### Unknown terminology

`STE-TERM-001` is `blocked`, not `error`, because absence from the active runtime lexicon does not establish that a domain term is invalid. A repository may legitimately classify it as a technical noun, technical verb, or both in `.ste/terms.json`.

### Autofix boundary

Only diagnostics carrying explicit `autofix` metadata can be changed mechanically. The current implementation only autofixes semicolons. Lexical substitutions are intentionally not automatic even when the runtime dictionary contains alternatives.

## Planned families

The architecture reserves stable families for POS, morphology, approved senses, syntax, references, relationships, context restrictions, style, additional glossary integrity, and semantic repair safety. A reserved family does not imply that its checks are implemented yet.
