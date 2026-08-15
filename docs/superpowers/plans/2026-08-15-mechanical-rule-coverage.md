# Issue 9 Mechanical Rule Coverage Execution Record

**Goal:** Replace the first-slice whitespace sentence counter with a source-grounded Issue 9 mechanical structure model, then implement deterministic adjacent rules that do not require speculative grammar.

## Implemented structure model

`crates/ste-lint/src/structure.rs` separates two concepts that Issue 9 uses differently:

1. **word-limit sentence units** for Rules 5.1 / 6.3 with counting semantics from 8.4–8.7;
2. **paragraph prose-sentence count** for Rule 6.6.

The word-limit analyzer implements deterministic text-level behavior for:

- normal `.?!` sentence boundaries while preserving decimal points and common abbreviation periods;
- vertical-list introductions and list items as independent word-limit units when a colon introduces a recognized list;
- parenthetical groups as one outer word plus separately checked inner word-limit units;
- quoted text as one word;
- `No.` plus an alphanumeric identifier as one word;
- numeric values plus recognized units, temperature scales, or clock abbreviations as one word;
- hyphenated word groups as one word.

The analyzer does not guess arbitrary unquoted titles/headings/labels or multiword proper nouns because those require document or identity context that raw prose does not carry. Diagnostics state this limitation explicitly.

## New executable diagnostics

- `STE-SYN-001` — clear contractions prohibited by Rule 4.2. Generic possessive `'s` is not blanket-flagged. No autofix is offered because contraction expansion can be grammatically ambiguous.
- `STE-PARA-001` — descriptive paragraphs over six prose sentences, Rule 6.6. Vertical-list items are not added to this paragraph count even though they are independent units for Rule 8.4 word limits.
- `STE-LEN-001` / `STE-LEN-002` now use `issue9_mechanical_v1` instead of the original whitespace counter.

## Source-grounded regression coverage

Tests cover:

- parenthetical group as one outer word;
- over-limit parenthetical content as its own word-limit unit;
- hyphenated groups;
- number + unit at the exact 20-word boundary;
- quoted multiword text at the boundary;
- `No. 1` grouping;
- decimal punctuation;
- vertical-list prefix/item independence;
- an over-limit vertical-list item;
- six versus seven descriptive paragraph sentences;
- seven list items not inflating the Rule 6.6 prose-sentence count;
- straight and curly contractions;
- possessive apostrophe distinction;
- ambiguous `'d` contractions remaining non-autofixable.

A behavioral RED run isolated an early paragraph bug: the list detector treated a normal prose line beginning `USE.` as an alphabetic list label. The detector was corrected to accept numeric labels or a single alphabetic label, preserving `1.` / `A.` lists without swallowing ordinary prose.

## Verification boundary

Repository CI covers authority/compiler tests, format, Clippy with warnings denied, and the full Rust workspace suite. This gate advances trustworthy mechanical coverage; it does not claim that all of Rule 8.6 is inferable from unstructured text or that grammar-dependent Issue 9 rules are implemented.

## Successor work under GitHub Issue #3

1. part-of-speech and allowed-form enforcement for general and technical terms;
2. approved-meaning and restriction semantics;
3. grammar-dependent procedural/descriptive rules;
4. reference and relationship rules;
5. explicit per-rule coverage evidence toward all 53 rules.
