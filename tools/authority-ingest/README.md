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

## Compile a private runtime lexicon

`build_runtime_lexicon.py` maps the verified private authority directory into the JSON contract consumed by `ste-data`.

The compiler is a maintenance tool. It does not run during ordinary linting, and the populated Issue 9 runtime output is source-derived private data. Do not commit that output to this public repository without a separate redistribution basis.

Run it with the exact private-bundle coordinate recorded in `data/issue9-source.manifest.json`:

```bash
python tools/authority-ingest/build_runtime_lexicon.py \
  --authority-dir private-authority/issue9 \
  --verified-manifest data/issue9-source.manifest.json \
  --private-bundle-sha256 <verified-private-bundle-sha256> \
  --out private-authority/issue9/runtime-lexicon.json
```

The explicit bundle hash matters. An extracted directory cannot independently reconstruct the identity of the ZIP that contained it, so the compiler requires that coordinate and checks it against Git's verified manifest instead of pretending to infer it.

The compiler validates retained-source identity, both dictionary cardinality bases, and structural record counts before it emits anything. It preserves source record order, source-page provenance, explicit forms, and the four retained source-semantic cells.

`interpretation_state: structural` means the source semantics are preserved but have not yet been fully classified into executable `senses`, `alternatives`, or `restrictions`. `interpretation_state: interpreted` means those structured fields are intended to represent the source semantics needed by the runtime checks that consume them. The compiler does not promote raw source help text into an executable restriction by guesswork.

The current embedded `crates/ste-data/data/test-lexicon.json` remains in place during this gate. Compiling a private Issue 9 runtime artifact does not, by itself, make STE-Lint a full Issue 9 implementation.

Run the maintenance regression tests separately from the source-dependent ingest:

```bash
python -m unittest discover -s tools/authority-ingest -p 'test_*.py' -v
```

## Verification boundary

The ingest verifies source identity, rule and recommendation counts, table coverage, and deterministic artifacts. Dictionary pages that contain no extracted table must be literal blank pages; every nonblank dictionary page must yield table evidence.

The Issue 9 introduction states 875 approved words and 1274 unapproved words. The normalized projection contains 878 approved and 1318 unapproved structural headword records. These are intentionally different measures. The publication states a word count, while the projection counts dictionary records, including expression records and distinct part-of-speech records. The ingest records and verifies both measures independently instead of forcing equality.

Approval classification follows the source's uppercase convention without treating a lowercase connector inside an approved alternative spelling as lexical content. The regression case `MATT (or MATTE)` prevents that defect from returning.

The next dictionary gate is semantic enrichment of the private structural records into executable senses, alternatives, restrictions, and technical-term references. It is not source re-acquisition and it must not discard the retained `source_semantics` evidence.
