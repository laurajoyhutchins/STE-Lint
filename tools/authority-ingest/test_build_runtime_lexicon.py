from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from build_runtime_lexicon import (
    AuthorityValidationError,
    compile_document,
    map_part_of_speech,
    normalize_forms,
    validate_authority,
    write_document,
)


ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "fixtures" / "authority-ingest"
BUNDLE_SHA256 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"


def load(name: str):
    return json.loads((FIXTURE / name).read_text())


class RuntimeLexiconCompilerTests(unittest.TestCase):
    def test_maps_source_part_of_speech_without_inventing_expression_pos(self):
        self.assertEqual(map_part_of_speech("v"), "verb")
        self.assertEqual(map_part_of_speech("adj"), "adjective")
        self.assertIsNone(map_part_of_speech(None))

    def test_normalizes_explicit_forms_without_inventing_morphology(self):
        entry = {
            "headword": "CHECK",
            "part_of_speech": "v",
            "forms": ["CHECK", "CHECKS", "CHECKED No other verb forms"],
        }
        self.assertEqual(normalize_forms(entry), ["CHECK", "CHECKS", "CHECKED"])
        self.assertEqual(
            normalize_forms(
                {
                    "headword": "CHECK AGAIN",
                    "part_of_speech": None,
                    "forms": [],
                }
            ),
            ["CHECK AGAIN"],
        )

    def test_rejects_source_identity_mismatch(self):
        source = load("source.json")
        private_manifest = load("manifest.json")
        verified_manifest = load("verified-manifest.json")
        dictionary = load("dictionary.json")
        source["sha256"] = "f" * 64

        with self.assertRaisesRegex(AuthorityValidationError, "source sha256"):
            validate_authority(
                source,
                private_manifest,
                verified_manifest,
                dictionary,
                BUNDLE_SHA256,
            )

    def test_rejects_structural_cardinality_mismatch(self):
        source = load("source.json")
        private_manifest = load("manifest.json")
        verified_manifest = load("verified-manifest.json")
        dictionary = load("dictionary.json")
        verified_manifest = copy.deepcopy(verified_manifest)
        verified_manifest["verified_ingest"]["approved_headword_records"] = 4

        with self.assertRaisesRegex(AuthorityValidationError, "approved structural"):
            validate_authority(
                source,
                private_manifest,
                verified_manifest,
                dictionary,
                BUNDLE_SHA256,
            )

    def test_compiles_losslessly_and_deterministically(self):
        first = compile_document(
            FIXTURE,
            FIXTURE / "verified-manifest.json",
            BUNDLE_SHA256,
        )
        second = compile_document(
            FIXTURE,
            FIXTURE / "verified-manifest.json",
            BUNDLE_SHA256,
        )

        self.assertEqual(first, second)
        self.assertEqual(first["metadata"]["scope"], "issue9_private_authority_runtime")
        self.assertEqual(
            first["metadata"]["dictionary_cardinalities"],
            {
                "source_declared_approved_words": 2,
                "source_declared_unapproved_words": 2,
                "structural_approved_records": 3,
                "structural_unapproved_records": 2,
            },
        )

        check = first["entries"][0]
        self.assertEqual(check["forms"], ["CHECK", "CHECKS", "CHECKED"])
        self.assertEqual(check["interpretation_state"], "structural")
        self.assertEqual(check["provenance"]["source_pages"], [1])
        self.assertEqual(
            check["source_semantics"]["meaning_or_alternatives"],
            "Synthetic approved meaning.",
        )

        simplebad = first["entries"][1]
        self.assertEqual(simplebad["interpretation_state"], "interpreted")
        self.assertEqual(simplebad["alternatives"][0]["kind"], "approved_word")
        self.assertEqual(simplebad["alternatives"][0]["strategy"], "word_replacement")

        domainthing = first["entries"][2]
        self.assertEqual(domainthing["alternatives"][0]["kind"], "technical_noun")
        self.assertIsNone(domainthing["alternatives"][0]["part_of_speech"])

        expression = first["entries"][3]
        self.assertIsNone(expression["part_of_speech"])
        self.assertEqual(expression["forms"], ["CHECK AGAIN"])

        contextual = first["entries"][4]
        self.assertEqual(contextual["interpretation_state"], "structural")
        self.assertIn("source help", contextual["source_semantics"]["meaning_or_alternatives"])

        with tempfile.TemporaryDirectory() as tmp:
            a = Path(tmp) / "a.json"
            b = Path(tmp) / "b.json"
            write_document(first, a)
            write_document(second, b)
            self.assertEqual(a.read_bytes(), b.read_bytes())
            self.assertTrue(a.read_bytes().endswith(b"\n"))


if __name__ == "__main__":
    unittest.main()
