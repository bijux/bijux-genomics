# Contract Spine

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-engine, bijux-runtime, bijux-runner, bijux-api

## What
Defines the contract artifacts that bind planning, execution, and reporting.

## Why
Stable contracts allow deterministic caching, replay, and audits.

## Non-goals
- Informal, ad‑hoc output formats.

## Contracts
- ExecutionGraph
- RunManifest
- ToolInvocation
- MetricsEnvelope

## Examples
- Planner emits ExecutionGraph → Engine executes → Runtime writes RunManifest.

## Failure modes
- Missing contract fields causes validation error.
