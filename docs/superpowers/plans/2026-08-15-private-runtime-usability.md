# Verified Private Runtime Usability Execution Record

**Goal:** Make STE-Lint usable on real technical prose with the verified private ASD-STE100 Issue 9 runtime corpus without publishing that corpus.

## Landed design

`ste-data` owns the runtime identity contract. `data/issue9-runtime.manifest.json` records only the authorized runtime's hashes and cardinalities. `RuntimeLexicon::verified_issue9_from_bytes` rejects bytes that do not match that identity and then validates parsed Issue 9 metadata, retained-source/private-bundle provenance, declared cardinalities, total structural records, and approval counts derived from the entries.

`ste-cli` owns selection. `--lexicon <PATH>` takes precedence over `STE_LINT_LEXICON`. A configured file that is missing or fails verification returns exit code 3 and never falls back. With no selector, the small embedded public lexicon remains available as explicitly identified test/development data.

The runtime compiler recovers source-listed verb forms from the retained source word-cell line structure. Private corpus review found five records where the earlier normalized intermediate form array had collapsed adjacent inflections or removed a meaningful parenthesis. The compiler now reconstructs those explicit forms without inventing morphology.

The lexical pass and dictionary command use multi-candidate form lookup. The lexical pass performs longest-match phrase lookup over adjacent prose tokens before component classification, including project glossary phrases. When all records for a matched form are approved, the lexical approval check passes. When all are unapproved, `STE-LEX-001` is emitted. When approved and unapproved records share the form, `STE-LEX-002` blocks pending grammatical or sense resolution rather than guessing. Phrase matching does not cross punctuation.

## Verification

Repository-owned verification passes on the coherent implementation candidate:

- authority/compiler Python tests, including source-cell verb form recovery;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`, including approved/unapproved phrase and punctuation-boundary cases.

Independent private recompilation from the retained authority inputs is byte-identical across runs and confirms:

- 1,538,351 bytes;
- SHA-256 `1e8e016bbdebce02483c95743183d479fa19e23214edcd3524817a66f3e08c22`;
- 2,196 structural records;
- 878 approved structural records;
- 1,318 unapproved structural records;
- 193 normalized forms with multiple structural candidates;
- 399 candidate record instances in those collisions;
- 70 ambiguous forms that cross approved/unapproved status.

The corrected private runtime replaces the previous derivative in the private Library. No populated source-derived Issue 9 dictionary data is committed publicly.

## Remaining Issue #3 gates

This gate makes the full verified dictionary operational, but it does not establish full ASD-STE100 compliance. Remaining work stays under GitHub Issue #3:

1. resolve POS/sense-dependent dictionary semantics beyond conservative blocking;
2. make approved meanings, restrictions, alternatives, and technical-noun/technical-verb references executable where required;
3. implement and verify the remaining writing-rule families toward all 53 rules;
4. only claim full Issue 9 compliance when dictionary semantics and all 53 rule behaviors have explicit executable evidence.
