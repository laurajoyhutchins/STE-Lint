#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import subprocess
from datetime import datetime, timezone
from pathlib import Path

import pdfplumber
from pypdf import PdfReader

POS_RE = re.compile(r"\((adj|adv|art|conj|n|prep|pron|v)\)", re.I)
RULE_RE = re.compile(r"^\s*Rule\s+(\d+\.\d+)\s+(.*)$", re.I)
GR_RE = re.compile(r"^\s*GR-(\d+)\s+(.*)$", re.I)
SECTION_SUMMARY = {1: 45, 2: 63, 3: 67, 4: 77, 5: 87, 6: 95, 7: 103, 8: 107, 9: 115}
SPECIAL_HEADWORDS = {"FOR EXAMPLE", "such as"}
ODD_X = [72.0, 176.4, 306.0, 435.7, 565.3]
EVEN_X = [50.4, 154.8, 284.4, 414.1, 543.7]
TABLE_SETTINGS_BASE = dict(
    vertical_strategy="explicit",
    horizontal_strategy="lines",
    snap_tolerance=2,
    join_tolerance=3,
    intersection_tolerance=3,
    text_tolerance=2,
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def clean_lines(lines: list[str]) -> list[str]:
    cleaned = []
    for line in lines:
        text = line.rstrip()
        stripped = text.strip()
        if not stripped:
            cleaned.append("")
            continue
        if stripped == "ASD-STE100 Simplified Technical English":
            continue
        if re.match(r"^(Issue 9|2025-01-15)$", stripped):
            continue
        if re.search(r"Part 1 - Writing [Rr]ules", stripped) and "Page" in stripped:
            continue
        if re.search(r"Part 2 - Dictionary", stripped) and "Page" in stripped:
            continue
        if re.match(r"^Page\s+[12]-", stripped):
            continue
        cleaned.append(text)
    while cleaned and not cleaned[0].strip():
        cleaned.pop(0)
    while cleaned and not cleaned[-1].strip():
        cleaned.pop()
    return cleaned


def logical_page_label(text: str) -> str | None:
    match = re.search(r"\bPage\s+([12]-[0-9A-Za-z-]+)", text)
    return match.group(1) if match else None


def extract_pages(pdf: Path, outdir: Path) -> list[str]:
    text_path = outdir / "issue9-layout.txt"
    subprocess.run(["pdftotext", "-layout", str(pdf), str(text_path)], check=True)
    pages = text_path.read_text(errors="replace").split("\f")
    if pages and not pages[-1].strip():
        pages = pages[:-1]
    return pages


def extract_rules(pages: list[str]) -> list[dict]:
    occurrences: dict[str, list[tuple[int, int, str]]] = {}
    for page_number in range(43, 129):
        lines = pages[page_number - 1].splitlines()
        for index, line in enumerate(lines):
            match = RULE_RE.match(line)
            if match:
                occurrences.setdefault(match.group(1), []).append(
                    (page_number, index, match.group(2).strip())
                )

    ordered = []
    for section in range(1, 10):
        ids = sorted(
            [rule_id for rule_id in occurrences if rule_id.startswith(f"{section}.")],
            key=lambda rule_id: int(rule_id.split(".")[1]),
        )
        summary_page = SECTION_SUMMARY[section]
        for rule_id in ids:
            candidates = [item for item in occurrences[rule_id] if item[0] >= summary_page]
            start = candidates[1] if len(candidates) > 1 else candidates[0]
            ordered.append((rule_id, start))

    seen = set()
    starts = []
    for rule_id, start in ordered:
        if rule_id not in seen:
            starts.append((rule_id, start))
            seen.add(rule_id)

    rules = []
    for index, (rule_id, (start_page, start_line, title_first)) in enumerate(starts):
        if index + 1 < len(starts):
            end_page, end_line = starts[index + 1][1][0], starts[index + 1][1][1]
        else:
            end_page, end_line = 123, 0

        chunks = []
        for page_number in range(start_page, end_page + 1):
            lines = pages[page_number - 1].splitlines()
            first = start_line if page_number == start_page else 0
            last = end_line if page_number == end_page else len(lines)
            chunks.extend(clean_lines(lines[first:last]))

        title_parts = [title_first]
        lines = pages[start_page - 1].splitlines()
        cursor = start_line + 1
        while cursor < len(lines) and lines[cursor].strip() and not lines[cursor].lstrip().startswith(
            ("Examples:", "Example:", "A ", "The ", "In ", "You ", "Use ", "Do ", "This ", "If ", "When ", "STE ")
        ):
            continuation = lines[cursor].strip()
            if len(continuation) > 110:
                break
            title_parts.append(continuation)
            cursor += 1

        rules.append(
            {
                "id": rule_id,
                "section": int(rule_id.split(".")[0]),
                "title": " ".join(part for part in title_parts if part).strip(),
                "start_pdf_page": start_page,
                "end_pdf_page": end_page if end_line > 0 else end_page - 1,
                "logical_page_start": logical_page_label(pages[start_page - 1]),
                "text": "\n".join(chunks).strip(),
            }
        )
    return rules


def extract_general_recommendations(pages: list[str]) -> list[dict]:
    starts = []
    for page_number in range(123, 128):
        lines = pages[page_number - 1].splitlines()
        for index, line in enumerate(lines):
            match = GR_RE.match(line)
            if match:
                starts.append((int(match.group(1)), page_number, index, match.group(2).strip()))

    recommendations = []
    for index, (number, start_page, start_line, title) in enumerate(starts):
        if index + 1 < len(starts):
            end_page, end_line = starts[index + 1][1], starts[index + 1][2]
        else:
            end_page, end_line = 128, 0
        chunks = []
        for page_number in range(start_page, end_page + 1):
            lines = pages[page_number - 1].splitlines()
            first = start_line if page_number == start_page else 0
            last = end_line if page_number == end_page else len(lines)
            chunks.extend(clean_lines(lines[first:last]))
        recommendations.append(
            {
                "id": f"GR-{number}",
                "title": title,
                "start_pdf_page": start_page,
                "end_pdf_page": end_page if end_line > 0 else end_page - 1,
                "text": "\n".join(chunks).strip(),
            }
        )
    return recommendations


def table_settings(page_number: int) -> dict:
    settings = TABLE_SETTINGS_BASE.copy()
    settings["explicit_vertical_lines"] = ODD_X if page_number % 2 else EVEN_X
    return settings


def is_new_headword(cell: str | None) -> bool:
    text = (cell or "").strip()
    return bool(text and (POS_RE.search(text) or text in SPECIAL_HEADWORDS))


def normalize_headword(raw: str) -> tuple[str, str | None, list[str]]:
    raw = " ".join(raw.replace("\n", " ").split())
    if raw in SPECIAL_HEADWORDS:
        return raw, None, []
    match = POS_RE.search(raw)
    part_of_speech = match.group(1).lower() if match else None
    if match:
        headword = raw[: match.start()].strip().rstrip(",")
    else:
        headword = raw
    headword = headword.split(",")[0].strip()
    forms = []
    if part_of_speech == "v":
        pieces = [piece.strip(" ,.") for piece in re.split(r"[,\n]+", raw) if piece.strip()]
        for piece in pieces:
            piece = POS_RE.sub("", piece).strip()
            if piece and piece.lower() != "no other verb forms" and not piece.startswith("No other"):
                forms.append(piece)
    return headword, part_of_speech, forms


def approved_from_headword(headword: str) -> bool:
    letters = "".join(character for character in headword if character.isalpha())
    return bool(letters and letters.upper() == letters)


def extract_dictionary(pdf: Path) -> tuple[list[dict], list[dict]]:
    raw_rows = []
    entries = []
    current = None
    with pdfplumber.open(pdf) as document:
        for page_number in range(149, 435):
            table = document.pages[page_number - 1].extract_table(table_settings(page_number))
            if not table:
                continue
            for row_index, row in enumerate(table):
                row = [(cell or "").strip() if cell is not None else None for cell in row]
                raw_rows.append({"pdf_page": page_number, "row_index": row_index, "cells": row})
                first_cell = (row[0] or "").strip()
                if is_new_headword(first_cell):
                    if current:
                        entries.append(current)
                    current = {"headword_raw": first_cell, "fragments": [], "source_pages": []}
                elif first_cell and current:
                    current["headword_raw"] += "\n" + first_cell
                if current:
                    current["fragments"].append(
                        {"pdf_page": page_number, "row_index": row_index, "cells": row}
                    )
                    if page_number not in current["source_pages"]:
                        current["source_pages"].append(page_number)
    if current:
        entries.append(current)

    for entry in entries:
        headword, part_of_speech, forms = normalize_headword(entry["headword_raw"])
        entry["headword"] = headword
        entry["part_of_speech"] = part_of_speech
        entry["forms"] = forms
        entry["approved"] = approved_from_headword(headword)
        columns = [[], [], [], []]
        for fragment in entry["fragments"]:
            for index, cell in enumerate(fragment["cells"]):
                if cell:
                    columns[index].append(cell)
        entry["word_cell"] = "\n".join(columns[0])
        entry["meaning_or_alternatives"] = "\n".join(columns[1])
        entry["ste_example"] = "\n".join(columns[2])
        entry["non_ste_example"] = "\n".join(columns[3])
    return raw_rows, entries


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def build_sqlite(path: Path, source: dict, pages: list[str], rules: list[dict], recommendations: list[dict], entries: list[dict], raw_rows: list[dict]) -> None:
    if path.exists():
        path.unlink()
    connection = sqlite3.connect(path)
    connection.executescript(
        """
        create table source(key text primary key, value text not null);
        create table pages(pdf_page integer primary key, logical_page text, text text not null, text_sha256 text not null);
        create table rules(id text primary key, section integer, title text, start_pdf_page integer, end_pdf_page integer, text text);
        create table recommendations(id text primary key, title text, start_pdf_page integer, end_pdf_page integer, text text);
        create table dictionary_entries(id integer primary key, headword text, headword_raw text, part_of_speech text, approved integer, forms_json text, meaning_or_alternatives text, ste_example text, non_ste_example text, source_pages_json text);
        create table dictionary_rows(pdf_page integer,row_index integer,cells_json text,primary key(pdf_page,row_index));
        """
    )
    connection.executemany(
        "insert into source values (?,?)",
        [(key, json.dumps(value, ensure_ascii=False)) for key, value in source.items()],
    )
    connection.executemany(
        "insert into pages values (?,?,?,?)",
        [
            (index + 1, logical_page_label(text), text, hashlib.sha256(text.encode()).hexdigest())
            for index, text in enumerate(pages)
        ],
    )
    connection.executemany(
        "insert into rules values (?,?,?,?,?,?)",
        [
            (rule["id"], rule["section"], rule["title"], rule["start_pdf_page"], rule["end_pdf_page"], rule["text"])
            for rule in rules
        ],
    )
    connection.executemany(
        "insert into recommendations values (?,?,?,?,?)",
        [
            (item["id"], item["title"], item["start_pdf_page"], item["end_pdf_page"], item["text"])
            for item in recommendations
        ],
    )
    connection.executemany(
        "insert into dictionary_entries values (?,?,?,?,?,?,?,?,?,?)",
        [
            (
                index + 1,
                entry["headword"],
                entry["headword_raw"],
                entry["part_of_speech"],
                int(entry["approved"]),
                json.dumps(entry["forms"], ensure_ascii=False),
                entry["meaning_or_alternatives"],
                entry["ste_example"],
                entry["non_ste_example"],
                json.dumps(entry["source_pages"]),
            )
            for index, entry in enumerate(entries)
        ],
    )
    connection.executemany(
        "insert into dictionary_rows values (?,?,?)",
        [
            (row["pdf_page"], row["row_index"], json.dumps(row["cells"], ensure_ascii=False))
            for row in raw_rows
        ],
    )
    connection.commit()
    connection.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("pdf", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    out = args.out
    out.mkdir(parents=True, exist_ok=True)
    pages = extract_pages(args.pdf, out)
    reader = PdfReader(str(args.pdf))
    metadata = {str(key).lstrip("/"): str(value) for key, value in (reader.metadata or {}).items()}
    source = {
        "title": "ASD-STE100 Simplified Technical English",
        "issue": 9,
        "publication_date": "2025-01-15",
        "drive_file_id": "1GfSldRfzXs91pG1BbgLjbzJFJML_wifP",
        "mime_type": "application/pdf",
        "byte_size": args.pdf.stat().st_size,
        "sha256": sha256_file(args.pdf),
        "pdf_pages": len(reader.pages),
        "encrypted": bool(reader.is_encrypted),
        "metadata": metadata,
        "ingested_at": datetime.now(timezone.utc).isoformat(),
    }

    rules = extract_rules(pages)
    recommendations = extract_general_recommendations(pages)
    raw_rows, entries = extract_dictionary(args.pdf)
    page_records = [
        {
            "pdf_page": index + 1,
            "logical_page": logical_page_label(text),
            "text": text,
            "text_sha256": hashlib.sha256(text.encode()).hexdigest(),
        }
        for index, text in enumerate(pages)
    ]

    write_json(out / "source.json", source)
    write_json(out / "rules.json", rules)
    write_json(out / "general-recommendations.json", recommendations)
    write_json(out / "dictionary.json", entries)
    with (out / "pages.jsonl").open("w") as stream:
        for record in page_records:
            stream.write(json.dumps(record, ensure_ascii=False) + "\n")
    with (out / "dictionary-rows.jsonl").open("w") as stream:
        for row in raw_rows:
            stream.write(json.dumps(row, ensure_ascii=False) + "\n")

    build_sqlite(out / "issue9-authority.sqlite3", source, pages, rules, recommendations, entries, raw_rows)
    approved = [entry for entry in entries if entry["approved"]]
    validations = {
        "pdf_page_count_434": len(reader.pages) == 434,
        "rules_count_53": len(rules) == 53,
        "general_recommendations_count_8": len(recommendations) == 8,
        "dictionary_entries": len(entries),
        "approved_entry_count": len(approved),
        "standard_states_875_approved_words": 875,
        "raw_dictionary_rows": len(raw_rows),
        "dictionary_pages_without_table": [
            page_number
            for page_number in range(149, 435)
            if not any(row["pdf_page"] == page_number for row in raw_rows)
        ],
    }
    manifest = {
        "source": source,
        "counts": {
            "pages": len(pages),
            "rules": len(rules),
            "general_recommendations": len(recommendations),
            "dictionary_entries": len(entries),
            "approved_entries": len(approved),
            "unapproved_entries": len(entries) - len(approved),
            "dictionary_rows": len(raw_rows),
        },
        "validations": validations,
        "artifacts": {},
    }
    for name in [
        "source.json",
        "rules.json",
        "general-recommendations.json",
        "dictionary.json",
        "pages.jsonl",
        "dictionary-rows.jsonl",
        "issue9-authority.sqlite3",
        "issue9-layout.txt",
    ]:
        path = out / name
        manifest["artifacts"][name] = {"bytes": path.stat().st_size, "sha256": sha256_file(path)}
    write_json(out / "manifest.json", manifest)
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
