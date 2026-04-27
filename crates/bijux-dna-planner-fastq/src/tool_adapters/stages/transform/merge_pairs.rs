use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    merge::{MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION},
    PairedMode,
};
use bijux_dna_domain_fastq::{MERGE_PAIRS_REPORT_SCHEMA_VERSION, STAGE_MERGE_PAIRS};
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_MERGE_PAIRS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
const DEFAULT_MERGE_THREADS: u32 = 6;

#[derive(Debug, Clone)]
pub struct MergePlanOptions {
    pub threads: Option<u32>,
    pub merge_overlap: Option<u32>,
    pub min_length: Option<u32>,
    pub unmerged_read_policy: UnmergedReadPolicy,
}

impl Default for MergePlanOptions {
    fn default() -> Self {
        Self {
            threads: None,
            merge_overlap: None,
            min_length: None,
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        }
    }
}

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a merge plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_merge(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_merge_with_options(tool, r1, r2, out_dir, &MergePlanOptions::default())
}

pub fn plan_merge_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    options: &MergePlanOptions,
) -> Result<StagePlanV1> {
    validate_merge_options(&tool.tool_id.0, options)?;
    let outputs = merge_outputs(&tool.tool_id.0, out_dir)?;
    let merge_engine = merge_engine(&tool.tool_id.0)?;
    let effective_threads = options.threads.unwrap_or(DEFAULT_MERGE_THREADS).max(1);
    let effective_params = MergeEffectiveParams {
        schema_version: MERGE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: effective_threads,
        merge_overlap: options.merge_overlap,
        min_len: options.min_length,
        merge_engine,
        unmerged_read_policy: options.unmerged_read_policy.clone(),
    };
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
        command: CommandSpecV1 {
            template: merge_command_template(
                &tool.tool_id.0,
                r1,
                r2,
                out_dir,
                &outputs,
                tool,
                &effective_params,
            )?,
        },
        resources,
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r1"),
                    r1.to_path_buf(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r2"),
                    r2.to_path_buf(),
                    ArtifactRole::Reads,
                ),
            ],
            outputs: merge_artifacts(&outputs, &effective_params.unmerged_read_policy),
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "r1": r1,
            "r2": r2,
            "merged_reads": outputs.merged_reads,
            "unmerged_reads_r1": if effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
                outputs.unmerged_reads_r1.clone()
            } else {
                None
            },
            "unmerged_reads_r2": if effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
                outputs.unmerged_reads_r2.clone()
            } else {
                None
            },
            "threads": effective_params.threads,
            "merge_overlap": effective_params.merge_overlap,
            "min_length": effective_params.min_len,
            "unmerged_read_policy": effective_params.unmerged_read_policy,
            "report_json": outputs.report_json,
            "raw_backend_report_txt": outputs.raw_backend_report_txt
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize merge effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

#[derive(Debug)]
struct MergeOutputs {
    merged_reads: PathBuf,
    unmerged_reads_r1: Option<PathBuf>,
    unmerged_reads_r2: Option<PathBuf>,
    report_json: PathBuf,
    raw_backend_report_txt: Option<PathBuf>,
}

