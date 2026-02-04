use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_core::alignment::AlignmentBoundary;
use bijux_env_runtime::ReferenceRecord;
use bijux_infra::hash_file_sha256;
use bijux_pipelines::PipelineProfile;

use crate::cross_router::CROSS_STAGE_ID;
use crate::fastq_router::StageExecutionSummary;

pub fn write_alignment_boundary(out_dir: &Path, boundary: &AlignmentBoundary) -> Result<PathBuf> {
    let boundaries_dir = out_dir.join("run_artifacts").join("boundaries");
    bijux_infra::ensure_dir(&boundaries_dir).context("create boundaries dir")?;
    let path = boundaries_dir.join("alignment_boundary.json");
    bijux_infra::atomic_write_json(&path, boundary)
        .with_context(|| "write alignment_boundary.json")?;
    Ok(path)
}

pub fn write_defaults_ledger(out_dir: &Path, profile: &PipelineProfile) -> Result<PathBuf> {
    let path = out_dir.join("defaults_ledger.json");
    let ledger = profile.defaults_ledger();
    bijux_infra::atomic_write_json(&path, &ledger).context("write defaults_ledger.json")?;
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
                "out_dir": relative_path_string(out_dir, &entry.plan.out_dir),
                "metrics": relative_path_string(out_dir, &entry.plan.out_dir.join("run_artifacts").join("metrics.json")),
                "stage_report": relative_path_string(out_dir, &entry.plan.out_dir.join("run_artifacts").join("stage_report.json")),
            },
        }));
    }

    let boundary_hash = boundary_path
        .map(hash_file_sha256)
        .transpose()?
        .unwrap_or_default();
    let mut domains = profile.input_domains.clone();
    for domain in &profile.output_domains {
        if !domains.contains(domain) {
            domains.push(*domain);
        }
    }
    let defaults_path = out_dir.join("defaults_ledger.json");
    let defaults_hash = hash_file_sha256(&defaults_path)?;
    let pipeline_id = profile.id.as_str().to_string();
    let _git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let _build_profile =
        std::env::var("BIJUX_BUILD_PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let run_provenance = run_provenance_from_cross(out_dir, fastq_summary, bam_runs, &pipeline_id);
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v2",
        "run_id": run_id,
        "profile_id": profile.id,
        "domains": domains,
        "stages": stages,
        "defaults_ledger": relative_path_string(out_dir, &defaults_path),
        "defaults_ledger_sha256": defaults_hash,
        "run_provenance": run_provenance,
        "domain_transitions": [{
            "from": "fastq",
            "to": "bam",
            "boundary": boundary_path.map(|path| relative_path_string(out_dir, path)),
        }],
        "boundaries": boundary_path.map(|path| serde_json::json!({
            "name": "alignment_boundary",
            "path": relative_path_string(out_dir, path),
            "sha256": boundary_hash,
        })),
    });
    let path = out_dir.join("run_manifest.json");
    bijux_infra::atomic_write_json(&path, &manifest).context("write run_manifest.json")?;
    Ok(())
}

pub fn write_reference_manifest(out_dir: &Path, record: &ReferenceRecord) -> Result<PathBuf> {
    let root = out_dir.join("run_artifacts").join("boundaries");
    bijux_infra::ensure_dir(&root).context("create boundaries dir")?;
    let path = root.join("reference_manifest.json");
    bijux_infra::atomic_write_json(&path, record)
        .with_context(|| "write reference_manifest.json")?;
    Ok(path)
}

