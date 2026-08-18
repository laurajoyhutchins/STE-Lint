#!/usr/bin/env python3
import argparse
import hashlib
import json
import math
import pathlib
import resource
import statistics
import sys
import time


def load_json(path):
    return json.loads(pathlib.Path(path).read_text(encoding="utf-8"))


def dump_json(path, value):
    pathlib.Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def char_to_byte_table(text):
    table = [0]
    total = 0
    for ch in text:
        total += len(ch.encode("utf-8"))
        table.append(total)
    return table


def byte_span(text, start_char, end_char):
    table = char_to_byte_table(text)
    if not (0 <= start_char <= end_char <= len(text)):
        raise ValueError(f"invalid character span {start_char}:{end_char}")
    return table[start_char], table[end_char]


def aligned_surface(text, start_char, end_char, surface):
    start, end = byte_span(text, start_char, end_char)
    raw = text.encode("utf-8")[start:end].decode("utf-8")
    return raw == surface, start, end


def tree_identity(root):
    root = pathlib.Path(root)
    if not root.exists():
        return {"path": str(root), "bytes": 0, "files": 0, "sha256": None}
    digest = hashlib.sha256()
    total = 0
    files = 0
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        rel = path.relative_to(root).as_posix()
        data = path.read_bytes()
        file_sha = hashlib.sha256(data).hexdigest()
        total += len(data)
        files += 1
        digest.update(rel.encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(len(data)).encode("ascii"))
        digest.update(b"\0")
        digest.update(file_sha.encode("ascii"))
        digest.update(b"\n")
    return {"path": str(root), "bytes": total, "files": files, "sha256": digest.hexdigest()}


def peak_rss_mib():
    rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return rss / (1024 * 1024) if sys.platform == "darwin" else rss / 1024


def percentile(values, q):
    if not values:
        return None
    values = sorted(values)
    index = (len(values) - 1) * q
    lo = math.floor(index)
    hi = math.ceil(index)
    if lo == hi:
        return values[lo]
    return values[lo] + (values[hi] - values[lo]) * (index - lo)


def normalize_dep(label):
    return {"dobj": "obj", "pobj": "obl", "nsubjpass": "nsubj:pass", "auxpass": "aux:pass"}.get(label, label)


def normalize_entity(label):
    return {"PER": "PERSON", "LOC": "GPE"}.get(label, label)


def broad_pos(upos):
    return {
        "NOUN": "noun", "PROPN": "noun", "ADJ": "adjective", "VERB": "verb", "AUX": "verb",
        "DET": "determiner", "CCONJ": "conjunction", "SCONJ": "conjunction",
    }.get(upos)


def harper_expected_pos(token):
    labels = [name for name in ("determiner", "conjunction", "noun", "adjective", "verb") if token.get(name)]
    return labels[0] if len(labels) == 1 else None


def form_from_ud(pos, feats):
    values = {}
    for part in str(feats or "").split("|"):
        if "=" in part:
            key, value = part.split("=", 1)
            values[key] = value
    if values.get("VerbForm") == "Part" and values.get("Tense") == "Past":
        return "PastParticiple"
    if values.get("VerbForm") == "Fin" and values.get("Tense") == "Past":
        return "SimplePast"
    if values.get("VerbForm") == "Fin" and values.get("Tense") == "Pres":
        return "SimplePresent"
    if values.get("Mood") == "Imp" or values.get("VerbForm") == "Inf":
        return "Base"
    if pos == "VERB" and values.get("VerbForm") is None and values.get("Tense") is None:
        return "Base"
    return None


def extract_constituency(tree):
    if tree is None:
        return []
    rows = []

    def walk(node):
        children = list(getattr(node, "children", []) or [])
        if not children:
            return [str(getattr(node, "label", ""))]
        leaves = []
        for child in children:
            leaves.extend(walk(child))
        label = str(getattr(node, "label", ""))
        text = " ".join(item for item in leaves if item)
        if label and text:
            rows.append({"label": label, "text": text})
        return leaves

    walk(tree)
    return rows


def stanza_coref_mentions(doc):
    mentions = []
    for sentence in getattr(doc, "sentences", []):
        for word in getattr(sentence, "words", []):
            for attachment in getattr(word, "coref_chains", None) or []:
                representative = getattr(attachment, "representative_text", None)
                if callable(representative):
                    representative = representative()
                chain = getattr(attachment, "chain", None)
                if representative is None and chain is not None:
                    representative = getattr(chain, "representative_text", None)
                    if callable(representative):
                        representative = representative()
                mentions.append({
                    "mention": word.text,
                    "representative": str(representative) if representative is not None else None,
                    "attachment": repr(attachment),
                })
    return mentions


