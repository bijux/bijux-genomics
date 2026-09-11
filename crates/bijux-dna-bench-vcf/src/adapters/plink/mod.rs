//! Owner: bijux-dna-bench-vcf
//! plink adapter report construction from explicit admitted inputs.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids;
use serde::Serialize;

use crate::fixtures::inputs::load_governed_vcf_fixture_inputs;
use crate::inputs::VcfAdapterInputs;
use bijux_dna_bench_model::VcfStageCatalogRow;
use bijux_dna_bench_model::VcfStageMatrixRow;

mod admission;
use admission::{
    ensure_vcf_plink_family_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{
    artifact, build_plink_family_row, path_relative_to_repo, repo_relative_path,
    schema_version_for_tool,
};
mod fixtures;
use fixtures::governed_inputs_for_stage;
mod model;
use model::{RegistryToolContract, GOVERNED_COHORT_VCF_PATH};
pub use model::{
    VcfPlinkFamilyAdapterArtifact, VcfPlinkFamilyAdapterCommandStep, VcfPlinkFamilyAdapterReport,
    VcfPlinkFamilyAdapterRow, DEFAULT_VCF_PLINK2_ADAPTER_PATH, DEFAULT_VCF_PLINK_ADAPTER_PATH,
    VCF_PLINK2_ADAPTER_SCHEMA_VERSION, VCF_PLINK_ADAPTER_SCHEMA_VERSION,
};
mod registry;
use registry::load_registry_tool_contract;
pub(crate) mod service;

pub use service::{
    collect_vcf_plink_family_adapter_rows_for_tool, render_vcf_plink_family_adapter,
};

mod stages;
