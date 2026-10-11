//! Owner: bijux-dna-bench-population
//! Compose admitted inputs, typed stage execution, evidence and publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::inputs::PopulationBenchmarkInputs;
use crate::pca::admission::binding::resolve_governed_vcf_pca_smoke_contract;
use crate::pca::execution::timing::timestamp_marker;
use crate::pca::fixtures::proxy::materialize_proxy_eigensoft_outputs;
use crate::pca::model::LocalVcfPcaSmokeReport;
use crate::pca::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::pca::model::DEFAULT_OUTPUT_TSV_NAME;
use crate::pca::model::DEFAULT_STAGE_RESULT_NAME;
use crate::pca::model::DEFAULT_VCF_PCA_SMOKE_ROOT;
use crate::pca::model::LOCAL_VCF_PCA_SMOKE_COMMAND;
use crate::pca::model::LOCAL_VCF_PCA_SMOKE_SCHEMA_VERSION;
use crate::pca::publication::build_pca_tsv;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted population benchmark and return its persisted report.
///
/// # Errors
/// Rejects incompatible inputs and propagates scientific execution or publication failure.
pub fn run_local_vcf_pca_smoke(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPcaSmokeReport> {
    let contract = resolve_governed_vcf_pca_smoke_contract(inputs, tool_id)?;
    let cohort = super::admission::input::admit_cohort(repo_root)?;
    let super::admission::input::CohortAdmission {
        fixture_manifest_path, metadata_by_sample, ..
    } = &cohort;

    let published_output_root = repo_root.join(DEFAULT_VCF_PCA_SMOKE_ROOT).join(&contract.tool);
    let staging_parent = published_output_root.parent().ok_or_else(|| {
        anyhow!("VCF PCA smoke root has no parent: {}", published_output_root.display())
    })?;
    let staging_dir =
        bijux_dna_infra::temp_dir_in(staging_parent, &format!("{}-staging-", contract.tool))
            .with_context(|| format!("create staging directory in {}", staging_parent.display()))?;
    let output_root = staging_dir.path().to_path_buf();
    let staged = super::execution::cohort::stage_cohort(&cohort, &output_root)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = super::execution::stage::execute_stage(&staged, &contract)?;

    let super::observations::stage::PcaEvidence {
        execution_mode,
        tool_ok,
        variant_count,
        rows,
        eigenvalues,
        excluded_samples,
        unexpected_samples,
    } = super::observations::stage::observe_stage(&cohort, &stage_outputs, &output_root)?;

    let pca_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(&pca_tsv_path, build_pca_tsv(&rows).as_bytes())?;
    let pca_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let published = super::execution::layout::publication_layout(&published_output_root);
    if contract.tool == "eigensoft" {
        materialize_proxy_eigensoft_outputs(
            &staged.input_vcf_path,
            metadata_by_sample,
            &staged.stage_root,
        )?;
    }
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let sample_count = u64::try_from(rows.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let report = LocalVcfPcaSmokeReport {
        schema_version: LOCAL_VCF_PCA_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_PCA_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, &published.input_vcf),
        sample_metadata_path: path_relative_to_repo(repo_root, &published.sample_metadata),
        population_metadata_path: path_relative_to_repo(repo_root, &published.population_metadata),
        population_labels_manifest_path: path_relative_to_repo(
            repo_root,
            &published.population_labels_manifest,
        ),
        output_root: path_relative_to_repo(repo_root, &published_output_root),
        pca_tsv_path: path_relative_to_repo(repo_root, &published.pca_tsv),
        pca_json_path: path_relative_to_repo(repo_root, &published.pca_json),
        source_eigenvec_path: path_relative_to_repo(repo_root, &published.source_eigenvec),
        source_eigenval_path: path_relative_to_repo(repo_root, &published.source_eigenval),
        source_pca_manifest_path: path_relative_to_repo(repo_root, &published.source_pca_manifest),
        source_logs_path: path_relative_to_repo(repo_root, &published.source_logs),
        stage_result_manifest_path: path_relative_to_repo(
            repo_root,
            &published.stage_result_manifest,
        ),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        tool_ok,
        variant_count,
        sample_count,
        excluded_samples,
        unexpected_samples,
        eigenvalues,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&pca_json_path, &report)?;

    let stage_result_manifest = super::execution::stage_result::stage_result(repo_root, &report)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    crate::execution::publication::publish(
        &output_root,
        &published_output_root,
        &crate::execution::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_JSON_NAME,
            schema: LOCAL_VCF_PCA_SMOKE_SCHEMA_VERSION,
            stage: &contract.stage,
            tool: &contract.tool,
        },
    )?;

    Ok(report)
}
