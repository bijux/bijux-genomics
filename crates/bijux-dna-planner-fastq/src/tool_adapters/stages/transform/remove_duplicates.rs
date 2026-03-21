use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    remove_duplicates::{
        DedupMode, RemoveDuplicatesEffectiveParams, REMOVE_DUPLICATES_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_REMOVE_DUPLICATES;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_REMOVE_DUPLICATES;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveDuplicatesPlanOptions {
    pub dedup_mode: DedupMode,
    pub keep_order: bool,
}

impl Default for RemoveDuplicatesPlanOptions {
    fn default() -> Self {
        Self {
            dedup_mode: DedupMode::Exact,
            keep_order: true,
        }
    }
}

pub fn normalize_deduplicate_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

fn deduplicate_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastuniq" => Some("fastuniq.fastq.gz"),
        "clumpify" => Some("clumpify.fastq.gz"),
        _ => None,
    }
}

/// Build a deduplicate plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_deduplicate(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_deduplicate_with_options(
        tool,
        r1,
        r2,
        out_dir,
        &RemoveDuplicatesPlanOptions::default(),
    )
}

/// Build a deduplicate plan with governed stage options.
///
/// # Errors
/// Returns an error if the tool is unsupported or the requested options are not yet supported
/// by the backend-specific adapter.
pub fn plan_deduplicate_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<StagePlanV1> {
    let paired_mode = r2.is_some();
    validate_deduplicate_options(&tool.tool_id.0, paired_mode, options)?;
    let output_r1 = if paired_mode {
        out_dir.join(format!("{}.dedup.R1.fastq.gz", tool.tool_id))
    } else {
        let output_name = deduplicate_output_name(&tool.tool_id.0)
            .ok_or_else(|| anyhow!("unsupported deduplicate tool"))?;
        out_dir.join(output_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("{}.dedup.R2.fastq.gz", tool.tool_id)));
    let report = out_dir.join("deduplicate_report.json");
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
        ArtifactId::from_static("dedup_reads_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("dedup_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
    let effective_params = RemoveDuplicatesEffectiveParams {
        schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(paired_mode),
        dedup_mode: options.dedup_mode.clone(),
        keep_order: options.keep_order,
    };
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
            template: deduplicate_command(
                &tool.tool_id.0,
                r1,
                r2,
                &output_r1,
                output_r2.as_deref(),
                &report,
                out_dir,
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
            "dedup_mode": options.dedup_mode,
            "keep_order": options.keep_order,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize deduplicate effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn deduplicate_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report: &Path,
    out_dir: &Path,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<Vec<String>> {
    match tool_id {
        "fastuniq" => {
            let r2 = r2.ok_or_else(|| anyhow!("fastuniq requires paired-end reads"))?;
            let output_r2 =
                output_r2.ok_or_else(|| anyhow!("fastuniq requires paired deduplicated output"))?;
            let input_manifest = out_dir.join("fastuniq_inputs.txt");
            let backend_log = out_dir.join("fastuniq.log");
            let report_payload = serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v1",
                "stage_id": STAGE_ID.as_str(),
                "tool_id": tool_id,
                "dedup_mode": options.dedup_mode,
                "keep_order": options.keep_order,
                "input_r1": r1,
                "input_r2": r2,
                "output_r1": output_r1,
                "output_r2": output_r2,
                "backend_log": backend_log,
            });
            let script = format!(
                "set -euo pipefail\nprintf '%s\\n%s\\n' {} {} > {}\nfastuniq -i {} -t q -o {} -p {} > {} 2>&1\nprintf '%s\\n' {} > {}\n",
                shell_quote_path(r1),
                shell_quote_path(r2),
                shell_quote_path(&input_manifest),
                shell_quote_path(&input_manifest),
                shell_quote_path(output_r1),
                shell_quote_path(output_r2),
                shell_quote_path(&backend_log),
                shell_quote_str(&report_payload.to_string()),
                shell_quote_path(report),
            );
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        "clumpify" => {
            let backend_log = out_dir.join("clumpify.log");
            let mut script = format!(
                "set -euo pipefail\nclumpify.sh in={} out={} dedupe=t",
                shell_quote_arg(&format!("in={}", r1.display())),
                shell_quote_arg(&format!("out={}", output_r1.display())),
            );
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                script.push(' ');
                script.push_str(&shell_quote_arg(&format!("in2={}", r2.display())));
                script.push(' ');
                script.push_str(&shell_quote_arg(&format!("out2={}", output_r2.display())));
            }
            script.push_str(&format!(" > {} 2>&1\n", shell_quote_path(&backend_log)));
            let report_payload = serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v1",
                "stage_id": STAGE_ID.as_str(),
                "tool_id": tool_id,
                "dedup_mode": options.dedup_mode,
                "keep_order": options.keep_order,
                "input_r1": r1,
                "input_r2": r2,
                "output_r1": output_r1,
                "output_r2": output_r2,
                "backend_log": backend_log,
            });
            script.push_str(&format!(
                "printf '%s\\n' {} > {}\n",
                shell_quote_str(&report_payload.to_string()),
                shell_quote_path(report),
            ));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!("unsupported deduplicate tool {tool_id}")),
    }
}

