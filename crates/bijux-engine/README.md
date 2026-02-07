# bijux-engine

## What this crate owns (SSOT)
- Orchestration logic (no execution backend).

## What this crate must never do (purity)
- Spawn processes or invoke docker/local executors.

## What depends on this crate
- API, CLI, tests.

## What this crate depends on
- bijux-core, bijux-runtime (Runner trait).
