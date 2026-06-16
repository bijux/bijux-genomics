use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_stage_commands;
use super::local_stage_result_manifest::path_relative_to_repo;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const LOCAL_BAM_STAGE_SMOKE_SOURCE_PATH: &str =
    "crates/bijux-dna/src/commands/benchmark/local_bam_stage_smoke.rs";
const LOCAL_BAM_STAGE_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_bam_stage_smoke.v1";
const LOCAL_BAM_STAGE_SMOKE_COMMAND: &str = "bijux-dna bench local run-bam-stage-smoke";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalBamStageSmokeArtifact {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) artifact_path: String,
    pub(crate) artifact_format: String,
}

pub(crate) fn run_bam_stage_smoke(args: &parse::BenchLocalRunBamStageSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let artifact_path = run_local_bam_stage_smoke(&repo_root, &args.stage_id)?;
    if args.json {
        if artifact_path.extension().and_then(std::ffi::OsStr::to_str) == Some("json") {
            let raw = fs::read_to_string(&artifact_path)
                .with_context(|| format!("read {}", artifact_path.display()))?;
            let payload: serde_json::Value = serde_json::from_str(&raw)
                .with_context(|| format!("parse {}", artifact_path.display()))?;
            render::json::print_pretty(&payload)?;
        } else {
            render::json::print_pretty(&LocalBamStageSmokeArtifact {
                schema_version: LOCAL_BAM_STAGE_SMOKE_SCHEMA_VERSION,
                stage_id: args.stage_id.clone(),
                artifact_path: path_relative_to_repo(&repo_root, &artifact_path),
                artifact_format: artifact_format_label(&artifact_path).to_string(),
            })?;
        }
    } else {
        println!("{}", path_relative_to_repo(&repo_root, &artifact_path));
    }
    Ok(())
}

pub(crate) fn run_local_bam_stage_smoke(repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
    ensure_supported_bam_stage_smoke(stage_id)?;
    local_stage_commands::materialize_local_stage(repo_root, stage_id)
        .with_context(|| format!("materialize governed BAM local smoke for `{stage_id}`"))
}

pub(crate) fn bam_stage_smoke_command(stage_id: &str) -> Result<String> {
    ensure_supported_bam_stage_smoke(stage_id)?;
    Ok(format!("{LOCAL_BAM_STAGE_SMOKE_COMMAND} --stage-id {stage_id}"))
}

pub(crate) fn bam_stage_smoke_support_path(
    repo_root: &Path,
    stage_id: &str,
) -> Result<Option<String>> {
    if !supports_bam_stage_smoke(stage_id) {
        return Ok(None);
    }
    let absolute_path = repo_root.join(LOCAL_BAM_STAGE_SMOKE_SOURCE_PATH);
    if !absolute_path.is_file() {
        return Ok(None);
    }
    Ok(Some(path_relative_to_repo(repo_root, &absolute_path)))
}

pub(crate) fn governed_bam_local_smoke_tool_id(
    repo_root: &Path,
    stage_id: &str,
) -> Result<Option<String>> {
    if !supports_bam_stage_smoke(stage_id) {
        return Ok(None);
    }
    let suffix = stage_id.strip_prefix("bam.").ok_or_else(|| {
        anyhow!("BAM stage smoke expected a `bam.*` stage id, found `{stage_id}`")
    })?;
    let config_path =
        repo_root.join(format!("benchmarks/configs/local/bam-{}.toml", suffix.replace('_', "-")));
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let parsed: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    let tool_id = parsed
        .get("tool_id")
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("{} is missing governed `tool_id`", config_path.display()))?;
    Ok(Some(tool_id.to_string()))
}

pub(crate) fn supports_bam_stage_smoke(stage_id: &str) -> bool {
    match stage_id {
        "bam.authenticity"
        | "bam.complexity"
        | "bam.coverage"
        | "bam.damage"
        | "bam.duplication_metrics"
        | "bam.endogenous_content"
        | "bam.filter"
        | "bam.gc_bias"
        | "bam.insert_size"
        | "bam.length_filter"
        | "bam.mapping_summary"
        | "bam.mapq_filter"
        | "bam.markdup"
        | "bam.overlap_correction"
        | "bam.qc_pre"
        | "bam.recalibration"
        | "bam.sex"
        | "bam.validate" => true,
        "bam.bias_mitigation" | "bam.kinship" => cfg!(feature = "bam_downstream"),
        _ => false,
    }
}

pub(crate) fn has_bam_local_ready_only_contract(stage_id: &str) -> bool {
    matches!(stage_id, "bam.align" | "bam.contamination" | "bam.genotyping" | "bam.haplogroups")
}

fn ensure_supported_bam_stage_smoke(stage_id: &str) -> Result<()> {
    if supports_bam_stage_smoke(stage_id) {
        return Ok(());
    }
    if !stage_id.starts_with("bam.") {
        return Err(anyhow!(
            "BAM stage smoke wrapper expected a `bam.*` stage id, found `{stage_id}`"
        ));
    }
    if has_bam_local_ready_only_contract(stage_id) {
        return Err(anyhow!(
            "stage `{stage_id}` keeps governed local-ready plan coverage but has no BAM tiny-fixture smoke wrapper"
        ));
    }
    Err(anyhow!("stage `{stage_id}` has no governed BAM tiny-fixture smoke wrapper"))
}

fn artifact_format_label(path: &Path) -> &'static str {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("json") => "json",
        Some("tsv") => "tsv",
        _ => "artifact",
    }
}
