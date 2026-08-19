#!/usr/bin/env python3
import argparse
import hashlib
import importlib.util
import json
import pathlib


HERE = pathlib.Path(__file__).resolve().parent
PROCESSORS = "tokenize,mwt,pos,lemma,depparse,constituency,ner,coref"
CONFIGURATION = (
    "lang=en;package=default_accurate;"
    f"processors={PROCESSORS};use_gpu=false;offline=true"
)


def load_evaluator():
    spec = importlib.util.spec_from_file_location("semantic_eval", HERE / "evaluate.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load semantic evaluation helpers")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def word_span(evaluator, text, sentence, word):
    start_char = getattr(word, "start_char", None)
    end_char = getattr(word, "end_char", None)
    if start_char is None or end_char is None:
        token = next(
            (token for token in sentence.tokens if word in getattr(token, "words", [])),
            None,
        )
        start_char = getattr(token, "start_char", None)
        end_char = getattr(token, "end_char", None)
    if start_char is None or end_char is None:
        return None
    ok, start, end = evaluator.aligned_surface(text, start_char, end_char, word.text)
    if not ok:
        return None
    return {"start": start, "end": end, "surface": word.text}


def unique_surface_span(evaluator, text, surface):
    if not surface:
        return None

    def positions(haystack, needle):
        found = []
        cursor = 0
        while True:
            index = haystack.find(needle, cursor)
            if index < 0:
                return found
            found.append(index)
            cursor = index + 1

    matches = positions(text, surface)
    if len(matches) != 1 and text.isascii() and surface.isascii():
        matches = positions(text.lower(), surface.lower())
    if len(matches) != 1:
        return None
    start_char = matches[0]
    end_char = start_char + len(surface)
    start, end = evaluator.byte_span(text, start_char, end_char)
    actual = text.encode("utf-8")[start:end].decode("utf-8")
    return {"start": start, "end": end, "surface": actual}


def constituency_evidence(tree, spans):
    evidence = []
    cursor = 0

    def walk(node):
        nonlocal cursor
        children = list(getattr(node, "children", []) or [])
        if not children:
            if cursor >= len(spans):
                return None
            span = spans[cursor]
            cursor += 1
            return span
        child_spans = [walk(child) for child in children]
        child_spans = [span for span in child_spans if span is not None]
        if not child_spans:
            return None
        span = {
            "start": child_spans[0]["start"],
            "end": child_spans[-1]["end"],
            "surface": "",
        }
        label = str(getattr(node, "label", "") or "")
        if label:
            evidence.append({"kind": "constituency", "label": label, "span": span})
        return span

    walk(tree)
    return evidence


def build_bundle(evaluator, text, pipeline, model_dir):
    doc = pipeline(text)
    evidence = []
    word_span_by_object = {}

    for sentence in doc.sentences:
        sentence_spans = []
        for word in sentence.words:
            span = word_span(evaluator, text, sentence, word)
            sentence_spans.append(span)
            if span is not None:
                word_span_by_object[id(word)] = span

        for index, word in enumerate(sentence.words):
            dependent = sentence_spans[index]
            if dependent is None or word.head == 0:
                continue
            head = sentence_spans[word.head - 1]
            if head is None:
                continue
            evidence.append(
                {
                    "kind": "dependency",
                    "relation": evaluator.normalize_dep(word.deprel),
                    "source": dependent,
                    "target": head,
                }
            )

        tree = getattr(sentence, "constituency", None)
        if tree is not None and all(span is not None for span in sentence_spans):
            rows = constituency_evidence(tree, sentence_spans)
            source_bytes = text.encode("utf-8")
            for row in rows:
                span = row["span"]
                span["surface"] = source_bytes[span["start"] : span["end"]].decode("utf-8")
                evidence.append(row)

    for entity in getattr(doc, "ents", []):
        ok, start, end = evaluator.aligned_surface(
            text, entity.start_char, entity.end_char, entity.text
        )
        if not ok:
            continue
        evidence.append(
            {
                "kind": "named_entity",
                "class": evaluator.normalize_entity(entity.type),
                "span": {"start": start, "end": end, "surface": entity.text},
            }
        )

    seen_coreference = set()
    for sentence in getattr(doc, "sentences", []):
        for word in getattr(sentence, "words", []):
            mention = word_span_by_object.get(id(word))
            if mention is None:
                continue
            for attachment in getattr(word, "coref_chains", None) or []:
                representative = getattr(attachment, "representative_text", None)
                if callable(representative):
                    representative = representative()
                chain = getattr(attachment, "chain", None)
                if representative is None and chain is not None:
                    representative = getattr(chain, "representative_text", None)
                    if callable(representative):
                        representative = representative()
                representative = str(representative) if representative is not None else ""
                antecedent = unique_surface_span(evaluator, text, representative)
                if antecedent is None or (
                    mention["start"] == antecedent["start"]
                    and mention["end"] == antecedent["end"]
                ):
                    continue
                key = (
                    mention["start"],
                    mention["end"],
                    antecedent["start"],
                    antecedent["end"],
                )
                if key in seen_coreference:
                    continue
                seen_coreference.add(key)
                evidence.append(
                    {
                        "kind": "coreference",
                        "representative": antecedent["surface"],
                        "source": mention,
                        "target": antecedent,
                    }
                )

    artifact = evaluator.tree_identity(model_dir)
    artifact_sha256 = artifact.get("sha256")
    if not artifact_sha256:
        raise RuntimeError("Stanza model directory has no verifiable resource tree")

    import stanza

    source_bytes = text.encode("utf-8")
    return {
        "schema_version": 1,
        "source": {
            "sha256": hashlib.sha256(source_bytes).hexdigest(),
            "bytes": len(source_bytes),
        },
        "provider": {"name": "stanza", "version": stanza.__version__},
        "model": {
            "name": "en-default_accurate",
            "version": "default_accurate",
            "artifact_sha256": artifact_sha256,
        },
        "configuration": CONFIGURATION,
        "configuration_sha256": hashlib.sha256(CONFIGURATION.encode("utf-8")).hexdigest(),
        "evidence": evidence,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Emit provider-neutral Stanza shadow evidence for one UTF-8 source document."
    )
    parser.add_argument("--input", required=True, help="UTF-8 source document")
    parser.add_argument("--model-dir", required=True, help="explicitly hydrated Stanza model root")
    parser.add_argument("--output", required=True, help="shadow bundle JSON")
    args = parser.parse_args()

    evaluator = load_evaluator()
    text = pathlib.Path(args.input).read_text(encoding="utf-8")
    model_dir = pathlib.Path(args.model_dir)
    pipeline = evaluator.stanza_load(model_dir, offline=True)
    bundle = build_bundle(evaluator, text, pipeline, model_dir)
    pathlib.Path(args.output).write_text(
        json.dumps(bundle, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()