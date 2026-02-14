use bijux_dna_core::ids::id_catalog;
use bijux_dna_core::prelude::params_hash;

use std::fs;
use std::path::Path;

use super::STAGE_QC_POST;
use anyhow::{Context, Result};
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep};
use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::ArtifactId;
use bijux_dna_planner_fastq::report_stage_step as build_report_stage_step;
use bijux_dna_planner_fastq::{CorrectDecisionTrace, MergeDecisionTrace};
use bijux_dna_runner::execute::StageResultV1;

pub(crate) struct StageExecutionSummary {
    pub plan: ExecutionStep,
    pub result: StageResultV1,
}

#[derive(Debug, Clone)]
#[allow(dead_code, clippy::struct_field_names)]
pub(crate) struct ReportArtifacts {
    pub run_summary_path: std::path::PathBuf,
    pub run_summary_html_path: std::path::PathBuf,
    pub summary_json_path: std::path::PathBuf,
    pub summary_tsv_path: std::path::PathBuf,
    pub report_html_path: std::path::PathBuf,
}

include!("summary_rendering.rs");
include!("manifest_writing.rs");
include!("provenance_capture.rs");
include!("html_rendering.rs");

#[cfg(test)]
include!("summary_contracts.rs");
