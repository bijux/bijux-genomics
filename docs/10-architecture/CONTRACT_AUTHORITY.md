# Contract Authority

This document defines the authoritative contract source for each category.

## What
Defines the single source of truth for stage/tool/param/profile contract artifacts.

## Why
Prevents contract drift between generated registries, pipeline profiles, and policy checks.

## Purpose
Declare authoritative contract sources per category and prevent conflicting ownership.

## Scope
Applies to stage/tool/param/profile/metrics authority and migration policy.

## Non-goals
- Repeating full schema definitions.
- Replacing domain-level scientific docs.

## Contracts
- Stage/tool/param/profile authority must resolve to exactly one canonical source.
- Generated artifacts and snapshots must track authority changes.

## Stages
- Authoritative source:
  - [../../configs/ci/stages/stages.toml](../../configs/ci/stages/stages.toml) (FASTQ/BAM)
  - [../../configs/ci/stages/stages_vcf.toml](../../configs/ci/stages/stages_vcf.toml) (VCF)
- Generated from domain SSOT through the domain compiler.

## Tools
- Authoritative source:
  - [../../configs/ci/registry/tool_registry.toml](../../configs/ci/registry/tool_registry.toml) (production FASTQ/BAM)
  - [../../configs/ci/registry/tool_registry_vcf.toml](../../configs/ci/registry/tool_registry_vcf.toml) (VCF)
  - [../../configs/ci/registry/tool_registry_experimental.toml](../../configs/ci/registry/tool_registry_experimental.toml) (experimental-only tools)
- Generated from domain SSOT through the domain compiler.

## Params
- Authoritative source:
  - [../../configs/ci/params/param_registry.toml](../../configs/ci/params/param_registry.toml)
  - [../../configs/ci/params/param_registry_vcf.toml](../../configs/ci/params/param_registry_vcf.toml)
- Code must not hardcode param schema IDs outside domain/config generation code.

## Metrics
- Authoritative source:
  - stage rows in `configs/stages*.toml` (`metrics_schema` field)
  - tool rows in `configs/tool_registry*.toml` (`metrics_schema` field)

## Profiles
- Authoritative source:
  - pipeline profile constructors in `bijux-dna-pipelines`
  - profile manifest produced by [../../crates/bijux-dna-pipelines/src/contract/profile_manifest.rs](../../crates/bijux-dna-pipelines/src/contract/profile_manifest.rs)
- Profile hash authority:
  - hash is derived from canonicalized profile manifest only.

## Migration Policy
- Schema versions are explicit and pinned in generated config headers and manifest payloads.
- Any schema version bump must include:
  - migration note in changelog/docs
  - snapshot updates where schema is serialized
  - compatibility decision (breaking vs non-breaking) documented in PR.

## Examples
- Stage authority: `configs/ci/stages/stages.toml`.
- Tool authority: `configs/ci/registry/tool_registry.toml`.

## Failure modes
- Registry/profile snapshots diverge when authority source changes without regeneration.
- Conflicting parallel authorities create non-deterministic policy results.
