import base64
import pathlib
import subprocess
import unittest


class TemporaryLockGeneration(unittest.TestCase):
    def test_emit_generated_lockfile(self):
        root = pathlib.Path(__file__).resolve().parents[2]
        subprocess.run(
            [
                "rustup",
                "toolchain",
                "install",
                "1.97.1",
                "--profile",
                "minimal",
                "--no-self-update",
            ],
            cwd=root,
            check=True,
        )
        subprocess.run(["cargo", "+1.97.1", "generate-lockfile"], cwd=root, check=True)
        payload = base64.b64encode((root / "Cargo.lock").read_bytes()).decode("ascii")
        print("LJH364_LOCKFILE_BASE64_BEGIN")
        print(payload)
        print("LJH364_LOCKFILE_BASE64_END")
        self.fail("temporary lockfile extraction probe")


if __name__ == "__main__":
    unittest.main()