# Reproducible verification

STE-Lint pins the Rust verification toolchain to Rust 1.97.1 in `rust-toolchain.toml` and commits the workspace `Cargo.lock`.

The lockfile was generated with Cargo 1.97.1 from the repository dependency manifests. CI runs Rust formatting, Clippy, and tests with the pinned toolchain; dependency-consuming checks use `--locked` so a manifest change that requires lockfile regeneration fails instead of silently changing the resolved dependency set.

Push CI applies to all branches, not only branches with a particular naming convention. Pull requests run the same CI workflow.

## Residual environment limits

Exact repository inputs do not fully pin the hosted CI machine. GitHub's `ubuntu-24.04` runner image is platform-managed and changes over time. The authority-ingest job requests Python 3.13, not a specific patch release. `actions/checkout@v4` and `actions/setup-python@v5` intentionally follow their maintained major-version action lines rather than exact action commit SHAs.

These limits describe the CI execution environment. They do not relax the repository's source, runtime-identity, standards-authority, or exact-head evidence requirements.
