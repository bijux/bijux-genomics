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
- Container versions, source identities, runtime support, executable names, and recipe paths are
  read exclusively from `containers/contracts/catalog.toml`.

FASTQ/BAM active defaults and supported stage matrices require finite positive `memory_gb`
resource hints and finite nonnegative output size estimates in MB. Fractional and arbitrarily
small positive memory values remain valid. Output estimates may be zero, including negative zero.
`time_minutes` and `threads` retain their positive integer requirements.

## Output contracts

`compile_domain_configs` writes only declared generated files under `CompileOptions::configs_dir`:

- `ci/registry/tool_registry.toml`
- `ci/registry/tool_registry_experimental.toml`
- `ci/registry/tool_registry_container_experimental.toml`
- `ci/registry/tool_registry_vcf.toml`
- `ci/registry/tool_registry_vcf_downstream.toml`
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
- `ci/tools/required_tools_vcf_downstream.toml`

Generated files must include the compiler header, source hash, schema version, owner, authority,
purpose, and stable ordering. Active generated configs must not contain unsupported placeholder
tokens.

The container-experimental view contains retained catalog tools without an authored domain binding.
The VCF-downstream view is a compatibility projection of authored VCF tools excluding the common
`bcftools` runtime. Neither view may introduce packaging facts independently of the catalog.

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

A domain tool may claim `supported` only when the catalog declares `product_support = "supported"`
and its delivery contract agrees with the runtime declaration. `redistributable_image` delivery
requires at least one catalog-authorized runtime and `external_artifact_supported = false`.
`external_artifact`, `private_site_build`, and `site_module` delivery require
`external_artifact_supported = true` and no redistributable runtime. Domain metadata cannot promote
an unsupported packaging contract.

Artifact vocabulary admission requires a supported schema (`bijux.domain.artifacts.v1` or `bijux.artifacts.v1`), the matching domain, and exactly one nonempty declaration: legacy `artifact_ids` or typed `artifacts`. Typed entries require `id` and `type`; unknown fields, null declarations, duplicate IDs, and blank or padded IDs/types reject. Legacy IDs retain an absent type in registry snapshots. Admission verifies declaration shape, not scientific compatibility or completeness of the artifact type authority.
