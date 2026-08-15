# Diagnostics

Diagnostic codes are the stable external API. ASD-STE100 rule numbers are attached as provenance where applicable, but they are not encoded into stable diagnostic identities.

## Implemented diagnostics

| Code | Severity | Meaning | Rule provenance | Autofix |
| --- | --- | --- | --- | --- |
| `STE-PUNC-001` | error | A semicolon is present. | 8.1 | `;` to `.` |
| `STE-LEN-001` | error | A procedural sentence exceeds 20 words. | 5.1 | none |
| `STE-LEN-002` | error | A descriptive sentence exceeds 25 words. | 6.3 | none |
| `STE-LEX-001` | error | Every runtime dictionary record for a token form is unapproved. | 1.1, 9.2 | none |
| `STE-LEX-002` | blocked | A token form has both approved and unapproved runtime dictionary records, so grammar or sense must be resolved before approval can be determined. | 1.1, 9.2 | none |
| `STE-TERM-001` | blocked | A prose token is absent from both the runtime lexicon and project glossary. | 1.1 | none |
| `TERM-DUP-001` | error | Two glossary entries normalize to the same term identity. | project glossary integrity | none |
| `SEM-MODALITY-001` | error | A proposed rewrite changes protected modality tokens. | STE-Lint repair safety | none |
| `SEM-NEGATION-001` | error | A proposed rewrite changes protected negation tokens. | STE-Lint repair safety | none |
| `SEM-QUANTITY-001` | error | A proposed rewrite changes the ordered numeric-literal sequence. | STE-Lint repair safety | none |

## Current behavior

### Sentence counting

The current sentence pass splits on `.`, `?`, and `!`. Word counting splits Unicode whitespace and trims surrounding ASCII punctuation. Diagnostics identify this implementation as `first_slice_whitespace` in their evidence.

This is not yet the complete ASD-STE100 word-count algorithm.

### Lexical token handling

The lexical pass strips surrounding sentence punctuation before it classifies a token. It then ignores tokens that still contain `_`, `/`, `\\`, `-`, `.`, or a digit. This keeps normal sentence-final words such as `acceptable.` visible to the lexicon while avoiding false prose diagnostics for identifiers, paths, versions, and similar machine tokens such as `occurrence_id`, `path/to/file`, or `1.2`.

A token ignored by this pass is not therefore declared STE-compliant. It is simply outside this lexical check.

The full Issue 9 structural dictionary contains spellings shared by multiple records. STE-Lint preserves every candidate rather than selecting the last record. When all candidates are approved, the lexical approval check passes. When all candidates are unapproved, `STE-LEX-001` is emitted. When approval status differs between candidates, `STE-LEX-002` blocks rather than guessing a part of speech or approved sense.

### Unknown terminology

`STE-TERM-001` is `blocked`, not `error`, because absence from the active runtime lexicon does not establish that a domain term is invalid. A repository may legitimately classify it as a technical noun or technical verb in `.ste/terms.json`.

### Autofix boundary

Only diagnostics carrying explicit `autofix` metadata can be changed mechanically. The current implementation only autofixes semicolons. Lexical substitutions are intentionally not automatic even when the runtime dictionary contains alternatives.

## Planned families

The architecture reserves stable families for POS, morphology, approved senses, syntax, references, relationships, context restrictions, style, additional glossary integrity, and semantic repair safety. A reserved family does not imply that its checks are implemented yet.
