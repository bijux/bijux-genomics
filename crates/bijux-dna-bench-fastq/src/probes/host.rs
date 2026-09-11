use super::inputs::{optional_input_path, required_input_path};
use super::parameters::{bool_json, f64_json, optional_string_json, string_json, u64_json};
use super::paths::path_relative_to_repo;
use super::publication::{copy_file, write_empty_gzip};
use super::streams::count_fastq_stats;
use super::MaterializedFastqProof;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
/// Publish the governed synthetic deplete host report for parser verification.
///
/// # Errors
/// Rejects foreign plan identity, unavailable inputs and report publication failures.
pub fn generate_deplete_host_probe(
    repo_root: &Path,
    probe_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<MaterializedFastqProof> {
    super::inputs::admit_stage(plan, bijux_dna_domain_fastq::stages::STAGE_DEPLETE_HOST.as_str())?;

    let input_r1 = required_input_path(repo_root, plan, "reads_r1")?;
    let input_r2 = optional_input_path(repo_root, plan, "reads_r2");
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let outputs = materialize_host_reads(probe_root, &input_r1, input_r2.as_deref())?;
    let report_path = outputs.report_path.clone();
    let report = project_host_report(
        repo_root,
        plan,
        HostProbeEvidence { input_r1, input_r2, stats_r1, stats_r2, outputs },
    );
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedFastqProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}

struct HostProbeOutputs {
    output_r1: std::path::PathBuf,
    output_r2: Option<std::path::PathBuf>,
    removed_r1: std::path::PathBuf,
    removed_r2: Option<std::path::PathBuf>,
    report_path: std::path::PathBuf,
}

fn materialize_host_reads(
    probe_root: &Path,
    input_r1: &Path,
    input_r2: Option<&Path>,
) -> Result<HostProbeOutputs> {
    let probe_dir = probe_root.join("deplete_host");
    bijux_dna_infra::ensure_dir(&probe_dir)
        .with_context(|| format!("create {}", probe_dir.display()))?;

    let output_r1 = probe_dir.join("host_depleted_R1.fastq.gz");
    copy_file(input_r1, &output_r1)?;
    let output_r2 = if let Some(input) = input_r2 {
        let path = probe_dir.join("host_depleted_R2.fastq.gz");
        copy_file(input, &path)?;
        Some(path)
    } else {
        None
    };
    let removed_r1 = probe_dir.join("removed_host_R1.fastq.gz");
    write_empty_gzip(&removed_r1)?;
    let removed_r2 = if input_r2.is_some() {
        let path = probe_dir.join("removed_host_R2.fastq.gz");
        write_empty_gzip(&path)?;
        Some(path)
    } else {
        None
    };

    Ok(HostProbeOutputs {
        output_r1,
        output_r2,
        removed_r1,
        removed_r2,
        report_path: probe_dir.join("host_depletion_report.json"),
    })
}

struct HostProbeEvidence {
    input_r1: std::path::PathBuf,
    input_r2: Option<std::path::PathBuf>,
    stats_r1: super::model::FastqReadStats,
    stats_r2: Option<super::model::FastqReadStats>,
    outputs: HostProbeOutputs,
}

fn project_host_report(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    evidence: HostProbeEvidence,
) -> bijux_dna_domain_fastq::DepleteHostReportV1 {
    let HostProbeEvidence { input_r1, input_r2, stats_r1, stats_r2, outputs } = evidence;
    let HostProbeOutputs { output_r1, output_r2, removed_r1, removed_r2, report_path } = outputs;
    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    bijux_dna_domain_fastq::DepleteHostReportV1 {
        schema_version: bijux_dna_domain_fastq::DEPLETE_HOST_REPORT_SCHEMA_VERSION.to_string(),
        stage: bijux_dna_domain_fastq::stages::STAGE_DEPLETE_HOST.as_str().to_string(),
        stage_id: bijux_dna_domain_fastq::stages::STAGE_DEPLETE_HOST.as_str().to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: u64_json(&plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        reference_scope: serde_json::from_value(
            plan.effective_params
                .get("reference_scope")
                .cloned()
                .unwrap_or(Value::String("host".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReferenceScope::Host),
        reference_catalog_id: string_json(&plan.effective_params, "reference_catalog_id")
            .unwrap_or_else(|| "host_reference".to_string()),
        reference_index_artifact_id: string_json(
            &plan.effective_params,
            "reference_index_artifact_id",
        )
        .unwrap_or_else(|| "reference_index".to_string()),
        reference_index_backend: string_json(&plan.effective_params, "reference_index_backend")
            .unwrap_or_else(|| "bowtie2_build".to_string()),
        reference_build_id: optional_string_json(&plan.effective_params, "reference_build_id"),
        reference_digest: optional_string_json(&plan.effective_params, "reference_digest"),
        masking_policy: serde_json::from_value(
            plan.effective_params
                .get("masking_policy")
                .cloned()
                .unwrap_or(Value::String("unmasked".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReferenceMaskingPolicy::Unmasked),
        decoy_policy: serde_json::from_value(
            plan.effective_params
                .get("decoy_policy")
                .cloned()
                .unwrap_or(Value::String("none".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReferenceDecoyPolicy::None),
        decoy_catalog_id: optional_string_json(&plan.effective_params, "decoy_catalog_id"),
        identity_threshold: f64_json(&plan.effective_params, "identity_threshold").unwrap_or(0.95),
        retained_read_policy: serde_json::from_value(
            plan.effective_params
                .get("retained_read_policy")
                .cloned()
                .unwrap_or(Value::String("keep_non_host_reads".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReadRetentionPolicy::KeepNonHostReads),
        emit_removed_reads: bool_json(&plan.effective_params, "emit_removed_reads").unwrap_or(true),
        report_format: serde_json::from_value(
            plan.effective_params
                .get("report_format")
                .cloned()
                .unwrap_or(Value::String("bowtie2_metrics_file".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::MappingReportFormat::Bowtie2MetricsFile),
        retain_unmapped_pairs: bool_json(&plan.effective_params, "retain_unmapped_pairs")
            .unwrap_or(false),
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        removed_host_r1: path_relative_to_repo(repo_root, &removed_r1),
        removed_host_r2: removed_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, &report_path),
        reads_in,
        reads_out: reads_in,
        reads_removed: 0,
        bases_in,
        bases_out: bases_in,
        bases_removed: 0,
        pairs_in,
        pairs_out: pairs_in,
        host_fraction_removed: 0.0,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(
            serde_json::json!({ "reads_removed": 0_u64, "bases_removed": 0_u64 }),
        ),
    }
}
