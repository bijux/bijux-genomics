//! Owner: bijux-dna-bench
//! Immutable stage-plugin normalization of terminal benchmark execution units.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_execution_spec, validate_utc_timestamp, BenchmarkExecutionSpec,
    BenchmarkStageNormalizationCheckV1, BenchmarkStageNormalizationUnitV1,
    BenchmarkStageNormalizationV1, BENCHMARK_STAGE_NORMALIZATION_ARTIFACT,
    BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1,
};
use bijux_dna_core::contract::ArtifactRef;
use bijux_dna_stage_contract::{canonical_tool_bindings, DownstreamProbeAssertionV1, StagePlanV1};

use super::load_stage_plan_manifest;
use crate::BenchmarkStagePluginRegistry;

pub use bijux_dna_bench_model::BENCHMARK_STAGE_NORMALIZATION_ARTIFACT as STAGE_PLUGIN_NORMALIZATION_ARTIFACT;

/// Normalize every immutable execution unit through the selected binding's domain plugin.
///
/// Existing evidence is replayed without changing its original timestamp. A completed scheduler
/// marker is required, while raw output closure validation remains the admission owner's next
/// operation.
///
/// # Errors
/// Returns an error when execution authority, paths, plugin output, probe evidence, or retained
/// immutable bytes do not close over the attempt.
pub fn normalize_benchmark_execution_outputs(
    benchmark_root: &Path,
    spec: &BenchmarkExecutionSpec,
    plugins: &BenchmarkStagePluginRegistry<'_>,
    normalized_at_utc: &str,
) -> Result<BenchmarkStageNormalizationV1> {
    validate_execution_spec(spec).context("validate plugin normalization execution authority")?;
    validate_utc_timestamp(normalized_at_utc, "stage-plugin normalization timestamp")?;
    if spec.interface.downstream_probe.artifact != BENCHMARK_STAGE_NORMALIZATION_ARTIFACT {
        bail!("benchmark execution does not declare stage-plugin normalization");
    }
    let result_root = Path::new(&spec.outputs.result_root);
    if result_root.canonicalize().context("canonicalize normalization result root")? != result_root
    {
        bail!("stage-plugin normalization result root is not canonical");
    }
    let output_path = result_root.join(BENCHMARK_STAGE_NORMALIZATION_ARTIFACT);
    require_regular_file(&result_root.join("scheduler/completed.json"))?;

    let (manifest, plans) = load_stage_plan_manifest(benchmark_root, &spec.identity)?;
    let manifest_path = result_root.join("stage-plan-manifest.json");
    let manifest_sha256 = bijux_dna_infra::hash_file_sha256(&manifest_path)
        .context("hash stage-plan manifest for normalization")?;
    if manifest.execution_spec_sha256
        != bijux_dna_infra::hash_file_sha256(&result_root.join("execution-spec.json"))?
    {
        bail!("stage-plan manifest differs from plugin normalization authority");
    }
    let existing = read_existing(&output_path)?;
    let evidence_timestamp = existing
        .as_ref()
        .map_or(normalized_at_utc, |publication| publication.normalized_at_utc.as_str());
    validate_utc_timestamp(evidence_timestamp, "retained stage-plugin normalization timestamp")?;
    let bindings = canonical_tool_bindings().context("load normalization binding authority")?;
    let mut units = plans
        .iter()
        .map(|plan| normalize_unit(spec, plugins, &bindings, plan))
        .collect::<Result<Vec<_>>>()?;
    units.sort_by(|left, right| left.stage_instance_id.cmp(&right.stage_instance_id));
    if units.is_empty() {
        bail!("stage-plugin normalization has no execution units");
    }
    let publication = BenchmarkStageNormalizationV1 {
        schema_version: BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1.to_string(),
        identity: spec.identity.clone(),
        normalized_at_utc: evidence_timestamp.to_string(),
        stage_plan_manifest_sha256: manifest_sha256.clone(),
        passed: units.iter().all(|unit| unit.passed),
        units,
    };
    validate_publication(&publication, spec, &plans, &manifest_sha256)?;
    if let Some(existing) = existing {
        if publication != existing {
            bail!("replayed stage-plugin normalization differs from retained evidence");
        }
        return Ok(existing);
    }
    let parent = output_path.parent().context("normalization output has no parent")?;
    ensure_regular_directory(parent)?;
    bijux_dna_infra::atomic_create_json(&output_path, &publication)
        .context("publish immutable stage-plugin normalization")?;
    Ok(publication)
}

