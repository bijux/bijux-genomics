#![allow(clippy::too_many_arguments)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::STAGE_FILTER_LOW_COMPLEXITY;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_FILTER_LOW_COMPLEXITY;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default)]
pub struct LowComplexityPlanOptions {
    pub entropy_threshold: Option<f64>,
    pub polyx_threshold: Option<u32>,
}

impl LowComplexityPlanOptions {
    fn resolved_entropy_threshold(&self) -> f64 {
        self.entropy_threshold.unwrap_or(0.5)
    }
}

/// # Errors
/// Returns an error if any requested low-complexity filter tool is not admitted for
/// `fastq.filter_low_complexity`.
pub fn normalize_low_complexity_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}

fn low_complexity_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "prinseq" => Some("prinseq_good.fastq"),
        "bbduk" => Some("bbduk.fastq.gz"),
        _ => None,
    }
}

/// Build a low-complexity filter plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_low_complexity(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &LowComplexityPlanOptions,
) -> Result<StagePlanV1> {
    let output_name = low_complexity_output_name(&tool.tool_id.0)
        .ok_or_else(|| anyhow!("unsupported low-complexity tool"))?;
    ensure_low_complexity_option_support(&tool.tool_id.0, options)?;
    let output_r1 = if r2.is_some() {
        out_dir.join(format!("R1.{output_name}"))
    } else {
        out_dir.join(output_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{output_name}")));
    let report = out_dir.join("low_complexity_report.json");
    let (raw_backend_report, raw_backend_report_format) =
        raw_backend_report_contract(&tool.tool_id.0, out_dir);
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("filtered_fastq_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("filtered_fastq_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("filter_report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: low_complexity_command_template(
                tool,
                r1,
                r2,
                &output_r1,
                output_r2.as_deref(),
                &report,
                raw_backend_report.as_deref(),
                options,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
            "raw_backend_report": raw_backend_report,
            "raw_backend_report_format": raw_backend_report_format,
            "entropy_threshold": options.resolved_entropy_threshold(),
            "polyx_threshold": options.polyx_threshold,
        }),
        effective_params: serde_json::json!({
            "paired_mode": if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
            "threads": tool.resources.threads,
            "entropy_threshold": options.resolved_entropy_threshold(),
            "polyx_threshold": options.polyx_threshold,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn low_complexity_command_template(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
    options: &LowComplexityPlanOptions,
) -> Result<Vec<String>> {
    if tool.tool_id.as_str() == "bbduk" {
        let mut command = vec![
            "bbduk".to_string(),
            format!("in={}", r1.display()),
            format!("out={}", output_r1.display()),
            format!("entropy={}", options.resolved_entropy_threshold()),
        ];
        if let Some(raw_backend_report) = raw_backend_report {
            command.push(format!("stats={}", raw_backend_report.display()));
        }
        if let Some(polyx_threshold) = options.polyx_threshold {
            command.push(format!("maxpoly={polyx_threshold}"));
        }
        if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
            command.push(format!("in2={}", r2.display()));
            command.push(format!("out2={}", output_r2.display()));
        }
        return Ok(command);
    }
    if tool.tool_id.as_str() == "prinseq" {
        let mut command = vec![
            "prinseq++".to_string(),
            "-threads".to_string(),
            tool.resources.threads.max(1).to_string(),
            "-fastq".to_string(),
            r1.display().to_string(),
            "-out_good".to_string(),
            output_r1.display().to_string(),
            "-out_bad".to_string(),
            "/dev/null".to_string(),
            "-lc_entropy".to_string(),
            options.resolved_entropy_threshold().to_string(),
        ];
        if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
            command.extend([
                "-fastq2".to_string(),
                r2.display().to_string(),
                "-out_good2".to_string(),
                output_r2.display().to_string(),
                "-out_bad2".to_string(),
                "/dev/null".to_string(),
                "-out_single".to_string(),
                "/dev/null".to_string(),
                "-out_single2".to_string(),
                "/dev/null".to_string(),
            ]);
        }
        return Ok(command);
    }
    crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("reads", Some(r1.display().to_string())),
            ("reads_r1", Some(r1.display().to_string())),
            ("reads_r2", r2.map(|path| path.display().to_string())),
            ("filtered_reads", Some(output_r1.display().to_string())),
            ("filtered_reads_r1", Some(output_r1.display().to_string())),
            ("filtered_reads_r2", output_r2.map(|path| path.display().to_string())),
            ("filtered_fastq", Some(output_r1.display().to_string())),
            ("filtered_fastq_r1", Some(output_r1.display().to_string())),
            ("filtered_fastq_r2", output_r2.map(|path| path.display().to_string())),
            ("filter_report_json", Some(report_json.display().to_string())),
            ("raw_backend_report", raw_backend_report.map(|path| path.display().to_string())),
        ],
    )
}

fn raw_backend_report_contract(
    tool: &str,
    out_dir: &Path,
) -> (Option<PathBuf>, Option<&'static str>) {
    match tool {
        "bbduk" => (Some(out_dir.join("bbduk.low_complexity.stats")), Some("bbduk_stats")),
        _ => (None, None),
    }
}

fn ensure_low_complexity_option_support(
    tool_id: &str,
    options: &LowComplexityPlanOptions,
) -> Result<()> {
    if tool_id == "prinseq" && options.polyx_threshold.is_some() {
        return Err(anyhow!(
            "prinseq low-complexity planning does not support explicit polyx_threshold"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{low_complexity_output_name, plan_low_complexity, LowComplexityPlanOptions};
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use std::path::Path;

    fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id.to_string()),
            tool_version: "test".to_string(),
            image: ContainerImageRefV1 { image: format!("example/{tool_id}"), digest: None },
            command: CommandSpecV1 { template: vec!["placeholder".to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 8,
            },
        }
    }

    #[test]
    fn low_complexity_output_names_reject_planned_only_tools() {
        assert_eq!(low_complexity_output_name("dustmasker"), None);
        assert_eq!(low_complexity_output_name("fastp"), None);
    }

    #[test]
    fn bbduk_low_complexity_plan_maps_entropy_and_polyx_thresholds() {
        let plan = plan_low_complexity(
            &tool("bbduk"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &LowComplexityPlanOptions { entropy_threshold: Some(0.8), polyx_threshold: Some(24) },
        )
        .expect("plan");
        assert!(plan.command.template.iter().any(|token| token == "entropy=0.8"));
        assert!(plan.command.template.iter().any(|token| token == "maxpoly=24"));
    }

    #[test]
    fn prinseq_low_complexity_plan_rejects_explicit_polyx_threshold() {
        let error = plan_low_complexity(
            &tool("prinseq"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &LowComplexityPlanOptions { entropy_threshold: Some(0.5), polyx_threshold: Some(20) },
        )
        .expect_err("explicit polyx threshold should fail");
        assert!(error.to_string().contains("does not support explicit polyx_threshold"));
    }

    #[test]
    fn prinseq_low_complexity_plan_uses_default_entropy_threshold() {
        let plan = plan_low_complexity(
            &tool("prinseq"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &LowComplexityPlanOptions::default(),
        )
        .expect("plan");
        assert!(plan.command.template.windows(2).any(|window| window == ["-lc_entropy", "0.5"]));
    }
}
