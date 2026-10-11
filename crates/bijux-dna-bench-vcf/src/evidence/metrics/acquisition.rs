use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn collect_retained_tools_by_stage(
    observations: &crate::adapters::observations::VcfAdapterObservations<'_>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let repo_root = observations.repo_root();
    let scratch_root = repo_root.join("artifacts/bench-readiness/vcf-comparable-metrics");
    bijux_dna_infra::ensure_dir(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;

    let mut stage_tools = BTreeMap::<String, BTreeSet<String>>::new();

    genotype_tools(observations, &mut stage_tools)?;
    population_tools(observations, &mut stage_tools, &scratch_root)?;
    haplotype_tools(observations, &mut stage_tools, &scratch_root)?;
    Ok(stage_tools)
}
fn register_stage_tool(
    stage_tools: &mut BTreeMap<String, BTreeSet<String>>,
    stage_id: &str,
    tool_id: &str,
    argv_validation_passed: bool,
    missing_input_test_passed: bool,
) {
    if argv_validation_passed && missing_input_test_passed {
        stage_tools.entry(stage_id.to_string()).or_default().insert(tool_id.to_string());
    }
}
fn genotype_tools(
    observations: &crate::adapters::observations::VcfAdapterObservations<'_>,
    stage_tools: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    for row in &observations.bcftools()?.rows {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in &observations.angsd()?.rows {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    Ok(())
}
fn population_tools(
    observations: &crate::adapters::observations::VcfAdapterObservations<'_>,
    stage_tools: &mut BTreeMap<String, BTreeSet<String>>,
    scratch_root: &std::path::Path,
) -> Result<()> {
    for row in observations.publish_plink("plink", &scratch_root.join("plink.vcf.json"))?.rows {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in observations.publish_plink("plink2", &scratch_root.join("plink2.vcf.json"))?.rows {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in observations.publish_eigensoft(&scratch_root.join("eigensoft.vcf.json"))?.rows {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    Ok(())
}
fn haplotype_tools(
    observations: &crate::adapters::observations::VcfAdapterObservations<'_>,
    stage_tools: &mut BTreeMap<String, BTreeSet<String>>,
    scratch_root: &std::path::Path,
) -> Result<()> {
    for tool_id in ["shapeit", "shapeit5", "eagle", "beagle"] {
        for row in observations
            .publish_phasing(tool_id, &scratch_root.join(format!("{tool_id}.phasing.vcf.json")))?
            .rows
        {
            register_stage_tool(
                stage_tools,
                &row.stage_id,
                &row.tool_id,
                row.argv_validation_passed,
                row.missing_input_test_passed,
            );
        }
    }
    for row in
        observations.publish_imputation(&scratch_root.join("imputation-family.vcf.json"))?.rows
    {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in observations.publish_descent(&scratch_root.join("descent-family.vcf.json"))?.rows {
        register_stage_tool(
            stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }

    Ok(())
}
