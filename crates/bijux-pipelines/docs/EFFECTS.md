# Effects

## What
Defines what effects `bijux-pipelines` is allowed to use.

## Why
Preserves architectural purity and reproducibility.

## Non-goals
- Granting permissions not required by the crate.

## Contracts
- Effects allowed/forbidden are enforced by policy tests.

## Examples
- Runner crates may spawn processes; most others may not.

## Failure modes
- Using forbidden effects fails policy checks.
