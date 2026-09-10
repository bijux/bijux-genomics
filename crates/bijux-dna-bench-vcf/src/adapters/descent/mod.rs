//! Owner: bijux-dna-bench-vcf
//! descent adapter report construction from explicit admitted inputs.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids;
use serde::Serialize;

use crate::inputs::VcfAdapterInputs;
use bijux_dna_bench_model::VcfStageCatalogRow;
use bijux_dna_bench_model::VcfStageMatrixRow;

mod admission;
use admission::{
    ensure_vcf_descent_family_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{build_descent_family_row, path_relative_to_repo, repo_relative_path};
mod fixtures;
use fixtures::materialize_governed_demography_inputs;
mod model;
use model::{
    RegistryToolContract, GOVERNED_COHORT_VCF_PATH, GOVERNED_DEMOGRAPHY_MAP_CONTENT,
    GOVERNED_DEMOGRAPHY_SEGMENTS_CONTENT, GOVERNED_DESCENT_ROWS, GOVERNED_IBDHAP_MAP_PATH,
    GOVERNED_IBDHAP_VCF_PATH,
};
pub use model::{
    VcfDescentFamilyAdapterArtifact, VcfDescentFamilyAdapterCommandStep,
    VcfDescentFamilyAdapterReport, VcfDescentFamilyAdapterRow,
    DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH, VCF_DESCENT_FAMILY_ADAPTER_SCHEMA_VERSION,
};
mod registry;
use registry::load_registry_tool_contract;
mod service;
pub use service::{collect_vcf_descent_family_adapter_rows, render_vcf_descent_family_adapter};

mod stages;