fn validate_deduplicate_options(
    tool_id: &str,
    paired_mode: bool,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<()> {
    if tool_id == "fastuniq" && !paired_mode {
        return Err(anyhow!("fastuniq requires paired-end reads"));
    }
    if options.dedup_mode != DedupMode::Exact {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports only dedup_mode=exact"
        ));
    }
    if !options.keep_order {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports only keep_order=true"
        ));
    }
    Ok(())
}

pub fn parse_dedup_mode(value: &str) -> Result<DedupMode> {
    match value {
        "exact" => Ok(DedupMode::Exact),
        "sequence_identity" => Ok(DedupMode::SequenceIdentity),
        "optical_aware" => Ok(DedupMode::OpticalAware),
        _ => Err(anyhow!("unsupported dedup_mode: {value}")),
    }
}

fn shell_quote_arg(value: &str) -> String {
    shell_quote_str(value)
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::ids::ToolId;
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints};

    #[test]
    fn deduplicate_output_name_rejects_unadmitted_tools() {
        assert!(deduplicate_output_name("prinseq").is_none());
    }

    #[test]
    fn plan_deduplicate_fastuniq_builds_paired_command_and_report() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::new("fastuniq"),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["unused".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        };

        let plan = plan_deduplicate(
            &tool,
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
        )
        .expect("deduplicate planner should build fastuniq command");
        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("fastuniq_inputs.txt"));
        assert!(script.contains("fastuniq.log"));
        assert!(script.contains("\"tool_id\":\"fastuniq\""));
        assert_eq!(
            plan.params["report_json"],
            serde_json::json!("out/deduplicate_report.json")
        );
        assert_eq!(
            plan.effective_params["schema_version"],
            serde_json::json!("bijux.fastq.params.remove_duplicates.v1")
        );
        assert_eq!(
            plan.effective_params["dedup_mode"],
            serde_json::json!("exact")
        );
        assert_eq!(plan.effective_params["keep_order"], serde_json::json!(true));
        assert!(script.contains("reads_R1.fastq.gz"));
        assert!(script.contains("reads_R2.fastq.gz"));
        assert!(script.contains("deduplicate_report.json"));
    }

    #[test]
    fn plan_deduplicate_fastuniq_rejects_single_end_inputs() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::new("fastuniq"),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["unused".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        };

        let error = plan_deduplicate(
            &tool,
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
        )
        .expect_err("fastuniq must reject single-end dedup planning");
        assert!(error.to_string().contains("paired-end"));
    }

    #[test]
    fn plan_deduplicate_clumpify_emits_governed_report() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::new("clumpify"),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["unused".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        };

        let plan = plan_deduplicate(&tool, Path::new("reads.fastq.gz"), None, Path::new("out"))
            .expect("clumpify single-end dedup planning should succeed");

        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("clumpify.sh"));
        assert!(script.contains("clumpify.log"));
        assert!(script.contains("\"tool_id\":\"clumpify\""));
    }

    #[test]
    fn plan_deduplicate_rejects_non_exact_mode_until_backend_support_exists() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::new("clumpify"),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["unused".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        };

        let error = plan_deduplicate_with_options(
            &tool,
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::SequenceIdentity,
                keep_order: true,
            },
        )
        .expect_err("non-exact dedup mode must fail until backend mapping exists");

        assert!(error.to_string().contains("dedup_mode=exact"));
    }

    #[test]
    fn plan_deduplicate_rejects_keep_order_false_until_backend_mapping_exists() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::new("clumpify"),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["unused".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        };

        let error = plan_deduplicate_with_options(
            &tool,
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::Exact,
                keep_order: false,
            },
        )
        .expect_err("keep_order=false must fail until backend mapping exists");

        assert!(error.to_string().contains("keep_order=true"));
    }
}
