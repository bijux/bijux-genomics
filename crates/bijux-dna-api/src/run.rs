use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info_span, warn};

use crate::request_args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, PlanRunRequest, PlanRunResult,
    RenderReportRequest, RenderReportResult, RunRequest, RunResult, RunStatus,
};
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_core::contract::{Profile, RunSpec, ToolRegistry};
use bijux_dna_core::ids::RunId;
use bijux_dna_engine::Engine;
use bijux_dna_pipelines::registry::PipelineRegistry;
use bijux_dna_pipelines::{Domain, PipelineProfile};
use bijux_dna_runner::DockerRunner;
use bijux_dna_runtime::{ensure_stage_supported_by_runner, RunnerContractKind};
use bijux_dna_stage_contract::{build_run_execution_plan, RunExecutionPlan};
use cargo_metadata::MetadataCommand;

include!("run_sections/planning_and_hpc.rs");
include!("run_sections/execute_run.rs");
include!("run_sections/reporting_and_audit.rs");
