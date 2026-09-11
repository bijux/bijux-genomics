//! Owner: bijux-dna-bench-vcf
//! eigensoft adapter report construction from explicit admitted inputs.
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
    ensure_vcf_eigensoft_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{artifact, build_eigensoft_row, path_relative_to_repo, repo_relative_path};
mod model;
use model::{RegistryToolContract, StageContract, GOVERNED_COHORT_VCF_PATH};
pub use model::{
    VcfEigensoftAdapterArtifact, VcfEigensoftAdapterCommandStep, VcfEigensoftAdapterReport,
    VcfEigensoftAdapterRow, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
    VCF_EIGENSOFT_ADAPTER_SCHEMA_VERSION,
};
mod registry;
use registry::load_registry_tool_contract;
pub(crate) mod service;

pub use service::{collect_vcf_eigensoft_adapter_rows_for_tool, render_vcf_eigensoft_adapter};

mod invocation;
