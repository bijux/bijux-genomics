# scripts/experimental

Purpose: quarantined non-supported scripts not called from make/CI.

Back to index: `scripts/README.md`.


Requires:
- `./bin/isolate`
- `cargo-nextest`

Exit codes:
- `0`: all runs passed
- `1`: at least one run failed
- `2`: usage error