def stanza_load(model_dir, offline):
    import stanza

    processors = "tokenize,mwt,pos,lemma,depparse,constituency,ner,coref"
    if not offline:
        stanza.download("en", model_dir=str(model_dir), package="default_accurate", processors=processors, verbose=False)
    return stanza.Pipeline(
        "en",
        dir=str(model_dir),
        package="default_accurate",
        processors=processors,
        use_gpu=False,
        download_method=None if offline else stanza.pipeline.core.DownloadMethod.REUSE_RESOURCES,
        verbose=False,
    )


def stanza_extract(pipeline, case):
    text = case["text"]
    doc = pipeline(text)
    tokens = []
    dependencies = []
    constituency = []
    alignment = []
    for sentence in doc.sentences:
        for word in sentence.words:
            start_char = getattr(word, "start_char", None)
            end_char = getattr(word, "end_char", None)
            if start_char is None or end_char is None:
                token = next((token for token in sentence.tokens if word in getattr(token, "words", [])), None)
                start_char = getattr(token, "start_char", None)
                end_char = getattr(token, "end_char", None)
            if start_char is None or end_char is None:
                alignment.append(False)
                start_byte = end_byte = None
            else:
                ok, start_byte, end_byte = aligned_surface(text, start_char, end_char, word.text)
                alignment.append(ok)
            head = "ROOT" if word.head == 0 else sentence.words[word.head - 1].text
            dependencies.append({"dependent": word.text, "head": head, "relation": normalize_dep(word.deprel)})
            tokens.append({
                "text": word.text, "start": start_byte, "end": end_byte, "upos": word.upos, "xpos": word.xpos,
                "feats": word.feats, "lemma": word.lemma, "broad_pos": broad_pos(word.upos),
                "verb_form": form_from_ud(word.upos, word.feats),
            })
        constituency.extend(extract_constituency(getattr(sentence, "constituency", None)))
    entities = []
    for entity in getattr(doc, "ents", []):
        ok, start_byte, end_byte = aligned_surface(text, entity.start_char, entity.end_char, entity.text)
        alignment.append(ok)
        entities.append({"text": entity.text, "type": normalize_entity(entity.type), "start": start_byte, "end": end_byte})
    return {
        "tokens": tokens, "dependencies": dependencies, "constituency": constituency, "entities": entities,
        "coreference": stanza_coref_mentions(doc), "alignment_ok": all(alignment) if alignment else True,
    }


def spacy_load():
    import en_core_web_sm
    return en_core_web_sm.load(), pathlib.Path(en_core_web_sm.__path__[0])


def spacy_extract(pipeline, case):
    text = case["text"]
    doc = pipeline(text)
    tokens = []
    dependencies = []
    alignment = []
    for token in doc:
        ok, start_byte, end_byte = aligned_surface(text, token.idx, token.idx + len(token.text), token.text)
        alignment.append(ok)
        dependencies.append({
            "dependent": token.text,
            "head": "ROOT" if token.head.i == token.i else token.head.text,
            "relation": normalize_dep(token.dep_),
        })
        tokens.append({
            "text": token.text, "start": start_byte, "end": end_byte, "upos": token.pos_, "xpos": token.tag_,
            "feats": str(token.morph), "lemma": token.lemma_, "broad_pos": broad_pos(token.pos_),
            "verb_form": form_from_ud(token.pos_, str(token.morph)),
        })
    entities = []
    for entity in doc.ents:
        ok, start_byte, end_byte = aligned_surface(text, entity.start_char, entity.end_char, entity.text)
        alignment.append(ok)
        entities.append({"text": entity.text, "type": normalize_entity(entity.label_), "start": start_byte, "end": end_byte})
    return {
        "tokens": tokens, "dependencies": dependencies, "constituency": None, "entities": entities,
        "coreference": None, "alignment_ok": all(alignment) if alignment else True,
    }


def exact_metric(cases, observations, key):
    correct = 0
    total = 0
    misses = []
    for case in cases:
        expected = case.get("gold", {}).get(key, [])
        if not expected:
            continue
        actual = observations[case["id"]].get(key)
        if actual is None:
            total += len(expected)
            misses.extend({"case": case["id"], "expected": item, "actual": None} for item in expected)
            continue
        keys = set(expected[0])
        actual_set = {tuple(sorted((str(k), str(v)) for k, v in item.items() if k in keys)) for item in actual}
        for item in expected:
            total += 1
            marker = tuple(sorted((str(k), str(v)) for k, v in item.items()))
            if marker in actual_set:
                correct += 1
            else:
                misses.append({"case": case["id"], "expected": item})
    return {"correct": correct, "total": total, "accuracy": correct / total if total else None, "misses": misses}


