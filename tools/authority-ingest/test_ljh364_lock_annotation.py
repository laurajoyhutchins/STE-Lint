import base64
import hashlib
import pathlib
import subprocess
import unittest


class TemporaryLockAnnotation(unittest.TestCase):
    def test_emit_lockfile_annotations(self):
        root = pathlib.Path(__file__).resolve().parents[2]
        subprocess.run(["rustup", "toolchain", "install", "1.97.1", "--profile", "minimal", "--no-self-update"], cwd=root, check=True)
        subprocess.run(["cargo", "+1.97.1", "generate-lockfile"], cwd=root, check=True)
        lock = (root / "Cargo.lock").read_bytes()
        payload = base64.b64encode(lock).decode("ascii")
        chunks = [payload[i:i + 18000] for i in range(0, len(payload), 18000)]
        print(f"::error file=tools/authority-ingest/test_ljh364_lock_annotation.py,line=1,title=LJH364_LOCK_META::sha256={hashlib.sha256(lock).hexdigest()};bytes={len(lock)};chunks={len(chunks)}", flush=True)
        for index, chunk in enumerate(chunks):
            print(f"::error file=tools/authority-ingest/test_ljh364_lock_annotation.py,line=1,title=LJH364_LOCK_CHUNK_{index:02d}::LJH364_LOCK_CHUNK_{index:02d}:{chunk}", flush=True)
        tree = subprocess.run(["cargo", "+1.97.1", "tree", "-p", "ste-lint", "--edges", "normal"], cwd=root, check=True, capture_output=True, text=True).stdout
        tree64 = base64.b64encode(tree.encode()).decode("ascii")
        tree_chunks = [tree64[i:i + 18000] for i in range(0, len(tree64), 18000)]
        for index, chunk in enumerate(tree_chunks):
            print(f"::error file=tools/authority-ingest/test_ljh364_lock_annotation.py,line=1,title=LJH364_TREE_CHUNK_{index:02d}::LJH364_TREE_CHUNK_{index:02d}:{chunk}", flush=True)
        self.fail("emitted Cargo.lock and active dependency tree")


if __name__ == "__main__":
    unittest.main()