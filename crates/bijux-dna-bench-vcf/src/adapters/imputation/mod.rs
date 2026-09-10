//! Owner: bijux-dna-bench-vcf
//! imputation adapter report construction from explicit admitted inputs.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids;
use serde::Serialize;

use crate::fixtures::inputs::materialize_indexed_vcf_input;
use crate::inputs::VcfAdapterInputs;
use bijux_dna_bench_corpus::panel_metadata::{
    materialize_local_vcf_panel_fixture, with_local_vcf_panel_fixture_lock,
};
use bijux_dna_bench_model::VcfStageCatalogRow;
use bijux_dna_bench_model::VcfStageMatrixRow;

mod admission;
use admission::{
    ensure_vcf_imputation_family_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{artifact, build_imputation_family_row, path_relative_to_repo, repo_relative_path};
mod fixtures;
use fixtures::materialize_panel_inputs;
mod model;
use model::{
    MaterializedPanelInputs, RegistryToolContract, StageContract, GOVERNED_GLLIKE_VCF_PATH,
    GOVERNED_GTCOHORT_VCF_PATH, GOVERNED_IMPUTATION_STAGE_IDS, GOVERNED_IMPUTATION_TOOL_IDS,
    GOVERNED_REGION_LITERAL, VCF_IMPUTATION_REGISTRY_PATHS,
};
pub use model::{
    VcfImputationFamilyAdapterArtifact, VcfImputationFamilyAdapterCommandStep,
    VcfImputationFamilyAdapterReport, VcfImputationFamilyAdapterRow,
    DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH, VCF_IMPUTATION_FAMILY_ADAPTER_SCHEMA_VERSION,
};
mod registry;
use registry::load_registry_tool_contract;
mod service;
pub use service::{
    collect_vcf_imputation_family_adapter_rows, render_vcf_imputation_family_adapter,
};

mod invocation;

mod outputs;