def coref_metric(cases, observations):
    correct = 0
    total = 0
    misses = []
    ambiguous_controls = []
    for case in cases:
        if case.get("ambiguity", {}).get("coreference"):
            actual = observations[case["id"]].get("coreference")
            committed = any(
                item.get("mention", "").lower() == "it" and item.get("representative") for item in (actual or [])
            )
            ambiguous_controls.append({"case": case["id"], "provider_committed": committed})
        for expected in case.get("gold", {}).get("coreference", []):
            total += 1
            actual = observations[case["id"]].get("coreference") or []
            matched = any(
                item.get("mention", "").lower() == expected["mention"].lower()
                and expected["antecedent"].lower() in (item.get("representative") or "").lower()
                for item in actual
            )
            if matched:
                correct += 1
            else:
                misses.append({"case": case["id"], "expected": expected, "actual": actual})
    return {
        "correct": correct, "total": total, "accuracy": correct / total if total else None,
        "misses": misses, "ambiguous_controls": ambiguous_controls,
    }


def harper_parity(cases, observations, harper):
    harper_by_id = {case["id"]: case for case in harper["cases"]}
    pos_correct = pos_total = morph_correct = morph_total = 0
    pos_misses = []
    morph_misses = []
    for case in cases:
        observed = observations[case["id"]]
        model_tokens = {
            (token.get("start"), token.get("end")): token for token in observed.get("tokens", [])
            if token.get("start") is not None and token.get("end") is not None
        }
        for token in harper_by_id[case["id"]]["tokens"]:
            model = model_tokens.get((token["start"], token["end"]))
            if not model:
                continue
            expected_pos = harper_expected_pos(token)
            if expected_pos:
                pos_total += 1
                if model.get("broad_pos") == expected_pos:
                    pos_correct += 1
                else:
                    pos_misses.append({"case": case["id"], "token": token["text"], "harper": expected_pos, "model": model.get("broad_pos")})
            roles = token.get("dictionary_verb_roles", [])
            if len(roles) == 1 and model.get("verb_form"):
                morph_total += 1
                if model["verb_form"] == roles[0]:
                    morph_correct += 1
                else:
                    morph_misses.append({"case": case["id"], "token": token["text"], "harper": roles[0], "model": model["verb_form"]})
    return {
        "pos": {"correct": pos_correct, "total": pos_total, "accuracy": pos_correct / pos_total if pos_total else None, "misses": pos_misses},
        "morphology": {"correct": morph_correct, "total": morph_total, "accuracy": morph_correct / morph_total if morph_total else None, "misses": morph_misses},
    }


def evaluate_provider(provider, cases_path, harper_path, model_dir, output, offline):
    cases = load_json(cases_path)["cases"]
    harper = load_json(harper_path)
    started = time.perf_counter()
    if provider == "stanza":
        pipeline = stanza_load(model_dir, offline=offline)
        extractor = stanza_extract
        artifact_root = pathlib.Path(model_dir)
        model_identity = {
            "provider": "stanza", "provider_version": __import__("stanza").__version__, "package": "default_accurate",
            "processors": ["tokenize", "mwt", "pos", "lemma", "depparse", "constituency", "ner", "coref"],
        }
    elif provider == "spacy":
        pipeline, artifact_root = spacy_load()
        extractor = spacy_extract
        model_identity = {
            "provider": "spacy", "provider_version": __import__("spacy").__version__, "model": "en_core_web_sm",
            "model_version": __import__("en_core_web_sm").__version__, "processors": list(pipeline.pipe_names),
        }
    else:
        raise ValueError(provider)
    init_seconds = time.perf_counter() - started
    rss_after_init = peak_rss_mib()
    extractor(pipeline, cases[0])
    observations = {}
    per_doc_ms = []
    for iteration in range(3):
        for case in cases:
            before = time.perf_counter()
            observed = extractor(pipeline, case)
            per_doc_ms.append((time.perf_counter() - before) * 1000)
            if iteration == 0:
                observations[case["id"]] = observed
    result = {
        "schema_version": 1,
        "provider": provider,
        "model_identity": model_identity,
        "offline_probe": offline,
        "runtime": {
            "python": sys.version.split()[0], "platform": sys.platform, "init_seconds": init_seconds,
            "peak_rss_mib_after_init": rss_after_init, "peak_rss_mib_after_inference": peak_rss_mib(),
            "mean_ms_per_document": statistics.mean(per_doc_ms), "p95_ms_per_document": percentile(per_doc_ms, 0.95),
            "measurement_documents": len(per_doc_ms),
        },
        "artifact": tree_identity(artifact_root),
        "span_alignment": {"correct": sum(1 for value in observations.values() if value["alignment_ok"]), "total": len(observations)},
        "metrics": {
            "dependencies": exact_metric(cases, observations, "dependencies"),
            "constituency": exact_metric(cases, observations, "constituency"),
            "entities": exact_metric(cases, observations, "entities"),
            "coreference": coref_metric(cases, observations),
            "harper_parity": harper_parity(cases, observations, harper),
        },
        "observations": observations,
    }
    result["span_alignment"]["accuracy"] = result["span_alignment"]["correct"] / result["span_alignment"]["total"] if result["span_alignment"]["total"] else None
    dump_json(output, result)


