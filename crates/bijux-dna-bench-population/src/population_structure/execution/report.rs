//! Owner: bijux-dna-bench-population
//! Validate the stage-result contract and publish the population report artifact set.
use crate::population_structure::model::LocalVcfPopulationStructureSmokeReport;
use crate::population_structure::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::population_structure::model::DEFAULT_STAGE_RESULT_NAME;
use crate::population_structure::model::LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION;
use anyhow::Result;
use std::path::Path;

pub(crate) fn publish_report(
    repo_root: &Path,
    report: &LocalVcfPopulationStructureSmokeReport,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::PopulationStructureStageOutputs,
    output_root: &Path,
    published_output_root: &Path,
) -> Result<()> {
    let population_structure_json_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest =
        super::stage_result::stage_result(repo_root, report, stage_outputs)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&population_structure_json_path, report)?;

    crate::execution::publication::publish(
        output_root,
        published_output_root,
        &crate::execution::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_REPORT_NAME,
            schema: LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION,
            stage: &report.stage_id,
            tool: &report.tool_id,
        },
    )?;

    Ok(())
}
