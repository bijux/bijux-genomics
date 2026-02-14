use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use bijux_dna_domain_vcf::params::{VcfCallParams, VcfFilterParams, VcfStatsParams};
use bijux_dna_domain_vcf::{VcfDomainStage, VCF_STAGE_ORDER_DOWNSTREAM};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json, hash_file_sha256};
use serde::Serialize;

use crate::invariants::{run_vcf_preflight, InputRegime, InvariantConfig, VcfPreflightResult};
use crate::pipeline::{
    run_admixture_stage, run_call_diploid_stage, run_call_gl_stage, run_call_pseudohaploid_stage,
    run_damage_filter_stage, run_demography_stage, run_filter_stage_real, run_gl_propagation_stage,
    run_ibd_stage, run_imputation_orchestration_stage, run_impute_stage, run_pca_stage,
    run_phasing_stage, run_population_structure_stage, run_postprocess_stage,
    run_prepare_reference_panel_stage, run_qc_stage, run_roh_stage, run_stats_stage_real,
    AdmixtureStageParams, DamageFilterStageParams, DemographyStageParams, GlPropagationStageParams,
    IbdStageParams, ImputeStageParams, PcaStageParams, PhasingStageParams,
    PopulationStructureStageParams, PostprocessStageParams, PrepareReferencePanelParams,
    QcStageParams, RohStageParams,
};

include!("engine_sections/types_and_dispatch.rs");
include!("engine_sections/stage_runner_impl.rs");
include!("engine_sections/pipeline_entrypoints.rs");
