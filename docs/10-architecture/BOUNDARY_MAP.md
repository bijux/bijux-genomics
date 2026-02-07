# Boundary Map

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-engine, bijux-runtime, bijux-runner, bijux-api

## What
Defines which effects (IO, process, network) are allowed in each crate.

## Why
Enforces architectural boundaries and prevents accidental coupling.

## Non-goals
- Granting broad execution privileges to non‑runner crates.

## Contracts
- Effect boundary policy tests.

## Examples
| Crate | Allowed effects | Forbidden effects |
| --- | --- | --- |
| bijux-core | None | IO, process, network |
| bijux-engine | Filesystem writes under run layout | Process spawn, docker APIs |
| bijux-runner | Process spawn, docker | Planner logic |

## Failure modes
- Any non‑allowlisted process spawn fails CI policy.
