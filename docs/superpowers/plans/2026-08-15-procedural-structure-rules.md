# Procedural note and simple-list execution record

Date: 2026-08-15  
Parent: GitHub Issue #3

## Goal

Add source-backed procedural structure checks that can be established from bounded document shape without pretending to solve general sentence grammar.

## Implemented

- Recognize `NOTE:` lines in procedural text without requiring blank-line separation.
- Treat only more-indented following lines as note continuation; peer-level text remains outside the note.
- Apply descriptive-writing sentence length inside notes: 25 words rather than the procedural 20-word limit.
- Add `STE-NOTE-001` for an unambiguous source-backed approved imperative base form at the start of a note sentence.
- Add `STE-NOTE-002` when the same sentence-initial spelling has another approved dictionary identity, so grammar must be resolved instead of guessed.
- Recognize bounded same-indent vertical-list blocks using bullets, numeric/single-letter labels, and parenthesized alphanumeric labels.
- Add mechanical list diagnostics for missing colon introduction, lowercase item starts, comma/semicolon endings, and missing period on the final item.
- Move Rules 4.3 and 5.5 from `context_required` to `partial` in the 53-rule coverage manifest.

## TDD evidence

Initial behavioral RED: Actions run `31895915871`.

The existing suite passed through format and Clippy; the new tests failed because note/list diagnostics and note-specific length behavior did not yet exist. The implementation then reached a full green run at `31896135883`.

A subsequent review found that blank-line-delimited note detection was too restrictive for real procedures. New tests require a `NOTE:` immediately after a peer step to be recognized and require only indented continuation lines to remain inside the note. The final parser refinement is verified on the documentation-complete candidate before merge.

## Boundary

This is not complete Rule 4.3 or 5.5 coverage.

The list pass does not yet decide sentence versus fragment, article choice, nested/wrapped lists, or more complex document-list semantics. The note pass does not prove every possible requirement or limit expressed without a sentence-initial imperative. Unindented wrapped note continuation is not inferred because doing so could swallow the next procedural instruction.

No new autofixes are introduced. Repairs to note instructions and list punctuation can require grammar or meaning.
