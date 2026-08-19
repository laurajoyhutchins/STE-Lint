#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

: "${PYTHON:=python3}"
MODEL_DIR="${STANZA_RESOURCES_DIR:-$ROOT/.semantic-models/stanza}"
OUT_DIR="${SEMANTIC_EVAL_OUT:-$ROOT/.semantic-eval-output}"
mkdir -p "$MODEL_DIR" "$OUT_DIR"

export PYTHONHASHSEED=0
export TOKENIZERS_PARALLELISM=false
export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1
export OPENBLAS_NUM_THREADS=1

"$PYTHON" -m pip install --disable-pip-version-check --no-input \
  stanza==1.14.0 \
  transformers==5.14.1 \
  peft==0.20.0 \
  spacy==3.8.14
"$PYTHON" -m pip install --disable-pip-version-check --no-input \
  https://github.com/explosion/spacy-models/releases/download/en_core_web_sm-3.8.0/en_core_web_sm-3.8.0-py3-none-any.whl

cargo +1.97.1 run --locked -p ste-lint --example semantic_eval_harper -- \
  tools/semantic-eval/cases.json > "$OUT_DIR/semantic-harper.json"

"$PYTHON" tools/semantic-eval/evaluate.py hydrate-stanza \
  --model-dir "$MODEL_DIR" > "$OUT_DIR/stanza-hydration.json"

HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 \
  "$PYTHON" tools/semantic-eval/evaluate.py provider \
    --provider stanza --offline \
    --cases tools/semantic-eval/cases.json \
    --harper "$OUT_DIR/semantic-harper.json" \
    --model-dir "$MODEL_DIR" \
    --output "$OUT_DIR/stanza-results.json"

"$PYTHON" tools/semantic-eval/evaluate.py provider \
  --provider spacy --offline \
  --cases tools/semantic-eval/cases.json \
  --harper "$OUT_DIR/semantic-harper.json" \
  --model-dir "$ROOT/.semantic-models/unused" \
  --output "$OUT_DIR/spacy-results.json"

"$PYTHON" tools/semantic-eval/evaluate.py combine \
  --stanza "$OUT_DIR/stanza-results.json" \
  --spacy "$OUT_DIR/spacy-results.json" \
  --output-json "$OUT_DIR/semantic-provider-results.json" \
  --output-md "$OUT_DIR/semantic-provider-results.md"

cat "$OUT_DIR/semantic-provider-results.md"
printf '%s\n' 'SEMANTIC_PROVIDER_RESULTS_JSON_BEGIN'
cat "$OUT_DIR/semantic-provider-results.json"
printf '%s\n' 'SEMANTIC_PROVIDER_RESULTS_JSON_END'

"$PYTHON" - "$OUT_DIR/semantic-provider-results.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    results = json.load(source)

summary = {"schema_version": 1, "providers": {}}
for name, provider in sorted(results["providers"].items()):
    metrics = provider["metrics"]
    summary["providers"][name] = {
        "model_identity": provider["model_identity"],
        "offline_probe": provider["offline_probe"],
        "runtime": provider["runtime"],
        "artifact": {
            key: provider["artifact"].get(key)
            for key in ("bytes", "files", "sha256")
        },
        "span_alignment": provider["span_alignment"],
        "metrics": {
            "dependencies": metrics["dependencies"],
            "constituency": metrics["constituency"],
            "entities": metrics["entities"],
            "coreference": metrics["coreference"],
            "harper_parity": metrics["harper_parity"],
        },
    }

print(
    "SEMANTIC_PROVIDER_SUMMARY_JSON="
    + json.dumps(summary, sort_keys=True, separators=(",", ":"))
)
PY
