#![allow(clippy::too_many_arguments)]

use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{
        RrnaEffectiveParams, RrnaReportFormat, RrnaScreeningEngine, RRNA_DEPLETION_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_RRNA;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type DepleteRrnaPlanOptions = crate::DepleteRrnaStageParams;

fn rrna_database_artifact_id(rrna_db: &str) -> String {
    let path = Path::new(rrna_db);
    if path.components().count() > 1 {
        return path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or(rrna_db)
            .to_string();
    }
    rrna_db.to_string()
}

/// # Errors
/// Returns an error if any requested rRNA depletion tool is not admitted for
/// `fastq.deplete_rrna`.
pub fn normalize_rrna_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

/// Build an rRNA screening plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_rrna(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_rrna_with_options(tool, r1, r2, out_dir, &DepleteRrnaPlanOptions::baseline())
}

/// Build an rRNA screening plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_rrna_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &DepleteRrnaPlanOptions,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_rrna_tool_list(std::slice::from_ref(&tool_id))?;
    if options.rrna_db.trim().is_empty() {
        return Err(anyhow!("rrna_db must be provided for {}", tool.tool_id));
    }
    if (options.min_identity - 0.95).abs() > f64::EPSILON {
        return Err(anyhow!(
            "sortmerna does not support governed min_identity overrides; requested {}",
            options.min_identity
        ));
    }
    let filtered_reads_r1 = if r2.is_some() {
        out_dir.join("rrna_filtered_R1.fastq.gz")
    } else {
        out_dir.join("rrna_filtered.fastq.gz")
    };
    let filtered_reads_r2 = r2.map(|_| out_dir.join("rrna_filtered_R2.fastq.gz"));
    let report = out_dir.join("rrna_report.tsv");
    let metrics = out_dir.join("rrna_report.json");
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let database_artifact_id = rrna_database_artifact_id(&options.rrna_db);
    let effective_params = RrnaEffectiveParams {
        schema_version: RRNA_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_threads,
        contaminant_db: Some(options.rrna_db.clone()),
        database_artifact_id: database_artifact_id.clone(),
        database_build_id: None,
        screening_engine: RrnaScreeningEngine::Sortmerna,
        report_format: RrnaReportFormat::SummaryTsvAndJson,
        emit_removed_reads: false,
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
    inputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_reference"),
        Path::new(&options.rrna_db).to_path_buf(),
        ArtifactRole::Reference,
    ));
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("rrna_filtered_reads_r1"),
        filtered_reads_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(filtered_reads_r2) = &filtered_reads_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("rrna_filtered_reads_r2"),
            filtered_reads_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_report_tsv"),
        report.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_report_json"),
        metrics.clone(),
        ArtifactRole::MetricsJson,
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
        command: CommandSpecV1 {
            template: rrna_command(
                &tool.tool_id.0,
                r1,
                r2,
                out_dir,
                &filtered_reads_r1,
                filtered_reads_r2.as_deref(),
                &report,
                &metrics,
                effective_threads,
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
            "rrna_db": options.rrna_db,
            "database_artifact_id": database_artifact_id,
            "min_identity": options.min_identity,
            "threads": effective_threads,
            "filtered_reads_r1": filtered_reads_r1,
            "filtered_reads_r2": filtered_reads_r2,
            "report_tsv": report,
            "report_json": metrics
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize rrna effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn rrna_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    filtered_reads_r1: &Path,
    filtered_reads_r2: Option<&Path>,
    report_tsv: &Path,
    report_json: &Path,
    threads: u32,
    options: &DepleteRrnaPlanOptions,
) -> Result<Vec<String>> {
    match tool_id {
        "sortmerna" => {
            let work_dir = out_dir.join("sortmerna_workdir");
            let idx_dir = work_dir.join("idx");
            let kvdb_dir = work_dir.join("kvdb");
            let readb_dir = work_dir.join("readb");
            let out_subdir = work_dir.join("out");
            let other_prefix = out_subdir.join("other");
            let mut command = vec![
                "sortmerna".to_string(),
                "--ref".to_string(),
                options.rrna_db.clone(),
                "--reads".to_string(),
                r1.display().to_string(),
                "--workdir".to_string(),
                format!("{}/", work_dir.display()),
                "--idx-dir".to_string(),
                idx_dir.display().to_string(),
                "--kvdb".to_string(),
                kvdb_dir.display().to_string(),
                "--readb".to_string(),
                readb_dir.display().to_string(),
                "--threads".to_string(),
                threads.to_string(),
                "--fastx".to_string(),
                "--zip-out".to_string(),
                "yes".to_string(),
            ];
            if let Some(r2) = r2 {
                command.push("--reads".to_string());
                command.push(r2.display().to_string());
            }
            let single_output_globs = format!(
                "{} {} {} {}",
                shell_quote_str(&format!("{}*.fastq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*.fq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*.fastq", other_prefix.display())),
                shell_quote_str(&format!("{}*.fq", other_prefix.display())),
            );
            let paired_fwd_globs = format!(
                "{} {} {} {} {} {} {} {} {} {} {} {}",
                shell_quote_str(&format!("{}*paired*fwd*.fastq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*paired*fwd*.fq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*paired*fwd*.fastq", other_prefix.display())),
                shell_quote_str(&format!("{}*paired*fwd*.fq", other_prefix.display())),
                shell_quote_str(&format!("{}*fwd*.fastq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*fwd*.fq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*fwd*.fastq", other_prefix.display())),
                shell_quote_str(&format!("{}*fwd*.fq", other_prefix.display())),
                shell_quote_str(&format!("{}/fwd_*.fastq.gz", readb_dir.display())),
                shell_quote_str(&format!("{}/fwd_*.fq.gz", readb_dir.display())),
                shell_quote_str(&format!("{}/fwd_*.fastq", readb_dir.display())),
                shell_quote_str(&format!("{}/fwd_*.fq", readb_dir.display())),
            );
            let paired_rev_globs = format!(
                "{} {} {} {} {} {} {} {} {} {} {} {}",
                shell_quote_str(&format!("{}*paired*rev*.fastq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*paired*rev*.fq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*paired*rev*.fastq", other_prefix.display())),
                shell_quote_str(&format!("{}*paired*rev*.fq", other_prefix.display())),
                shell_quote_str(&format!("{}*rev*.fastq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*rev*.fq.gz", other_prefix.display())),
                shell_quote_str(&format!("{}*rev*.fastq", other_prefix.display())),
                shell_quote_str(&format!("{}*rev*.fq", other_prefix.display())),
                shell_quote_str(&format!("{}/rev_*.fastq.gz", readb_dir.display())),
                shell_quote_str(&format!("{}/rev_*.fq.gz", readb_dir.display())),
                shell_quote_str(&format!("{}/rev_*.fastq", readb_dir.display())),
                shell_quote_str(&format!("{}/rev_*.fq", readb_dir.display())),
            );
            let readb_single_globs = format!(
                "{} {} {} {}",
                shell_quote_str(&format!("{}/fwd_*.fastq.gz", readb_dir.display())),
                shell_quote_str(&format!("{}/fwd_*.fq.gz", readb_dir.display())),
                shell_quote_str(&format!("{}/fwd_*.fastq", readb_dir.display())),
                shell_quote_str(&format!("{}/fwd_*.fq", readb_dir.display())),
            );
            let script = format!(
                "set -euo pipefail\nshopt -s nullglob\ncollect_output_from_globs() {{ dest=\"$1\"; shift; local pattern candidate matches=(); local -a inputs=(); for pattern in \"$@\"; do matches=( $pattern ); for candidate in \"${{matches[@]}}\"; do if [ -f \"$candidate\" ]; then inputs+=( \"$candidate\" ); fi; done; done; if [ \"${{#inputs[@]}}\" -eq 0 ]; then printf 'missing expected SortMeRNA output for %s\\n' \"$dest\" >&2; return 1; fi; {{ for candidate in \"${{inputs[@]}}\"; do case \"$candidate\" in *.gz) gzip -cd -- \"$candidate\" ;; *) cat -- \"$candidate\" ;; esac; done; }} | gzip -c > \"$dest\"; }}\nrm -rf {kvdb_dir} {readb_dir} {out_subdir}\nmkdir -p {work_dir} {idx_dir} {kvdb_dir} {readb_dir} {out_subdir}\nmkdir -p \"$(dirname {filtered_reads_r1})\" \"$(dirname {report_tsv})\" \"$(dirname {report_json})\"\nrm -f {filtered_reads_r1} {filtered_reads_r2_cleanup} {report_tsv} {report_json}\n{sortmerna_command}\n",
                kvdb_dir = shell_quote_path(&kvdb_dir),
                readb_dir = shell_quote_path(&readb_dir),
                out_subdir = shell_quote_path(&out_subdir),
                work_dir = shell_quote_path(&work_dir),
                idx_dir = shell_quote_path(&idx_dir),
                filtered_reads_r1 = shell_quote_path(filtered_reads_r1),
                filtered_reads_r2_cleanup = filtered_reads_r2
                    .map_or_else(|| "''".to_string(), shell_quote_path),
                report_tsv = shell_quote_path(report_tsv),
                report_json = shell_quote_path(report_json),
                sortmerna_command = shell_join(&command),
            );
            let script = if let Some(filtered_reads_r2) = filtered_reads_r2 {
                format!(
                    "{script}collect_output_from_globs {filtered_reads_r1} {paired_fwd_globs}\ncollect_output_from_globs {filtered_reads_r2} {paired_rev_globs}\n",
                    filtered_reads_r1 = shell_quote_path(filtered_reads_r1),
                    paired_fwd_globs = paired_fwd_globs,
                    filtered_reads_r2 = shell_quote_path(filtered_reads_r2),
                    paired_rev_globs = paired_rev_globs,
                )
            } else {
                format!(
                    "{script}collect_output_from_globs {filtered_reads_r1} {single_output_globs} {readb_single_globs}\n",
                    filtered_reads_r1 = shell_quote_path(filtered_reads_r1),
                    single_output_globs = single_output_globs,
                    readb_single_globs = readb_single_globs,
                )
            };
            let script = format!(
                "{script}if [ -f {aligned_log} ]; then cp -- {aligned_log} {report_tsv}; else : > {report_tsv}; fi\nprintf '{{}}\\n' > {report_json}\n",
                aligned_log = shell_quote_path(&out_subdir.join("aligned.log")),
                report_tsv = shell_quote_path(report_tsv),
                report_json = shell_quote_path(report_json),
            );
            Ok(vec!["bash".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!("unsupported tool {tool_id}")),
    }
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_str(part)).collect::<Vec<_>>().join(" ")
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
