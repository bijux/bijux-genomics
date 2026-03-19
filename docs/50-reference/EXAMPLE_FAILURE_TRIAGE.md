# Example Failure Triage

## Purpose
Provide a fast, repeatable workflow for debugging failed example runs.

## Scope
Applies to failures from `cargo run -q -p bijux-dev-dna -- examples run run` and related example policy checks.

## Non-goals
- Replacing crate-level debugging docs.
- Covering non-example integration test failures.

## Contracts
- Example runs must write artifacts under `artifacts/examples/<example-id>/`.
- Triage decisions should be based on generated `plan.json`, `explain.json`, `report.json`, `run_report.json`, `manifest.json`, and `logs.txt`.

## Common Failure Modes
- Missing corpus metadata (`MANIFEST.toml` / `CHECKSUMS.sha256`).
- Golden drift between expected and produced `plan.json` / `explain.json`.
- Snapshot gate mismatch for CLI command surface.
- Running outside the shared artifact contract causing policy-gated abort.

## Triage Steps
1. Re-run the example:
   - `cargo run -q -p bijux-dev-dna -- examples run run <example-id>`
2. Inspect generated bundle and logs:
   - `artifacts/examples/<example-id>/bundle.tar.gz`
   - `.../run_report.json`
   - `.../logs.txt`
3. Diff produced vs golden:
   - `diff -u examples/.../golden/plan.json .../plan.json`
   - `diff -u examples/.../golden/explain.json .../explain.json`
4. Validate corpus inputs:
   - `cargo run -q -p bijux-dev-dna -- checks run check-examples-corpus-manifests`
   - `cargo run -q -p bijux-dev-dna -- checks run check-examples-corpus-checksums`
5. Validate CLI snapshot if command surface changed:
   - `cargo run -q -p bijux-dev-dna -- checks run check-cli-command-snapshot`

## Examples
- `fastq_qc_pre_bench` fails with golden drift:
  check `run_report.json` then diff `plan.json` and `explain.json` before editing goldens.

## Failure modes
- Updating goldens without diagnosing root cause can hide regressions.
- Running outside the shared artifact contract can produce misleading paths/artifacts and invalidate triage output.
