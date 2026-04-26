# Change Rules

These rules define safe changes for `bijux-dna-bench`.

## Compatible Changes

- Add a checked-in benchmark suite that validates against
  `bijux.bench.suite.v1`.
- Add an internal helper under `src/workflow/`, `src/repo/`, or `src/artifacts/`
  without changing public exports.
- Add a public operation when it is listed in `docs/COMMANDS.md`, exported
  through `src/public_api/stable_surface.rs`, and covered by tests.
- Add optional artifact fields when old fixtures still deserialize and stable
  ordering is preserved.
- Add stricter validation when valid existing suites and artifacts keep the same
  output shape.

## Breaking Changes

- Rename, remove, or change the meaning of a public operation.
- Change the stable serialized meaning of `summary.json`, `decision.json`,
  `decisions.json`, or `observations.jsonl`.
- Change `BenchmarkSuiteSpec`, `BenchmarkObservation`, `BenchmarkSummary`, or
  gate policy behavior in a way that invalidates governed fixtures.
- Add product execution, runner, network, or hidden global-state effects.
- Add normal dependencies on API, planner, runner, or domain execution crates.

## Required Updates

For public or artifact contract changes, update the relevant files together:

- `docs/PUBLIC_API.md`
- `docs/COMMANDS.md`
- `docs/BENCH_CONTRACT.md`
- `docs/BENCH_FORMAT.md`
- `docs/REPRODUCIBILITY.md`
- `docs/SUITE_DESIGN.md` for suite catalog changes
- `tests/contracts/` for public behavior and artifact shape
- `tests/determinism/` for ordering, snapshot, or comparison changes
- `tests/boundaries/architecture_tree.rs` for intentional layout changes

## Review Rule

When a change could affect summaries, decisions, suite validation, or serialized
artifact shape, treat it as breaking until the contract and determinism tests
prove otherwise.
