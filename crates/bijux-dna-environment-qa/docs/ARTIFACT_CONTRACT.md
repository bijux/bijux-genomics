# ARTIFACT_CONTRACT

QA emits:
- manifest.json
- report.json

## Fixtures discipline
Fixtures must be minimal and named like production artifacts (`manifest.json`, `report.json`).
See `tests/fixtures/qa_artifacts/` and `tests/artifacts/qa_artifact_contract.rs`.

## Compatibility
The QA report/manifest schema must remain compatible with runtime/analyze expectations.
See:
- `crates/bijux-dna-runtime/docs/ARTIFACTS.md`
- `crates/bijux-dna-analyze` report loaders
