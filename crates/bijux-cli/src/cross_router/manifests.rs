use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_core::alignment::AlignmentBoundary;
use bijux_engine::api::hash_file_sha256;
use bijux_env_runtime::ReferenceRecord;
use bijux_pipelines::PipelineProfile;

use crate::cross_router::CROSS_STAGE_ID;
use crate::fastq_router::StageExecutionSummary;

pub fn write_alignment_boundary(out_dir: &Path, boundary: &AlignmentBoundary) -> Result<PathBuf> {
    let boundaries_dir = out_dir.join("run_artifacts").join("boundaries");
    fs::create_dir_all(&boundaries_dir).context("create boundaries dir")?;
    let path = boundaries_dir.join("alignment_boundary.json");
    fs::write(&path, serde_json::to_vec_pretty(boundary)?)
        .context("write alignment_boundary.json")?;
    Ok(path)
}

pub fn write_defaults_ledger(out_dir: &Path, profile: &PipelineProfile) -> Result<PathBuf> {
    let path = out_dir.join("defaults_ledger.json");
    let ledger = profile.defaults_ledger();
    fs::write(&path, serde_json::to_vec_pretty(&ledger)?).context("write defaults_ledger.json")?;
    Ok(path)
}

pub fn write_cross_run_manifest(
    out_dir: &Path,
    profile: &PipelineProfile,
    fastq_summary: &serde_json::Value,
    bam_runs: &[StageExecutionSummary],
    boundary_path: Option<&Path>,
    prepare_reference_path: Option<&Path>,
) -> Result<()> {
    let run_id = fastq_summary
        .get("run_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    let mut stages = Vec::new();
    if let Some(fastq_stages) = fastq_summary
        .get("stages")
        .and_then(serde_json::Value::as_array)
    {
        for stage in fastq_stages {
            let stage_id = stage
                .get("stage_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let tool_id = stage
                .get("tool_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            stages.push(serde_json::json!({
                "stage_id": stage_id,
                "tool_id": tool_id,
                "domain": "fastq",
                "artifacts": stage.get("artifacts").cloned().unwrap_or(serde_json::json!({})),
            }));
        }
    }

    if let Some(boundary_path) = boundary_path {
        stages.push(serde_json::json!({
            "stage_id": CROSS_STAGE_ID,
            "tool_id": "alignment_boundary",
            "domain": "cross",
            "artifacts": {
                "alignment_boundary": boundary_path,
            },
        }));
    }
    if let Some(reference_path) = prepare_reference_path {
        stages.push(serde_json::json!({
            "stage_id": "core.prepare_reference",
            "tool_id": "reference_registry",
            "domain": "cross",
            "artifacts": {
                "reference_manifest": reference_path,
            },
        }));
    }

    for entry in bam_runs {
        stages.push(serde_json::json!({
            "stage_id": entry.plan.stage_id.0,
            "tool_id": entry.plan.tool_id.0,
            "domain": "bam",
            "artifacts": {
                "out_dir": entry.plan.out_dir,
                "metrics": entry.plan.out_dir.join("run_artifacts").join("metrics.json"),
                "stage_report": entry.plan.out_dir.join("run_artifacts").join("stage_report.json"),
            },
        }));
    }

    let boundary_hash = boundary_path
        .map(hash_file_sha256)
        .transpose()?
        .unwrap_or_default();
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v2",
        "run_id": run_id,
        "profile_id": profile.id,
        "domains": profile.domains,
        "stages": stages,
        "domain_transitions": [{
            "from": "fastq",
            "to": "bam",
            "boundary": boundary_path,
        }],
        "boundaries": boundary_path.map(|path| serde_json::json!({
            "name": "alignment_boundary",
            "path": path,
            "sha256": boundary_hash,
        })),
    });
    let path = out_dir.join("run_manifest.json");
    fs::write(&path, serde_json::to_vec_pretty(&manifest)?).context("write run_manifest.json")?;
    Ok(())
}

pub fn write_reference_manifest(out_dir: &Path, record: &ReferenceRecord) -> Result<PathBuf> {
    let root = out_dir.join("run_artifacts").join("boundaries");
    fs::create_dir_all(&root).context("create boundaries dir")?;
    let path = root.join("reference_manifest.json");
    fs::write(&path, serde_json::to_vec_pretty(record)?)
        .context("write reference_manifest.json")?;
    Ok(path)
}
