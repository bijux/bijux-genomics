use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::measure::SeqkitMetrics;
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{validate::ValidateEffectiveParams, PairedMode};

pub const STAGE_ID: &str = "fastq.validate_pre";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct ValidatePreUserConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidatePreEffectiveConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

pub fn plan(tool: &ToolExecutionSpecV1, r1: &Path, out_dir: &Path) -> StagePlanV1 {
    let effective_params = ValidateEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        q_cutoff: None,
    };
    StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs: Vec::new(),
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "out_dir": out_dir
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize validate effective params"),
        aux_images: std::collections::BTreeMap::new(),
    }
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

pub fn resolve_config(user: ValidatePreUserConfig) -> ValidatePreEffectiveConfig {
    ValidatePreEffectiveConfig {
        tool: user.tool,
        r1: user.r1,
        out_dir: user.out_dir,
    }
}

pub fn plan_from_config(
    tool: &ToolExecutionSpecV1,
    config: &ValidatePreEffectiveConfig,
) -> StagePlanV1 {
    plan(tool, &config.r1, &config.out_dir)
}

pub fn validate_reads_total(tool: &str, input_stats: &SeqkitMetrics, stdout: &str) -> Result<u64> {
    let reads_total = match tool {
        "seqtk" | "fastqc" => input_stats.reads,
        "fastqvalidator" | "fastqvalidator_official" => {
            parse_fastqvalidator_count(stdout).unwrap_or(input_stats.reads)
        }
        "fqtools" => stdout
            .lines()
            .next()
            .ok_or_else(|| anyhow!("fqtools output missing"))?
            .parse::<u64>()?,
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };
    Ok(reads_total)
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

fn parse_fastqvalidator_count(stdout: &str) -> Option<u64> {
    for token in stdout.split_whitespace() {
        if let Ok(count) = token.parse::<u64>() {
            return Some(count);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::validate_reads_total;
    use anyhow::Result;
    use bijux_core::measure::SeqkitMetrics;

    #[test]
    fn validate_reads_total_uses_input_for_fastqc() -> Result<()> {
        let input = SeqkitMetrics {
            reads: 12,
            bases: 120,
            mean_q: 30.0,
            gc_percent: 50.0,
        };
        let count = validate_reads_total("fastqc", &input, "")?;
        assert_eq!(count, 12);
        Ok(())
    }

    #[test]
    fn validate_reads_total_parses_fqtools() -> Result<()> {
        let input = SeqkitMetrics {
            reads: 1,
            bases: 10,
            mean_q: 30.0,
            gc_percent: 50.0,
        };
        let count = validate_reads_total("fqtools", &input, "42\n")?;
        assert_eq!(count, 42);
        Ok(())
    }

    #[test]
    fn validate_reads_total_rejects_unknown_tool() {
        let input = SeqkitMetrics {
            reads: 1,
            bases: 10,
            mean_q: 30.0,
            gc_percent: 50.0,
        };
        match validate_reads_total("mystery", &input, "") {
            Ok(_) => panic!("expected unsupported tool"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }
}
