# Contract Authority

This document defines the authoritative contract source for each category.

## Stages
- Authoritative source:
  - `configs/stages.toml` (FASTQ/BAM)
  - `configs/stages_vcf.toml` (VCF)
- Generated from domain SSOT through the domain compiler.

## Tools
- Authoritative source:
  - `configs/tool_registry.toml` (production FASTQ/BAM)
  - `configs/tool_registry_vcf.toml` (VCF)
  - `configs/tool_registry_experimental.toml` (experimental-only tools)
- Generated from domain SSOT through the domain compiler.

## Params
- Authoritative source:
  - `configs/param_registry.toml`
  - `configs/param_registry_vcf.toml`
- Code must not hardcode param schema IDs outside domain/config generation code.

## Metrics
- Authoritative source:
  - stage rows in `configs/stages*.toml` (`metrics_schema` field)
  - tool rows in `configs/tool_registry*.toml` (`metrics_schema` field)

## Profiles
- Authoritative source:
  - pipeline profile constructors in `crates/bijux-dna-pipelines`
  - profile manifest produced by `PipelineProfile::profile_manifest`
- Profile hash authority:
  - hash is derived from canonicalized profile manifest only.

## Migration Policy
- Schema versions are explicit and pinned in generated config headers and manifest payloads.
- Any schema version bump must include:
  - migration note in changelog/docs
  - snapshot updates where schema is serialized
  - compatibility decision (breaking vs non-breaking) documented in PR.
