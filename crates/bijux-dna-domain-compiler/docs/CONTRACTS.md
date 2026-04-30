# bijux-dna-domain-compiler Contracts

This crate turns authored domain source into deterministic validation results and generated
config files. The contracts below are the review checklist for any change to compiler behavior.

## Input contracts

- Domain source lives under `domain/` by default and is selected with `CompileOptions::domain_dir`
  or `ValidateOptions::domain_dir`.
- Supported source domains are FASTQ, BAM, and VCF.
- Each domain must provide stage schemas, tool schemas, artifact vocabularies, metric
  vocabularies, and an index.
- Shared tool-domain ownership is read from `configs/domain/shared_tools.toml` relative to the
  workspace root.

## Output contracts

`compile_domain_configs` writes only declared generated files under `CompileOptions::configs_dir`:

- `ci/registry/tool_registry.toml`
- `ci/registry/tool_registry_experimental.toml`
- `ci/registry/tool_registry_vcf.toml`
- `ci/registry/domain_registry_release_bundle.json`
- `ci/registry/domain_defaults_snapshot.json`
- `ci/registry/domain_artifact_contract_snapshots.json`
- `ci/registry/domain_metric_catalogs.json`
- `ci/registry/domain_deprecations_snapshot.json`
- `ci/registry/domain_invariant_catalogs.json`
- `ci/registry/domain_evidence_contracts.json`
- `ci/stages/stages.toml`
- `ci/stages/stages_vcf.toml`
- `ci/tools/images.toml`
- `ci/tools/required_tools.toml`

Generated files must include the compiler header, source hash, schema version, owner, authority,
purpose, and stable ordering. Active generated configs must not contain unsupported placeholder
tokens.

`write_domain_registry_bundle` must write only the declared JSON bundle surfaces beneath
`configs/ci/registry/`. `load_domain_registry_bundle` and `query_domain_registry_bundle` must be
read-only over an existing release bundle.

## Scope contracts

- `pre_hpc_pre_vcf` is the default compiler scope.
- `pre_hpc_pre_vcf` must not emit VCF tools or VCF stages into the governed production registry.
- Planned alternatives may remain visible as `planned_out_of_scope`, but planned-only tools must
  stay out of runtime-governed registry entries.

## Failure contracts

Validation or compilation must fail for missing required source files, duplicate IDs, invalid
status values, unknown stage/tool references, incomplete supported provenance, invalid shared-tool
domain mappings, deprecated tool replacements that point outside the domain/tool inventory, stage
schemas that omit governed required fields, inconsistent tool fixture claims, and unsupported
generated output vocabulary.
