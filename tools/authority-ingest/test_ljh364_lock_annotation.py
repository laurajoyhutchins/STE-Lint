import base64
import pathlib
import subprocess
import unittest


class TemporaryLockAnnotation(unittest.TestCase):
    def test_emit_lockfile_annotations(self):
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
        chunks = [payload[i : i + 6000] for i in range(0, len(payload), 6000)]
        for index, chunk in enumerate(chunks):
            print(
                f"::error file=tools/authority-ingest/test_ljh364_lock_annotation.py,line=1,title=LJH364_LOCK_CHUNK_{index:02d}::LJH364_LOCK_CHUNK_{index:02d}:{chunk}",
                flush=True,
            )
        self.fail(f"emitted {len(chunks)} Cargo.lock chunks")


if __name__ == "__main__":
    unittest.main()