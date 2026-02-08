# SCHEMA

## Authority
The report schema is defined in `docs/REPORT_CONTRACT.md`. Use that document for field semantics
and artifact expectations.

## Schema fixtures
Canonical fixtures used by schema/contract tests:
- `tests/fixtures/report/happy/default/report.json`
- `tests/fixtures/report/missing/default/report.json`
- `tests/fixtures/report/failure/default/report.json`
- `tests/fixtures/report/provenance/default/report.json`
- `tests/fixtures/report/perf_budget/default/report.json`
- `tests/fixtures/report/sections/default/report.json`

## Schema tests
- `tests/report/report_contract.rs`
- `tests/report/report_determinism.rs`
- `tests/contracts/contract_handshake.rs`

## Related
- `docs/DATA_MODEL.md` for the higher-level data model view.
- `docs/DECISIONS.md` for decision inputs that flow into report sections.
