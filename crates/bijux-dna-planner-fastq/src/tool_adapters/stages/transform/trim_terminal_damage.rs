#![allow(clippy::format_push_string, clippy::too_many_arguments, clippy::uninlined_format_args)]

use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::trim::{
    resolve_terminal_damage_policy_with_override, TrimTerminalDamageParams,
    TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE;
use bijux_dna_domain_fastq::{TerminalDamageReportV1, TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION};
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_TRIM_TERMINAL_DAMAGE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type TrimTerminalDamagePlanOptions = crate::TrimTerminalDamageStageParams;

struct TrimTerminalDamagePaths {
    output_r1: std::path::PathBuf,
    output_r2: Option<std::path::PathBuf>,
    report: std::path::PathBuf,
    raw_backend_report: Option<std::path::PathBuf>,
}

fn output_name(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "adapterremoval" => Some("trim_terminal_damage.adapterremoval.fastq.gz"),
        "cutadapt" => Some("trim_terminal_damage.cutadapt.fastq.gz"),
        "seqkit" => Some("trim_terminal_damage.seqkit.fastq.gz"),
        _ => None,
    }
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_terminal_damage`.
pub fn plan_trim_terminal_damage(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    damage_mode: &str,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
) -> Result<StagePlanV1> {
    let damage_mode = damage_mode.parse::<DamageMode>().map_err(|error| {
        anyhow!("invalid fastq.trim_terminal_damage damage_mode `{damage_mode}`: {error}")
    })?;
    plan_trim_terminal_damage_with_options(
        tool,
        r1,
        r2,
        out_dir,
        &TrimTerminalDamagePlanOptions {
            threads: None,
            damage_mode,
            execution_policy: None,
            trim_5p_bases,
            trim_3p_bases,
        },
    )
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_terminal_damage`.
pub fn plan_trim_terminal_damage_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &TrimTerminalDamagePlanOptions,
) -> Result<StagePlanV1> {
    let resolved_policy = resolve_terminal_damage_policy_with_override(
        options.damage_mode,
        options.trim_5p_bases,
        options.trim_3p_bases,
        options.execution_policy,
    )?;
    let out_name = output_name(tool.tool_id.as_str())
        .ok_or_else(|| anyhow!("unsupported trim_terminal_damage tool {}", tool.tool_id))?;
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let paths = trim_terminal_damage_paths(tool.tool_id.as_str(), out_name, r2.is_some(), out_dir);
    let command_template = trim_terminal_damage_command(
        &tool.tool_id.0,
        r1,
        r2,
        &paths.output_r1,
        paths.output_r2.as_deref(),
        &paths.report,
        paths.raw_backend_report.as_deref(),
        effective_threads,
        options.damage_mode,
        resolved_policy.execution_policy,
        resolved_policy.effective_trim_5p_bases,
        resolved_policy.effective_trim_3p_bases,
        resolved_policy.requested_trim_5p_bases,
        resolved_policy.requested_trim_3p_bases,
    )?;
    let effective_params = trim_terminal_damage_effective_params(
        options,
        r2.is_some(),
        effective_threads,
        resolved_policy,
    );
    let inputs = trim_terminal_damage_inputs(r1, r2);
    let outputs = trim_terminal_damage_outputs(&paths);
    let mut resources = tool.resources.clone();
    resources.threads = effective_threads;
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
        command: CommandSpecV1 { template: command_template },
        resources,
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: trim_terminal_damage_params(&tool.tool_id.0, r1, r2, &paths, effective_threads),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "damage-aware terminal trimming"),
    })
}

fn trim_terminal_damage_paths(
    tool_id: &str,
    out_name: &str,
    paired: bool,
    out_dir: &Path,
) -> TrimTerminalDamagePaths {
    let output_r1 =
        if paired { out_dir.join(format!("R1.{out_name}")) } else { out_dir.join(out_name) };
    TrimTerminalDamagePaths {
        output_r1,
        output_r2: paired.then(|| out_dir.join(format!("R2.{out_name}"))),
        report: out_dir.join("trim_terminal_damage_report.json"),
        raw_backend_report: match tool_id {
            "cutadapt" => Some(out_dir.join("trim_terminal_damage.cutadapt.raw.json")),
            _ => None,
        },
    }
}

