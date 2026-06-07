# Commands

`bijux-dna-planner-vcf` is a planner crate. It does not expose CLI commands, subcommands, or runtime entrypoints. It manages planned VCF stage command specifications that downstream runtime crates may execute later.

## Runtime Commands
None.

## Planned Stage Command Specs
The authoritative VCF stage catalog is `bijux_dna_domain_vcf::taxonomy::VcfDomainStage::all()`. This crate can plan command specs for these stage IDs when the requested coverage regime and ordering allow them:

- `vcf.admixture`
- `vcf.call`
- `vcf.call_diploid`
- `vcf.call_gl`
- `vcf.call_pseudohaploid`
- `vcf.damage_filter`
- `vcf.demography`
- `vcf.filter`
- `vcf.gl_propagation`
- `vcf.ibd`
- `vcf.imputation_metrics`
- `vcf.impute`
- `vcf.pca`
- `vcf.phasing`
- `vcf.population_structure`
- `vcf.postprocess`
- `vcf.prepare_reference_panel`
- `vcf.qc`
- `vcf.roh`
- `vcf.stats`

## Default Stage Sets
Default stage sets are selected by `src/stage_sequence.rs` after reference and coverage resolution.

- `low_cov_gl`: `vcf.prepare_reference_panel`, `vcf.call_gl`, `vcf.damage_filter`, `vcf.filter`, `vcf.gl_propagation`, `vcf.impute`, `vcf.postprocess`, `vcf.population_structure`, `vcf.stats`
- `diploid`: `vcf.prepare_reference_panel`, `vcf.call_diploid`, `vcf.damage_filter`, `vcf.filter`, `vcf.phasing`, `vcf.impute`, `vcf.postprocess`, `vcf.population_structure`, `vcf.roh`, `vcf.ibd`, `vcf.demography`, `vcf.stats`
- `pseudohaploid`: `vcf.call_pseudohaploid`, `vcf.damage_filter`, `vcf.filter`, `vcf.roh`, `vcf.stats`

## Registry Authorities
This crate reads repository-owned registry configuration for validation only:

- `configs/ci/stages/stages_vcf.toml`
- `configs/ci/stages/stages_vcf_downstream.toml`
- `configs/ci/tools/required_tools_vcf.toml`
- `configs/ci/tools/required_tools_vcf_downstream.toml`
- `configs/ci/registry/tool_registry_vcf.toml`
- `configs/ci/registry/tool_registry_vcf_downstream.toml`
- `configs/ci/params/param_registry_downstream.toml`

## Ownership Rules
- Add, rename, or remove VCF stage IDs in `bijux-dna-domain-vcf` first.
- Keep this file aligned with the domain stage catalog and `src/stage_sequence.rs`.
- Do not add `src/bin`, CLI parsing, process spawning, or runtime execution here.
