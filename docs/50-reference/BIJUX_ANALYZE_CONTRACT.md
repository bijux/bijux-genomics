# Bijux Analyze Contract

## What
Defines analysis output expectations.

## Why
Ensures report compatibility across releases.

## Non-goals
- UI presentation rules.

## Contracts
- `report.json` schema must remain stable under
  [REPORT_CONTRACT.md](../30-operations/REPORT_CONTRACT.md).
- Analysis output reasoning must stay aligned with
  [EXPLAINABILITY.md](../30-operations/EXPLAINABILITY.md).

## Examples
- Report completeness checks ensure required fields.

## Failure modes
- Missing report fields cause policy failures.
