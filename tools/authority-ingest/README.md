# Authority ingest

This directory contains maintenance tooling that builds and verifies private, versioned STE authority data from authorized source material.

It is deliberately outside the normal lint path. A released `ste` binary or package must contain the runtime language data it needs and must not fetch an ASD-STE100 PDF during ordinary use.

## Issue 9 source

The verified retained source for the current Issue 9 ingest is recorded in `data/issue9-source.manifest.json`. Google Drive owns the retained PDF bytes. Git owns only safe expected identity, extraction code, counts, hashes, and verification contracts.

The source PDF and full source-derived outputs are not committed to this public repository. The PDF's copyright notice limits reproduction/publication, so extracted rules, definitions, examples, and page text stay in private storage unless a separate redistribution authority permits publication.

## Reproduce the private authority bundle

Requirements:

- Python 3
- Poppler `pdftotext`
- `pdfplumber==0.11.9`
- `pypdf==5.9.0`

Install Python dependencies in an isolated environment:

```bash
python -m pip install pdfplumber==0.11.9 pypdf==5.9.0
```

Run the ingest against an authorized local hydration of the exact retained PDF:

```bash
python tools/authority-ingest/ingest_issue9.py \
  /path/to/ASD-STE100_ISSUE9.pdf \
  --out private-authority/issue9
```

The command writes complete page-level text, 53 structured writing rules, eight general recommendations, raw dictionary rows, normalized dictionary entries, a SQLite projection, and a manifest with hashes and validation evidence.

## Verification boundary

The ingest verifies source identity and structural invariants. It does not silently repair discrepancies. In the verified January 2025 Issue 9 source, the introduction states that the dictionary contains 875 approved words and 1274 unapproved words. The current structural extraction identifies 877 approved and 1319 unapproved headword records. Those are parsing/modeling discrepancies, not values to normalize away. They remain explicit evidence until the dictionary reconstruction is reconciled against the source semantics.
