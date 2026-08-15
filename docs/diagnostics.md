# Diagnostics

Diagnostic codes are the stable external API. ASD-STE100 rule numbers are attached as provenance where applicable, but they are not encoded into stable diagnostic identities.

## Implemented diagnostics

| Code | Severity | Meaning | Rule provenance | Autofix |
| --- | --- | --- | --- | --- |
| `STE-PUNC-001` | error | A semicolon is present. | 8.1 | `;` to `.` |
| `STE-SYN-001` | error | A clear contraction is present. | 4.2 | none |
| `STE-VERB-001` | error | Direct `HAVE`/`HAS`/`HAD` plus an unambiguous approved past participle makes a prohibited perfect-tense construction. | 3.2, 3.4 | none |
| `STE-VERB-002` | blocked | A direct `HAVE`/`HAS`/`HAD` plus participle-looking form has a competing approved dictionary identity, so grammatical use must be resolved before Rule 3.4 can be asserted. | 3.2, 3.4 | none |
| `STE-NOTE-001` | error | A procedural `NOTE:` sentence starts with an unambiguous approved imperative base form. | 5.5 | none |
| `STE-NOTE-002` | blocked | A procedural `NOTE:` sentence starts with a spelling that can be an approved imperative base form but also has another approved identity. | 5.5 | none |
| `STE-LIST-001` | error | A recognized simple vertical list is not introduced by a colon. | 4.3 | none |
| `STE-LIST-002` | error | A recognized simple vertical-list item starts with a lowercase alphabetic character. | 4.3 | none |
| `STE-LIST-003` | error | A recognized simple vertical-list item ends with a comma or semicolon. | 4.3, 8.1 | none |
| `STE-LIST-004` | error | The last item in a recognized simple vertical list does not end with a period. | 4.3 | none |
| `STE-LEN-001` | error | A procedural sentence unit exceeds 20 words. | 5.1; counting semantics 8.4–8.7 | none |
| `STE-LEN-002` | error | A descriptive sentence unit exceeds 25 words, including a sentence inside a recognized note. | 5.5, 6.3; counting semantics 8.4–8.7 | none |
| `STE-PARA-001` | error | A descriptive paragraph contains more than six prose sentences. | 6.6 | none |
| `STE-LEX-001` | error | Every runtime dictionary record for a matched word or phrase form is unapproved and no approved project technical-term identity governs the match. | 1.1, 9.2 | none |
| `STE-LEX-002` | blocked | A matched word or phrase form has both approved and unapproved runtime dictionary records, so grammar or sense must be resolved before approval can be determined. | 1.1, 9.2 | none |
| `STE-TERM-001` | blocked | A prose token is absent from both the runtime lexicon and project glossary after phrase matching. | 1.1 | none |
| `STE-TERM-002` | error | A matched technical term is deprecated by the project glossary. | project terminology governance | none |
| `TERM-DUP-001` | error | Two glossary entries normalize to the same term identity. | project glossary integrity | none |
| `SEM-MODALITY-001` | error | A proposed rewrite changes protected modality tokens. | STE-Lint repair safety | none |
| `SEM-NEGATION-001` | error | A proposed rewrite changes protected negation tokens. | STE-Lint repair safety | none |
| `SEM-QUANTITY-001` | error | A proposed rewrite changes the ordered numeric-literal sequence. | STE-Lint repair safety | none |

## Current behavior

### Issue 9 mechanical sentence and word counting

Sentence-length diagnostics use the `issue9_mechanical_v1` analyzer rather than the original whitespace counter.

For the 20/25-word limits it implements the deterministic text-level behavior established by Issue 9 Rules 8.4–8.7:

- `.`, `?`, and `!` close normal sentence units, while decimal points and common abbreviation periods are not treated as sentence boundaries;
- when a colon introduces a recognized vertical list, the introductory segment and each list item are independent word-limit units;
- a parenthetical group counts as one word in the outer sentence, while its content is checked as its own word-limit unit;
- quoted multiword text counts as one word;
- `No.` plus an alphanumeric identifier counts as one word;
- numeric values paired with recognized units, temperature scales, or clock abbreviations count as one word;
- hyphenated groups count as one word because they remain one lexical token.

Recognized `NOTE:` paragraphs are removed from the ordinary procedural 20-word pass and their content is checked with the descriptive 25-word limit. The `NOTE:` label itself is not counted as part of the note sentence.

This does not silently infer document semantics that raw prose cannot establish. In particular, arbitrary unquoted titles/headings/labels and multiword proper nouns require document or identity context before they can safely be collapsed to one word under Rule 8.6. Those cases remain explicit limitations rather than guessed classifications.

### Notes in procedures

`STE-NOTE-001` implements a bounded Rule 5.5 check. In procedural mode, each sentence in a blank-line-delimited paragraph beginning with `NOTE:` is inspected for a source-backed approved base verb at its start. Lexical verbs and the irregular auxiliary `BE` can establish an imperative candidate; defective modal auxiliaries are excluded because the same base-spelling test does not establish a command.

