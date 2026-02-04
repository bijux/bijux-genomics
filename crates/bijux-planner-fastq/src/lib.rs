use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_core::execution_plan::{default_edges_for_stages, ExecutionPlan, PlanPolicy};
use bijux_core::{ContainerImageRefV1, ToolExecutionSpecV1};

pub const PLANNER_VERSION: &str = "bijux-planner-fastq.v1";

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

pub fn normalize_trim_tool_list(tools: &[String]) -> Result<Vec<String>> {
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
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "seqtk",
        "fastqc",
        "fastqvalidator",
        "fastqvalidator_official",
        "fqtools",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["rcorrector", "spades", "bayeshammer", "lighter", "musket"];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool == "rcorrector");
    }
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_stats_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["seqkit_stats"];
    normalize_tools_with_allowlist(tools, &allowed)
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
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
    fn normalize_trim_tools_dedup_and_sort() {
        let tools = vec![
            "fastp".to_string(),
            "FASTP".to_string(),
            "cutadapt".to_string(),
        ];
        match normalize_trim_tool_list(&tools) {
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
    fn normalize_trim_tools_blocks_experimental_by_default() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");
        let tools = vec!["seqpurge".to_string()];
        match normalize_trim_tool_list(&tools) {
            Ok(_) => panic!("expected failure"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }

    #[test]
    fn normalize_trim_tools_allows_experimental_when_enabled() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").ok();
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
        let tools = vec!["seqpurge".to_string()];
        match normalize_trim_tool_list(&tools) {
            Ok(normalized) => assert_eq!(normalized, vec!["seqpurge".to_string()]),
            Err(err) => panic!("normalize failed: {err}"),
        }
        match prev {
            Some(value) => std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", value),
            None => std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS"),
        }
    }

    #[test]
    fn normalize_tools_rejects_empty() {
        match normalize_validate_tool_list(&[]) {
            Ok(_) => panic!("expected empty failure"),
            Err(err) => assert!(err.to_string().contains("no tools specified")),
        }
    }
}
