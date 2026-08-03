use std::path::Path;

use anyhow::Result;
use bijux_dna_stage_contract::StagePlanV1;
use serde::{Deserialize, Serialize};

const READS_R1_PLACEHOLDER: &str = "__BIJUX_READS_R1__";
const READS_R2_PLACEHOLDER: &str = "__BIJUX_READS_R2__";
const RESULT_ROOT_PLACEHOLDER: &str = "__BIJUX_RESULT_ROOT__";

pub const BENCHMARK_STAGE_CONTRACT_SCHEMA_VERSION: &str =
    "bijux.benchmark.planner_stage_contract.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageContractV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub parameter_schema_version: String,
    pub default_provider: String,
    pub output_contract: String,
    pub observation_authority: String,
    pub result_attribution: String,
    pub domain_defaults: BenchmarkLayoutDefaultsV1,
    pub tools: Vec<BenchmarkToolPlansV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLayoutDefaultsV1 {
    pub single_end: serde_json::Value,
    pub paired_end: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkToolPlansV1 {
    pub tool_id: String,
    pub single_end: StagePlanV1,
    pub paired_end: StagePlanV1,
}

/// Resolve the complete retained-tool planner contract for read-length profiling.
///
/// The returned plans use stable path sentinels so an execution orchestrator can replace paths
/// without reimplementing backend argv construction. Effective parameters remain serialized in
/// each plan, while `domain_defaults` records the crate-owned values before explicit overrides.
///
/// # Errors
/// Returns an error if a retained tool contract cannot be loaded or planned.
pub fn profile_read_lengths_benchmark_contract(
    repo_root: &Path,
    threads_override: Option<u32>,
    histogram_bins_override: Option<u32>,
) -> Result<BenchmarkStageContractV1> {
    let stage_id = &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS;
    let tools = crate::selection::allowed_tools_for_stage(stage_id)
        .into_iter()
        .map(|tool_id| {
            let tool = crate::selection::load_fastq_domain_tool_execution_spec(
                repo_root, stage_id, &tool_id,
            )?;
            let single_end = crate::tool_adapters::fastq::profile_read_lengths::plan_with_options(
                &tool,
                Path::new(READS_R1_PLACEHOLDER),
                None,
                Path::new(RESULT_ROOT_PLACEHOLDER),
                threads_override,
                histogram_bins_override,
            )?;
            let paired_end = crate::tool_adapters::fastq::profile_read_lengths::plan_with_options(
                &tool,
                Path::new(READS_R1_PLACEHOLDER),
                Some(Path::new(READS_R2_PLACEHOLDER)),
                Path::new(RESULT_ROOT_PLACEHOLDER),
                threads_override,
                histogram_bins_override,
            )?;
            Ok(BenchmarkToolPlansV1 {
                tool_id: tool_id.as_str().to_string(),
                single_end,
                paired_end,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BenchmarkStageContractV1 {
        schema_version: BENCHMARK_STAGE_CONTRACT_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.as_str().to_string(),
        parameter_schema_version:
            bijux_dna_domain_fastq::params::stats::READ_LENGTH_PROFILE_SCHEMA_VERSION.to_string(),
        default_provider: "bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults"
            .to_string(),
        output_contract: bijux_dna_domain_fastq::PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION
            .to_string(),
        observation_authority: "bijux_dna_api::internal::fastq::stages::profile_read_lengths"
            .to_string(),
        result_attribution: "shared_observer_after_backend_execution".to_string(),
        domain_defaults: BenchmarkLayoutDefaultsV1 {
            single_end: serde_json::to_value(
                bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults(false),
            )?,
            paired_end: serde_json::to_value(
                bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults(true),
            )?,
        },
        tools,
    })
}
