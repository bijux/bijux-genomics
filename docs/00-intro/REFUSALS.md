# Refusals

## What
Explicit statements of what Bijux DNA does not support.

## Why
Refusals prevent accidental scope creep and keep policies enforceable.

## Non-goals
- Supporting ad‑hoc shell execution in stages.
- Accepting untyped identifiers in contracts.

## Contracts
- Effect boundary policy.

## Examples
- Stages never build command lines.

## Failure modes
- Attempts to bypass contracts are rejected by policy tests.
