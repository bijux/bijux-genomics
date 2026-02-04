use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_core::domain::PipelineSpec;
use bijux_core::execution_plan::{default_edges_for_stages, ExecutionPlan, PlanPolicy};
use bijux_core::{ContainerImageRefV1, ToolExecutionSpecV1};

pub const PLANNER_VERSION: &str = "bijux-planner-fastq.v1";

fn required_stage_ids() -> Vec<String> {
    vec![
        "fastq.validate_pre".to_string(),
        "fastq.detect_adapters".to_string(),
        "fastq.trim".to_string(),
        "fastq.filter".to_string(),
        "fastq.stats_neutral".to_string(),
        "fastq.qc_post".to_string(),
    ]
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct DefaultPipelineOptions {
    pub paired: bool,
    pub enable_merge: bool,
    pub enable_correct: bool,
    pub enable_qc_post: bool,
    pub enable_screen: bool,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: false,
            enable_qc_post: true,
            enable_screen: false,
        }
    }
}

#[must_use]
pub fn default_pipeline_spec(options: DefaultPipelineOptions) -> PipelineSpec {
    let mut stages = required_stage_ids();
    if options.paired && options.enable_correct {
        stages.push("fastq.correct".to_string());
    }
    if options.paired && options.enable_merge {
        stages.push("fastq.merge".to_string());
    }
    if options.enable_screen && !stages.iter().any(|stage| stage == "fastq.screen") {
        stages.push("fastq.screen".to_string());
    }
    if options.enable_qc_post && !stages.iter().any(|stage| stage == "fastq.qc_post") {
        stages.push("fastq.qc_post".to_string());
    }
    PipelineSpec { stages }
}

#[derive(Debug, Clone)]
pub struct PreprocessPolicyDecision {
    pub adapter_inference: Option<serde_json::Value>,
    pub adapter_bank_preset_override: Option<String>,
    pub pipeline_stages: Vec<String>,
    pub pipeline_tools: Vec<String>,
    pub stage_skips: Vec<serde_json::Value>,
}

#[must_use]
pub fn apply_preprocess_policy(
    pipeline_stages: Vec<String>,
    pipeline_tools: Vec<String>,
) -> PreprocessPolicyDecision {
    let mut pipeline_stages = pipeline_stages;
    let mut pipeline_tools = pipeline_tools;
    let mut stage_skips = Vec::new();

    if let (Some(trim_idx), Some(filter_idx)) = (
        pipeline_stages
            .iter()
            .position(|stage| stage == "fastq.trim"),
        pipeline_stages
            .iter()
            .position(|stage| stage == "fastq.filter"),
    ) {
        let trim_tool = pipeline_tools.get(trim_idx).map(String::as_str);
        let filter_tool = pipeline_tools.get(filter_idx).map(String::as_str);
        if trim_tool == Some("fastp") && filter_tool == Some("fastp") {
            let skipped_stage = pipeline_stages.remove(filter_idx);
            let skipped_tool = pipeline_tools.remove(filter_idx);
            stage_skips.push(serde_json::json!({
                "stage_id": skipped_stage,
                "tool_id": skipped_tool,
                "reason": "fastp trimming already performs quality filtering; filter stage skipped",
                "equivalent_params": {
                    "quality_filtering": true
                }
            }));
        }
    }

    PreprocessPolicyDecision {
        adapter_inference: None,
        adapter_bank_preset_override: None,
        pipeline_stages,
        pipeline_tools,
        stage_skips,
    }
}

#[derive(Debug, Clone)]
pub struct FastqPlanConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub stages: Vec<String>,
    pub tools: Vec<ToolExecutionSpecV1>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out_dir: PathBuf,
}

pub struct FastqPlanner;

impl FastqPlanner {
    /// # Errors
    /// Returns an error if planning fails or the plan lint fails.
    pub fn plan(config: &FastqPlanConfig) -> Result<ExecutionPlan> {
        if config.stages.len() != config.tools.len() {
            return Err(anyhow!(
                "pipeline stages/tools length mismatch: {} vs {}",
                config.stages.len(),
                config.tools.len()
            ));
        }
        let out_dir = config.out_dir.clone();
        let plans = bijux_stages_fastq::fastq::preprocess::plan_preprocess_pipeline(
            &config.stages,
            &config.tools,
            &config.aux_images,
            config.adapter_bank.as_ref(),
            config.polyx_bank.as_ref(),
            config.contaminant_bank.as_ref(),
            config.enable_contaminant_removal,
            &config.r1,
            config.r2.as_deref(),
            |stage, tool, _r1, _r2| {
                let stage_dir = stage.trim_start_matches("fastq.");
                Ok(out_dir.join(stage_dir).join(&tool.tool_id.0))
            },
        )?;
        let edges = default_edges_for_stages(&plans);
        ExecutionPlan::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            plans,
            edges,
        )
    }
}

pub fn select_trim_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "fastp",
        "cutadapt",
        "bbduk",
        "adapterremoval",
        "trimmomatic",
        "trim_galore",
        "atropos",
        "seqpurge",
    ];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool != "seqpurge");
    }
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_validate_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "seqtk",
        "fastqc",
        "fastqvalidator",
        "fastqvalidator_official",
        "fqtools",
    ];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_filter_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_merge_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_correct_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["rcorrector", "spades", "bayeshammer", "lighter", "musket"];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool == "rcorrector");
    }
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_qc_post_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_umi_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_screen_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_stats_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["seqkit_stats"];
    select_tools_with_allowlist(tools, &allowed)
}

#[must_use]
pub fn scale_tool_spec_for_jobs(tool: &ToolExecutionSpecV1, jobs: usize) -> ToolExecutionSpecV1 {
    if jobs <= 1 {
        return tool.clone();
    }
    let mut scaled = tool.clone();
    let threads = scaled.resources.threads;
    let denom = u32::try_from(jobs).unwrap_or(1);
    scaled.resources.threads = (threads / denom).max(1);
    scaled
}

fn select_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn select_trim_tools_dedup_and_sort() {
        let tools = vec![
            "fastp".to_string(),
            "FASTP".to_string(),
            "cutadapt".to_string(),
        ];
        match select_trim_tools(&tools) {
            Ok(normalized) => {
                assert_eq!(
                    normalized,
                    vec!["cutadapt".to_string(), "fastp".to_string()]
                );
            }
            Err(err) => panic!("normalize failed: {err}"),
        }
    }

    #[test]
    fn select_trim_tools_blocks_experimental_by_default() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");
        let tools = vec!["seqpurge".to_string()];
        match select_trim_tools(&tools) {
            Ok(_) => panic!("expected failure"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }

    #[test]
    fn select_trim_tools_allows_experimental_when_enabled() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").ok();
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
        let tools = vec!["seqpurge".to_string()];
        match select_trim_tools(&tools) {
            Ok(normalized) => assert_eq!(normalized, vec!["seqpurge".to_string()]),
            Err(err) => panic!("normalize failed: {err}"),
        }
        match prev {
            Some(value) => std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", value),
            None => std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS"),
        }
    }

    #[test]
    fn select_tools_rejects_empty() {
        match select_validate_tools(&[]) {
            Ok(_) => panic!("expected empty failure"),
            Err(err) => assert!(err.to_string().contains("no tools specified")),
        }
    }
}
