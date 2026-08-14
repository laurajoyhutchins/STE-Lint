# Issue 9 Authority-to-Runtime Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic, tested maintenance compiler that maps the verified private Issue 9 dictionary authority into the existing STE-Lint runtime JSON contract while preserving exact source provenance and lossless private source semantics.

**Architecture:** Extend `ste-data` instead of introducing a second runtime model. The maintenance compiler reads the existing private authority bundle products, validates them against `data/issue9-source.manifest.json`, emits enriched `LexiconDocument` JSON, and never becomes a normal runtime dependency. The populated Issue 9 runtime artifact remains private.

**Tech Stack:** Rust stable workspace, Serde/serde_json, Python 3 standard library, GitHub Actions.

## Global Constraints

- Normal STE-Lint runtime must not fetch or read the ASD-STE100 PDF.
- The populated Issue 9 dictionary and source-derived prose remain private unless a separate redistribution authority permits publication.
- Source-declared word counts and structural headword-record counts are distinct measures and are validated independently.
- The compiler must preserve source record order and source-page provenance.
- The compiler must never invent ordinary English morphology or a part of speech absent from the source.
- `crates/ste-data/data/test-lexicon.json` stays the embedded runtime fixture for this gate.
- Source semantics that are not fully interpreted stay losslessly preserved and are marked `structural`, not guessed into executable rules.

---

### Task 1: Extend the runtime dictionary contract

**Files:**
- Modify: `crates/ste-data/src/lib.rs`
- Modify: `schemas/dictionary.schema.json`
- Test: `crates/ste-data/src/lib.rs`

**Interfaces:**
- Produces: `AuthorityProvenance`, `DictionaryCardinalities`, `EntryProvenance`, `SourceSemantics`, `InterpretationState` and an optional `LexiconEntry.part_of_speech`.
- Preserves: `RuntimeLexicon::embedded()`, `RuntimeLexicon::from_json`, explicit-form lookup semantics, and the existing toy lexicon.

- [ ] **Step 1: Add failing Rust tests for enriched runtime data**

Add tests that parse this invented JSON through `RuntimeLexicon::from_json`:

```rust
let json = r#"{
  "metadata": {
    "standard": "ASD-STE100",
    "issue": 9,
    "date": "2025-01-15",
    "scope": "synthetic_authority_mapping",
    "authority": {
      "drive_file_id": "synthetic-drive-object",
      "source_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "source_byte_size": 123,
      "physical_pages": 4,
      "private_bundle_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    },
    "dictionary_cardinalities": {
      "source_declared_approved_words": 2,
      "source_declared_unapproved_words": 2,
      "structural_approved_records": 3,
      "structural_unapproved_records": 2
    }
  },
  "entries": [{
    "lemma": "CHECK AGAIN",
    "status": "approved",
    "part_of_speech": null,
    "forms": ["CHECK AGAIN"],
    "senses": [],
    "alternatives": [],
    "restrictions": [],
    "interpretation_state": "structural",
    "provenance": {"structural_record_index": 3, "source_pages": [7, 8]},
    "source_semantics": {
      "word_cell": "CHECK AGAIN",
      "meaning_or_alternatives": "synthetic source meaning",
      "ste_example": "CHECK AGAIN.",
      "non_ste_example": ""
    }
  }]
}"#;

let lexicon = RuntimeLexicon::from_json(json).unwrap();
let entry = lexicon.lookup_form("check again").unwrap();
assert_eq!(entry.part_of_speech, None);
assert_eq!(entry.interpretation_state, InterpretationState::Structural);
assert_eq!(entry.provenance.as_ref().unwrap().source_pages, vec![7, 8]);
assert_eq!(
    lexicon.metadata().dictionary_cardinalities.as_ref().unwrap().structural_approved_records,
    3
);
```

Also retain the existing tests proving the embedded toy lexicon parses and lookup does not invent forms.

- [ ] **Step 2: Run the Rust test and observe RED**

Run in repository CI or an equivalent Rust environment:

```bash
cargo test -p ste-data
```

Expected: compile failure because the new types and fields do not exist yet.

- [ ] **Step 3: Implement the minimal typed model**

