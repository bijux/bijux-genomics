//! Owner: bijux-dna-bench-population
//! Orchestrate admitted upstream reports, scientific execution, normalized observations and publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::inputs::PopulationBenchmarkInputs;
use crate::population_structure::admission::binding::resolve_governed_vcf_population_structure_smoke_contract;
use crate::population_structure::execution::timing::timestamp_marker;
use crate::population_structure::model::LocalVcfPopulationStructureSmokeReport;
use crate::population_structure::model::DEFAULT_VCF_POPULATION_STRUCTURE_SMOKE_ROOT;
use crate::population_structure::model::LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_COMMAND as COMMAND;
use crate::population_structure::model::LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted population benchmark and return its persisted report.
///
/// # Errors
/// Rejects incompatible inputs and propagates scientific execution or publication failure.
pub fn run_local_vcf_population_structure_smoke(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPopulationStructureSmokeReport> {
    let contract = resolve_governed_vcf_population_structure_smoke_contract(inputs, tool_id)?;
    let upstream = super::admission::upstream::acquire_upstream(inputs, repo_root, &contract)?;

    let published_output_root =
        repo_root.join(DEFAULT_VCF_POPULATION_STRUCTURE_SMOKE_ROOT).join(&contract.tool);
    let staging_parent = published_output_root.parent().ok_or_else(|| {
        anyhow!(
            "VCF population-structure smoke root has no parent: {}",
            published_output_root.display()
        )
    })?;
    let staging_dir =
        bijux_dna_infra::temp_dir_in(staging_parent, &format!("{}-staging-", contract.tool))
            .with_context(|| format!("create staging directory in {}", staging_parent.display()))?;
    let output_root = staging_dir.path().to_path_buf();
    let layout = super::execution::layout::prepare_layout(&published_output_root, &output_root)?;
    let super::execution::layout::PopulationArtifactLayout {
        published_population_structure_json_path,
        published_source_population_structure_path,
        published_source_pruned_variants_path,
        published_source_logs_path,
        published_source_pca_report_path,
        published_source_admixture_report_path,
        published_stage_result_manifest_path,
        ..
    } = &layout;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = super::execution::stage::execute_stage(&upstream, &layout, &contract)?;

    let collected = super::execution::evidence::collect_evidence(
        repo_root,
        &upstream,
        &layout,
        &output_root,
        &stage_outputs,
    )?;

    let super::observations::stage::NormalizedPopulationStage {
        selected_pca_tool_id,
        selected_pca_tool_ok,
        selected_admixture_tool_id,
        selected_admixture_tool_ok,
        consumed_pca,
        consumed_admixture,
        sample_groups,
        distance_summary,
        status,
    } = super::observations::stage::normalize_stage(repo_root, &collected, &layout, &contract)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfPopulationStructureSmokeReport {
        schema_version: LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION,
        command: format!("{COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        output_root: path_relative_to_repo(repo_root, &published_output_root),
        population_structure_json_path: path_relative_to_repo(
            repo_root,
            published_population_structure_json_path,
        ),
        source_population_structure_path: path_relative_to_repo(
            repo_root,
            published_source_population_structure_path,
        ),
        source_pruned_variants_path: path_relative_to_repo(
            repo_root,
            published_source_pruned_variants_path,
        ),
        source_logs_path: path_relative_to_repo(repo_root, published_source_logs_path),
        source_pca_report_path: path_relative_to_repo(repo_root, published_source_pca_report_path),
        source_admixture_report_path: path_relative_to_repo(
            repo_root,
            published_source_admixture_report_path,
        ),
        stage_result_manifest_path: path_relative_to_repo(
            repo_root,
            published_stage_result_manifest_path,
        ),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        selected_pca_tool_id,
        selected_pca_tool_ok,
        selected_admixture_tool_id,
        selected_admixture_tool_ok,
        consumed_pca,
        consumed_admixture,
        sample_groups,
        distance_summary,
        status,
    };
    super::execution::report::publish_report(
        repo_root,
        &report,
        &stage_outputs,
        &output_root,
        &published_output_root,
    )?;

    Ok(report)
}
