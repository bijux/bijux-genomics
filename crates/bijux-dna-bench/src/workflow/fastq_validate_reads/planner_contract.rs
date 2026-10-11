//! Owner: bijux-dna-bench
//! Checked-in planner projection consumed by FASTQ validation campaigns.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

const CONTRACT_PATH: &str =
    "benchmarks/contracts/planner/fastq-validate-reads.planner-contract.json";
const READS_R1_PLACEHOLDER: &str = "__BIJUX_READS_R1__";
const READS_R2_PLACEHOLDER: &str = "__BIJUX_READS_R2__";
const RESULT_ROOT_PLACEHOLDER: &str = "__BIJUX_RESULT_ROOT__";

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ValidationPlannerContract {
    schema_version: String,
    stage_id: String,
    pub parameter_schema_version: String,
    pub default_provider: String,
    pub domain_defaults: PlannerLayoutDefaults,
    tools: Vec<PlannerToolContract>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct PlannerLayoutDefaults {
    pub single_end: PlannerEffectiveParameters,
    pub paired_end: PlannerEffectiveParameters,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PlannerEffectiveParameters {
    pub schema_version: String,
    pub threads: u64,
    pub validation_mode: String,
    pub paired_mode: String,
    pub pair_sync_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct PlannerToolContract {
    tool_id: String,
    single_end: PlannerStagePlan,
    paired_end: PlannerStagePlan,
}

#[derive(Debug, Clone, Deserialize)]
struct PlannerStagePlan {
    command: PlannerCommand,
    effective_params: PlannerEffectiveParameters,
}

#[derive(Debug, Clone, Deserialize)]
struct PlannerCommand {
    template: Vec<String>,
}

pub(super) fn load_validation_planner_contract(
    repository_root: &Path,
) -> Result<ValidationPlannerContract> {
    let path = repository_root.join(CONTRACT_PATH);
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, 4 * 1024 * 1024)
        .with_context(|| format!("read FASTQ validation planner contract {}", path.display()))?;
    let contract: ValidationPlannerContract = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse FASTQ validation planner contract {}", path.display()))?;
    if contract.schema_version != "bijux.benchmark.planner_stage_contract.v1"
        || contract.stage_id != "fastq.validate_reads"
        || contract.tools.len() != 5
    {
        bail!("FASTQ validation planner contract identity is invalid");
    }
    Ok(contract)
}

pub(super) fn validation_planner_contract_sha256(repository_root: &Path) -> Result<String> {
    bijux_dna_infra::hash_file_sha256(&repository_root.join(CONTRACT_PATH))
        .context("hash FASTQ validation planner contract")
}

impl ValidationPlannerContract {
    pub(super) fn tool(&self, tool_id: &str) -> Result<&PlannerToolContract> {
        self.tools
            .iter()
            .find(|tool| tool.tool_id == tool_id)
            .context("planner contract omits requested FASTQ validation backend")
    }
}

impl PlannerToolContract {
    pub(super) fn materialize_command(
        &self,
        r1: &str,
        r2: Option<&str>,
        output_root: &str,
    ) -> Result<String> {
        ensure_projection_path(r1)?;
        if let Some(r2) = r2 {
            ensure_projection_path(r2)?;
        }
        ensure_projection_path(output_root)?;
        let plan = if r2.is_some() { &self.paired_end } else { &self.single_end };
        if plan.command.template.len() != 3
            || plan.command.template[0] != "sh"
            || plan.command.template[1] != "-lc"
            || plan.effective_params.validation_mode != "strict"
            || plan.effective_params.threads != 4
        {
            bail!("FASTQ validation planner command or effective parameters are invalid");
        }
        let mut command = plan.command.template[2]
            .replace(READS_R1_PLACEHOLDER, r1)
            .replace(RESULT_ROOT_PLACEHOLDER, output_root);
        if let Some(r2) = r2 {
            command = command.replace(READS_R2_PLACEHOLDER, r2);
        } else if command.contains(READS_R2_PLACEHOLDER) {
            bail!("single-end planner command retains a mate placeholder");
        }
        if command.contains("__BIJUX_") {
            bail!("materialized planner command retains an unresolved placeholder");
        }
        Ok(command)
    }
}

fn ensure_projection_path(path: &str) -> Result<()> {
    if !path.starts_with('/')
        || path.contains("..")
        || !path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
    {
        bail!("FASTQ validation planner path is not safe for sentinel projection");
    }
    Ok(())
}
