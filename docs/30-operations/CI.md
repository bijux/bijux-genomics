# CI

## What
CI enforces the minimal deterministic gate for the workspace.

## Purpose
Define the canonical CI gate contract and shared artifact invocation for the repository.

## Command
- `make ci`

## Current `make ci` Gates
- `fmt`
- `lint`
- `audit`
- `test`
- `coverage`

## CI Profiles
- Fast CI profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-fast`
- Slow CI profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-slow`
- Fast profile intent: static/policy/contract gates with deterministic runner settings.
- Slow profile intent: heavier coverage/docs/release-readiness checks.
- Test/coverage runner defaults are pinned in
  [configs/rust/nextest.toml](../../configs/rust/nextest.toml) and
  [configs/coverage/runner.toml](../../configs/coverage/runner.toml).

## Artifact Contract
- See [docs/30-operations/ISOLATION.md](ISOLATION.md).

## HPC Forward-compat
- With HPC enabled, `make ci` still enforces the same gate order and policy checks.
- Path roots and container storage may resolve to HPC profile locations, not local defaults.
- Use profile-aware commands and avoid hardcoded local paths in docs automation.

## Non-goals
- Documenting non-`make ci` target suites.

## Scope
Applies only to the files and workflows referenced in this document.

## Contracts
- Content here is normative where explicitly stated.

## Examples
- Local: `make ci`
- HPC profile enabled: `ARTIFACT_ROOT=artifacts make ci`
- Fast profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-fast`
- Slow profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-slow`

## Failure modes
- Running CI-related scripts outside the shared artifact contract fails by policy.
- HPC path assumptions in docs/scripts can cause false failures when profile roots differ.
