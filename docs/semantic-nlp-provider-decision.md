# Semantic NLP provider decision

This document records the measured Gate 3 decision for GitHub #68. It is a decision about the richer generic NLP evidence layer used for Gate 4 shadow integration. It is not a standards-authority decision and it is not the final production-runtime selection.

## Decision

Use **Stanford Stanza 1.14.0 `default_accurate` as the full-pipeline reference and Gate 4 shadow provider** for dependency, constituency, named-entity, and coreference evidence.

Run Stanza through an explicitly hydrated, local Python sidecar for the shadow phase. Normal `ste` lint execution remains Rust-local and does not acquire Python, Stanza, Transformers, PEFT, or model-download dependencies.

Keep **spaCy 3.8.14 with `en_core_web_sm` 3.8.0 as the small deployment/control candidate**, not as the sole Gate 4 provider. Its measured footprint and latency are attractive, but this model does not supply constituency or coreference and missed two adjudicated dependency relations in the Gate 3 corpus.

Do **not** treat Stanza coreference output as resolved truth by itself. The ambiguous-control case showed that the selected Stanza pipeline can commit to a coreference chain when the repository expects ambiguity. Gate 4 must expose that output as provenance-bearing evidence and preserve `Ambiguous`/`Unknown` unless repository-owned resolution logic has enough corroborating evidence to resolve it.

Defer the final production packaging/runtime choice until representative technical-English evidence from GitHub #64 is available. Gate 3 therefore selects the shadow/reference runtime now while explicitly leaving the release-runtime decision open. A self-contained Rust-consumable runtime remains preferable if later measurements show that it preserves the required evidence quality.

## Measurement identity

The measurements below were produced by the reproducible harness in `tools/semantic-eval/` on pull request #72 at exact head:

`a4c57ad330cd63f5b31fd9c612dc83e309ae7f48`

GitHub Actions evidence:

- workflow run: `32201649840`
- Rust job: `95916555234`
- Python: `3.12.3`
- corpus: `tools/semantic-eval/cases.json`, schema version 1

The synthetic corpus is independently authored technical English. It is suitable for early provider characterization, span/provenance tests, and ambiguity controls. It is not representative real-world benchmark evidence and does not satisfy the later production-selection or rule-promotion requirement from GitHub #68 by itself.

## Measured results

| Measurement | Stanza 1.14.0 `default_accurate` | spaCy 3.8.14 `en_core_web_sm` 3.8.0 |
| --- | ---: | ---: |
| Canonical UTF-8 span alignment | 12/12 | 12/12 |
| Dependency relations | 13/13 | 11/13 |
| Constituency spans | 5/5 | unavailable (0/5) |
| Named entities | 6/6 | 6/6 |
| Expected coreference links | 2/2 | unavailable (0/2) |
| Ambiguous coreference control | committed (unsafe alone) | no commitment |
| POS parity vs Harper | 32/36 | 32/36 |
| Initialization | 20.54 s | 2.82 s |
| Mean inference latency | 5654.9 ms/document | 4.4 ms/document |
| Peak RSS after inference | 6276.7 MiB | 608.0 MiB |
| Captured model/resource bytes | 779,233,413 | 15,242,267 |
| Captured model/resource size | 743.1 MiB | 14.5 MiB |
| Offline inference probe | pass | pass |

The two spaCy dependency misses were both in `modifier-subject`: the expected `indicator -> flashes` subject relation and `red -> indicator` adjectival-modifier relation. The four Harper POS mismatches were shared by both candidates, so neither richer provider replaces Harper as the cheap lexical source.

Captured model/resource tree identities from the exact run:

- Stanza: SHA-256 `296460d5669f5b0587c17e2200a6e22498a0a978ded8bd90c155cead6ef91902`, 779,233,413 bytes, 12 files.
- spaCy: SHA-256 `60a20496b0139d4f0fe306736082d7a42374b04cd0231b8617fcacd06f410055`, 15,242,267 bytes, 29 files.

