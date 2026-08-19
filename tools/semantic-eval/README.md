# Semantic provider evaluation

This directory is the reproducible Gate 3 harness and Gate 4 shadow bridge for GitHub #68.

The corpus is independently authored synthetic technical English. It exercises lexical/POS comparison against Harper, dependency relations, constituency spans, named entities, coreference, ambiguity behavior, and UTF-8 byte-span alignment.

The harness evaluates two deliberately different candidates:

- Stanford Stanza 1.14.0 `default_accurate` with Transformers 5.14.1 and PEFT 0.20.0 as the full-pipeline reference for dependency, constituency, NER, and coreference evidence.
- spaCy 3.8.14 with `en_core_web_sm` 3.8.0 as a small CPU/deployment control for overlapping POS, dependency, and NER evidence.

`run.sh` explicitly hydrates Stanza before inference, then evaluates it with Hugging Face and Transformers network access disabled. It records provider versions, resource-tree identity, artifact bytes, initialization time, per-document latency, peak RSS, exact synthetic-task metrics, Harper parity, ambiguity controls, and canonical UTF-8 byte-span alignment.

The measured Gate 3 decision is recorded in `docs/semantic-nlp-provider-decision.md`. Stanza is the Gate 4 full-pipeline reference/shadow provider; spaCy remains the small deployment control. The production runtime remains contingent on representative technical-English benchmark evidence from GitHub #64.

## Gate 4 shadow bundle

`shadow.py` is an explicit offline sidecar. It never hydrates or downloads a model. First hydrate the selected Stanza resources deliberately with the existing Gate 3 helper, then run the shadow bridge against one UTF-8 source document:

```bash
python tools/semantic-eval/evaluate.py hydrate-stanza --model-dir .semantic-models/stanza
python tools/semantic-eval/shadow.py \
  --input path/to/source.txt \
  --model-dir .semantic-models/stanza \
  --output .semantic-eval-output/source.shadow.json
```

The bundle records source SHA-256 and byte length, Stanza/provider identity, the exact hydrated resource-tree SHA-256, a hashed offline configuration string, and provider-neutral dependency, constituency, generic NER, and coreference observations. Every observation carries source surfaces plus UTF-8 byte coordinates.

Rust imports a bundle only through `AnalysisDocument::import_shadow_evidence_json`. Import fails closed if the source digest or byte length differs, a span is not a canonical UTF-8 boundary, a recorded surface does not match the source bytes, the model artifact/configuration identity is malformed, or confidence is outside `[0, 1]`. Imported evidence remains separate from Harper lexical evidence and is not consumed by rule passes in this shadow gate.

Coreference output is intentionally retained as provider evidence even when Stanza commits on an ambiguous case. The importer does not resolve or authorize the link. Later repository-owned semantic resolution must preserve `Ambiguous`/`Unknown` when evidence is insufficient or conflicting.

The branch-specific Rust CI bridge used to obtain the original measurements was removed after the decision. Run the evaluation harness deliberately when measurement evidence needs to be regenerated; it is not part of ordinary workspace CI.

These results and shadow bundles are provider evidence, not ASD-STE100 authority. The synthetic corpus contains no protected Issue 9 prose. Normal lint execution never invokes Python, downloads models, or reads a shadow bundle implicitly.