fn trim_terminal_damage_effective_params(
    options: &TrimTerminalDamagePlanOptions,
    paired: bool,
    effective_threads: u32,
    resolved_policy: bijux_dna_domain_fastq::params::trim::ResolvedTerminalDamagePolicy,
) -> TrimTerminalDamageParams {
    TrimTerminalDamageParams {
        schema_version: TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(paired),
        threads: effective_threads,
        damage_mode: options.damage_mode,
        execution_policy: resolved_policy.execution_policy,
        trim_5p_bases: resolved_policy.effective_trim_5p_bases,
        trim_3p_bases: resolved_policy.effective_trim_3p_bases,
        requested_trim_5p_bases: Some(resolved_policy.requested_trim_5p_bases),
        requested_trim_3p_bases: Some(resolved_policy.requested_trim_3p_bases),
    }
}

fn trim_terminal_damage_inputs(r1: &Path, r2: Option<&Path>) -> Vec<ArtifactRef> {
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
    inputs
}

fn trim_terminal_damage_outputs(paths: &TrimTerminalDamagePaths) -> Vec<ArtifactRef> {
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("trimmed_reads_r1"),
        paths.output_r1.clone(),
        ArtifactRole::TrimmedReads,
    )];
    if let Some(output_r2) = &paths.output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("trimmed_reads_r2"),
            output_r2.clone(),
            ArtifactRole::TrimmedReads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        paths.report.clone(),
        ArtifactRole::ReportJson,
    ));
    if let Some(raw_backend_report) = &paths.raw_backend_report {
        outputs.push(ArtifactRef::optional(
            ArtifactId::from_static("raw_backend_report_json"),
            raw_backend_report.clone(),
            ArtifactRole::ReportJson,
        ));
    }
    outputs
}

fn trim_terminal_damage_params(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    paths: &TrimTerminalDamagePaths,
    effective_threads: u32,
) -> serde_json::Value {
    serde_json::json!({
        "tool": tool_id,
        "input_r1": r1,
        "input_r2": r2,
        "output_r1": paths.output_r1,
        "output_r2": paths.output_r2,
        "report_json": paths.report,
        "raw_backend_report": paths.raw_backend_report,
        "threads": effective_threads,
    })
}

fn trim_terminal_damage_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report: &Path,
    raw_backend_report: Option<&Path>,
    threads: u32,
    damage_mode: DamageMode,
    execution_policy: bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
    requested_trim_5p_bases: u32,
    requested_trim_3p_bases: u32,
) -> Result<Vec<String>> {
    let governed_report = build_governed_terminal_damage_report(
        tool_id,
        r1,
        r2,
        output_r1,
        output_r2,
        raw_backend_report,
        threads,
        damage_mode,
        execution_policy,
        trim_5p_bases,
        trim_3p_bases,
        requested_trim_5p_bases,
        requested_trim_3p_bases,
    )?;
    match tool_id {
        "adapterremoval" => {
            let mut script = format!(
                "set -eu\nadapterremoval --threads {threads} --trim5p {trim_5p_bases} --trim3p {trim_3p_bases} --file1 {} --output1 {}",
                shell_quote_path(r1),
                shell_quote_path(output_r1),
            );
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                script.push_str(&format!(
                    " --file2 {} --output2 {} --singleton /dev/null",
                    shell_quote_path(r2),
                    shell_quote_path(output_r2),
                ));
            }
            script.push_str(" --discarded /dev/null");
            script.push('\n');
            script.push_str(&format!(
                "printf '%s\\n' {} > {}\n",
                shell_quote_str(&governed_report),
                shell_quote_path(report),
            ));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        "cutadapt" => {
            let raw_backend_report = raw_backend_report.ok_or_else(|| {
                anyhow!("cutadapt terminal-damage planning requires raw backend report path")
            })?;
            let mut script = format!("set -eu\ncutadapt --cores {threads}");
            if trim_5p_bases > 0 {
                script.push_str(&format!(" -u {}", trim_5p_bases));
            }
            if trim_3p_bases > 0 {
                script.push_str(&format!(" -u -{trim_3p_bases}"));
            }
            script.push_str(&format!(
                " --json {} -o {}",
                shell_quote_path(raw_backend_report),
                shell_quote_path(output_r1),
            ));
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                script.push_str(&format!(
                    " -p {} {} {}",
                    shell_quote_path(output_r2),
                    shell_quote_path(r1),
                    shell_quote_path(r2),
                ));
            } else {
                script.push_str(&format!(" {}", shell_quote_path(r1)));
            }
            script.push('\n');
            script.push_str(&format!(
                "printf '%s\\n' {} > {}\n",
                shell_quote_str(&governed_report),
                shell_quote_path(report),
            ));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        "seqkit" => {
            let region = terminal_trim_region(trim_5p_bases, trim_3p_bases);
            let mut script = format!(
                "set -eu\nseqkit subseq -j {threads} -r {} {} -o {}\n",
                shell_quote_str(&region),
                shell_quote_path(r1),
                shell_quote_path(output_r1),
            );
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                script.push_str(&format!(
                    "seqkit subseq -j {threads} -r {} {} -o {}\n",
                    shell_quote_str(&region),
                    shell_quote_path(r2),
                    shell_quote_path(output_r2),
                ));
            }
            script.push_str(&format!(
                "printf '%s\\n' {} > {}\n",
                shell_quote_str(&governed_report),
                shell_quote_path(report),
            ));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!("unsupported trim_terminal_damage tool for stage planning: {tool_id}")),
    }
}