fn normalize_unit(
    spec: &BenchmarkExecutionSpec,
    plugins: &BenchmarkStagePluginRegistry<'_>,
    bindings: &bijux_dna_stage_contract::ToolBindingRegistry,
    plan: &StagePlanV1,
) -> Result<BenchmarkStageNormalizationUnitV1> {
    let mut host_plan = plan.clone();
    host_plan.io.inputs = plan
        .io
        .inputs
        .iter()
        .map(|artifact| host_input_artifact(spec, artifact))
        .collect::<Result<Vec<_>>>()?;
    host_plan.io.outputs = plan
        .io
        .outputs
        .iter()
        .map(|artifact| host_output_artifact(spec, artifact))
        .collect::<Result<Vec<_>>>()?;
    let stage_instance_id = plan
        .stage_instance_id
        .as_ref()
        .map(ToString::to_string)
        .context("normalization stage plan omits instance identity")?;
    host_plan.out_dir =
        Path::new(&spec.outputs.result_root).join("repetitions").join(&stage_instance_id);
    let normalized = plugins.normalize(bindings, &host_plan, &host_plan.io.outputs)?;
    let normalized_output =
        serde_json::to_value(&normalized.output).context("serialize typed stage-plugin output")?;
    let normalized_output_sha256 = bijux_dna_infra::hash_bytes_sha256(
        &serde_json::to_vec(&normalized_output).context("encode normalized plugin output")?,
    );
    let downstream = normalized.downstream;
    Ok(BenchmarkStageNormalizationUnitV1 {
        stage_instance_id,
        normalized_output,
        normalized_output_sha256,
        probe_id: downstream.probe_id,
        binding_id: downstream.binding_id,
        producer_stage_id: downstream.producer_stage_id.to_string(),
        tool_id: downstream.tool_id.to_string(),
        passed: downstream.passed,
        checks: downstream
            .checks
            .into_iter()
            .map(|check| BenchmarkStageNormalizationCheckV1 {
                assertion: assertion_name(check.assertion).to_string(),
                passed: check.passed,
                detail: check.detail,
            })
            .collect(),
    })
}

fn host_input_artifact(
    spec: &BenchmarkExecutionSpec,
    artifact: &ArtifactRef,
) -> Result<ArtifactRef> {
    let mount = artifact.path.to_str().context("stage input path is not UTF-8")?;
    let input = spec
        .inputs
        .iter()
        .find(|input| input.mount_path == mount)
        .with_context(|| format!("stage input `{mount}` has no execution input authority"))?;
    let path = PathBuf::from(&input.path);
    require_regular_file(&path)?;
    if path.canonicalize().context("canonicalize normalization input")? != path {
        bail!("stage-plugin normalization input path is not canonical");
    }
    let mut host = artifact.clone();
    host.path = path;
    Ok(host)
}

fn host_output_artifact(
    spec: &BenchmarkExecutionSpec,
    artifact: &ArtifactRef,
) -> Result<ArtifactRef> {
    let relative = artifact
        .path
        .strip_prefix(&spec.outputs.mount_path)
        .context("stage output escapes the execution result mount")?;
    let path = Path::new(&spec.outputs.result_root).join(relative);
    require_regular_artifact(&path)?;
    let mut host = artifact.clone();
    host.path = path;
    Ok(host)
}