fn merge_outputs(tool: &str, out_dir: &Path) -> Result<MergeOutputs> {
    let report_json = out_dir.join("merge_report.json");
    let outputs = match tool {
        "adapterremoval" => MergeOutputs {
            merged_reads: out_dir.join("adapterremoval.collapsed.gz"),
            unmerged_reads_r1: Some(out_dir.join("adapterremoval.pair1.truncated.gz")),
            unmerged_reads_r2: Some(out_dir.join("adapterremoval.pair2.truncated.gz")),
            report_json,
            raw_backend_report_txt: Some(out_dir.join("adapterremoval.settings")),
        },
        "pear" => {
            let prefix = out_dir.join("pear");
            MergeOutputs {
                merged_reads: prefix.with_extension("assembled.fastq"),
                unmerged_reads_r1: Some(out_dir.join("pear.unassembled.forward.fastq")),
                unmerged_reads_r2: Some(out_dir.join("pear.unassembled.reverse.fastq")),
                report_json,
                raw_backend_report_txt: Some(out_dir.join("pear.log")),
            }
        }
        "vsearch" => MergeOutputs {
            merged_reads: out_dir.join("vsearch.merged.fastq"),
            unmerged_reads_r1: Some(out_dir.join("vsearch.unmerged_r1.fastq")),
            unmerged_reads_r2: Some(out_dir.join("vsearch.unmerged_r2.fastq")),
            report_json,
            raw_backend_report_txt: Some(out_dir.join("vsearch.log")),
        },
        "bbmerge" => MergeOutputs {
            merged_reads: out_dir.join("bbmerge.merged.fastq"),
            unmerged_reads_r1: Some(out_dir.join("bbmerge.unmerged_r1.fastq")),
            unmerged_reads_r2: Some(out_dir.join("bbmerge.unmerged_r2.fastq")),
            report_json,
            raw_backend_report_txt: Some(out_dir.join("bbmerge.log")),
        },
        "flash2" => MergeOutputs {
            merged_reads: out_dir.join("flash2.extendedFrags.fastq"),
            unmerged_reads_r1: Some(out_dir.join("flash2.notCombined_1.fastq")),
            unmerged_reads_r2: Some(out_dir.join("flash2.notCombined_2.fastq")),
            report_json,
            raw_backend_report_txt: Some(out_dir.join("flash2.log")),
        },
        "leehom" => MergeOutputs {
            merged_reads: out_dir.join("leehom.fq.gz"),
            unmerged_reads_r1: Some(out_dir.join("leehom_r1.fq.gz")),
            unmerged_reads_r2: Some(out_dir.join("leehom_r2.fq.gz")),
            report_json,
            raw_backend_report_txt: Some(out_dir.join("leehom.log")),
        },
        _ => return Err(anyhow!("unsupported merge tool")),
    };
    Ok(outputs)
}

fn merge_engine(tool: &str) -> Result<MergeEngine> {
    let engine = match tool {
        "adapterremoval" => MergeEngine::AdapterRemoval,
        "pear" => MergeEngine::Pear,
        "vsearch" => MergeEngine::Vsearch,
        "bbmerge" => MergeEngine::Bbmerge,
        "flash2" => MergeEngine::Flash2,
        "leehom" => MergeEngine::Leehom,
        _ => return Err(anyhow!("unsupported merge tool")),
    };
    Ok(engine)
}

fn merge_artifacts(
    outputs: &MergeOutputs,
    unmerged_policy: &UnmergedReadPolicy,
) -> Vec<ArtifactRef> {
    let mut artifacts = vec![ArtifactRef::required(
        ArtifactId::from_static("merged_reads"),
        outputs.merged_reads.clone(),
        ArtifactRole::Reads,
    )];
    if *unmerged_policy == UnmergedReadPolicy::EmitUnmergedPairs {
        if let Some(unmerged_reads_r1) = &outputs.unmerged_reads_r1 {
            artifacts.push(ArtifactRef::optional(
                ArtifactId::from_static("unmerged_reads_r1"),
                unmerged_reads_r1.clone(),
                ArtifactRole::Reads,
            ));
        }
        if let Some(unmerged_reads_r2) = &outputs.unmerged_reads_r2 {
            artifacts.push(ArtifactRef::optional(
                ArtifactId::from_static("unmerged_reads_r2"),
                unmerged_reads_r2.clone(),
                ArtifactRole::Reads,
            ));
        }
    }
    artifacts.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        outputs.report_json.clone(),
        ArtifactRole::MetricsJson,
    ));
    if let Some(raw_backend_report_txt) = &outputs.raw_backend_report_txt {
        artifacts.push(ArtifactRef::optional(
            ArtifactId::from_static("raw_backend_report_txt"),
            raw_backend_report_txt.clone(),
            ArtifactRole::Log,
        ));
    }
    artifacts
}

