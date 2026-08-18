# Semantic provider evaluation

This directory is the reproducible Gate 3 harness for GitHub #68.

The corpus is independently authored synthetic technical English. It exercises lexical/POS comparison against Harper, dependency relations, constituency spans, named entities, coreference, ambiguity behavior, and UTF-8 byte-span alignment.

The harness evaluates two deliberately different candidates:

- Stanford Stanza 1.14.0 `default_accurate` as the full-pipeline reference for dependency, constituency, NER, and coreference evidence.
- spaCy 3.8.14 with `en_core_web_sm` 3.8.0 as a small CPU/deployment control for overlapping POS, dependency, and NER evidence.

`run.sh` explicitly hydrates Stanza before inference, then evaluates it with Hugging Face and Transformers network access disabled. It records provider versions, resource-tree identity, artifact bytes, initialization time, per-document latency, peak RSS, exact synthetic-task metrics, Harper parity, ambiguity controls, and canonical UTF-8 byte-span alignment.

These results are provider evidence, not ASD-STE100 authority. The corpus contains no protected Issue 9 prose. Normal lint execution never invokes the harness, downloads these models, or acquires a Python dependency.
