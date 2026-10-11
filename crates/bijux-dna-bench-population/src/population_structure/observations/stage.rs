//! Owner: bijux-dna-bench-population
//! Population structure observations stage ownership.
use crate::population_structure::execution::artifacts::read_json;
use crate::population_structure::observations::evidence::required_bool_at;
use crate::population_structure::observations::evidence::required_string_at;
use crate::population_structure::observations::evidence::summarize_consumed_admixture;
use crate::population_structure::observations::evidence::summarize_consumed_pca;
use crate::population_structure::observations::evidence::validate_consumed_stage_inputs;
use crate::population_structure::observations::records::parse_admixture_rows;
use crate::population_structure::observations::records::parse_pca_rows;
use crate::population_structure::publication::build_distance_summary;
use crate::population_structure::publication::build_sample_groups;
use crate::population_structure::ConsumedAdmixtureReport;
use crate::population_structure::ConsumedPcaReport;
use crate::population_structure::LocalVcfPopulationStructureDistanceSummary;
use crate::population_structure::LocalVcfPopulationStructureSampleGroup;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) struct NormalizedPopulationStage {
    pub(crate) selected_pca_tool_id: String,
    pub(crate) selected_pca_tool_ok: bool,
    pub(crate) selected_admixture_tool_id: String,
    pub(crate) selected_admixture_tool_ok: bool,
    pub(crate) consumed_pca: ConsumedPcaReport,
    pub(crate) consumed_admixture: ConsumedAdmixtureReport,
    pub(crate) sample_groups: Vec<LocalVcfPopulationStructureSampleGroup>,
    pub(crate) distance_summary: LocalVcfPopulationStructureDistanceSummary,
    pub(crate) status: String,
}

pub(crate) fn normalize_stage(
    repo_root: &Path,
    collected: &crate::population_structure::execution::evidence::CollectedPopulationEvidence,
    layout: &crate::population_structure::execution::layout::PopulationArtifactLayout,
    contract: &crate::population_structure::model::GovernedVcfPopulationStructureSmokeContract,
) -> Result<NormalizedPopulationStage> {
    let crate::population_structure::execution::evidence::CollectedPopulationEvidence {
        stage: source_population_structure_json,
        pca: source_pca_report_path,
        admixture: source_admixture_report_path,
        ..
    } = collected;
    let crate::population_structure::execution::layout::PopulationArtifactLayout {
        published_source_pca_report_path,
        published_source_admixture_report_path,
        stage_root,
        ..
    } = layout;
    let consumed_pca_json = read_json(source_pca_report_path)?;
    let consumed_admixture_json = read_json(source_admixture_report_path)?;
    let selected_pca_tool_id = required_string_at(
        source_population_structure_json,
        "/pca/tool_id",
        "population-structure selected PCA tool",
    )?;
    let selected_pca_tool_ok = required_bool_at(
        source_population_structure_json,
        "/pca/tool_ok",
        "population-structure selected PCA execution",
    )?;
    let selected_admixture_tool_id = required_string_at(
        source_population_structure_json,
        "/admixture/tool_id",
        "population-structure selected admixture tool",
    )?;
    let selected_admixture_tool_ok = required_bool_at(
        source_population_structure_json,
        "/admixture/tool_ok",
        "population-structure selected admixture execution",
    )?;
    if selected_pca_tool_id != contract.pca_tool_id
        || selected_admixture_tool_id != contract.admixture_tool_id
    {
        bail!(
            "population-structure component identity drift: expected PCA `{}` and admixture `{}`, observed PCA `{selected_pca_tool_id}` and admixture `{selected_admixture_tool_id}`",
            contract.pca_tool_id,
            contract.admixture_tool_id
        );
    }
    let ld_prune_out_path = stage_root.join("ld_prune_out.tsv");
    bijux_dna_infra::atomic_write_bytes(&ld_prune_out_path, b"")?;

    let consumed_pca =
        summarize_consumed_pca(repo_root, published_source_pca_report_path, &consumed_pca_json)?;
    let consumed_admixture = summarize_consumed_admixture(
        repo_root,
        published_source_admixture_report_path,
        &consumed_admixture_json,
    )?;
    let pca_rows = parse_pca_rows(&consumed_pca_json)?;
    let admixture_rows = parse_admixture_rows(&consumed_admixture_json)?;
    validate_consumed_stage_inputs(source_population_structure_json, &pca_rows, &admixture_rows)?;
    let sample_groups = build_sample_groups(&pca_rows, &admixture_rows)?;
    let distance_summary = build_distance_summary(&sample_groups)?;
    let status = source_population_structure_json
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("complete")
        .to_string();

    Ok(NormalizedPopulationStage {
        selected_pca_tool_id,
        selected_pca_tool_ok,
        selected_admixture_tool_id,
        selected_admixture_tool_ok,
        consumed_pca,
        consumed_admixture,
        sample_groups,
        distance_summary,
        status,
    })
}