fn validate_merge_options(tool: &str, options: &MergePlanOptions) -> Result<()> {
    if options.min_length.is_some()
        && !matches!(tool, "adapterremoval" | "pear" | "vsearch" | "bbmerge")
    {
        return Err(anyhow!("merge planning does not yet map min_length for {tool}"));
    }
    if options.merge_overlap.is_some() && matches!(tool, "leehom") {
        return Err(anyhow!("merge planning does not yet map merge_overlap for {tool}"));
    }
    Ok(())
}

fn merge_command_template(
    tool: &str,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    outputs: &MergeOutputs,
    tool_spec: &ToolExecutionSpecV1,
    effective_params: &MergeEffectiveParams,
) -> Result<Vec<String>> {
    let base_command =
        base_merge_command(tool, r1, r2, out_dir, outputs, tool_spec, effective_params)?;
    let script = format!(
        "set -euo pipefail\ncount_fastq_reads() {{\n  local path=\"$1\"\n  if [ ! -f \"$path\" ]; then printf '0'; return; fi\n  case \"$path\" in\n    *.gz) gzip -dc \"$path\" ;;\n    *) cat \"$path\" ;;\n  esac | awk 'END {{ printf \"%d\", int(NR/4) }}'\n}}\n{base_command}\nreads_r1=$(count_fastq_reads {input_r1})\nreads_r2=$(count_fastq_reads {input_r2})\nreads_merged=$(count_fastq_reads {merged_reads})\npairs_in=$reads_r1\nif [ \"$reads_r2\" -lt \"$pairs_in\" ]; then pairs_in=$reads_r2; fi\nreads_unmerged=$(( pairs_in - reads_merged ))\nif [ \"$reads_unmerged\" -lt 0 ]; then reads_unmerged=0; fi\nif [ {emit_unmerged} = 1 ]; then\n  if [ -n {unmerged_r1_shell} ] && [ -n {unmerged_r2_shell} ]; then\n    reads_unmerged_r1=$(count_fastq_reads {unmerged_r1_shell})\n    reads_unmerged_r2=$(count_fastq_reads {unmerged_r2_shell})\n    reads_unmerged=$reads_unmerged_r1\n    if [ \"$reads_unmerged_r2\" -lt \"$reads_unmerged\" ]; then reads_unmerged=$reads_unmerged_r2; fi\n  fi\nelse\n  rm -f {cleanup_unmerged_r1} {cleanup_unmerged_r2}\nfi\nmerge_rate=$(awk -v merged=\"$reads_merged\" -v pairs=\"$pairs_in\" 'BEGIN {{ if (pairs > 0) printf \"%.6f\", merged / pairs; else printf \"0.000000\" }}')\ncat > {report_json_shell} <<EOF\n{{\n  \"schema_version\": {schema_version},\n  \"stage\": {stage_id},\n  \"stage_id\": {stage_id},\n  \"tool_id\": {tool_id},\n  \"paired_mode\": {paired_mode},\n  \"merge_engine\": {merge_engine},\n  \"threads\": {threads},\n  \"merge_overlap\": {merge_overlap},\n  \"min_len\": {min_len},\n  \"unmerged_read_policy\": {unmerged_policy},\n  \"input_r1\": {input_r1_json},\n  \"input_r2\": {input_r2_json},\n  \"merged_reads\": {merged_json},\n  \"unmerged_reads_r1\": {unmerged_r1_json},\n  \"unmerged_reads_r2\": {unmerged_r2_json},\n  \"reads_r1\": $reads_r1,\n  \"reads_r2\": $reads_r2,\n  \"reads_merged\": $reads_merged,\n  \"reads_unmerged\": $reads_unmerged,\n  \"merge_rate\": $merge_rate,\n  \"runtime_s\": null,\n  \"memory_mb\": null,\n  \"raw_backend_report\": {raw_backend_report_json},\n  \"raw_backend_report_format\": {raw_backend_report_format}\n}}\nEOF\n",
        base_command = base_command,
        input_r1 = shell_quote_path(r1),
        input_r2 = shell_quote_path(r2),
        merged_reads = shell_quote_path(&outputs.merged_reads),
        emit_unmerged = i32::from(
            effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs
        ),
        unmerged_r1_shell = outputs
            .unmerged_reads_r1
            .as_ref()
            .map_or_else(|| "''".to_string(), |path| shell_quote_path(path)),
        unmerged_r2_shell = outputs
            .unmerged_reads_r2
            .as_ref()
            .map_or_else(|| "''".to_string(), |path| shell_quote_path(path)),
        cleanup_unmerged_r1 = outputs
            .unmerged_reads_r1
            .as_ref()
            .map_or_else(|| "''".to_string(), |path| shell_quote_path(path)),
        cleanup_unmerged_r2 = outputs
            .unmerged_reads_r2
            .as_ref()
            .map_or_else(|| "''".to_string(), |path| shell_quote_path(path)),
        report_json_shell = shell_quote_path(&outputs.report_json),
        schema_version = json_literal(MERGE_PAIRS_REPORT_SCHEMA_VERSION)?,
        stage_id = json_literal(STAGE_ID.as_str())?,
        tool_id = json_literal(tool)?,
        paired_mode = serde_json::to_string(&effective_params.paired_mode)?,
        merge_engine = serde_json::to_string(&effective_params.merge_engine)?,
        threads = effective_params.threads,
        merge_overlap = option_json_u32(effective_params.merge_overlap),
        min_len = option_json_u32(effective_params.min_len),
        unmerged_policy = serde_json::to_string(&effective_params.unmerged_read_policy)?,
        input_r1_json = json_literal(&r1.display().to_string())?,
        input_r2_json = json_literal(&r2.display().to_string())?,
        merged_json = json_literal(&outputs.merged_reads.display().to_string())?,
        raw_backend_report_json = option_json_path_literal(outputs.raw_backend_report_txt.as_ref())?,
        raw_backend_report_format = option_json_string(merge_raw_backend_report_format(tool)),
        unmerged_r1_json = option_json_path_literal(if effective_params.unmerged_read_policy
            == UnmergedReadPolicy::EmitUnmergedPairs
        {
            outputs.unmerged_reads_r1.as_ref()
        } else {
            None
        })?,
        unmerged_r2_json = option_json_path_literal(if effective_params.unmerged_read_policy
            == UnmergedReadPolicy::EmitUnmergedPairs
        {
            outputs.unmerged_reads_r2.as_ref()
        } else {
            None
        })?,
    );
    Ok(vec!["bash".to_string(), "-lc".to_string(), script])
}

