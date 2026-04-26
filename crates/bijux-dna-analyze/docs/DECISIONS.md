# Decisions

## Authority
Decision behavior is implemented in `src/decision/`. Public exports are curated through
`src/public_api/decision.rs`.

## Compare Mode
`AnalyzeMode::Compare` compares two completed run directories and writes `compare.json`.

- Input: two run directories with produced report/facts artifacts.
- Objective: currently resolves to the balanced objective from core contracts.
- Ordering: comparison output must be deterministic for identical inputs.
- Boundary: compare logic must not load through report renderers or mutate run artifacts.

## Ranking Modes
`src/decision/score/` owns three stable ranking modes:

- `FastestAcceptable`: lower `runtime_s` wins; ties sort by `tool`.
- `MostConservative`: higher combined retention and error-reduction proxy wins; ties sort by
  `tool`.
- `BalancedPareto`: combines inverted runtime, retention, and inverted memory with stable weights;
  ties sort by `tool`.

## Metric Semantics
Rankings require semantics for:

- `runtime_s`
- `memory_mb`
- `read_retention`
- `base_retention`
- `error_reduction_proxy`
- `merge_rate`

Missing semantics are errors because silent defaulting would make reports misleading.

## Explainability Contract
Each ranking entry must carry:

- `score`
- `explain`
- `score_breakdown`
- `penalties`
- `why_not_first`
- `decision_trace`

The trace must make the input metrics, ranking mode, contribution weights, missing metrics, and
tie-breaking rationale inspectable.

## Coverage
- `tests/semantics/decision/compare_ranking.rs`
- `tests/semantics/decision/compare_stats.rs`
- `tests/semantics/decision/decision_trace.rs`
- `tests/semantics/decision/ranking_ties.rs`
- `tests/semantics/decision/selection_fastq.rs`
