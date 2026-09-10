//! Owner: bijux-dna-bench-vcf
//! phasing adapter report construction from explicit admitted inputs.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids;
use serde::Serialize;

use crate::fixtures::inputs::{load_governed_vcf_fixture_inputs, materialize_indexed_vcf_input};
use crate::fixtures::panel::{
    materialize_local_vcf_panel_fixture, with_local_vcf_panel_fixture_lock,
};
use crate::inputs::VcfAdapterInputs;
use bijux_dna_bench_model::VcfStageCatalogRow;
use bijux_dna_bench_model::VcfStageMatrixRow;

mod admission;
use admission::{
    ensure_vcf_phasing_family_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{
    artifact, build_phasing_family_row, path_relative_to_repo, repo_relative_path,
    schema_version_for_tool,
};
mod fixtures;
use fixtures::materialize_panel_inputs;
mod model;
use model::{
    MaterializedPanelInputs, RegistryToolContract, StageContract, GOVERNED_COHORT_VCF_PATH,
};
pub use model::{
    VcfPhasingFamilyAdapterArtifact, VcfPhasingFamilyAdapterCommandStep,
    VcfPhasingFamilyAdapterReport, VcfPhasingFamilyAdapterRow, DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    DEFAULT_VCF_EAGLE_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
    DEFAULT_VCF_SHAPEIT_ADAPTER_PATH, VCF_BEAGLE_ADAPTER_SCHEMA_VERSION,
    VCF_EAGLE_ADAPTER_SCHEMA_VERSION, VCF_SHAPEIT5_ADAPTER_SCHEMA_VERSION,
    VCF_SHAPEIT_ADAPTER_SCHEMA_VERSION,
};
mod registry;
use registry::load_registry_tool_contract;
mod service;

pub use service::{
    collect_vcf_phasing_family_adapter_rows_for_tool, render_vcf_phasing_family_adapter,
};

mod invocation;
