# bijux-runner

## What this crate owns (SSOT)
- Execution backends (local, docker) implementing Runner.

## What this crate must never do (purity)
- Planner logic or domain semantics.

## What depends on this crate
- Engine via Runner trait (runtime).

## What this crate depends on
- bijux-runtime, bijux-core.
