# STE-Lint agent entrypoint

This file is a navigation index, not a parallel source of rules or policy.

- Start with [README.md](README.md) for installation, the pinned Rust toolchain, verification commands, and the runtime-data boundary.
- Follow [skills/STE.SKILL.md](skills/STE.SKILL.md) for lint, repair, rewrite checks, and coverage reporting.
- Follow [skills/STE-GLOSSARY.SKILL.md](skills/STE-GLOSSARY.SKILL.md) when editing governed terminology or reusable profiles.
- Read [data/rules.json](data/rules.json) and [docs/rule-coverage.md](docs/rule-coverage.md) before changing a rule implementation or compliance claim.
- Read [docs/reproducibility.md](docs/reproducibility.md) before adjusting CI or verification.
- For benchmark work, follow [issue #64](https://github.com/laurajoyhutchins/STE-Lint/issues/64); for semantic evidence/provider work, follow [issue #68](https://github.com/laurajoyhutchins/STE-Lint/issues/68).

Keep PRs bounded. Verify the exact proposed head with the existing pinned Rust and authority-ingest checks. Never use the test lexicon, model evidence, or incomplete rule coverage to claim full ASD-STE100 compliance. Do not commit protected source material or the private runtime dictionary.
