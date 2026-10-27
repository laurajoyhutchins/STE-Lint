#!/usr/bin/env python3
"""Verify staged public STE Linux release bytes against independently supplied Git identity.

This is byte-integrity and custody checking, not signed provenance, build
reproducibility, distribution authorization or ASD-STE100 compliance proof.
Never extracts archive contents or executes its binary.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tarfile

SHA40 = re.compile(r"[0-9a-f]{40}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
SUPPORTED = {"x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"}
MAX_ARCHIVE_BYTES = 128 * 1024 * 1024
MAX_UNPACKED_BYTES = 128 * 1024 * 1024
MAX_MEMBERS = 3


def sha256(path: Path) -> str:
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for part in iter(lambda: stream.read(1 << 20), b""):
            result.update(part)
    return result.hexdigest()


def verify(archive: Path, *, expected_tag: str,
           expected_source: str, expected_tree: str) -> dict:
    if not SHA40.fullmatch(expected_source) or not SHA40.fullmatch(expected_tree):
        raise ValueError("expected source commit/tree must be full lowercase Git SHAs")
    if not re.fullmatch(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", expected_tag):
        raise ValueError("expected tag must be a stable vMAJOR.MINOR.PATCH")
    if not archive.is_file() or archive.stat().st_size > MAX_ARCHIVE_BYTES:
        raise ValueError("archive absent or exceeds bounded review size")

    receipt_path = archive.with_name(archive.name + ".provenance.json")
    sum_path = archive.with_name(archive.name + ".sha256")
    if not receipt_path.is_file() or not sum_path.is_file():
        raise ValueError("release is incomplete: checksum or provenance receipt absent")
    if receipt_path.stat().st_size > 8192 or sum_path.stat().st_size > 256:
        raise ValueError("release evidence exceeds size limits")
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    if not isinstance(receipt, dict) or receipt.get("schema") != "ste-release-provenance/v1":
        raise ValueError("unrecognized provenance schema")
    if (receipt.get("tag") != expected_tag
            or receipt.get("source_commit") != expected_source
            or receipt.get("source_tree") != expected_tree):
        raise ValueError("provenance identity differs from independently supplied Git identity")
    version = expected_tag[1:]
    target = receipt.get("target")
    if target not in SUPPORTED or receipt.get("toolchain") != "1.97.1":
        raise ValueError("unsupported target or toolchain")
    expected_name = f"ste-{version}-{target}.tar.gz"
    if receipt.get("archive_name") != expected_name or archive.name != expected_name:
        raise ValueError("archive filename does not match versioned target")
    if receipt.get("private_runtime_included") is not False:
        raise ValueError("public archive may not claim private runtime inclusion")
    if receipt.get("reproducibility_claimed") is not False:
        raise ValueError("unsupported reproducibility assertion")
    for field in ("archive_sha256", "binary_sha256", "cargo_lock_sha256"):
        if not isinstance(receipt.get(field), str) or not SHA256.fullmatch(receipt[field]):
            raise ValueError(f"invalid {field}")

    archive_hash = sha256(archive)
    if receipt["archive_sha256"] != archive_hash:
        raise ValueError("archive digest differs from retained provenance")
    expected_sum = f"{archive_hash}  {expected_name}\n"
    if sum_path.read_text(encoding="utf-8") != expected_sum:
        raise ValueError("checksum file does not bind this exact archive")
    prefix = f"ste-{version}-{target}"
    expected_members = {prefix + "/ste", prefix + "/README.md", prefix + "/LICENSE"}
    with tarfile.open(archive, mode="r:gz") as tar:
        members = tar.getmembers()
        if len(members) != MAX_MEMBERS or {member.name for member in members} != expected_members:
            raise ValueError("archive contains missing, extra, duplicate or unsafe paths")
        if any(not member.isfile() for member in members):
            raise ValueError("archive cannot contain symlinks, directories or device members")
        if sum(member.size for member in members) > MAX_UNPACKED_BYTES:
            raise ValueError("archive members exceed bounded decompressed size")
        binary = next(member for member in members if member.name == prefix + "/ste")
        if binary.mode != 0o755 or any(m.mode != 0o644 for m in members if m != binary):
            raise ValueError("archive has incorrect executable/file permissions")
        stream = tar.extractfile(binary)
        if stream is None:
            raise ValueError("archive binary member absent")
        binary_hash = hashlib.sha256()
        with stream:
            for part in iter(lambda: stream.read(1 << 20), b""):
                binary_hash.update(part)
        if binary_hash.hexdigest() != receipt["binary_sha256"]:
            raise ValueError("binary content differs from provenance receipt")
    return {
        "schema": "ste-release-readback/v1",
        "tag": expected_tag,
        "source_commit": expected_source,
        "source_tree": expected_tree,
        "archive_sha256": archive_hash,
        "target": target,
        "byte_integrity_verified": True,
        "signed_provenance_verified": False,
        "cross_builder_reproducibility_verified": False,
        "private_runtime_authenticated": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--expected-tag", required=True)
    parser.add_argument("--expected-source", required=True)
    parser.add_argument("--expected-tree", required=True)
    args = parser.parse_args()
    result = verify(args.archive, expected_tag=args.expected_tag,
                    expected_source=args.expected_source,
                    expected_tree=args.expected_tree)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, tarfile.TarError, json.JSONDecodeError) as exc:
        print(f"release readback rejected: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
