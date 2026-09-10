//! Owner: bijux-dna-bench-vcf
//! bcftools adapter report construction from explicit admitted inputs.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids;
use serde::Serialize;

use crate::fixtures::inputs::materialize_reference_fasta_with_index;
use crate::inputs::VcfAdapterInputs;
use bijux_dna_bench_model::VcfStageCatalogRow;
use bijux_dna_bench_model::VcfStageMatrixRow;

mod admission;
use admission::{
    ensure_vcf_bcftools_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{artifact, build_bcftools_row, path_relative_to_repo, repo_relative_path};
mod fixtures;
use fixtures::governed_inputs_for_stage;
mod model;
pub use model::{
    VcfBcftoolsAdapterArtifact, VcfBcftoolsAdapterCommandStep, VcfBcftoolsAdapterReport,
    VcfBcftoolsAdapterRow, DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH, VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
};
use model::{
    GOVERNED_BAM_INDEX_PATH, GOVERNED_BAM_PATH, GOVERNED_BCFTOOLS_TOOL_ID,
    GOVERNED_FILTERED_SINGLE_SAMPLE_VCF_PATH, GOVERNED_MULTISAMPLE_VCF_PATH,
    GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH, GOVERNED_REFERENCE_FASTA_PATH,
    GOVERNED_REFERENCE_PANEL_VCF_PATH,
};
mod service;
pub use service::{collect_vcf_bcftools_adapter_rows, render_vcf_bcftools_adapter};

mod stages;