Add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityProvenance {
    pub drive_file_id: String,
    pub source_sha256: String,
    pub source_byte_size: u64,
    pub physical_pages: u32,
    pub private_bundle_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryCardinalities {
    pub source_declared_approved_words: u32,
    pub source_declared_unapproved_words: u32,
    pub structural_approved_records: u32,
    pub structural_unapproved_records: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryProvenance {
    pub structural_record_index: u32,
    pub source_pages: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSemantics {
    pub word_cell: String,
    pub meaning_or_alternatives: String,
    pub ste_example: String,
    pub non_ste_example: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationState {
    Structural,
    #[default]
    Interpreted,
}
```

Change `LexiconEntry.part_of_speech` to `Option<PartOfSpeech>` and add backward-compatible optional/default fields:

```rust
#[serde(default)]
pub interpretation_state: InterpretationState,
#[serde(default)]
pub provenance: Option<EntryProvenance>,
#[serde(default)]
pub source_semantics: Option<SourceSemantics>,
```

Add to `LexiconMetadata`:

```rust
#[serde(default)]
pub authority: Option<AuthorityProvenance>,
#[serde(default)]
pub dictionary_cardinalities: Option<DictionaryCardinalities>,
```

Update the JSON Schema to represent the same optional fields and nullable POS while keeping the original required top-level entry fields.

- [ ] **Step 4: Run Rust verification and observe GREEN**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all existing tests and the new enriched-model tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/ste-data/src/lib.rs schemas/dictionary.schema.json
git commit -m "feat: extend runtime dictionary provenance model"
```

---

### Task 2: Define the synthetic authority fixture and compiler tests

**Files:**
- Create: `fixtures/authority-ingest/source.json`
- Create: `fixtures/authority-ingest/manifest.json`
- Create: `fixtures/authority-ingest/dictionary.json`
- Create: `fixtures/authority-ingest/verified-manifest.json`
- Create: `tools/authority-ingest/test_build_runtime_lexicon.py`

**Interfaces:**
- Consumes: the private-authority field names already produced by `ingest_issue9.py`.
- Produces test expectations for `map_part_of_speech`, `normalize_forms`, `validate_authority`, `compile_entry`, and `compile_document` in Task 3.

- [ ] **Step 1: Create a synthetic authority fixture**

Use invented prose only. The dictionary fixture must include five records:

```json
[
  {
    "headword": "CHECK",
    "headword_raw": "CHECK (v),\nCHECKS,\nCHECKED\nNo other verb forms.",
    "part_of_speech": "v",
    "forms": ["CHECK", "CHECKS", "CHECKED No other verb forms"],
    "approved": true,
    "word_cell": "CHECK (v),\nCHECKS,\nCHECKED\nNo other verb forms.",
    "meaning_or_alternatives": "Synthetic approved meaning.",
    "ste_example": "CHECK THE ITEM.",
    "non_ste_example": "",
    "source_pages": [1]
  },
  {
    "headword": "simplebad",
    "headword_raw": "simplebad (adj)",
    "part_of_speech": "adj",
    "forms": [],
    "approved": false,
    "word_cell": "simplebad (adj)",
    "meaning_or_alternatives": "GOOD (adj)",
    "ste_example": "THE ITEM IS GOOD.",
    "non_ste_example": "The item is simplebad.",
    "source_pages": [2],
    "senses": [],
    "alternatives": [{
      "kind": "approved_word",
      "text": "GOOD",
      "part_of_speech": "adjective",
      "strategy": "word_replacement"
    }],
    "restrictions": [],
    "interpretation_state": "interpreted"
  },
  {
    "headword": "domainthing",
    "headword_raw": "domainthing (n)",
    "part_of_speech": "n",
    "forms": [],
    "approved": false,
    "word_cell": "domainthing (n)",
    "meaning_or_alternatives": "PROJECT TERM (TN)",
    "ste_example": "INSPECT THE PROJECT TERM.",
    "non_ste_example": "Inspect the domainthing.",
    "source_pages": [3],
    "alternatives": [{
      "kind": "technical_noun",
      "text": "PROJECT TERM",
      "part_of_speech": null,
      "strategy": "sentence_reconstruction"
    }],
    "restrictions": [],
    "interpretation_state": "interpreted"
  },
  {
    "headword": "CHECK AGAIN",
    "headword_raw": "CHECK AGAIN",
    "part_of_speech": null,
    "forms": [],
    "approved": true,
    "word_cell": "CHECK AGAIN",
    "meaning_or_alternatives": "Synthetic expression meaning.",
    "ste_example": "CHECK AGAIN.",
    "non_ste_example": "",
    "source_pages": [4]
  },
  {
    "headword": "contextual",
    "headword_raw": "contextual (adj)",
    "part_of_speech": "adj",
    "forms": [],
    "approved": true,
    "word_cell": "contextual (adj)",
    "meaning_or_alternatives": "Synthetic meaning plus source help that is not yet interpreted.",
    "ste_example": "THE VALUE IS CONTEXTUAL.",
    "non_ste_example": "",
    "source_pages": [4]
  }
]
```

Set the synthetic manifests to internally consistent identity and counts: three approved structural records, two unapproved structural records, and independently declared word counts of two approved and two unapproved words.

- [ ] **Step 2: Write compiler tests before the compiler exists**

Tests must assert:

```python
self.assertEqual(map_part_of_speech("v"), "verb")
self.assertEqual(map_part_of_speech(None), None)
self.assertEqual(
    normalize_forms({"headword": "CHECK", "part_of_speech": "v", "forms": ["CHECK", "CHECKS", "CHECKED No other verb forms"]}),
    ["CHECK", "CHECKS", "CHECKED"],
)
self.assertEqual(
    normalize_forms({"headword": "CHECK AGAIN", "part_of_speech": None, "forms": []}),
    ["CHECK AGAIN"],
)
```

End-to-end tests must build twice into separate temporary paths and assert identical bytes. They must also assert that the compiled technical-noun alternative remains `technical_noun`, the phrase record has `part_of_speech: null`, every record retains `source_semantics`, and the intentionally uninterpreted record emits `interpretation_state: structural`.

Add negative tests that mutate source SHA-256 and structural counts and assert `AuthorityValidationError`.

- [ ] **Step 3: Run Python tests and observe RED**

```bash
python -m unittest tools/authority-ingest/test_build_runtime_lexicon.py -v
```

Expected: import failure because `build_runtime_lexicon.py` does not exist.

- [ ] **Step 4: Commit the RED fixture/tests**

```bash
git add fixtures/authority-ingest tools/authority-ingest/test_build_runtime_lexicon.py
git commit -m "test: specify authority runtime compiler contract"
```

---

### Task 3: Implement the deterministic authority compiler

**Files:**
- Create: `tools/authority-ingest/build_runtime_lexicon.py`
- Test: `tools/authority-ingest/test_build_runtime_lexicon.py`

**Interfaces:**
- `map_part_of_speech(source_pos: str | None) -> str | None`
- `normalize_forms(entry: dict) -> list[str]`
- `validate_authority(source: dict, private_manifest: dict, verified_manifest: dict, dictionary: list[dict]) -> None`
- `compile_entry(index: int, entry: dict) -> dict`
- `compile_document(authority_dir: Path, verified_manifest_path: Path) -> dict`
- CLI: `python tools/authority-ingest/build_runtime_lexicon.py --authority-dir DIR --verified-manifest data/issue9-source.manifest.json --out FILE`

- [ ] **Step 1: Implement exact source/POS/form helpers**

Use the standard library only. POS mapping is exact:

```python
POS = {
    "n": "noun",
    "v": "verb",
    "adj": "adjective",
    "adv": "adverb",
    "pron": "pronoun",
    "art": "article",
    "prep": "preposition",
    "conj": "conjunction",
}
```

`normalize_forms` must remove `No other verb forms.` text, strip a leading `(also ` marker and matching punctuation, split contaminated form strings when necessary, preserve source order, and deduplicate case-sensitively. It must not synthesize any unlisted conjugation. If no explicit forms remain, return `[headword]`.

- [ ] **Step 2: Implement source validation**

Require exact equality between private source evidence and the verified public manifest for:

```text
issue
publication_date
drive file id
SHA-256
byte size
physical page count
private bundle SHA-256 (from verified manifest versus the bundle identity supplied to the build)
source-declared approved count
source-declared unapproved count
structural approved count
structural unapproved count
```

For the compiler's directory input, derive the structural approved/unapproved counts directly from `dictionary.json` and compare them to both manifests.

Do not compare source-declared word counts to structural record counts.

- [ ] **Step 3: Implement lossless record compilation**

Each output record has the existing runtime fields plus:

```python
"interpretation_state": entry.get("interpretation_state", "structural"),
"provenance": {
    "structural_record_index": index,
    "source_pages": entry.get("source_pages", []),
},
"source_semantics": {
    "word_cell": entry.get("word_cell", ""),
    "meaning_or_alternatives": entry.get("meaning_or_alternatives", ""),
    "ste_example": entry.get("ste_example", ""),
    "non_ste_example": entry.get("non_ste_example", ""),
},
```

Copy pre-interpreted optional `senses`, `alternatives`, and `restrictions` when they exist. Otherwise emit empty arrays rather than guessing.

- [ ] **Step 4: Implement deterministic document emission**

Metadata must include the normal `standard`, `issue`, `date`, and `scope: "issue9_private_authority_runtime"` plus authority provenance and both cardinality bases.

Serialize with:

```python
text = json.dumps(document, ensure_ascii=False, indent=2) + "\n"
```

Preserve insertion order and input record order. Write the file atomically through a temporary sibling and `Path.replace`.

- [ ] **Step 5: Run Python tests and observe GREEN**

```bash
python -m unittest tools/authority-ingest/test_build_runtime_lexicon.py -v
```

Expected: all compiler tests pass.

- [ ] **Step 6: Run the complete repository gate**

```bash
python -m py_compile tools/authority-ingest/ingest_issue9.py tools/authority-ingest/test_ingest_issue9.py tools/authority-ingest/build_runtime_lexicon.py tools/authority-ingest/test_build_runtime_lexicon.py
python -m unittest discover -s tools/authority-ingest -p 'test_*.py' -v
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add tools/authority-ingest/build_runtime_lexicon.py
git commit -m "feat: compile private authority into runtime lexicon"
```

---

### Task 4: Verify the real private authority derivative and document the gate

**Files:**
- Modify: `tools/authority-ingest/README.md`
- Modify: `docs/dictionary-model.md`
- Private output only: `runtime-lexicon.json`
- Private output only: `runtime-manifest.json`

**Interfaces:**
- Consumes: the exact verified private authority bundle identified by `data/issue9-source.manifest.json`.
- Produces: a private runtime artifact plus SHA-256 evidence that can be referenced by GitHub Issue #3 / the execution queue without publishing source-derived prose.

- [ ] **Step 1: Run the compiler twice against the retained private authority bundle**

```bash
python tools/authority-ingest/build_runtime_lexicon.py \
  --authority-dir /private/issue9-authority \
  --verified-manifest data/issue9-source.manifest.json \
  --out /private/runtime-a.json

python tools/authority-ingest/build_runtime_lexicon.py \
  --authority-dir /private/issue9-authority \
  --verified-manifest data/issue9-source.manifest.json \
  --out /private/runtime-b.json

sha256sum /private/runtime-a.json /private/runtime-b.json
cmp /private/runtime-a.json /private/runtime-b.json
```

Expected: identical SHA-256 values and `cmp` success.

- [ ] **Step 2: Parse the real runtime artifact through `RuntimeLexicon` in a private-capable Rust executor when available**

The verification claim is narrow if only the compiler can run in the current environment. Hosted CI remains the exact-candidate repository gate for the public code. Do not publish the private artifact merely to make this step easier.

- [ ] **Step 3: Store derivative identity privately**

Create a private `runtime-manifest.json` containing:

```json
{
  "parent_private_bundle_sha256": "<verified bundle SHA-256>",
  "runtime_lexicon_sha256": "<computed SHA-256>",
  "runtime_lexicon_bytes": 0,
  "compiler_commit": "<exact branch head>",
  "source_sha256": "<verified retained PDF SHA-256>"
}
```

Replace `0` with the measured byte count and store the manifest beside the private runtime artifact. Do not commit either file publicly.

- [ ] **Step 4: Update public maintenance documentation**

Document:

- how to invoke the compiler;
- that its full Issue 9 output is private source-derived data;
- that `interpretation_state: structural` means source semantics are preserved but not yet executable as sense/restriction logic;
- that the embedded toy lexicon remains in place;
- that source-declared and structural counts are distinct;
- the next gate is semantic enrichment / executable coverage, not source re-acquisition.

- [ ] **Step 5: Run exact-head CI and review evidence**

Require the pull-request head to pass the Python authority tests and the full Rust workspace gate. Inspect any failure rather than weakening the tests.

- [ ] **Step 6: Commit documentation**

```bash
git add tools/authority-ingest/README.md docs/dictionary-model.md
git commit -m "docs: define private runtime authority boundary"
```

- [ ] **Step 7: Open a coherent non-draft pull request**

The PR body records:

- GitHub Issue #3 as authority;
- exact private source and parent-bundle identities by hash only;
- the runtime derivative hash and byte count without attaching source-derived content;
- exact-head CI evidence;
- `owner-impact: none` for this bounded implementation;
- that Issue #3 remains open for semantic enrichment and all-53-rule coverage after this first gate lands.
