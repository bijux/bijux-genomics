use super::inputs::{optional_input_path, required_input_path};
use super::paths::path_relative_to_repo;
use super::publication::write_empty_gzip;
use super::streams::count_fastq_stats;
use super::MaterializedFastqProof;
use anyhow::{Context, Result};
use std::path::Path;
/// Publish the governed synthetic screen taxonomy report for parser verification.
///
/// # Errors
/// Rejects foreign plan identity, unavailable inputs and report publication failures.
pub fn generate_screen_taxonomy_probe(
    repo_root: &Path,
    probe_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<MaterializedFastqProof> {
    super::inputs::admit_stage(
        plan,
        bijux_dna_domain_fastq::stages::STAGE_SCREEN_TAXONOMY.as_str(),
    )?;

    let input_r1 = required_input_path(repo_root, plan, "reads_r1")?;
    let input_r2 = optional_input_path(repo_root, plan, "reads_r2");
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let effective_params: bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("parse governed screen_taxonomy effective params")?;
    let probe_dir = probe_root.join("screen_taxonomy");
    bijux_dna_infra::ensure_dir(&probe_dir)
        .with_context(|| format!("create {}", probe_dir.display()))?;

    let unclassified_r1 = probe_dir.join("unclassified_R1.fastq.gz");
    write_empty_gzip(&unclassified_r1)?;
    let unclassified_r2 = if input_r2.is_some() {
        let path = probe_dir.join("unclassified_R2.fastq.gz");
        write_empty_gzip(&path)?;
        Some(path)
    } else {
        None
    };
    let report_tsv = probe_dir.join("taxonomy_screen.tsv");
    bijux_dna_infra::atomic_write_bytes(
        &report_tsv,
        b"tax_id\tname\treads\tfraction\n9606\tHomo sapiens\t4\t1.000000\n",
    )
    .with_context(|| format!("write {}", report_tsv.display()))?;

    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let entries = vec![bijux_dna_domain_fastq::TaxonomyScreenSummaryEntryV1 {
        label: "Homo sapiens".to_string(),
        percent: 100.0,
    }];
    let report_path = probe_dir.join("classification_report.json");
    let report = bijux_dna_domain_fastq::ScreenTaxonomyReportV1 {
        schema_version: bijux_dna_domain_fastq::SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
        stage: bijux_dna_domain_fastq::stages::STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        stage_id: bijux_dna_domain_fastq::stages::STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: effective_params.threads,
        classifier: effective_params.classifier,
        report_format: effective_params.report_format,
        assignment_format: effective_params.assignment_format,
        database_catalog_id: effective_params.database_catalog_id,
        database_artifact_id: effective_params.database_artifact_id,
        database_build_id: effective_params.database_build_id,
        database_digest: effective_params.database_digest,
        database_namespace: effective_params.database_namespace,
        database_scope: effective_params.database_scope,
        minimum_confidence: effective_params.minimum_confidence,
        emit_unclassified: effective_params.emit_unclassified,
        interpretation_boundary: effective_params.interpretation_boundary,
        truth_conditions: effective_params.truth_conditions,
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        screen_report_tsv: path_relative_to_repo(repo_root, &report_tsv),
        classification_report_json: path_relative_to_repo(repo_root, &report_path),
        unclassified_reads_r1: Some(path_relative_to_repo(repo_root, &unclassified_r1)),
        unclassified_reads_r2: unclassified_r2
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        reads_in: Some(reads_in),
        reads_out: Some(reads_in),
        bases_in: Some(bases_in),
        bases_out: Some(bases_in),
        pairs_in,
        pairs_out: pairs_in,
        contamination_rate: Some(0.0),
        classified_fraction: Some(1.0),
        unclassified_fraction: Some(0.0),
        summary_entries: entries.clone(),
        top_taxa: entries,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedFastqProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}
