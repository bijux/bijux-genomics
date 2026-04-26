# bijux-dna-environment-qa Contracts

This crate owns QA evidence contracts, not production execution contracts.

## Owned Contracts

- QA artifact fixture shape: `manifest.json` and `report.json` include `schema_version`.
- QA record shape: `ImageQaRecord` values written to JSONL and SQLite through
  `bijux-dna-analyze`.
- QA input inventory: expected input hashes per stage/platform/runner.
- QA pass lookup: required stage/tool/image/input tuples must have passing records.
- Docker image probe outcome: pass/fail summary with explicit failure reasons.
- FASTQ behavioral QA stages from the FASTQ domain execution-support roster.

## Artifact Locations

- JSONL: `artifacts/image-qa/<platform>/qa.jsonl`
- SQLite: `artifacts/image-qa/<platform>/qa.sqlite`
- Summary: `artifacts/image-qa/<platform>/qa.json`
- Per-run outputs: `artifacts/image-qa/runs/<stage>/<tool>-<uuid>/`
- Fixture contracts: `tests/fixtures/qa_artifacts/default/manifest.json` and `report.json`

## Dataset Rules

- In-repository fixtures must be small, synthetic, and deterministic.
- External datasets are operator-provided and must not be fetched by default tests.
- Dataset checksum/provenance changes require tests or docs that explain the changed invariant.

## Change Rules

- Changing QA artifact fields requires contract test updates.
- Changing QA output locations requires `README.md`, `EFFECTS.md`, and tests to change together.
- Adding a QA stage requires FASTQ-domain support, stage roster coverage, and pass validation.
- Adding a host command requires `COMMANDS.md` and boundary tests.

## Failure Patterns

- Missing image: Docker image probe returns `ImageNotFound`.
- Missing executable: image exists but executable probe fails.
- Probe drift: expected version text is absent or exit code is outside the allowed set.
- Missing QA evidence: validation helpers instruct operators to rerun image QA for the platform.
