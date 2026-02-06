use std::path::{Path, PathBuf};

use super::AlignmentBoundary;
use anyhow::{Context, Result};
use bijux_environment::resolve::ReferenceRecord;
use bijux_infra::hash_file_sha256;
use bijux_pipelines::PipelineProfile;

use crate::handlers::fastq::StageExecutionSummary;
use bijux_pipelines::STAGE_CORE_PREPARE_REFERENCE;
use bijux_pipelines::STAGE_CROSS_ALIGN_STUB;

pub fn write_alignment_boundary(out_dir: &Path, boundary: &AlignmentBoundary) -> Result<PathBuf> {
    let boundaries_dir =
        bijux_runtime::recording::run_artifacts_dir_for_out(out_dir).join("boundaries");
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

#[allow(clippy::too_many_lines)]
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
            "stage_id": STAGE_CROSS_ALIGN_STUB,
            "tool_id": "alignment_boundary",
            "domain": "cross",
            "artifacts": {
                "alignment_boundary": boundary_path,
            },
        }));
    }
    if let Some(reference_path) = prepare_reference_path {
        stages.push(serde_json::json!({
            "stage_id": STAGE_CORE_PREPARE_REFERENCE,
            "tool_id": "reference_registry",
            "domain": "cross",
            "artifacts": {
                "reference_manifest": reference_path,
            },
        }));
    }

    for entry in bam_runs {
        stages.push(serde_json::json!({
            "stage_id": entry.plan.step_id.0,
            "tool_id": entry.plan.image.image,
            "domain": "bam",
            "artifacts": {
                "out_dir": relative_path_string(out_dir, &entry.plan.out_dir),
                "metrics": relative_path_string(out_dir, &bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir).join("metrics.json")),
                "stage_report": relative_path_string(out_dir, &bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir).join("stage_report.json")),
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
    let graph_hash = std::env::var("BIJUX_PLAN_HASH")
        .ok()
        .unwrap_or_else(|| "unknown".to_string());
    let toolchain_versions = serde_json::json!({
        "planner": std::env::var("BIJUX_PLANNER_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        "engine": std::env::var("BIJUX_ENGINE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        "runner": std::env::var("BIJUX_RUNNER_VERSION").unwrap_or_else(|_| "unknown".to_string()),
    });
    let mut dataset_fingerprints = Vec::new();
    if let Some(value) = run_provenance
        .get("input_hashes")
        .and_then(serde_json::Value::as_array)
    {
        for item in value {
            if let Some(hash) = item.as_str() {
                dataset_fingerprints.push(hash.to_string());
            }
        }
    }
    dataset_fingerprints.sort();
    dataset_fingerprints.dedup();
    let mut output_artifacts: Vec<serde_json::Value> = Vec::new();
    for entry in bam_runs {
        for artifact in &entry.plan.io.outputs {
            let sha256 = artifact
                .path
                .exists()
                .then(|| bijux_infra::hash_file_sha256(&artifact.path).ok())
                .flatten();
            output_artifacts.push(serde_json::json!({
                "stage_id": entry.plan.step_id.to_string(),
                "name": artifact.name.to_string(),
                "role": artifact.role.as_str(),
                "optional": artifact.optional,
                "path": relative_path_string(out_dir, &artifact.path),
                "sha256": sha256,
            }));
        }
    }
    let bam_out_dir = out_dir.join("bam");
    let bam_root = bijux_runtime::recording::run_artifacts_dir_for_out(&bam_out_dir);
    for (name, role, path) in [
        (
            "summary",
            bijux_core::contract::ArtifactRole::SummaryJson,
            bam_root.join("summary.json"),
        ),
        (
            "summary_tsv",
            bijux_core::contract::ArtifactRole::SummaryTsv,
            bam_root.join("summary.tsv"),
        ),
        (
            "report_html",
            bijux_core::contract::ArtifactRole::ReportHtml,
            bam_root.join("report.html"),
        ),
    ] {
        let sha256 = path
            .exists()
            .then(|| bijux_infra::hash_file_sha256(&path).ok())
            .flatten();
        output_artifacts.push(serde_json::json!({
            "stage_id": "report.aggregate",
            "name": name,
            "role": role.as_str(),
            "optional": false,
            "path": relative_path_string(out_dir, &path),
            "sha256": sha256,
        }));
    }
    let tool_invocations: Vec<bijux_core::metrics::ToolInvocationV1> = bam_runs
        .iter()
        .filter_map(|entry| {
            let path = bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                .join("tool_invocation.json");
            let raw = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str(&raw).ok()
        })
        .collect();
    let cache_key = {
        let params_hash = run_provenance
            .get("params_hash")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let input_hashes = run_provenance
            .get("input_hashes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(std::string::ToString::to_string))
            .collect::<Vec<_>>();
        let tool_version = run_provenance
            .get("tool_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let env_digest = run_provenance
            .get("tool_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        bijux_core::primitives::CacheKey::new(
            bijux_core::primitives::input_fingerprint(&input_hashes),
            params_hash,
            tool_version,
            env_digest,
        )
    };
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_core::contract::ContractVersion::v1(),
        "run_id": run_id,
        "pipeline_id": pipeline_id,
        "profile_id": profile.id,
        "graph_hash": graph_hash,
        "cache_key": cache_key,
        "toolchain_versions": toolchain_versions,
        "dataset_fingerprints": dataset_fingerprints,
        "tool_invocations": tool_invocations,
        "output_artifacts": output_artifacts,
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
    let payload = bijux_core::primitives::to_canonical_json_bytes(&manifest)?;
    bijux_infra::atomic_write_bytes(&path, payload.as_slice())
        .context("write run_manifest.json")?;
    Ok(())
}

pub fn write_reference_manifest(out_dir: &Path, record: &ReferenceRecord) -> Result<PathBuf> {
    let root = bijux_runtime::recording::run_artifacts_dir_for_out(out_dir).join("boundaries");
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
            .get("parameters_fingerprint")
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
        tool_versions.insert("unknown".to_string());
        if let Some(digest) = entry.plan.image.digest.clone() {
            image_digests.insert(digest);
        }
        let envelope_path =
            bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                .join("metrics_envelope.json");
        if let Ok(raw) = std::fs::read_to_string(&envelope_path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(hash) = value
                    .get("input_fingerprint")
                    .and_then(serde_json::Value::as_str)
                {
                    input_hashes.push(hash.to_string());
                }
                if let Some(hash) = value
                    .get("parameters_fingerprint")
                    .and_then(serde_json::Value::as_str)
                {
                    params_by_stage.insert(entry.plan.step_id.to_string(), hash.to_string());
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
    let plan_hash = std::env::var("BIJUX_PLAN_HASH").ok();
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
        "plan_hash": plan_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::{write_cross_run_manifest, write_defaults_ledger};
    use bijux_pipelines::registry::profile_by_id;
    use bijux_pipelines::Domain;
    use bijux_planner_fastq::stage_api::STAGE_TRIM;

    #[test]
    fn cross_run_manifest_includes_defaults_ledger() -> anyhow::Result<()> {
        let temp = bijux_infra::temp_dir("bijux-cross-manifest")?;
        let out_dir = temp.path();
        let profile = profile_by_id(Domain::Cross, "fastq-to-bam__default__v1")?;
        write_defaults_ledger(out_dir, &profile)?;
        let fastq_summary = serde_json::json!({
            "run_id": "run-1",
            "stages": [{
                "stage_id": STAGE_TRIM.as_str(),
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
