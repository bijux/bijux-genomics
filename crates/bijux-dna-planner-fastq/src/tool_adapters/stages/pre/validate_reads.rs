use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{validate::ValidateEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_VALIDATE_READS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_VALIDATE_READS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default)]
pub struct ValidatePlanOptions {
    pub q_cutoff: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ValidateReadsUserConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub out_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateReadsEffectiveConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub out_dir: std::path::PathBuf,
}

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, r1, r2, out_dir, &ValidatePlanOptions::default())
}

pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &ValidatePlanOptions,
) -> Result<StagePlanV1> {
    validate_option_support(&tool.tool_id.0, options)?;
    let report_path = out_dir.join("validation.json");
    let validated_reads_manifest = out_dir.join("validated_reads_manifest.json");
    let effective_params = ValidateEffectiveParams {
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        q_cutoff: options.q_cutoff,
    };
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
    let command_template = validation_command(
        tool,
        r1,
        r2,
        &report_path,
        &validated_reads_manifest,
        out_dir,
        options,
    )?;
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
            template: command_template,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("validation_report"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("validated_reads_manifest"),
                    validated_reads_manifest.clone(),
                    ArtifactRole::StageReport,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "out_dir": out_dir,
            "report_json": report_path,
            "validated_reads_manifest": validated_reads_manifest,
            "q_cutoff": options.q_cutoff,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize validate effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

pub fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub fn resolve_config(user: ValidateReadsUserConfig) -> ValidateReadsEffectiveConfig {
    ValidateReadsEffectiveConfig {
        tool: user.tool,
        r1: user.r1,
        r2: user.r2,
        out_dir: user.out_dir,
    }
}

pub fn plan_from_config(
    tool: &ToolExecutionSpecV1,
    config: &ValidateReadsEffectiveConfig,
) -> Result<StagePlanV1> {
    plan(tool, &config.r1, config.r2.as_deref(), &config.out_dir)
}

fn validate_option_support(tool_id: &str, options: &ValidatePlanOptions) -> Result<()> {
    if options.q_cutoff.is_some() {
        return Err(anyhow!(
            "{tool_id} validate-reads adapter does not yet map q_cutoff into a governed backend-native validation contract"
        ));
    }
    Ok(())
}

fn validation_command(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    report_path: &Path,
    validated_reads_manifest: &Path,
    out_dir: &Path,
    options: &ValidatePlanOptions,
) -> Result<Vec<String>> {
    let single_command = |reads: &Path, log_path: &Path, status_var: &str| -> Result<String> {
        let rendered = crate::tool_adapters::template_render::render_command_template(
            &tool.command.template,
            &[
                ("reads", Some(reads.display().to_string())),
                ("reads_r1", Some(reads.display().to_string())),
            ],
        )?;
        Ok(format!(
            "{} > {} 2>&1\n{status_var}=$?",
            shell_join(&rendered),
            shell_quote(log_path)
        ))
    };

    let r1_log = out_dir.join("validation_r1.log");
    let mut commands = vec!["set +e".to_string(), single_command(r1, &r1_log, "status_r1")?];
    let r2_log = r2.map(|_| out_dir.join("validation_r2.log"));
    if let Some(r2) = r2 {
        commands.push(single_command(
            r2,
            r2_log
                .as_deref()
                .ok_or_else(|| anyhow!("paired validation log path missing"))?,
            "status_r2",
        )?);
    } else {
        commands.push("status_r2=0".to_string());
    }
    commands.push("strict_pass=true".to_string());
    commands.push("exit_code=0".to_string());
    commands.push(
        "if [ \"$status_r1\" -ne 0 ]; then strict_pass=false; exit_code=$status_r1; fi"
            .to_string(),
    );
    commands.push(
        "if [ \"$status_r2\" -ne 0 ]; then strict_pass=false; if [ \"$exit_code\" -eq 0 ]; then exit_code=$status_r2; fi; fi"
            .to_string(),
    );
    let report_format = format!(
        "{{\"schema_version\":\"bijux.fastq.validate.report.v1\",\"stage\":{},\"stage_id\":{},\"tool_id\":{},\"input_r1\":%s,\"input_r2\":%s,\"validation_log_r1\":%s,\"validation_log_r2\":%s,\"validated_inputs\":{},\"strict_pass\":%s,\"exit_code\":%s}}",
        json_string_literal(STAGE_ID.as_str()),
        json_string_literal(STAGE_ID.as_str()),
        json_string_literal(tool.tool_id.as_str()),
        if r2.is_some() { 2 } else { 1 },
    );
    let lineage_payload = serde_json::json!({
        "schema_version": "bijux.fastq.validate.lineage.v1",
        "stage_id": STAGE_ID.as_str(),
        "tool_id": tool.tool_id.as_str(),
        "input_r1": r1,
        "input_r2": r2,
        "validation_report": report_path,
        "q_cutoff": options.q_cutoff,
        "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
        "validated_stream_ids": if r2.is_some() {
            vec!["reads_r1", "reads_r2"]
        } else {
            vec!["reads_r1"]
        },
    });
    commands.push(format!(
        "printf '%s\\n' {} > {}",
        shell_quote_str(&lineage_payload.to_string()),
        shell_quote(validated_reads_manifest),
    ));
    commands.push(format!(
        "printf '{}' {} {} {} {} \"$strict_pass\" \"$exit_code\" > {}",
        escape_printf_format(&report_format),
        shell_quote_str(&json_path_token(r1)?),
        shell_quote_str(&json_optional_path_token(r2)?),
        shell_quote_str(&json_path_token(&r1_log)?),
        shell_quote_str(&json_optional_path_token(r2_log.as_deref())?),
        shell_quote(report_path),
    ));
    commands.push(format!(
        "exit \"$exit_code\""
    ));
    Ok(vec![
        "sh".to_string(),
        "-lc".to_string(),
        commands.join(" && "),
    ])
}

fn json_string_literal(value: &str) -> String {
    serde_json::to_string(value).expect("serialize static json string literal")
}

fn json_path_token(path: &Path) -> Result<String> {
    serde_json::to_string(&path.display().to_string())
        .map_err(|error| anyhow!("serialize path token for validation report: {error}"))
}

fn json_optional_path_token(path: Option<&Path>) -> Result<String> {
    serde_json::to_string(&path.map(|value| value.display().to_string()))
        .map_err(|error| anyhow!("serialize optional path token for validation report: {error}"))
}

fn escape_printf_format(value: &str) -> String {
    value.replace('%', "%%")
}

fn shell_quote(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolId};

    fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id.to_string()),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: match tool_id {
                    "fastqvalidator" => vec![
                        "fastqvalidator".to_string(),
                        "--file".to_string(),
                        "{{reads_r1}}".to_string(),
                    ],
                    "seqtk" => vec![
                        "seqtk".to_string(),
                        "seq".to_string(),
                        "{{reads_r1}}".to_string(),
                    ],
                    "fqtools" => vec![
                        "fqtools".to_string(),
                        "validate".to_string(),
                        "{{reads_r1}}".to_string(),
                    ],
                    _ => vec![tool_id.to_string(), "{{reads_r1}}".to_string()],
                },
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn resolve_config_preserves_optional_r2() {
        let config = resolve_config(ValidateReadsUserConfig {
            tool: "fastqvalidator".to_string(),
            r1: "reads_R1.fastq.gz".into(),
            r2: Some("reads_R2.fastq.gz".into()),
            out_dir: "out".into(),
        });

        assert_eq!(
            config.r2.as_deref(),
            Some(std::path::Path::new("reads_R2.fastq.gz"))
        );
    }

    #[test]
    fn plan_from_config_keeps_paired_validation_inputs() -> Result<()> {
        let config = resolve_config(ValidateReadsUserConfig {
            tool: "fastqvalidator".to_string(),
            r1: "reads_R1.fastq.gz".into(),
            r2: Some("reads_R2.fastq.gz".into()),
            out_dir: "out".into(),
        });

        let plan = plan_from_config(&dummy_tool("fastqvalidator"), &config)?;
        assert_eq!(plan.io.inputs.len(), 2);
        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        assert!(plan.command.template[2].contains("reads_R1.fastq.gz"));
        assert!(plan.command.template[2].contains("reads_R2.fastq.gz"));
        assert!(plan.command.template[2].contains("validation_r1.log"));
        assert!(plan.command.template[2].contains("validation_r2.log"));
        assert!(plan.command.template[2].contains("\"validated_inputs\":2"));
        assert!(
            plan.command.template[2].contains("\"schema_version\":\"bijux.fastq.validate.lineage.v1\"")
        );
        assert_eq!(
            plan.params["report_json"],
            serde_json::json!("out/validation.json")
        );
        assert_eq!(
            plan.params["validated_reads_manifest"],
            serde_json::json!("out/validated_reads_manifest.json")
        );
        assert_eq!(plan.io.outputs.len(), 2);
        Ok(())
    }

    #[test]
    fn plan_writes_governed_validation_report_for_seqtk() -> Result<()> {
        let plan = plan(
            &dummy_tool("seqtk"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
        )?;

        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("'seqtk' 'seq' 'reads.fastq.gz' > 'out/validation_r1.log' 2>&1"));
        assert!(script.contains("out/validation.json"));
        assert!(script.contains("out/validated_reads_manifest.json"));
        assert!(script.contains("\"tool_id\":\"seqtk\""));
        assert!(script.contains("\"validated_inputs\":1"));
        assert!(script.contains("\"stage\":\"fastq.validate_reads\""));
        assert!(script.contains("\"$strict_pass\""));
        assert!(script.contains("\"$exit_code\""));
        Ok(())
    }

    #[test]
    fn plan_validation_report_tracks_runtime_exit_code_instead_of_placeholder_success() -> Result<()> {
        let plan = plan(
            &dummy_tool("fastqvalidator"),
            std::path::Path::new("reads.fastq.gz"),
            Some(std::path::Path::new("reads_r2.fastq.gz")),
            std::path::Path::new("out"),
        )?;

        let script = &plan.command.template[2];
        assert!(script.contains("status_r1=$?"));
        assert!(script.contains("status_r2=$?"));
        assert!(script.contains("exit_code=$status_r1"));
        assert!(script.contains("exit \"$exit_code\""));
        assert!(!script.contains("\"strict_pass\":true"));
        Ok(())
    }

    #[test]
    fn plan_with_options_propagates_quality_cutoff_into_effective_params() -> Result<()> {
        let error = plan_with_options(
            &dummy_tool("fastqvalidator"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
            &ValidatePlanOptions { q_cutoff: Some(25) },
        )
        .expect_err("q_cutoff must be rejected until backend-native validation support exists");

        assert!(error
            .to_string()
            .contains("does not yet map q_cutoff into a governed backend-native validation contract"));
        Ok(())
    }

    #[test]
    fn validation_quality_cutoff_is_rejected_for_seqtk_until_backend_support_exists() {
        let error = plan_with_options(
            &dummy_tool("seqtk"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
            &ValidatePlanOptions { q_cutoff: Some(20) },
        )
        .expect_err("seqtk validate_reads should reject q_cutoff until validation semantics are real");

        assert!(error.to_string().contains("seqtk"));
        assert!(error.to_string().contains("q_cutoff"));
    }

    #[test]
    fn validation_quality_cutoff_is_rejected_for_fqtools_until_backend_support_exists() {
        let error = plan_with_options(
            &dummy_tool("fqtools"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
            &ValidatePlanOptions { q_cutoff: Some(20) },
        )
        .expect_err(
            "fqtools validate_reads should reject q_cutoff until validation semantics are real",
        );

        assert!(error.to_string().contains("fqtools"));
        assert!(error.to_string().contains("q_cutoff"));
    }
}

fn shell_join(command: &[String]) -> String {
    command
        .iter()
        .map(|part| shell_quote_str(part))
        .collect::<Vec<_>>()
        .join(" ")
}
