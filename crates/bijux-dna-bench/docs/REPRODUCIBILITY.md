# Reproducibility

Benchmark results must be reproducible from the suite spec, observations,
policy, and options used for a run.

## Reproduce A Benchmark Summary

1. Load the suite TOML from `crates/bijux-dna-bench/bench/suites/`.
2. Collect the governed `BenchmarkObservation` records for the run.
3. Use the same `BenchRunOptions`, including output directory and resume setting.
4. Call `summarize`.
5. Compare the emitted `BenchmarkSummary` with `summary.json` or test fixtures.

## Reproduce Gate Decisions

1. Recreate the `GatePolicy`.
2. Call `gate(policy, summary)` with the reproduced summary.
3. Compare `decision.json` and `decisions.json` against fixtures or persisted
   artifacts.

## Reproduce Comparisons

1. Load two completed `BenchmarkSummary` values.
2. Call `compare(summary_a, summary_b)`.
3. Compare the deterministic report against the stored compare snapshot.

## Reproduce Attempt State

1. Locate the permanent run, tool, campaign, and attempt directory.
2. Validate `attempt.json` as the immutable planned record.
3. Apply transition files in numeric sequence order.
4. Validate the reconstructed `BenchmarkAttemptRecordV2`; do not infer state
   from directory names or scheduler output alone.

## Reproduce Stage Coverage

1. Load the typed container catalog, campaign registry, and admitted reports.
2. Select the required repository commit for the publication cycle.
3. Call `derive_stage_coverage`; reports from any other commit remain `stale`.
4. Use `publish_checked_in_stage_coverage` in check mode to verify JSON,
   Markdown, and generated JSON Schema byte identity.

## Reproduce FASTQ Expectations

1. Select the checksummed STUDY data index and exact corpus identity.
2. Call `ensure_fastq_read_length_expectations` with the governed sample/file closure.
3. Re-admission verifies every catalog, file size, and compressed-file digest.
4. Science derivation streams the FASTQ records and reproduces the exact summary,
   histogram, row digest, and manifest bytes; any existing conflict is rejected.
5. Call `prepare_fastq_profile_read_lengths_material` for each planned tool identity;
   identical attempts reproduce parameter, mount, command, and required-file bytes.

## Allowed Variation

- Output directory paths may differ.
- Existing observations may be merged only when `BenchRunOptions.resume` is true.
- Test-local temporary directories may differ.

## Forbidden Variation

- Suite validation result.
- Observation grouping.
- Summary row ordering.
- Gate decision ordering.
- Numeric score, delta, outlier, and bootstrap results for the same inputs.
- Chosen pass/fail decision and rationale for the same policy.

Determinism is enforced by `tests/determinism.rs` and tests under
`tests/determinism/`.