fn base_merge_command(
    tool: &str,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    outputs: &MergeOutputs,
    _tool_spec: &ToolExecutionSpecV1,
    effective_params: &MergeEffectiveParams,
) -> Result<String> {
    let command = match tool {
        "adapterremoval" => {
            let unmerged_r1 =
                if effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
                    outputs
                        .unmerged_reads_r1
                        .as_ref()
                        .ok_or_else(|| {
                            anyhow!("adapterremoval merge requires unmerged_reads_r1 output")
                        })?
                        .display()
                        .to_string()
                } else {
                    "/dev/null".to_string()
                };
            let unmerged_r2 =
                if effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
                    outputs
                        .unmerged_reads_r2
                        .as_ref()
                        .ok_or_else(|| {
                            anyhow!("adapterremoval merge requires unmerged_reads_r2 output")
                        })?
                        .display()
                        .to_string()
                } else {
                    "/dev/null".to_string()
                };
            let mut command = vec![
                "adapterremoval".to_string(),
                "--threads".to_string(),
                effective_params.threads.to_string(),
                "--file1".to_string(),
                r1.display().to_string(),
                "--file2".to_string(),
                r2.display().to_string(),
                "--collapse-deterministic".to_string(),
                "--gzip".to_string(),
                "--output1".to_string(),
                unmerged_r1,
                "--output2".to_string(),
                unmerged_r2,
                "--outputcollapsed".to_string(),
                outputs.merged_reads.display().to_string(),
                "--outputcollapsedtruncated".to_string(),
                "/dev/null".to_string(),
                "--settings".to_string(),
                outputs
                    .raw_backend_report_txt
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow!("adapterremoval merge requires raw backend settings output")
                    })?
                    .display()
                    .to_string(),
                "--singleton".to_string(),
                "/dev/null".to_string(),
                "--discarded".to_string(),
                "/dev/null".to_string(),
            ];
            if let Some(merge_overlap) = effective_params.merge_overlap {
                command.extend(["--minalignmentlength".to_string(), merge_overlap.to_string()]);
            }
            if let Some(min_len) = effective_params.min_len {
                command.extend(["--minlength".to_string(), min_len.to_string()]);
            }
            command
        }
        "pear" => {
            let prefix = out_dir.join("pear");
            let mut command = vec![
                "pear".to_string(),
                "-f".to_string(),
                r1.display().to_string(),
                "-r".to_string(),
                r2.display().to_string(),
                "-o".to_string(),
                prefix.display().to_string(),
                "-j".to_string(),
                effective_params.threads.to_string(),
            ];
            if let Some(merge_overlap) = effective_params.merge_overlap {
                command.extend(["-v".to_string(), merge_overlap.to_string()]);
            }
            if let Some(min_len) = effective_params.min_len {
                command.extend(["-n".to_string(), min_len.to_string()]);
            }
            command
        }
        "vsearch" => {
            let mut command = vec![
                "vsearch".to_string(),
                "--fastq_mergepairs".to_string(),
                r1.display().to_string(),
                "--reverse".to_string(),
                r2.display().to_string(),
                "--fastqout".to_string(),
                outputs.merged_reads.display().to_string(),
                "--threads".to_string(),
                effective_params.threads.to_string(),
            ];
            if effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
                command.extend([
                    "--fastqout_notmerged_fwd".to_string(),
                    outputs
                        .unmerged_reads_r1
                        .as_ref()
                        .ok_or_else(|| anyhow!("vsearch merge requires unmerged_reads_r1 output"))?
                        .display()
                        .to_string(),
                    "--fastqout_notmerged_rev".to_string(),
                    outputs
                        .unmerged_reads_r2
                        .as_ref()
                        .ok_or_else(|| anyhow!("vsearch merge requires unmerged_reads_r2 output"))?
                        .display()
                        .to_string(),
                ]);
            }
            if let Some(merge_overlap) = effective_params.merge_overlap {
                command.extend(["--fastq_minovlen".to_string(), merge_overlap.to_string()]);
            }
            if let Some(min_len) = effective_params.min_len {
                command.extend(["--fastq_minmergelen".to_string(), min_len.to_string()]);
            }
            command
        }
        "bbmerge" => {
            let mut command = vec![
                "bbmerge".to_string(),
                format!("in1={}", r1.display()),
                format!("in2={}", r2.display()),
                format!("out={}", outputs.merged_reads.display()),
                format!("threads={}", effective_params.threads),
            ];
            if effective_params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
                command.push(format!(
                    "outu1={}",
                    outputs
                        .unmerged_reads_r1
                        .as_ref()
                        .ok_or_else(|| anyhow!("bbmerge merge requires unmerged_reads_r1 output"))?
                        .display()
                ));
                command.push(format!(
                    "outu2={}",
                    outputs
                        .unmerged_reads_r2
                        .as_ref()
                        .ok_or_else(|| anyhow!("bbmerge merge requires unmerged_reads_r2 output"))?
                        .display()
                ));
            }
            if let Some(merge_overlap) = effective_params.merge_overlap {
                command.push(format!("minoverlap={merge_overlap}"));
            }
            if let Some(min_len) = effective_params.min_len {
                command.push(format!("mininsert={min_len}"));
            }
            command
        }
        "flash2" => {
            let mut command = vec![
                "flash2".to_string(),
                "-o".to_string(),
                "flash2".to_string(),
                "-d".to_string(),
                out_dir.display().to_string(),
                "-t".to_string(),
                effective_params.threads.to_string(),
                r1.display().to_string(),
                r2.display().to_string(),
            ];
            if let Some(merge_overlap) = effective_params.merge_overlap {
                command.extend(["-m".to_string(), merge_overlap.to_string()]);
            }
            command
        }
        "leehom" => vec![
            "leehom".to_string(),
            "-fq1".to_string(),
            r1.display().to_string(),
            "-fq2".to_string(),
            r2.display().to_string(),
            "-fqo".to_string(),
            out_dir.join("leehom").display().to_string(),
            "-t".to_string(),
            effective_params.threads.to_string(),
        ],
        _ => return Err(anyhow!("unsupported merge tool")),
    };
    let command =
        command.into_iter().map(|part| shell_quote_str(&part)).collect::<Vec<_>>().join(" ");
    if tool == "adapterremoval" {
        return Ok(command);
    }
    Ok(outputs
        .raw_backend_report_txt
        .as_ref()
        .map_or(command.clone(), |path| format!("{command} > {} 2>&1", shell_quote_path(path))))
}