fn relative_path_string(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn run_provenance_from_cross(
    _out_dir: &Path,
    fastq_summary: &serde_json::Value,
    bam_runs: &[StageExecutionSummary],
    pipeline_id: &str,
) -> serde_json::Value {
    let mut params_by_stage = std::collections::BTreeMap::new();
    let mut input_hashes = Vec::new();
    let mut tool_versions = std::collections::BTreeSet::new();
    let mut image_digests = std::collections::BTreeSet::new();
    if let Some(fastq_prov) = fastq_summary.get("run_provenance") {
        if let Some(hash) = fastq_prov
            .get("params_hash")
            .and_then(serde_json::Value::as_str)
        {
            params_by_stage.insert("fastq".to_string(), hash.to_string());
        }
        if let Some(inputs) = fastq_prov
            .get("input_hashes")
            .and_then(serde_json::Value::as_array)
        {
            for input in inputs {
                if let Some(value) = input.as_str() {
                    input_hashes.push(value.to_string());
                }
            }
        }
        if let Some(value) = fastq_prov
            .get("tool_version")
            .and_then(serde_json::Value::as_str)
        {
            tool_versions.insert(value.to_string());
        }
        if let Some(value) = fastq_prov
            .get("tool_image_digest")
            .and_then(serde_json::Value::as_str)
        {
            image_digests.insert(value.to_string());
        }
    }
    for entry in bam_runs {
        tool_versions.insert(entry.plan.tool_version.clone());
        if let Some(digest) = entry.plan.image.digest.clone() {
            image_digests.insert(digest);
        }
        let envelope_path = entry
            .plan
            .out_dir
            .join("run_artifacts")
            .join("metrics_envelope.json");
        if let Ok(raw) = std::fs::read_to_string(&envelope_path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(hash) = value.get("input_hash").and_then(serde_json::Value::as_str) {
                    input_hashes.push(hash.to_string());
                }
                if let Some(hash) = value.get("params_hash").and_then(serde_json::Value::as_str) {
                    params_by_stage.insert(entry.plan.stage_id.0.clone(), hash.to_string());
                }
            }
        }
    }
    input_hashes.sort();
    input_hashes.dedup();
    let params_hash = bijux_core::params_hash(&serde_json::json!(params_by_stage))
        .unwrap_or_else(|_| "unknown".to_string());
    let tool_version = if tool_versions.len() == 1 {
        tool_versions
            .into_iter()
            .next()
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "multiple".to_string()
    };
    let tool_image_digest = if image_digests.len() == 1 {
        image_digests.into_iter().next()
    } else {
        None
    };
    let git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let build_profile =
        std::env::var("BIJUX_BUILD_PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let reference_genome = std::env::var("BIJUX_REFERENCE_GENOME").ok();
    serde_json::json!({
        "schema_version": "bijux.run_provenance.v1",
        "tool_image_digest": tool_image_digest,
        "tool_version": tool_version,
        "params_hash": params_hash,
        "input_hashes": input_hashes,
        "reference_genome": reference_genome,
        "pipeline_id": pipeline_id,
        "git_commit": git_commit,
        "build_profile": build_profile,
    })
}

#[cfg(test)]
mod tests {
    use super::{write_cross_run_manifest, write_defaults_ledger};
    use bijux_pipelines::registry::profile_by_id;
    use bijux_pipelines::Domain;

    #[test]
    fn cross_run_manifest_includes_defaults_ledger() -> anyhow::Result<()> {
        let temp = bijux_infra::temp_dir("bijux-cross-manifest")?;
        let out_dir = temp.path();
        let profile = profile_by_id(Domain::Cross, "fastq-to-bam__default__v1")?;
        write_defaults_ledger(out_dir, &profile)?;
        let fastq_summary = serde_json::json!({
            "run_id": "run-1",
            "stages": [{
                "stage_id": "fastq.trim",
                "tool_id": "fastp",
                "artifacts": {},
            }]
        });
        write_cross_run_manifest(out_dir, &profile, &fastq_summary, &[], None, None)?;
        let manifest_raw = std::fs::read_to_string(out_dir.join("run_manifest.json"))?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)?;
        assert!(manifest.get("defaults_ledger").is_some());
        assert!(manifest.get("defaults_ledger_sha256").is_some());
        Ok(())
    }
}
