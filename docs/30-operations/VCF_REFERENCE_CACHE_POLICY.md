# VCF Reference Cache Policy

## Purpose
Defines immutable acquisition and reuse policy for VCF panel/map reference assets.

## Scope
Applies to panel/map acquisition, cache layout, and runtime reuse for VCF downstream workflows.

## Non-goals
- Defining panel scientific suitability criteria or backend selection policy.

## Contracts
- Runtime and planner stages must not fetch panel/map assets from network.
- Asset materialization and lock checksums must match enabled panel/map configuration.

VCF panel/map assets are acquired once according to
[configs/vcf/panels/panels.toml](../../configs/vcf/panels/panels.toml), locked through
[configs/vcf/panels/locks/panel_locks.toml](../../configs/vcf/panels/locks/panel_locks.toml), and reused by
pipelines.

Rules:
- Network fetch for panel/map artifacts is allowed only in:
  - `cargo run -q -p bijux-dna-dev -- tooling run acquire-panels`
  - `cargo run -q -p bijux-dna-dev -- tooling run acquire-maps`
- Runtime/planner stages must not redownload references.
- Cache layout must be:
  - `raw/` immutable downloads
  - `normalized/` indexed/tool-ready outputs
  - `derived/` chunk indices and converted artifacts
- Catalog files with a derivation contract are never treated as downloads. Admit existing
  checksum-verified bytes with `acquire-panels --admission-root <dir>`; the Rust reference
  materialization policy places them under `derived/` and records the admission source and
  observed digest.
- Enabled panel/map in
  [configs/runtime/profiles/vcf_downstream_local.toml](../../configs/runtime/profiles/vcf_downstream_local.toml)
  must:
  - exist in catalog + lock entries,
  - be materialized under cache root,
  - match lock checksums.