When the sentence-initial spelling also has another approved dictionary identity, `STE-NOTE-002` blocks instead of guessing the grammatical role. Note checks have no autofix. This does not yet prove all Rule 5.5 restrictions, such as every possible requirement, limit, or domain-specific instruction expressed without a sentence-initial imperative.

### Simple vertical lists

The simple-list pass recognizes contiguous same-indentation list-item lines using bullets, numeric labels, single-letter labels, and parenthesized alphanumeric labels. For that bounded structure it checks four mechanical Rule 4.3 requirements: a colon before the first item, uppercase item starts, no comma/semicolon item endings, and a period on the final item.

This is deliberately not a full document-list grammar. Nested lists, wrapped continuation lines, article choice, and the sentence-versus-fragment distinction remain context-dependent. No list fix is applied automatically because changing punctuation can change whether an item is a sentence or fragment.

### Paragraph sentence counting

`STE-PARA-001` applies to descriptive writing. Blank-line-delimited paragraphs may contain at most six prose sentences. A vertical-list introduction contributes its prose sentence, but list-item sentence units used for Rule 8.4 word limits do not inflate the Rule 6.6 paragraph sentence count.

### Contractions

`STE-SYN-001` implements the deterministic part of Rule 4.2 by detecting clear English contractions, including straight and curly apostrophe forms. Generic possessive `'s` is not blanket-flagged. Contractions receive no autofix because forms such as `'d` can expand differently depending on grammar and meaning.

### Approved verb paradigms and direct perfect tense

The verified private runtime preserves source-listed roles for approved verbs. Ordinary lexical verbs can identify a listed past participle; exceptional auxiliaries and defective modal verbs are represented separately rather than forced through ordinary morphology.

`STE-VERB-001` implements one deterministic slice of Rules 3.2 and 3.4. It detects a directly adjacent `HAVE`, `HAS`, or `HAD` followed by a source-backed approved past participle. Multiword participles are matched longest-first and the pattern never crosses punctuation. The diagnostic carries no autofix because converting a perfect construction to an allowed tense without changing meaning requires sentence-level interpretation.

If the matched participle spelling also has another approved dictionary identity, `STE-VERB-002` blocks instead of asserting the grammatical role. This is deliberately narrower than a full Rule 3 grammar engine. Progressive constructions, passive/condition distinctions after `BE`, modal-plus-auxiliary constructions, participle-as-adjective validation, and general POS/sense resolution remain separate work.

### Lexical token and phrase handling

The lexical pass strips surrounding sentence punctuation before it classifies prose. It ignores tokens that still contain `_`, `/`, `\\`, `-`, `.`, or a digit. This keeps normal sentence-final words visible to the lexicon while avoiding false prose diagnostics for identifiers, paths, versions, and similar machine tokens.

A token ignored by this pass is not therefore declared STE-compliant. It is simply outside this lexical check.

Before classifying individual tokens, STE-Lint performs longest-match lookup over adjacent alphabetic tokens separated only by whitespace. This lets approved multiword dictionary or glossary forms suppress false component diagnostics and lets unapproved multiword dictionary forms produce one diagnostic over the full phrase. Phrase matching never crosses punctuation.

### Project technical-term authority

An exact project glossary identity is evaluated before the general dictionary status for the same word or phrase. This is necessary because STE permits valid technical nouns and technical verbs that are absent from the dictionary or unapproved for general use. An approved governed technical term therefore satisfies this lexical gate even if the same spelling has an unapproved dictionary record.

This precedence is narrow. It applies only to an exact canonical term or governed alias. It does not automatically add unknown words, infer new terminology, or prove that a technical noun is used as a noun or a technical verb is used as a verb. Those grammatical checks remain successor work.

Glossary kinds are `technical_noun`, `technical_verb`, and `technical_noun_and_verb`. A governed term marked `deprecated` emits `STE-TERM-002` even if the same spelling could otherwise pass a dictionary lookup.

### Dictionary ambiguity

The full Issue 9 structural dictionary contains forms shared by multiple records. STE-Lint preserves every candidate rather than selecting the last record. When all candidates are approved, the lexical approval check passes. When all candidates are unapproved, `STE-LEX-001` is emitted. When approval status differs between candidates, `STE-LEX-002` blocks rather than guessing a part of speech or approved sense.

### Unknown terminology

`STE-TERM-001` is `blocked`, not `error`, because absence from the active runtime lexicon does not establish that a domain term is invalid. A repository may legitimately classify it as a technical noun, technical verb, or both in `.ste/terms.json`.

### Autofix boundary

Only diagnostics carrying explicit `autofix` metadata can be changed mechanically. The current implementation only autofixes semicolons. Contractions, verb constructions, notes, list mechanics, and lexical substitutions are intentionally not automatic when a safe repair can depend on grammar, sense, or context.

## Planned families

The architecture reserves stable families for additional POS and morphology checks, approved senses, syntax, references, relationships, context restrictions, style, additional glossary integrity, and semantic repair safety. A reserved family does not imply that its checks are implemented yet.
