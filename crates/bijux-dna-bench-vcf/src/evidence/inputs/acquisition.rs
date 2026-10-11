use super::{
    model::VcfAdapterMissingInputTestRow,
    probes::{build_adapter_probe_row, build_support_probe_row, ProbeInput},
};
use crate::{
    adapters::observations::VcfAdapterObservations,
    fixtures::inputs::load_governed_vcf_fixture_inputs,
};
use anyhow::{anyhow, Result};
use std::path::Path;

struct AdapterProbeInputs<'a> {
    stage: &'a str,
    tool: &'a str,
    status: &'a str,
    reason: &'a str,
    inputs: Vec<ProbeInput>,
}
fn probe_roles(
    repo: &Path,
    adapter: &AdapterProbeInputs<'_>,
    roles: &[&str],
) -> Result<Vec<VcfAdapterMissingInputTestRow>> {
    roles
        .iter()
        .map(|role| {
            build_adapter_probe_row(
                repo,
                adapter.stage,
                adapter.tool,
                adapter.status,
                role,
                adapter.reason,
                &adapter.inputs,
            )
        })
        .collect()
}

pub(super) fn collect_vcf_adapter_missing_input_test_rows(
    observations: &VcfAdapterObservations<'_>,
) -> Result<Vec<VcfAdapterMissingInputTestRow>> {
    let repo_root = observations.repo_root();
    let temp_root = repo_root.join("artifacts/bench-readiness/vcf-missing-input-tests");
    let bcftools_report =
        observations.publish_bcftools(&temp_root.join("bcftools.adapter.json"))?;
    let plink2_report =
        observations.publish_plink("plink2", &temp_root.join("plink2.adapter.json"))?;
    let phasing_report =
        observations.publish_phasing("shapeit5", &temp_root.join("shapeit5.adapter.json"))?;
    let _ = observations.publish_eigensoft(&temp_root.join("eigensoft.adapter.json"))?;
    let _ = observations.publish_imputation(&temp_root.join("imputation-family.adapter.json"))?;
    let fixture_inputs = load_governed_vcf_fixture_inputs(repo_root)?;
    let mut rows = probe_roles(
        repo_root,
        &bcftools_inputs(&bcftools_report)?,
        &["input_bam", "input_bam_index", "reference_fasta", "reference_fai"],
    )?;
    rows.extend(probe_roles(
        repo_root,
        &phasing_inputs(&phasing_report)?,
        &["vcf", "vcf_index", "reference_panel_vcf", "genetic_map_tsv"],
    )?);
    rows.extend(probe_roles(
        repo_root,
        &plink_inputs(&plink2_report)?,
        &["sample_metadata_manifest"],
    )?);
    rows.push(build_support_probe_row(repo_root,"target_sites_bed",&fixture_inputs.target_sites_bed_path,"VCF target-sites BED is governed by the owned corpus fixture contract because no retained VCF adapter consumes it directly"));
    Ok(rows)
}

fn bcftools_inputs(
    report: &crate::adapters::bcftools::VcfBcftoolsAdapterReport,
) -> Result<AdapterProbeInputs<'_>> {
    let row = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.call")
        .ok_or_else(|| anyhow!("VCF bcftools adapter is missing `vcf.call`"))?;
    Ok(AdapterProbeInputs {
        stage: "vcf.call",
        tool: "bcftools",
        status: &row.benchmark_status,
        reason: &row.reason,
        inputs: row
            .required_inputs
            .iter()
            .map(|input| ProbeInput {
                artifact_id: input.artifact_id.clone(),
                path: input.path.clone(),
            })
            .collect(),
    })
}

fn phasing_inputs(
    report: &crate::adapters::phasing::VcfPhasingFamilyAdapterReport,
) -> Result<AdapterProbeInputs<'_>> {
    let row = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.phasing")
        .ok_or_else(|| anyhow!("VCF shapeit5 adapter is missing `vcf.phasing`"))?;
    Ok(AdapterProbeInputs {
        stage: "vcf.phasing",
        tool: "shapeit5",
        status: &row.benchmark_status,
        reason: &row.reason,
        inputs: row
            .required_inputs
            .iter()
            .map(|input| ProbeInput {
                artifact_id: input.artifact_id.clone(),
                path: input.path.clone(),
            })
            .collect(),
    })
}

fn plink_inputs(
    report: &crate::adapters::plink::VcfPlinkFamilyAdapterReport,
) -> Result<AdapterProbeInputs<'_>> {
    let row = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.pca")
        .ok_or_else(|| anyhow!("VCF plink2 adapter is missing `vcf.pca`"))?;
    Ok(AdapterProbeInputs {
        stage: "vcf.pca",
        tool: "plink2",
        status: &row.benchmark_status,
        reason: &row.reason,
        inputs: row
            .required_inputs
            .iter()
            .map(|input| ProbeInput {
                artifact_id: input.artifact_id.clone(),
                path: input.path.clone(),
            })
            .collect(),
    })
}
