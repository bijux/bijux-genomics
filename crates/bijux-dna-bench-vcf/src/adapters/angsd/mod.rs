//! Owner: bijux-dna-bench-vcf
//! angsd adapter report construction from explicit admitted inputs.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{vcf_domain_stage_adapter_id, vcf_domain_stage_parser_id};
use serde::Serialize;

use crate::inputs::VcfAdapterInputs;
use bijux_dna_bench_model::VcfStageCatalogRow;

mod admission;
use admission::{
    ensure_vcf_angsd_adapter_contract, run_missing_input_probe, validate_command_steps,
    validate_required_inputs,
};
mod commands;
use commands::{artifact, build_angsd_row, path_relative_to_repo, repo_relative_path};
mod fixtures;
use fixtures::{governed_inputs_for_stage, materialize_bam_list};
mod model;
pub use model::{
    VcfAngsdAdapterArtifact, VcfAngsdAdapterCommandStep, VcfAngsdAdapterReport, VcfAngsdAdapterRow,
    DEFAULT_VCF_ANGSD_ADAPTER_PATH, VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
};
use model::{
    GOVERNED_ANGSD_TOOL_ID, GOVERNED_ANGSD_TOOL_STATUS, GOVERNED_BAM_PATH, GOVERNED_CORPUS_ID,
    GOVERNED_REFERENCE_FASTA_PATH, GOVERNED_SITES_VCF_PATH,
};
mod registry;
use registry::load_admitted_angsd_stage_ids;
mod service;
pub use service::{collect_vcf_angsd_adapter_rows, render_vcf_angsd_adapter};

mod stages;