fn validate_publication(
    publication: &BenchmarkStageNormalizationV1,
    spec: &BenchmarkExecutionSpec,
    plans: &[StagePlanV1],
    manifest_sha256: &str,
) -> Result<()> {
    let planned_instances = plans
        .iter()
        .filter_map(|plan| plan.stage_instance_id.as_ref().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let observed_instances = publication
        .units
        .iter()
        .map(|unit| unit.stage_instance_id.clone())
        .collect::<BTreeSet<_>>();
    if publication.schema_version != BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1
        || publication.identity != spec.identity
        || publication.units.is_empty()
        || publication.stage_plan_manifest_sha256 != manifest_sha256
        || planned_instances.len() != plans.len()
        || observed_instances.len() != publication.units.len()
        || observed_instances != planned_instances
        || publication.passed != publication.units.iter().all(|unit| unit.passed)
        || publication.units.iter().any(|unit| {
            unit.producer_stage_id != spec.identity.run.stage_id
                || unit.tool_id != spec.identity.tool_id
                || unit.probe_id != spec.interface.downstream_probe.stage_id
                || unit.binding_id.trim().is_empty()
                || unit.checks.len() != 4
                || unit.checks.iter().any(|check| check.detail.trim().is_empty())
                || normalization_check_names(unit)
                    != [
                        "declared_artifacts",
                        "normalized_identity",
                        "scientific_invariants",
                        "scientific_metrics",
                    ]
                    .into_iter()
                    .collect()
                || unit.passed != unit.checks.iter().all(|check| check.passed)
                || match serde_json::to_vec(&unit.normalized_output) {
                    Ok(bytes) => {
                        bijux_dna_infra::hash_bytes_sha256(&bytes) != unit.normalized_output_sha256
                    }
                    Err(_) => true,
                }
        })
    {
        bail!("stage-plugin normalization publication differs from execution authority");
    }
    Ok(())
}

fn normalization_check_names(unit: &BenchmarkStageNormalizationUnitV1) -> BTreeSet<&str> {
    unit.checks.iter().map(|check| check.assertion.as_str()).collect()
}

fn assertion_name(assertion: DownstreamProbeAssertionV1) -> &'static str {
    match assertion {
        DownstreamProbeAssertionV1::NormalizedIdentity => "normalized_identity",
        DownstreamProbeAssertionV1::ScientificMetrics => "scientific_metrics",
        DownstreamProbeAssertionV1::DeclaredArtifacts => "declared_artifacts",
        DownstreamProbeAssertionV1::ScientificInvariants => "scientific_invariants",
    }
}

fn read_existing(path: &Path) -> Result<Option<BenchmarkStageNormalizationV1>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("stage-plugin normalization is not a regular file");
            }
            let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)?;
            Ok(Some(serde_json::from_slice(&bytes).context("parse stage-plugin normalization")?))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("inspect stage-plugin normalization"),
    }
}

fn require_regular_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect normalization evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("normalization evidence is not a regular file: {}", path.display());
    }
    Ok(())
}

fn require_regular_artifact(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect normalization artifact {}", path.display()))?;
    if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
        bail!("normalization output is not a regular artifact: {}", path.display());
    }
    if path.canonicalize().context("canonicalize normalization output")? != path {
        bail!("stage-plugin normalization output path is not canonical");
    }
    if metadata.is_dir() {
        let mut files = 0_usize;
        for entry in walkdir::WalkDir::new(path).min_depth(1).follow_links(false) {
            let entry = entry.context("walk normalization directory artifact")?;
            if entry.file_type().is_symlink()
                || (!entry.file_type().is_file() && !entry.file_type().is_dir())
            {
                bail!("normalization directory contains an unsafe artifact");
            }
            if entry.file_type().is_file() {
                files += 1;
            }
        }
        if files == 0 {
            bail!("normalization directory artifact is empty");
        }
    }
    Ok(())
}

fn ensure_regular_directory(path: &Path) -> Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!("stage-plugin normalization root is not a regular directory");
            }
            Ok(())
        }
        Err(error) => Err(error).context("create stage-plugin normalization root"),
    }
}
