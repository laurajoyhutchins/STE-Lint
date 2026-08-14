# ASD-STE100 Issue 9 Authority Ingest Design

## Goal

Turn the retained private ASD-STE100 Issue 9 PDF into a reproducible, provenance-bearing authority bundle without making the public STE-Lint repository a redistribution channel for copyrighted source-derived text.

## Authority boundary

Google Drive owns the retained PDF bytes. Git owns the expected source identity, extraction code, schemas/contracts, safe counts and digests, and verification evidence. Generated source-derived rule text, examples, dictionary definitions, and dictionary examples remain private derivatives unless a separate redistribution authority permits publication.

Normal STE-Lint execution must not fetch the PDF. The PDF is a maintenance input only.

## Canonical source identity

The authorized source is the Drive object `1GfSldRfzXs91pG1BbgLjbzJFJML_wifP`, titled `ASD-STE100_ISSUE9.pdf`.

The ingest must verify the retained object by byte size, SHA-256, publication identity, physical page count, and PDF metadata before source-dependent conclusions are accepted.

## Extraction products

One ingest produces:

- `source.json`: canonical source identity and metadata.
- `pages.jsonl`: complete page-level layout text with physical page number, logical page label, and text hash.
- `rules.json`: all 53 writing rules with section, title, source-page span, and extracted text.
- `general-recommendations.json`: GR-1 through GR-8 with source-page spans and extracted text.
- `dictionary-rows.jsonl`: raw four-column dictionary table rows with physical-page provenance.
- `dictionary.json`: dictionary entries reconstructed from the raw rows while retaining all row fragments and page provenance.
- `issue9-authority.sqlite3`: a queryable projection of the same evidence.
- `issue9-layout.txt`: the complete layout-preserving text derivative.
- `manifest.json`: artifact sizes, hashes, counts, and validation results.

The raw row and page derivatives are retained alongside normalized projections so parser defects can be diagnosed without repeating source acquisition.

## Dictionary extraction

Part 2 dictionary pages alternate left and right page margins. The parser uses the observed Word table geometry for odd and even physical pages and reads the explicit cell borders rather than guessing column boundaries from whitespace.

Rows with an empty word cell belong to the preceding dictionary entry. A non-empty word cell starts a new entry when it contains a recognized part-of-speech marker, with explicit handling for dictionary expressions that have no part-of-speech marker. Verb-form rows such as `PREVENTS` and `PREVENTED` are retained as continuation material instead of being counted as independent records.

Approval status follows the source's uppercase convention. Lowercase grammatical connectors inside an otherwise uppercase alternative spelling do not make the record unapproved. This behavior is regression-tested with `MATT (or MATTE)`.

Each normalized entry retains the original cell fragments and physical page numbers. Normalization never destroys the source-oriented representation.

## Validation

A successful ingest must establish:

- 434 physical PDF pages.
- 53 writing rules.
- 8 general recommendations.
- 3,052 raw dictionary rows and 2,196 reconstructed headword records for the pinned source.
- all dictionary pages without tables are literal blank pages and every nonblank dictionary page yields table evidence.
- artifact SHA-256 hashes and deterministic replay.
- exact source SHA-256 and byte size.

The source states 875 approved words and 1,274 unapproved words. The current structural projection contains 878 approved and 1,318 unapproved headword records. These counts use different cardinality bases, so equality is not a validation condition. Both measures are retained as evidence, and STE-Lint must not relabel structural records as source-declared words.

## Redistribution boundary

The PDF states that ASD owns the copyright and limits reproduction/publication. The public repository therefore receives the parser, safe source identity, counts, hashes, tests, and verification notes, but not the full extracted rules, definitions, examples, page text, SQLite database, or source PDF.

Private generated authority bundles may be retained in private storage for authorized use.