fn build_governed_terminal_damage_report(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    raw_backend_report: Option<&Path>,
    threads: u32,
    damage_mode: DamageMode,
    execution_policy: bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
    requested_trim_5p_bases: u32,
    requested_trim_3p_bases: u32,
) -> Result<String> {
    let report = TerminalDamageReportV1 {
        schema_version: TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool_id.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads,
        damage_mode,
        execution_policy,
        trim_5p_bases,
        trim_3p_bases,
        requested_trim_5p_bases: Some(requested_trim_5p_bases),
        requested_trim_3p_bases: Some(requested_trim_3p_bases),
        udg_classification: damage_mode_default_udg_classification(damage_mode).to_string(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        reads_in: None,
        reads_out: None,
        bases_in: None,
        bases_out: None,
        mean_q_before: None,
        mean_q_after: None,
        ct_ga_asymmetry_pre: None,
        ct_ga_asymmetry_post: None,
        ct_ga_asymmetry_pre_r1: None,
        ct_ga_asymmetry_post_r1: None,
        ct_ga_asymmetry_pre_r2: None,
        ct_ga_asymmetry_post_r2: None,
        terminal_base_composition_pre_r1: None,
        terminal_base_composition_post_r1: None,
        terminal_base_composition_pre_r2: None,
        terminal_base_composition_post_r2: None,
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: match tool_id {
            "cutadapt" => Some("cutadapt_json".to_string()),
            "seqkit" => Some("seqkit_subseq".to_string()),
            _ => None,
        },
        runtime_s: None,
        memory_mb: None,
        used_fallback: false,
        backend_metrics: None,
    };
    serde_json::to_string(&report)
        .map_err(|error| anyhow!("serialize terminal damage governed report: {error}"))
}

fn damage_mode_default_udg_classification(damage_mode: DamageMode) -> &'static str {
    match damage_mode {
        DamageMode::Ancient => "non_udg",
        DamageMode::UdgTrimmed => "udg",
    }
}

