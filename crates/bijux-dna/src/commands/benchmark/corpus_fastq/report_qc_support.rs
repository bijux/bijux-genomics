use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::{
    benchmark_runtime_env, benchmark_sample_root, default_stage_out_root, stage_command_spec,
    CorpusNormalizedSample, StageSamplePreparation, REPORT_QC_INPUTS_SCHEMA_VERSION,
};
use crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig;

#[derive(Debug, Clone, Copy)]
struct ReportQcContributorContract {
    stage_id: &'static str,
    candidate_tool_ids: &'static [&'static str],
    default_tool_id: &'static str,
    artifact_id: &'static str,
    artifact_role: &'static str,
    relative_path: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct ReportQcUpstreamStage {
    stage_id: &'static str,
    tool_id: &'static str,
    extra_args: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct ReportQcContributorArtifact {
    contract: ReportQcContributorContract,
    tool_id: String,
    path: PathBuf,
}

const REPORT_QC_CONTRIBUTORS: [ReportQcContributorContract; 6] = [
    ReportQcContributorContract {
        stage_id: "fastq.validate_reads",
        candidate_tool_ids: &["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
        default_tool_id: "fastqvalidator",
        artifact_id: "validation_report",
        artifact_role: "report_json",
        relative_path: "validation.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.validate_reads",
        candidate_tool_ids: &["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
        default_tool_id: "fastqvalidator",
        artifact_id: "validated_reads_manifest",
        artifact_role: "summary_json",
        relative_path: "validated_reads_manifest.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.detect_adapters",
        candidate_tool_ids: &["fastqc"],
        default_tool_id: "fastqc",
        artifact_id: "report_json",
        artifact_role: "report_json",
        relative_path: "adapter_report.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.detect_adapters",
        candidate_tool_ids: &["fastqc"],
        default_tool_id: "fastqc",
        artifact_id: "adapter_evidence_dir",
        artifact_role: "stage_report",
        relative_path: "fastqc",
    },
    ReportQcContributorContract {
        stage_id: "fastq.profile_reads",
        candidate_tool_ids: &["seqkit_stats", "seqkit", "seqfu"],
        default_tool_id: "seqkit_stats",
        artifact_id: "qc_json",
        artifact_role: "metrics_json",
        relative_path: "qc.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.profile_read_lengths",
        candidate_tool_ids: &["seqkit_stats"],
        default_tool_id: "seqkit_stats",
        artifact_id: "length_distribution_json",
        artifact_role: "metrics_json",
        relative_path: "length_distribution.json",
    },
];

const REPORT_QC_UPSTREAM_STAGES: [ReportQcUpstreamStage; 4] = [
    ReportQcUpstreamStage {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqvalidator",
        extra_args: &[],
    },
    ReportQcUpstreamStage {
        stage_id: "fastq.detect_adapters",
        tool_id: "fastqc",
        extra_args: &["--threads", "1"],
    },
    ReportQcUpstreamStage {
        stage_id: "fastq.profile_reads",
        tool_id: "seqkit_stats",
        extra_args: &["--threads", "1"],
    },
    ReportQcUpstreamStage {
        stage_id: "fastq.profile_read_lengths",
        tool_id: "seqkit_stats",
        extra_args: &["--threads", "1", "--histogram-bins", "100"],
    },
];

pub(super) fn prepare_report_qc_sample(
    program: &Path,
    repo_root: &Path,
    workspace_config: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    platform: &str,
    out_root: &Path,
    sample: &CorpusNormalizedSample,
    dry_run: bool,
) -> Result<StageSamplePreparation> {
    let artifacts =
        report_qc_required_contributor_artifacts(workspace_config, corpus_id, &sample.sample_id)?;
    let missing_stage_ids = artifacts
        .iter()
        .filter(|row| !row.path.exists())
        .map(|row| row.contract.stage_id)
        .collect::<BTreeSet<_>>();
    for stage_id in missing_stage_ids {
        ensure_report_qc_upstream_stage_outputs(
            program,
            repo_root,
            workspace_config,
            corpus_id,
            platform,
            sample,
            stage_id,
            dry_run,
        )?;
    }

    if !dry_run {
        let unresolved = artifacts
            .iter()
            .filter(|row| !row.path.exists())
            .map(|row| row.path.display().to_string())
            .collect::<Vec<_>>();
        if !unresolved.is_empty() {
            return Err(anyhow!(
                "report-qc governed input resolution failed for {}: missing {}",
                sample.sample_id,
                unresolved.join(", ")
            ));
        }
    }

    let raw_fastqc_dir = report_qc_contributor_artifact_path(
        workspace_config,
        corpus_id,
        &sample.sample_id,
        "fastq.detect_adapters",
        "fastqc",
        "fastqc",
    )?;
    let governed_manifest = report_qc_manifest_path(out_root, &sample.sample_id);
    if let Some(parent) = governed_manifest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::json!({
        "schema_version": REPORT_QC_INPUTS_SCHEMA_VERSION,
        "qc_inputs": artifacts
            .iter()
            .map(|row| serde_json::json!({
                "name": report_qc_artifact_name(row),
                "path": row.path.display().to_string(),
                "role": row.contract.artifact_role,
                "optional": false,
            }))
            .collect::<Vec<_>>(),
        "contributors": artifacts
            .iter()
            .map(|row| serde_json::json!({
                "contributor_id": report_qc_contributor_id(row),
                "stage_id": row.contract.stage_id,
                "tool_id": row.tool_id,
                "artifact_id": row.contract.artifact_id,
                "artifact_role": row.contract.artifact_role,
                "path": row.path.display().to_string(),
            }))
            .collect::<Vec<_>>(),
        "raw_fastqc_dir": raw_fastqc_dir.display().to_string(),
    });
    fs::write(&governed_manifest, format!("{}\n", serde_json::to_string_pretty(&payload)?))
        .with_context(|| format!("write {}", governed_manifest.display()))?;

    let extra_stage_args = vec![
        "--aggregation-engine".to_string(),
        "multiqc".to_string(),
        "--aggregation-scope".to_string(),
        "governed_qc_artifacts".to_string(),
        "--governed-qc-manifest".to_string(),
        governed_manifest.display().to_string(),
    ];

    let mut run_extra_fields = BTreeMap::new();
    run_extra_fields.insert(
        "governed_qc_manifest".to_string(),
        serde_json::Value::String(governed_manifest.display().to_string()),
    );
    run_extra_fields.insert(
        "governed_qc_input_count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(artifacts.len() as u64)),
    );

    Ok(StageSamplePreparation { extra_stage_args, run_extra_fields })
}

fn report_qc_manifest_path(out_root: &Path, sample_id: &str) -> PathBuf {
    out_root
        .join("bench")
        .join("report_qc")
        .join(sample_id)
        .join("governed_qc_inputs_manifest.json")
}

fn report_qc_required_contributor_artifacts(
    workspace_config: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    sample_id: &str,
) -> Result<Vec<ReportQcContributorArtifact>> {
    REPORT_QC_CONTRIBUTORS
        .iter()
        .copied()
        .map(|contract| {
            resolve_report_qc_contributor_artifact(workspace_config, corpus_id, sample_id, contract)
        })
        .collect()
}

fn resolve_report_qc_contributor_artifact(
    workspace_config: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    sample_id: &str,
    contract: ReportQcContributorContract,
) -> Result<ReportQcContributorArtifact> {
    for tool_id in contract.candidate_tool_ids {
        let path = report_qc_contributor_artifact_path(
            workspace_config,
            corpus_id,
            sample_id,
            contract.stage_id,
            tool_id,
            contract.relative_path,
        )?;
        if path.exists() {
            return Ok(ReportQcContributorArtifact {
                contract,
                tool_id: (*tool_id).to_string(),
                path,
            });
        }
    }

    let default_path = report_qc_contributor_artifact_path(
        workspace_config,
        corpus_id,
        sample_id,
        contract.stage_id,
        contract.default_tool_id,
        contract.relative_path,
    )?;
    Ok(ReportQcContributorArtifact {
        contract,
        tool_id: contract.default_tool_id.to_string(),
        path: default_path,
    })
}

fn report_qc_contributor_artifact_path(
    workspace_config: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    sample_id: &str,
    stage_id: &str,
    tool_id: &str,
    relative_path: &str,
) -> Result<PathBuf> {
    let stage_root = default_stage_out_root(workspace_config, corpus_id, stage_id)?;
    let stage_spec = stage_command_spec(stage_id)?;
    Ok(benchmark_sample_root(&stage_root, stage_spec.report_dir, sample_id)
        .join("tools")
        .join(tool_id)
        .join(relative_path))
}

fn ensure_report_qc_upstream_stage_outputs(
    program: &Path,
    repo_root: &Path,
    workspace_config: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    platform: &str,
    sample: &CorpusNormalizedSample,
    stage_id: &str,
    dry_run: bool,
) -> Result<()> {
    let upstream = report_qc_upstream_stage(stage_id)?;
    let stage_spec = stage_command_spec(upstream.stage_id)?;
    let out_root = default_stage_out_root(workspace_config, corpus_id, upstream.stage_id)?;
    fs::create_dir_all(&out_root).with_context(|| format!("create {}", out_root.display()))?;

    let mut command_args = vec![
        "--platform".to_string(),
        platform.to_string(),
        "bench".to_string(),
        "fastq".to_string(),
        stage_spec.bench_subcommand.to_string(),
        "--sample-id".to_string(),
        sample.sample_id.clone(),
        "--r1".to_string(),
        sample.r1.display().to_string(),
        "--out".to_string(),
        out_root.display().to_string(),
        "--tools".to_string(),
        upstream.tool_id.to_string(),
    ];
    if let Some(r2) = sample.r2.as_ref() {
        command_args.push("--r2".to_string());
        command_args.push(r2.display().to_string());
    }
    command_args.extend(upstream.extra_args.iter().copied().map(str::to_string));

    if dry_run {
        return Ok(());
    }

    let command_name =
        program.to_str().ok_or_else(|| anyhow!("benchmark executable path is not valid UTF-8"))?;
    let output = bijux_dna_api::v1::api::run::run_command_with_context(
        command_name,
        &command_args,
        Some(repo_root),
        Some(&benchmark_runtime_env(&out_root)),
    )
    .with_context(|| format!("run {}", command_args.join(" ")))?;
    if output.exit_code != 0 {
        return Err(anyhow!(
            "{} governed QC bootstrap failed for {} with exit code {}",
            upstream.stage_id,
            sample.sample_id,
            output.exit_code
        ));
    }
    Ok(())
}

fn report_qc_upstream_stage(stage_id: &str) -> Result<ReportQcUpstreamStage> {
    REPORT_QC_UPSTREAM_STAGES
        .iter()
        .copied()
        .find(|row| row.stage_id == stage_id)
        .ok_or_else(|| anyhow!("unsupported report-qc upstream stage `{stage_id}`"))
}

fn report_qc_contributor_id(artifact: &ReportQcContributorArtifact) -> String {
    format!("{}.{}", artifact.contract.stage_id, artifact.tool_id)
}

fn report_qc_artifact_name(artifact: &ReportQcContributorArtifact) -> String {
    format!(
        "{}.tool.{}.{}",
        artifact.contract.stage_id, artifact.tool_id, artifact.contract.artifact_id
    )
}

pub(super) fn report_qc_contributor_tool_ids() -> Vec<String> {
    REPORT_QC_CONTRIBUTORS
        .iter()
        .flat_map(|row| row.candidate_tool_ids.iter().copied())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn report_qc_upstream_stage_ids() -> Vec<String> {
    REPORT_QC_UPSTREAM_STAGES.iter().map(|row| row.stage_id.to_string()).collect()
}
