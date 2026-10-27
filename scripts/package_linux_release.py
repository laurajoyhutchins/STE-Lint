#!/usr/bin/env python3
"""Stage a checksummed, source-pinned Linux CLI archive. Does not publish a release."""

import argparse
from datetime import datetime, timezone
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parent.parent


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def add_file(tar: tarfile.TarFile, path: Path, name: str, epoch: int,
             executable: bool = False) -> None:
    info = tar.gettarinfo(str(path), arcname=name)
    info.uid = 0
    info.gid = 0
    info.uname = ""
    info.gname = ""
    info.mtime = epoch
    info.mode = 0o755 if executable else 0o644
    with path.open("rb") as stream:
        tar.addfile(info, stream)


def stage(tag: str, destination: Path) -> Path:
    if not tag.startswith("v") or not tag[1:].replace(".", "").isdigit():
        raise ValueError("release tag must be an explicit vMAJOR.MINOR.PATCH")
    segments = tag[1:].split(".")
    if len(segments) != 3 or any(not s or (len(s) > 1 and s[0] == "0") for s in segments):
        raise ValueError("release tag must be an explicit vMAJOR.MINOR.PATCH")

    with (ROOT / "Cargo.toml").open("rb") as stream:
        version = tomllib.load(stream)["workspace"]["package"]["version"]
    if tag != "v" + version:
        raise ValueError("tag and workspace package version differ")

    source_sha = git("rev-parse", "HEAD")
    tagged_sha = git("rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    if source_sha != tagged_sha:
        raise ValueError("checked-out source is not the exact release tag commit")
    if git("status", "--porcelain", "--untracked-files=no"):
        raise ValueError("tracked source is dirty; refusing to produce release bytes")

    if platform.system() != "Linux":
        raise ValueError("this packager produces Linux-only artifacts")
    # Never label an unqualified build as GNU merely from uname. Host triples
    # such as musl are not ABI-compatible with the advertised GNU release.
    host_output = subprocess.check_output(
        ["rustc", "+1.97.1", "-vV"], cwd=ROOT, text=True
    )
    hosts = [line.partition(":")[2].strip() for line in host_output.splitlines()
             if line.startswith("host:")]
    if len(hosts) != 1 or hosts[0] not in {
        "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"
    }:
        raise ValueError("unsupported Rust host triple; refuse misleading artifact target")
    target = hosts[0]
    archive_name = f"ste-{version}-{target}.tar.gz"
    destination = destination.resolve()
    if destination == ROOT.resolve() or destination.is_relative_to(ROOT.resolve()):
        raise ValueError("staged release outputs must be outside the source repository")
    if any((destination / name).exists() for name in
           (archive_name, archive_name + ".sha256", archive_name + ".provenance.json")):
        raise ValueError("release output already exists; never overwrite retained evidence")

    epoch = int(git("show", "-s", "--format=%ct", "HEAD"))
    env = os.environ.copy()
    # Ambient target/flags or a private dictionary path must not change the
    # advertised binary or accidentally enter a public release build.
    for name in (
        "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER", "RUSTC_BOOTSTRAP", "STE_LINT_LEXICON",
    ):
        env.pop(name, None)
    env["SOURCE_DATE_EPOCH"] = str(epoch)
    subprocess.run([
        "cargo", "+1.97.1", "build", "-p", "ste-cli",
        "--target", target, "--locked", "--release",
    ], cwd=ROOT, env=env, check=True)
    executable = ROOT / "target" / target / "release" / "ste"
    subprocess.run([sys.executable, str(ROOT / "scripts" / "release_smoke.py"),
                    str(executable)], cwd=ROOT, env=env, check=True)

    destination.mkdir(parents=True, exist_ok=True)
    # Temp and final paths must share a filesystem for create-only hard links.
    # The manifest is a *last-written* completeness marker, not a substitute
    # for verifying the archive digest. Abandoned partial outputs stay visible
    # and block future staging rather than being clobbered on retry.
    with tempfile.TemporaryDirectory(prefix=".ste-release-", dir=destination) as temporary:
        compressed = Path(temporary) / archive_name
        with compressed.open("wb") as output:
            with gzip.GzipFile(fileobj=output, mode="wb", filename="", mtime=0) as gz:
                with tarfile.open(fileobj=gz, mode="w", format=tarfile.GNU_FORMAT) as tar:
                    prefix = f"ste-{version}-{target}"
                    add_file(tar, executable, prefix + "/ste", epoch, executable=True)
                    add_file(tar, ROOT / "README.md", prefix + "/README.md", epoch)
                    add_file(tar, ROOT / "LICENSE", prefix + "/LICENSE", epoch)
        archive_hash = digest(compressed)
        # Only byte identities are claimed. Toolchain pin alone cannot prove
        # bit-for-bit reproducibility across all hosts.
        manifest = {
            "schema": "ste-release-provenance/v1",
            "tag": tag,
            "source_commit": source_sha,
            "source_tree": git("rev-parse", "HEAD^{tree}"),
            "build_epoch_utc": datetime.fromtimestamp(epoch, timezone.utc).isoformat(),
            "toolchain": "1.97.1",
            "target": target,
            "archive_name": archive_name,
            "archive_sha256": archive_hash,
            "binary_sha256": digest(executable),
            "cargo_lock_sha256": digest(ROOT / "Cargo.lock"),
            "private_runtime_included": False,
            "reproducibility_claimed": False,
        }
        staged_checksum = Path(temporary) / (archive_name + ".sha256")
        staged_receipt = Path(temporary) / (archive_name + ".provenance.json")
        staged_checksum.write_text(
            f"{archive_hash}  {archive_name}\n", encoding="utf-8"
        )
        staged_receipt.write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        archive = destination / archive_name
        # os.link fails atomically on existing destinations; os.replace or
        # shutil.copyfile could overwrite proof generated by a parallel worker.
        os.link(compressed, archive)
        os.link(staged_checksum, destination / staged_checksum.name)
        os.link(staged_receipt, destination / staged_receipt.name)
        if digest(archive) != archive_hash:
            raise ValueError("staged release archive changed after publication")
        return archive


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True, help="already-created immutable release tag")
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    archive = stage(args.tag, args.output)
    print(f"Staged checksummed archive: {archive}; no upload or release published")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, subprocess.CalledProcessError) as exc:
        print(f"release package rejected: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
