#!/usr/bin/env python3
"""Run public STE CLI release checks without loading protected runtime data."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib


def invoke(binary: Path, *args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.pop("STE_LINT_LEXICON", None)
    return subprocess.run([str(binary), *args], text=True, capture_output=True,
                          env=env, check=False)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: python3 scripts/release_smoke.py PATH-TO-STE", file=sys.stderr)
        return 2
    binary = Path(sys.argv[1]).resolve()
    if not binary.is_file():
        print(f"release smoke: binary missing: {binary}", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parent.parent
    with (root / "Cargo.toml").open("rb") as stream:
        version = tomllib.load(stream)["workspace"]["package"]["version"]

    help_result = invoke(binary, "--help")
    if help_result.returncode or not {"coverage", "dictionary"}.issubset(
        help_result.stdout.split()
    ):
        raise RuntimeError("public CLI help or command inventory invalid")

    coverage_result = invoke(binary, "coverage", "--format", "json")
    if coverage_result.returncode:
        raise RuntimeError("public coverage requires runtime or failed")
    coverage = json.loads(coverage_result.stdout)
    if (coverage.get("total_rules") != 53
            or coverage.get("full_compliance_claimed") is not False
            or not isinstance(coverage.get("rules"), list)
            or len(coverage["rules"]) != 53):
        raise RuntimeError("public coverage contract or disclaimer invalid")

    version_result = invoke(binary, "--allow-test-lexicon", "version")
    lines = version_result.stdout.splitlines()
    if (version_result.returncode or not lines or lines[0] != f"ste {version}"
            or "runtime source: embedded test lexicon" not in version_result.stdout):
        raise RuntimeError("binary version or explicit test-runtime identity invalid")

    dictionary_result = invoke(
        binary, "--allow-test-lexicon", "dictionary", "lookup", "USE",
        "--format", "json"
    )
    if dictionary_result.returncode or not json.loads(dictionary_result.stdout):
        raise RuntimeError("explicit public test lexicon lookup failed")

    # With no user-supplied private data, a production dictionary request must fail.
    missing = invoke(binary, "dictionary", "lookup", "USE", "--format", "json")
    if missing.returncode != 3:
        raise RuntimeError("unconfigured private runtime did not fail with exit 3")
    error = json.loads(missing.stdout)
    if (error.get("schema") != "ste-cli-error/v1"
            or error.get("error", {}).get("kind") != "invalid_data"
            or error.get("error", {}).get("exit_code") != 3):
        raise RuntimeError("unconfigured runtime did not return typed invalid_data")

    print(f"Public release smoke passed: ste {version}, 53 rules, no private runtime")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, RuntimeError, json.JSONDecodeError) as exc:
        print(f"release smoke failure: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
