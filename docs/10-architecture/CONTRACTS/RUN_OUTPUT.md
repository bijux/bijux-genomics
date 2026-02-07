# Run Output Contract

## What
Defines the expected output directory layout for runs.

## Why
Provides stable paths for analysis and benchmarking.

## Non-goals
- Custom directory layouts per user.

## Contracts
- Layout is derived from RunLayout.

## Examples
- `run_artifacts/` lives under the run output directory.

## Failure modes
- Layout drift breaks replay and audits.
