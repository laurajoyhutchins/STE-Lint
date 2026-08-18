import pathlib
import subprocess
import tomllib
import unittest


class TemporaryLockRecovery(unittest.TestCase):
    def test_report_parser_dependency_resolution(self):
        root = pathlib.Path(__file__).resolve().parents[2]
        subprocess.run(
            ["rustup", "toolchain", "install", "1.97.1", "--profile", "minimal", "--no-self-update"],
            cwd=root,
            check=True,
        )
        subprocess.run(["cargo", "+1.97.1", "generate-lockfile"], cwd=root, check=True)
        lock = tomllib.loads((root / "Cargo.lock").read_text())
        selected = [
            (package["name"], package["version"])
            for package in lock["package"]
            if package["name"] in {"harper-core", "harper-brill", "pulldown-cmark"}
        ]
        print(f"LJH364_SELECTED={selected!r}", flush=True)
        self.fail("reported parser dependency resolution")


if __name__ == "__main__":
    unittest.main()