fn terminal_trim_region(trim_5p_bases: u32, trim_3p_bases: u32) -> String {
    let start = trim_5p_bases.saturating_add(1);
    let end = if trim_3p_bases == 0 {
        "-1".to_string()
    } else {
        format!("-{}", trim_3p_bases.saturating_add(1))
    };
    format!("{start}:{end}")
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::{
        damage_mode_default_udg_classification, plan_trim_terminal_damage_with_options,
        TrimTerminalDamagePlanOptions,
    };
    use anyhow::Result;
    use bijux_dna_core::prelude::ToolExecutionSpecV1;
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolId};
    use bijux_dna_domain_fastq::params::DamageMode;

    fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id.to_string()),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn cutadapt_terminal_damage_plan_emits_governed_report_wrapper() -> Result<()> {
        let plan = plan_trim_terminal_damage_with_options(
            &dummy_tool("cutadapt"),
            std::path::Path::new("reads_R1.fastq.gz"),
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out"),
            &TrimTerminalDamagePlanOptions {
                threads: None,
                damage_mode: DamageMode::Ancient,
                execution_policy: None,
                trim_5p_bases: 2,
                trim_3p_bases: 1,
            },
        )?;

        let script = &plan.command.template[2];
        assert!(plan
            .io
            .outputs
            .iter()
            .any(|artifact| artifact.name.as_str() == "raw_backend_report_json"));
        assert!(script.contains("cutadapt --cores 1 -u 2 -u -1"));
        assert!(script.contains("out/trim_terminal_damage.cutadapt.raw.json"));
        assert!(script.contains("out/trim_terminal_damage_report.json"));
        assert!(
            script.contains("\"schema_version\":\"bijux.fastq.trim_terminal_damage.report.v2\"")
        );
        assert!(script.contains("\"raw_backend_report_format\":\"cutadapt_json\""));
        assert!(script.contains("\"udg_classification\":\"non_udg\""));
        assert!(script.contains("\"threads\":1"));
        assert!(script.contains("\"used_fallback\":false"));
        Ok(())
    }

    #[test]
    fn adapterremoval_terminal_damage_plan_emits_governed_report_contract() -> Result<()> {
        let plan = plan_trim_terminal_damage_with_options(
            &dummy_tool("adapterremoval"),
            std::path::Path::new("reads_R1.fastq.gz"),
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out"),
            &TrimTerminalDamagePlanOptions {
                threads: Some(3),
                damage_mode: DamageMode::Ancient,
                execution_policy: None,
                trim_5p_bases: 2,
                trim_3p_bases: 1,
            },
        )?;

        let script = &plan.command.template[2];
        assert!(!plan
            .io
            .outputs
            .iter()
            .any(|artifact| artifact.name.as_str() == "raw_backend_report_json"));
        assert!(script.contains("adapterremoval --threads 3 --trim5p 2 --trim3p 1"));
        assert!(script.contains("--file2 'reads_R2.fastq.gz' --output2 'out/R2.trim_terminal_damage.adapterremoval.fastq.gz'"));
        assert!(script.contains("--singleton /dev/null"));
        assert!(script.contains("--discarded /dev/null"));
        assert!(script.contains("\"tool_id\":\"adapterremoval\""));
        assert!(script.contains("\"raw_backend_report_format\":null"));
        assert!(script.contains("\"threads\":3"));
        assert!(script.contains("\"used_fallback\":false"));
        Ok(())
    }

    #[test]
    fn seqkit_terminal_damage_plan_emits_governed_report_contract() -> Result<()> {
        let plan = plan_trim_terminal_damage_with_options(
            &dummy_tool("seqkit"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
            &TrimTerminalDamagePlanOptions {
                threads: None,
                damage_mode: DamageMode::UdgTrimmed,
                execution_policy: None,
                trim_5p_bases: 2,
                trim_3p_bases: 2,
            },
        )?;

        let script = &plan.command.template[2];
        assert!(!plan
            .io
            .outputs
            .iter()
            .any(|artifact| artifact.name.as_str() == "raw_backend_report_json"));
        assert!(script.contains("seqkit subseq -j 1 -r '1:-1'"));
        assert!(script.contains("\"execution_policy\":\"preserve_udg_trimmed_ends\""));
        assert!(script.contains("\"raw_backend_report_format\":\"seqkit_subseq\""));
        assert!(script.contains("\"udg_classification\":\"udg\""));
        assert!(script.contains("\"threads\":1"));
        assert!(script.contains("\"used_fallback\":false"));
        Ok(())
    }

    #[test]
    fn ancient_damage_mode_defaults_to_non_udg_classification() {
        assert_eq!(damage_mode_default_udg_classification(DamageMode::Ancient), "non_udg");
        assert_eq!(damage_mode_default_udg_classification(DamageMode::UdgTrimmed), "udg");
    }

    #[test]
    fn terminal_damage_plan_honors_thread_override() -> Result<()> {
        let plan = plan_trim_terminal_damage_with_options(
            &dummy_tool("cutadapt"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
            &TrimTerminalDamagePlanOptions {
                threads: Some(6),
                damage_mode: DamageMode::Ancient,
                execution_policy: None,
                trim_5p_bases: 2,
                trim_3p_bases: 1,
            },
        )?;

        assert_eq!(plan.resources.threads, 6);
        assert_eq!(plan.effective_params["threads"], serde_json::json!(6));
        assert_eq!(plan.params["threads"], serde_json::json!(6));
        assert!(plan.command.template[2].contains("cutadapt --cores 6"));
        Ok(())
    }
}
