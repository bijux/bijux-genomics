# Commands

This file is the SSOT for callable operations managed by
`bijux-dna-stages-vcf`. The crate owns Rust operations, not CLI commands.

## CLI Commands

None. This crate does not own binaries, subcommands, CLI parsing, or user-facing
command routing.

## Registry Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `list-vcf-implemented-stages` | `implemented_stages` | Return the VCF stages implemented by this crate. |
| `list-vcf-stage-catalog` | `stage_specs::vcf_stage_catalog` | Return the stage metadata catalog with stage status, runtime default tool, metrics schema, and smoke/parser flags. |
| `list-vcf-supported-stages` | `stage_specs::supported_vcf_stages` | Return stages that are currently marked supported and smoke/parser ready. |
| `check-vcf-stage-completeness` | `stage_specs::vcf_stage_completeness` | Check whether a baseline VCF stage has smoke and parser support. |

## Pipeline Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `run-vcf-pipeline` | `engine::run_vcf_pipeline` | Dispatch a typed VCF pipeline request and emit stage artifacts. |
| `run-vcf-preflight` | `invariants::run_vcf_preflight` | Validate VCF invariants and emit normalized preflight artifacts. |
| `run-vcf-chunked-regions` | `pipeline::run_chunked_regions` | Build deterministic region chunks and merged outputs. |

## Stage Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `run-vcf-call-gl-stage` | `pipeline::run_call_gl_stage` / `run_call_gl_from_bam_stage` | Produce genotype-likelihood VCF call artifacts. |
| `run-vcf-call-diploid-stage` | `pipeline::run_call_diploid_stage` | Produce diploid call artifacts. |
| `run-vcf-call-pseudohaploid-stage` | `pipeline::run_call_pseudohaploid_stage` | Produce pseudohaploid call artifacts. |
| `run-vcf-filter-stage` | `pipeline::run_filter_stage` / `run_filter_stage_real` | Filter VCF records and emit breakdown metrics. |
| `run-vcf-damage-filter-stage` | `pipeline::run_damage_filter_stage` | Filter damage-sensitive ancient DNA genotypes and emit damage artifacts. |
| `run-vcf-gl-propagation-stage` | `pipeline::run_gl_propagation_stage` | Normalize and propagate genotype likelihoods. |
| `run-vcf-qc-stage` | `pipeline::run_qc_stage` | Emit VCF QC tables, JSON metrics, warnings, and readiness markers. |
| `run-vcf-stats-stage` | `pipeline::run_stats_stage` / `run_stats_stage_real` | Emit VCF stats artifacts and metrics. |
| `run-vcf-phasing-stage` | `pipeline::run_phasing_stage` | Produce phased VCF artifacts using typed phasing parameters. |
| `run-vcf-impute-stage` | `pipeline::run_impute_stage` | Produce imputed VCF artifacts using typed imputation parameters. |
| `run-vcf-imputation-orchestration-stage` | `pipeline::run_imputation_orchestration_stage` | Run the wrapper-level imputation orchestration artifact contract. |
| `run-vcf-postprocess-stage` | `pipeline::run_postprocess_stage` | Merge, normalize, validate, and summarize imputed VCF output. |
| `run-vcf-pca-stage` | `pipeline::run_pca_stage` | Emit PCA eigen artifacts and preprocessing contracts. |
| `run-vcf-population-structure-stage` | `pipeline::run_population_structure_stage` | Emit population structure outputs and metrics. |
| `run-vcf-admixture-stage` | `pipeline::run_admixture_stage` | Emit admixture Q-matrix and model selection artifacts. |
| `run-vcf-roh-stage` | `pipeline::run_roh_stage` | Emit runs-of-homozygosity artifacts and metrics. |
| `run-vcf-ibd-stage` | `pipeline::run_ibd_stage` | Emit identity-by-descent segments and metrics. |
| `run-vcf-demography-stage` | `pipeline::run_demography_stage` | Emit demography trajectory artifacts from IBD inputs. |
| `run-vcf-prepare-reference-panel-stage` | `pipeline::run_prepare_reference_panel_stage` | Prepare reference-panel artifacts and overlap metrics. |

## VCF IO Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `read-vcf-text` | `vcf_io::read_vcf_text` | Read plain or best-effort compressed VCF text. |
| `validate-vcf-input` | `vcf_io::vcf_validate_input` | Validate VCF records against field requirements. |
| `normalize-vcf-headers` | `vcf_io::vcf_normalize_headers` | Write normalized VCF header/sample ordering. |
| `index-vcf-bgzip-tabix` | `vcf_io::vcf_index_bgzip_tabix` | Write bgzip VCF and tabix index artifacts. |
| `split-vcf-by-chrom` | `vcf_io::vcf_split_by_chrom` | Split indexed VCF input into chromosome files. |
| `concat-vcf` | `vcf_io::vcf_concat` | Concatenate VCF shards into an indexed VCF. |
| `extract-vcf-region` | `vcf_io::vcf_region_extract` | Extract a genomic region into an indexed VCF. |
| `compute-vcf-stats-basic` | `vcf_io::vcf_stats_basic` | Emit basic stats output and typed metrics. |
| `compute-vcf-checksum-set` | `vcf_io::vcf_checksum_set` | Hash VCF artifact sets. |
| `check-vcf-reference-match` | `vcf_io::vcf_ref_match_check` | Validate VCF reference build compatibility. |
| `compute-vcf-panel-overlap` | `vcf_io::vcf_panel_overlap` | Compute overlap between input and panel VCFs. |

## Metrics And Wrapper Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `parse-vcf-call-summary` | `metrics::parse_vcf_call_summary` | Parse call-summary metrics from VCF records. |
| `parse-vcf-filter-breakdown` | `metrics::parse_vcf_filter_breakdown` | Parse filter breakdown metrics from VCF records. |
| `parse-vcf-stats` | `metrics::parse_vcf_stats` | Parse VCF stats text into typed metrics. |
| `summarize-vcf-metrics` | `metrics::summarize_vcf_metrics` | Convert typed VCF stats metrics into summary JSON. |
| `verify-vcf-tool-wrapper` | `wrappers::verify_tool_wrapper` | Validate local tool wrapper version/help contracts. |

## Commands Owned Elsewhere

- User-facing CLI commands belong in command/API crates.
- Plan construction and cross-domain profile selection belong in planner and
  pipeline crates.
- Runtime queueing and worker supervision belong in runtime and runner crates.
- Environment and container commands belong in environment crates.