These digests identify the hydrated resource trees observed by the harness. Gate 4 must establish its own pinned artifact/configuration identity before model evidence can be consumed by repository code.

## Task-family interpretation

### Dependency parsing

Stanza is the selected shadow reference because it covered all adjudicated dependency relations in this corpus. spaCy remains a useful performance/deployment control, but the measured misses make it an insufficient drop-in syntax provider for this gate without additional evidence or adaptation.

### Constituency

The selected Stanza pipeline supplied all expected constituency spans. The evaluated spaCy small pipeline has no constituency component, so it cannot satisfy the Gate 4 fact set alone.

### Named entities

Both candidates matched all six adjudicated NER expectations. Generic NER remains evidence only. It cannot grant governed terminology, project entity identity, or ASD-STE100 authority.

### Coreference

Stanza recovered both expected links but failed the explicit ambiguity control by committing to a chain. This is a useful negative result: provider output must retain alternatives/confidence/provenance and flow through a fail-closed repository resolver. No Gate 4 coreference fact may become authoritative merely because Stanza emitted it.

### POS and morphology

Both candidates reached the same 32/36 broad-POS parity against Harper. Harper remains the lexical provider. Gate 4 must not replace it just to duplicate cheap deterministic/local evidence.

## Performance and packaging interpretation

Stanza provides the broadest measured task coverage but is not acceptable as an always-on normal-lint dependency at the observed cost. The Gate 4 sidecar is therefore shadow-only and explicitly hydrated. Its purpose is to establish evidence quality, canonical-span mapping, provenance behavior, and replacement value before any production cutover.

spaCy's measured footprint and latency make it materially easier to package and operate, but the evaluated small pipeline does not cover the required fact families. Later work may re-evaluate a different spaCy pipeline, a distilled/exported model, an ONNX/Rust-consumable path, or another runtime if representative benchmark evidence justifies it. Gate 3 does not bless an unmeasured packaging path.

Both ecosystems support task-specific training/fine-tuning workflows. spaCy has a comparatively direct packaged-pipeline workflow; Stanza exposes training for its neural modules from source. Gate 5 must still evaluate STE-specific supervised tasks on independently authored/safely derived data rather than assuming either generic pipeline is sufficient.

## Licensing and redistribution boundary

The Stanza software package declares Apache License 2.0. spaCy declares the MIT license, and the evaluated `en_core_web_sm` model metadata declares MIT.

Those framework/package licenses do not by themselves prove that every upstream-trained model artifact can be redistributed under the same terms. The hydrated Stanza resource tree and any future model export must receive an artifact/source-specific redistribution review before bundling or public publication. This gate commits no third-party model bytes.

For Gate 4, model hydration remains explicit and external to normal lint execution. Exact model/config identities must be pinned and verified locally before use.

## Gate 4 constraints carried forward

1. Stanza evidence maps back to repository-owned canonical UTF-8 byte spans.
2. Dependency, constituency, NER, and coreference observations use the provider-neutral evidence IR from Gate 2.
3. Provider/model/config provenance is inspectable.
4. Harper and richer-provider evidence remain separately visible when they disagree.
5. Coreference and other materially ambiguous evidence fail closed.
6. No rule verdict or coverage status changes during initial shadow integration.
7. No model is downloaded implicitly by normal lint execution.
8. Large model artifacts remain outside public Git unless a redistribution basis is established.
9. Production selection remains contingent on representative technical-English evidence from GitHub #64.

## Reproduction

Run `tools/semantic-eval/run.sh` in an environment with the pinned Python packages available and network access for the explicit hydration/install phase. The runner hydrates Stanza before evaluation, then disables Hugging Face/Transformers network access for Stanza inference and records the result set under `.semantic-eval-output/`.

The temporary branch-specific Rust integration-test bridge used to obtain the measurements was intentionally removed after this decision. The harness remains available for deliberate research/benchmark execution and is not part of ordinary workspace CI.