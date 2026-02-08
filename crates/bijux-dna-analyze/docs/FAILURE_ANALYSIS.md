# FAILURE_ANALYSIS

## Start here
Use `docs/FAILURE_TAXONOMY.md` for the authoritative failure classes and remediation hints.

## Common failures
- missing metrics: inspect `stage_report.json` and metrics paths in the run manifest.
- missing artifacts: inspect `run_manifest.json` and artifact paths.
- parse error: inspect tool output fixture and parser contract tests.
