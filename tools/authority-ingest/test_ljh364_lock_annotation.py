import base64
import pathlib
import subprocess
import unittest


class TemporaryLockAnnotation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
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
        cls.payload = base64.b64encode((root / "Cargo.lock").read_bytes()).decode("ascii")


def _make_chunk_test(index):
    def test_chunk(self):
        start = index * 8000
        chunk = self.payload[start : start + 8000]
        if chunk:
            self.fail(f"LJH364_LOCK_CHUNK_{index:02d}:{chunk}")
    return test_chunk


for _index in range(24):
    setattr(TemporaryLockAnnotation, f"test_chunk_{_index:02d}", _make_chunk_test(_index))


if __name__ == "__main__":
    unittest.main()