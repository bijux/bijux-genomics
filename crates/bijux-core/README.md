# bijux-core

## What this crate owns (SSOT)
- Contract types, IDs, and canonical serialization.

## What this crate must never do (purity)
- IO, process execution, network access.

## What depends on this crate
- All crates: engine, runtime, planners, stages, api, analyze, benchmark.

## What this crate depends on
- Minimal utilities only.
