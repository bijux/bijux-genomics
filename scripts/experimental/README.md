# scripts/experimental

Purpose: quarantined non-supported scripts not called from make/CI.

Back to index: `scripts/README.md`.


Requires:
- shared `artifacts/` environment via `scripts/_lib/common.sh`
- `cargo-nextest`

Exit codes:
- `0`: all runs passed
- `1`: at least one run failed
- `2`: usage error
