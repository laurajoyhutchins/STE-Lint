"""Hostile offline tests for source-bound STE public release staging."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))
import package_linux_release as packaging  # noqa: E402
import verify_linux_release as verifier  # noqa: E402


class PackageAdmissionTests(unittest.TestCase):
    def test_malformed_tag_is_rejected_without_build(self):
        with tempfile.TemporaryDirectory() as output:
            for tag in ("v1.2", "v1..3", "v01.2.3", "v1.2.3-rc", "v1.2.3\ninjected"):
                with self.subTest(tag=tag):
                    with self.assertRaises(ValueError):
                        packaging.stage(tag, Path(output))

    def test_version_mismatch_rejected_before_git_or_cargo(self):
        with tempfile.TemporaryDirectory() as output:
            with patch.object(packaging, "git", side_effect=AssertionError("git should not run")):
                with self.assertRaisesRegex(ValueError, "version"):
                    packaging.stage("v99.99.99", Path(output))

    def test_wrong_source_commit_or_dirty_tree_prevents_build(self):
        with tempfile.TemporaryDirectory() as output:
            def wrong_tag(*args):
                return "a" * 40 if args == ("rev-parse", "HEAD") else "b" * 40

            with patch.object(packaging, "git", side_effect=wrong_tag):
                with self.assertRaisesRegex(ValueError, "exact release tag"):
                    packaging.stage("v0.1.0", Path(output))

            def dirty(*args):
                if args == ("status", "--porcelain", "--untracked-files=no"):
                    return " M README.md"
                return "a" * 40

            with patch.object(packaging, "git", side_effect=dirty):
                with self.assertRaisesRegex(ValueError, "dirty"):
                    packaging.stage("v0.1.0", Path(output))

    def test_existing_archive_cannot_be_overwritten(self):
        with tempfile.TemporaryDirectory() as output:
            location = Path(output) / "ste-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
            location.write_bytes(b"prior proof")
            with patch.object(packaging.platform, "system", return_value="Linux"), \
                 patch.object(packaging.subprocess, "check_output", return_value="host: x86_64-unknown-linux-gnu\n"), \
                 patch.object(packaging, "git", side_effect=lambda *args: "" if args[0] == "status" else "a" * 40):
                with self.assertRaisesRegex(ValueError, "already exists"):
                    packaging.stage("v0.1.0", Path(output))
            self.assertEqual(location.read_bytes(), b"prior proof")

    def test_unexpected_rust_host_is_not_mislabeled_as_linux_gnu(self):
        with tempfile.TemporaryDirectory() as output:
            with patch.object(packaging.platform, "system", return_value="Linux"), \
                 patch.object(packaging.subprocess, "check_output", return_value="host: x86_64-unknown-linux-musl\n"), \
                 patch.object(packaging, "git", side_effect=lambda *args: "" if args[0] == "status" else "a" * 40):
                with self.assertRaisesRegex(ValueError, "unsupported Rust host triple"):
                    packaging.stage("v0.1.0", Path(output))

    def test_staging_creates_checksum_and_last_written_receipt_without_overwrite(self):
        import hashlib
        with tempfile.TemporaryDirectory() as td:
            root = Path(td) / "checkout"
            output = Path(td) / "release"
            target = "x86_64-unknown-linux-gnu"
            binary = root / "target" / target / "release" / "ste"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"synthetic public executable fixture")
            (root / "Cargo.toml").write_text(
                '[workspace.package]\nversion = "0.1.0"\n', encoding="utf-8"
            )
            for name in ("README.md", "LICENSE", "Cargo.lock"):
                (root / name).write_text(name, encoding="utf-8")
            def git(*args):
                if args == ("status", "--porcelain", "--untracked-files=no"):
                    return ""
                if args[:2] == ("show", "-s"):
                    return "1730000000"
                if args == ("rev-parse", "HEAD^{tree}"):
                    return "b" * 40
                return "a" * 40
            with patch.object(packaging, "ROOT", root), \
                 patch.object(packaging.platform, "system", return_value="Linux"), \
                 patch.object(packaging.subprocess, "check_output",
                              return_value="host: x86_64-unknown-linux-gnu\n"), \
                 patch.object(packaging.subprocess, "run") as runner, \
                 patch.object(packaging, "git", side_effect=git):
                archive = packaging.stage("v0.1.0", output)
                receipt = json.loads((output / (archive.name + ".provenance.json")).read_text())
                checksum = (output / (archive.name + ".sha256")).read_text()
                actual_sha = hashlib.sha256(archive.read_bytes()).hexdigest()
                self.assertEqual(receipt["archive_sha256"], actual_sha)
                self.assertEqual(receipt["source_commit"], "a" * 40)
                self.assertEqual(receipt["source_tree"], "b" * 40)
                self.assertIn(actual_sha, checksum)
                self.assertFalse(receipt["private_runtime_included"])
                self.assertEqual(runner.call_count, 2)
                command = runner.call_args_list[0].args[0]
                self.assertIn("--target", command)
                self.assertEqual(command[command.index("--target") + 1], target)
                self.assertNotIn("STE_LINT_LEXICON", runner.call_args_list[0].kwargs["env"])
                self.assertNotIn("CARGO_BUILD_TARGET", runner.call_args_list[0].kwargs["env"])
                self.assertEqual(
                    Path(runner.call_args_list[1].args[0][2]), binary
                )
                independently_verified = verifier.verify(
                    archive, expected_tag="v0.1.0",
                    expected_source="a" * 40, expected_tree="b" * 40
                )
                self.assertTrue(independently_verified["byte_integrity_verified"])
                self.assertFalse(independently_verified["signed_provenance_verified"])
                for wrong_source, wrong_tree in (("c" * 40, "b" * 40),
                                                 ("a" * 40, "c" * 40)):
                    with self.assertRaisesRegex(ValueError, "identity"):
                        verifier.verify(archive, expected_tag="v0.1.0",
                                        expected_source=wrong_source,
                                        expected_tree=wrong_tree)
                before = archive.read_bytes()
                with self.assertRaisesRegex(ValueError, "already exists"):
                    packaging.stage("v0.1.0", output)
                self.assertEqual(before, archive.read_bytes())
                self.assertEqual(len(list(output.glob("*.provenance.json"))), 1)
                archive.write_bytes(before + b"tampered")
                with self.assertRaisesRegex(ValueError, "digest"):
                    verifier.verify(archive, expected_tag="v0.1.0",
                                    expected_source="a" * 40,
                                    expected_tree="b" * 40)


    def test_staging_inside_source_repo_is_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            with patch.object(packaging, "ROOT", root), \
                 patch.object(packaging, "git", side_effect=lambda *args: "" if args[0] == "status" else "a" * 40), \
                 patch.object(packaging.platform, "system", return_value="Linux"), \
                 patch.object(packaging.subprocess, "check_output",
                              return_value="host: x86_64-unknown-linux-gnu\n"):
                (root / "Cargo.toml").write_text(
                    '[workspace.package]\nversion = "0.1.0"\n'
                )
                with self.assertRaisesRegex(ValueError, "outside the source"):
                    packaging.stage("v0.1.0", root / "dist")


class PublicSmokeTests(unittest.TestCase):
    def _stub(self, directory: Path, *, version: str = "0.1.0",
              claim_compliance: bool = False, missing_status: int = 3) -> Path:
        binary = directory / "fake-ste"
        template = r'''#!/usr/bin/env python3
import json, os, sys
if "STE_LINT_LEXICON" in os.environ:
    print("ambient private data unexpectedly propagated", file=sys.stderr)
    sys.exit(7)
args = sys.argv[1:]
if args == ["--help"]:
    print("coverage dictionary lint version")
elif args == ["coverage", "--format", "json"]:
    print(json.dumps({"total_rules": 53, "full_compliance_claimed": __COMPLIANCE__,
                      "rules": [{"id": i} for i in range(53)]}))
elif args == ["--allow-test-lexicon", "version"]:
    print("ste __VERSION__\nruntime source: embedded test lexicon")
elif args == ["--allow-test-lexicon", "dictionary", "lookup", "USE", "--format", "json"]:
    print(json.dumps([{"lemma": "USE"}]))
elif args == ["dictionary", "lookup", "USE", "--format", "json"]:
    print(json.dumps({"schema": "ste-cli-error/v1",
        "error": {"kind": "invalid_data", "exit_code": __MISSING__}}))
    sys.exit(__MISSING__)
else:
    sys.exit(70)
'''
        binary.write_text(template.replace("__VERSION__", version)
                          .replace("__COMPLIANCE__", "True" if claim_compliance else "False")
                          .replace("__MISSING__", str(missing_status)), encoding="utf-8")
        binary.chmod(0o755)
        return binary

    def _run(self, binary: Path):
        environment = os.environ.copy()
        environment["STE_LINT_LEXICON"] = "/private/should-not-be-read.json"
        return subprocess.run([sys.executable, str(HERE / "release_smoke.py"), str(binary)],
                              env=environment, capture_output=True, text=True, check=False)

    def test_public_smoke_drops_ambient_private_runtime(self):
        with tempfile.TemporaryDirectory() as d:
            result = self._run(self._stub(Path(d)))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("no private runtime", result.stdout)

    def test_binary_version_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            result = self._run(self._stub(Path(d), version="9.9.9"))
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("version", result.stderr)

    def test_compliance_overstatement_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            result = self._run(self._stub(Path(d), claim_compliance=True))
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("coverage contract", result.stderr)

    def test_missing_runtime_must_exit_with_typed_code_three(self):
        with tempfile.TemporaryDirectory() as d:
            result = self._run(self._stub(Path(d), missing_status=0))
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unconfigured private runtime", result.stderr)

    def test_missing_executable_is_not_success(self):
        with tempfile.TemporaryDirectory() as d:
            result = self._run(Path(d) / "missing")
            self.assertEqual(result.returncode, 2)


if __name__ == "__main__":
    unittest.main()
