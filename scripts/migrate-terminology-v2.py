import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUPPORTS = ["admission", "definition", "role", "forms", "alias", "status"]
ROLE_MAP = {
    "technical_noun": ["noun"],
    "technical_verb": ["verb"],
    "technical_noun_and_verb": ["noun", "verb"],
}


def slug(value: str) -> str:
    return re.sub(r"(^-+|-+$)", "", re.sub(r"[^a-z0-9]+", "-", value.lower()))


def source_id(raw: str, catalog: dict) -> str:
    for key, value in catalog.items():
        if value.get("_raw") == raw:
            return key
    base = slug(raw.split("; reviewed")[0])[:48] or "source"
    if raw.startswith("http://") or raw.startswith("https://"):
        from urllib.parse import urlparse
        base = slug(urlparse(raw).netloc.removeprefix("www.")) or "source"
    key = base
    suffix = 1
    while key in catalog:
        key = f"{base}-{suffix}"
        suffix += 1
    reviewed = re.search(r"; reviewed (\d{4}-\d{2}-\d{2})$", raw)
    title = re.sub(r"; reviewed \d{4}-\d{2}-\d{2}$", "", raw)
    value = {"title": title, "_raw": raw}
    if raw.startswith("http://") or raw.startswith("https://"):
        value["url"] = raw
    if reviewed:
        value["reviewed_on"] = reviewed.group(1)
    catalog[key] = value
    return key


def alias_kind(text: str) -> str:
    letters = [c for c in text if c.isalpha()]
    if letters and all(c.isupper() for c in letters) and " " not in text:
        return "abbreviation"
    return "short_form"


def migrate_profile(path: Path) -> None:
    old = json.loads(path.read_text())
    if old.get("schema") == "ste-terminology/v2":
        return
    catalog = {}
    for raw in old["profile"].get("sources", []):
        source_id(raw, catalog)
    terms = []
    for old_term in old["terms"]:
        roles = ROLE_MAP[old_term["kind"]]
        provenance = old_term.get("provenance") or old["profile"].get("sources", [])
        refs = [source_id(raw, catalog) for raw in provenance]
        term = {
            "id": slug(old_term["term"]),
            "canonical": old_term["term"],
            "roles": roles,
            "definition": old_term["definition"],
            "forms": [{"text": text, "roles": roles} for text in old_term.get("forms", [])],
            "aliases": [
                {"text": text, "kind": alias_kind(text)} for text in old_term.get("aliases", [])
            ],
            "sources": [{"source": ref, "supports": SUPPORTS} for ref in refs],
            "status": old_term["status"],
        }
        if old_term.get("examples"):
            term["examples"] = old_term["examples"]
        terms.append(term)
    for value in catalog.values():
        value.pop("_raw", None)
    new = {
        "schema": "ste-terminology/v2",
        "profile": {
            "id": old["profile"]["id"],
            "version": old["profile"]["version"],
            "domain": old["profile"]["id"],
            "description": old["profile"]["description"],
        },
        "sources": catalog,
        "terms": terms,
    }
    path.write_text(json.dumps(new, indent=2) + "\n")


def migrate_fixture(path: Path) -> None:
    old = json.loads(path.read_text())
    if old.get("schema") == "ste-terminology/v2":
        return
    catalog = {"fixture": {"title": "STE-Lint glossary fixture"}}
    terms = []
    for old_term in old["terms"]:
        roles = ROLE_MAP[old_term["kind"]]
        terms.append({
            "id": slug(old_term["term"]),
            "canonical": old_term["term"],
            "roles": roles,
            "definition": old_term["definition"],
            "forms": [{"text": text, "roles": roles} for text in old_term.get("forms", [])],
            "aliases": [{"text": text, "kind": "synonym"} for text in old_term.get("aliases", [])],
            "sources": [{"source": "fixture", "supports": SUPPORTS}],
            "status": old_term["status"],
        })
    new = {
        "schema": "ste-terminology/v2",
        "domain": "fixture",
        "sources": catalog,
        "terms": terms,
    }
    path.write_text(json.dumps(new, indent=2) + "\n")


for name in ["software-core", "git", "github"]:
    migrate_profile(ROOT / "profiles" / f"{name}.json")
for path in (ROOT / "fixtures" / "glossary").glob("*.json"):
    migrate_fixture(path)
