# Verb paradigm and direct Rule 3.4 execution record

Date: 2026-08-15  
Parent: GitHub Issue #3

## Goal

Promote source-listed approved verb morphology into an executable runtime contract without treating a flat spelling list as sentence grammar, then use that contract for a narrow Rule 3.4 check that can be asserted from text and dictionary evidence alone.

## Implemented

- Added optional `verb_paradigm` data to approved verb entries.
- Preserved the raw ordered source form sequence separately from deduplicated lookup forms.
- Classified approved verbs as `lexical`, `irregular_auxiliary`, or `defective_modal` instead of forcing exceptional auxiliaries through ordinary verb slots.
- Kept unapproved verb records free of approved executable paradigms.
- Regenerated and privately audited the complete Issue 9 runtime: 208 approved paradigms, comprising 203 lexical verbs, 4 defective modal verbs, and 1 irregular auxiliary.
- Re-authorized the private runtime at 1,619,057 bytes, SHA-256 `34363ea2c8dc855edb180bb61b180d2dda4556b4bf93bdc89056c1b68639e157`.
- Added `STE-VERB-001` for directly adjacent `HAVE`/`HAS`/`HAD` plus an unambiguous approved past participle.
- Added `STE-VERB-002` as a blocker when that participle spelling has a competing approved dictionary identity.
- Supported multiword participles longest-first and refused to cross punctuation.
- Added no verb-construction autofix.

## RED findings that changed the design

1. A positional ordinary-verb interpretation incorrectly treated modal-auxiliary forms as ordinary conjugation. The compiler now preserves source sequence and verb class before assigning role fields.
2. A generic trailing-parenthesis cleanup broke legitimate parenthesized phrasal headwords such as the synthetic regression `zorb (from)` and the real unapproved form `prevent (from)`. Parentheses are now stripped only while parsing a source `(also ...)` alternate-form group.
3. The first Rule 3.4 pass enforced whitespace inside multiword participles but not between the auxiliary and participle. The RED punctuation test exposed `HAS, REMOVED`; direct adjacency now requires whitespace at that boundary too.

## Explicit boundary

This gate does not claim complete Rule 3 coverage. In particular, `BE + participle` cannot be blanket-rejected because Issue 9 permits some past participles as adjectives, so passive-versus-condition distinction needs grammatical or semantic context. Progressive constructions, modal-plus-auxiliary patterns, general POS/sense disambiguation, and participle-as-adjective validation remain successor work.

The populated Issue 9 runtime remains private. Git contains only executable code, schemas, tests, counts, hashes, and documentation that do not redistribute the protected dictionary prose.
