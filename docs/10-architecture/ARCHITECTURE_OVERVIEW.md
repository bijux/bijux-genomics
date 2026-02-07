# Architecture Overview

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-engine, bijux-runtime, bijux-runner, bijux-api

## What
Bijux DNA is a contract‑first pipeline system. Contracts are defined in `bijux-core` and consumed across planners, engine, runtime, and reporting.

## Why
A stable contract spine makes execution deterministic, auditable, and reproducible.

## Non-goals
- Tight coupling between engine and runner backends.
- Ad‑hoc artifact formats.

## Contracts
- ExecutionGraph
- RunManifest
- ToolInvocation

## Examples
```
core → engine → runtime → runner → api → stages → planners → pipelines → analyze/bench
```

## Failure modes
- Contract mismatches break validation and abort runs.
