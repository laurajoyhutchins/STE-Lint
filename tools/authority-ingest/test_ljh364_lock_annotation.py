import base64
import gzip
import pathlib
import subprocess
import unittest


class TemporaryLockRecovery(unittest.TestCase):
    def test_emit_complete_compressed_lockfile(self):
        root = pathlib.Path(__file__).resolve().parents[2]
        subprocess.run(
            ["rustup", "toolchain", "install", "1.97.1", "--profile", "minimal", "--no-self-update"],
            cwd=root,
            check=True,
        )
        subprocess.run(["cargo", "+1.97.1", "generate-lockfile"], cwd=root, check=True)
        payload = base64.b64encode(gzip.compress((root / "Cargo.lock").read_bytes())).decode("ascii")
        chunks = [payload[i : i + 1800] for i in range(0, len(payload), 1800)]
        print(f"LJH364_LOCK_CHUNK_COUNT={len(chunks)}", flush=True)
        for index, chunk in enumerate(chunks):
            print(f"LJH364_LOCK_CHUNK_{index:03d}={chunk}", flush=True)
        self.fail("emitted complete compressed Cargo.lock")


if __name__ == "__main__":
    unittest.main()