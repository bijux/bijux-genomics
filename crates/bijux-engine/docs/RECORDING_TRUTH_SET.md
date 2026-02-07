# RECORDING_TRUTH_SET

## Required per-step artifacts
Every executed step must emit the following files:

- `effective_config.json`
- `tool_invocation.json`
- `execution_record.json`
- `metrics.json` (when metrics are required)
- `stage_report.json` (when a stage report is required)

## Rationale
These files form the minimal truth set needed for reproducibility and analysis.

## Coverage
The test `tests/recording_completeness.rs` asserts this set for each step.
