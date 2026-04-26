# Failure Handling

## Authority
Failure classes and remediation hints are implemented in `src/failure/`. The public contract is the
structured `BenchmarkFailure` output and its serialized snapshots.

## Failure Classes
- `ContractError`: schema validation failures, invariant violations, and incompatible artifact
  contracts.
- `ToolError`: failed tool execution as recorded by produced artifacts, failed observers, or tool
  output parse errors.
- `EnvironmentError`: infrastructure failures such as image problems, timeouts, missing resources,
  or resource exhaustion.

## Failure Kinds
- `tool_exit`: tool execution failed before producing a valid artifact set.
- `contract_violation`: a schema, report, or invariant contract was not satisfied.
- `observer_parse`: recorded tool output could not be parsed into a typed report.
- `data_invalid`: input data failed validation.
- `resource_exhaustion`: memory, time, or disk limits were exceeded.
- `image_error`: a runtime image was missing or invalid.

## Remediation Hints
Hints are structured records with severity and suggested action. Keep wording stable enough for
snapshot review and precise enough for operators to act on without reading source code.

## Common Diagnosis Paths
- Missing metrics: inspect the stage `metrics_path`, `stage_report.json`, and metric provenance.
- Missing artifacts: inspect `run_manifest.json`, `facts.jsonl`, and report artifact paths.
- Parse errors: inspect the tool output fixture and the parser contract that owns that artifact.
- Incomplete reports: inspect `ReportCompletenessV1` and the report section required by
  `docs/REPORT_CONTRACT.md`.

## Coverage
- `tests/contracts/core/failure_hints.rs`
- `tests/snapshots/bijux-dna-analyze__schemas__failure_hint_adapter.json`
- `tests/snapshots/bijux-dna-analyze__schemas__failure_hint_timeout.json`
- `tests/snapshots/bijux-dna-analyze__schemas__failure_hint_invalid.json`
