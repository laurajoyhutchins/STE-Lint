import unittest
from ingest_issue9 import approved_from_headword, extract_declared_dictionary_counts, validate_dictionary_table_coverage


class ApprovalClassificationTests(unittest.TestCase):
    def test_uppercase_alternative_spelling_is_approved(self):
        self.assertTrue(approved_from_headword("MATT (or MATTE)"))

    def test_lowercase_headword_is_unapproved(self):
        self.assertFalse(approved_from_headword("acceptable"))


class DeclaredDictionaryCountTests(unittest.TestCase):
    def test_extracts_source_declared_counts_without_equating_them_to_record_counts(self):
        pages = [""] * 434
        pages[132] = (
            "The dictionary gives the words that are approved in STE (875 approved words) and the examples\n"
            "The dictionary also includes a selection of words that are not approved (1274 words)."
        )
        self.assertEqual(
            extract_declared_dictionary_counts(pages),
            {"approved_words": 875, "unapproved_words": 1274},
        )


class DictionaryCoverageTests(unittest.TestCase):
    def test_accepts_only_blank_dictionary_pages_without_tables(self):
        pages = ["entry"] * 4
        pages[1] = "Blank Page"
        rows = [{"pdf_page": 1}, {"pdf_page": 3}, {"pdf_page": 4}]
        self.assertEqual(validate_dictionary_table_coverage(pages, rows, start_page=1, end_page=4), [2])

    def test_rejects_nonblank_dictionary_page_without_table(self):
        pages = ["entry"] * 3
        rows = [{"pdf_page": 1}, {"pdf_page": 3}]
        with self.assertRaisesRegex(ValueError, "nonblank dictionary page"):
            validate_dictionary_table_coverage(pages, rows, start_page=1, end_page=3)


if __name__ == "__main__":
    unittest.main()