def combine(stanza_path, spacy_path, output_json, output_md):
    stanza = load_json(stanza_path)
    spacy = load_json(spacy_path)
    combined = {
        "schema_version": 1,
        "providers": {"stanza": stanza, "spacy": spacy},
        "selection_constraints": {
            "full_shadow_provider_requires": ["dependency", "constituency", "ner", "coreference", "canonical_byte_span_alignment", "offline_after_explicit_hydration"],
            "authority": "model output remains evidence and cannot grant ASD-STE100 or project/domain authority",
        },
    }
    dump_json(output_json, combined)

    def pct(metric):
        value = metric.get("accuracy")
        return "n/a" if value is None else f"{value * 100:.1f}%"

    rows = []
    for name, result in (("Stanza", stanza), ("spaCy", spacy)):
        metrics = result["metrics"]
        rows.append(
            "| {name} | {align} | {dep} | {con} | {ner} | {coref} | {pos} | {mean:.1f} | {rss:.0f} | {size:.1f} |".format(
                name=name, align=pct(result["span_alignment"]), dep=pct(metrics["dependencies"]), con=pct(metrics["constituency"]),
                ner=pct(metrics["entities"]), coref=pct(metrics["coreference"]), pos=pct(metrics["harper_parity"]["pos"]),
                mean=result["runtime"]["mean_ms_per_document"], rss=result["runtime"]["peak_rss_mib_after_inference"],
                size=result["artifact"]["bytes"] / (1024 * 1024),
            )
        )
    pathlib.Path(output_md).write_text("\n".join([
        "# Semantic provider evaluation", "", f"Python: `{stanza['runtime']['python']}`", "",
        "| Provider | Byte align | Dependency | Constituency | NER | Coref | POS vs Harper | Mean ms/doc | Peak RSS MiB | Artifact MiB |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |", *rows, "",
        "This is provider evidence, not standards authority. Ambiguous or conflicting evidence remains fail-closed.", "",
    ]), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    provider = sub.add_parser("provider")
    provider.add_argument("--provider", choices=["stanza", "spacy"], required=True)
    provider.add_argument("--cases", required=True)
    provider.add_argument("--harper", required=True)
    provider.add_argument("--model-dir", required=True)
    provider.add_argument("--output", required=True)
    provider.add_argument("--offline", action="store_true")
    hydrate = sub.add_parser("hydrate-stanza")
    hydrate.add_argument("--model-dir", required=True)
    combine_parser = sub.add_parser("combine")
    combine_parser.add_argument("--stanza", required=True)
    combine_parser.add_argument("--spacy", required=True)
    combine_parser.add_argument("--output-json", required=True)
    combine_parser.add_argument("--output-md", required=True)
    args = parser.parse_args()
    if args.command == "provider":
        evaluate_provider(args.provider, args.cases, args.harper, args.model_dir, args.output, args.offline)
    elif args.command == "hydrate-stanza":
        pipeline = stanza_load(pathlib.Path(args.model_dir), offline=False)
        print(json.dumps({
            "provider": "stanza", "version": __import__("stanza").__version__,
            "processors": list(getattr(pipeline, "processors", {}).keys()), "artifact": tree_identity(args.model_dir),
        }, indent=2, sort_keys=True))
    else:
        combine(args.stanza, args.spacy, args.output_json, args.output_md)


if __name__ == "__main__":
    main()