fn option_json_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn option_json_string(value: Option<&str>) -> String {
    value.map_or_else(
        || "null".to_string(),
        |value| serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
    )
}

fn merge_raw_backend_report_format(tool: &str) -> Option<&'static str> {
    match tool {
        "adapterremoval" => Some("adapterremoval_settings"),
        "pear" => Some("pear_log"),
        "vsearch" => Some("vsearch_log"),
        "bbmerge" => Some("bbmerge_log"),
        "flash2" => Some("flash2_log"),
        "leehom" => Some("leehom_log"),
        _ => None,
    }
}

fn option_json_path_literal(path: Option<&PathBuf>) -> Result<String> {
    match path {
        Some(path) => json_literal(&path.display().to_string()),
        None => Ok("null".to_string()),
    }
}

fn json_literal(value: &str) -> Result<String> {
    serde_json::to_string(value).map_err(|error| anyhow!("serialize merge json literal: {error}"))
}

fn shell_quote_path(path: &Path) -> String {
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
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::{merge_command_template, merge_outputs, MergeEffectiveParams, MergeEngine};
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use bijux_dna_domain_fastq::params::{merge::UnmergedReadPolicy, PairedMode};
    use std::path::Path;

    #[test]
    fn adapterremoval_merge_redirects_collapsed_truncated_side_output() {
        let outputs =
            merge_outputs("adapterremoval", Path::new("out")).expect("adapterremoval outputs");
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("adapterremoval"),
            tool_version: "latest-pinned".to_string(),
            image: ContainerImageRefV1 {
                image: "docker.io/bijuxdna/adapterremoval:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 { template: Vec::new() },
            resources: ToolConstraints::default(),
        };
        let params = MergeEffectiveParams {
            schema_version: "bijux.fastq.merge.v1".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            merge_overlap: None,
            min_len: None,
            merge_engine: MergeEngine::AdapterRemoval,
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        };

        let command = merge_command_template(
            "adapterremoval",
            Path::new("sample_R1.fastq.gz"),
            Path::new("sample_R2.fastq.gz"),
            Path::new("out"),
            &outputs,
            &tool,
            &params,
        )
        .expect("adapterremoval command");

        let script = &command[2];
        assert!(script.contains("'--outputcollapsedtruncated' '/dev/null'"));
    }
}
