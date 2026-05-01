# Manifest Schema Migration Policy

## Scope
- `bijux.profile_manifest.v1`
- run manifest/lock artifacts written by runner/runtime/report layers
- core report/facts artifacts:
  - `bijux.run_manifest.v1`
  - `bijux.report.v1`
  - `bijux.facts.v1`
- certification artifacts:
  - `bijux.certification_bundle.v2`
  - `bijux.certification_run_stamp.v1`
  - `bijux.frontend.mini_domain_validation.v1`
  - `bijux.example.bundle.v1`
- workflow and plan compatibility upgrades are documented in [UPGRADE_GUIDE.md](UPGRADE_GUIDE.md)
- schema compatibility classes and migration rules are documented in [SCHEMA_REGISTRY.md](SCHEMA_REGISTRY.md)

## Version pinning
- Every manifest payload must include an explicit `schema_version`.
- Schema version changes are intentional and reviewed.

## Breaking vs non-breaking
- Breaking:
  - remove required fields
  - change field semantics
  - change hash inputs
- Non-breaking:
  - add optional fields
  - add new metadata sections that do not affect semantic meaning

## Snapshot impact
- If schema version changes, snapshot updates are expected.
- PR must include a migration note describing:
  - old version
  - new version
  - compatibility behavior

## Certification Schema Notes
- `bijux.run_manifest.v1`
  - Legacy run-manifest schema observed in fixture-backed certification checks.
- `bijux.report.v1`
  - Canonical report payload used for fixture/domain summary assertions.
- `bijux.facts.v1`
  - Facts jsonl row schema used by report pipeline and certification checks.
- `bijux.certification_bundle.v2`
  - Captures per-domain certification status, warnings/errors, and golden key-drift policy.
  - Breaking changes: remove required domain keys or status fields.
- `bijux.certification_run_stamp.v1`
  - Captures production vs non-production mode and relaxed-threshold state.
  - Breaking changes: rename mode fields or semantics.
- `bijux.frontend.mini_domain_validation.v1`
  - Captures VCF mini-stack local validation output.
  - Breaking changes: remove required `ok` or `errors` fields.
- `bijux.example.bundle.v1`
  - Captures local example bundle membership (`plan/explain/report/metrics/logs`).
  - Breaking changes: remove required file keys from bundle manifest.

## Hash contract
- Profile hash is derived from canonicalized profile manifest only.
- Non-semantic ordering changes must not alter hash.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.
- Reviewed release-by-release migration actions must be generated from `configs/ci/compatibility/release_changes.toml` into [UPGRADE_GUIDE.md](UPGRADE_GUIDE.md).
- Deterministic migration inputs and compatibility expectations must be covered by the fixture-backed contract tests in `bijux-dna-core` and `bijux-dna-runtime`.
