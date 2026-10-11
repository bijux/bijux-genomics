use super::super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use super::super::tool_families::{validate_tool_families_path, DEFAULT_TOOL_FAMILIES_PATH};
use super::super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use super::{bam, fastq, vcf};
use crate::commands::benchmark::local_stage_commands::{
    local_stage_plans, materialize_local_stage,
};
use anyhow::{anyhow, Result};
use bijux_dna_bench_parser_proofs::{
    catalog::ProbeKind,
    ports::{MaterializedProof, ProbeMaterializer},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub(super) struct ConfiguredProducers<'a>(pub &'a Path);
impl ProbeMaterializer for ConfiguredProducers<'_> {
    fn local_stage(&self, stage_id: &str) -> Result<PathBuf> {
        materialize_local_stage(self.0, stage_id)
    }
    fn stage_tool(&self, stage_id: &str) -> Result<String> {
        local_stage_plans(self.0, stage_id)?
            .first()
            .map(|plan| plan.tool_id.as_str().to_string())
            .ok_or_else(|| anyhow!("unable to determine tool_id for stage `{stage_id}`"))
    }
    fn generate(&self, kind: ProbeKind, probe_root: &Path) -> Result<MaterializedProof> {
        match kind {
            ProbeKind::LocalStage { .. } => {
                Err(anyhow!("local stage proof requires the local-stage materialization port"))
            }
            ProbeKind::GeneratedIndexReference => {
                fastq::generate_index_reference_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedCorrectErrors => {
                fastq::generate_correct_errors_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedDepleteHost => {
                fastq::generate_deplete_host_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedDepleteRrna => {
                fastq::generate_deplete_rrna_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedScreenTaxonomy => {
                fastq::generate_screen_taxonomy_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedBamAlign => bam::generate_bam_align_probe(self.0, probe_root),
            ProbeKind::GeneratedBamContamination => {
                bam::generate_bam_contamination_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedBamDamage => bam::generate_bam_damage_probe(self.0, probe_root),
            ProbeKind::GeneratedBamAuthenticity => {
                bam::generate_bam_authenticity_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedBamGenotyping => {
                bam::generate_bam_genotyping_probe(self.0, probe_root)
            }
            ProbeKind::GeneratedBamKinship => bam::generate_bam_kinship_probe(self.0, probe_root),
            ProbeKind::GeneratedBamSex => bam::generate_bam_sex_probe(self.0, probe_root),
            ProbeKind::GeneratedVcfCall => vcf::generate_vcf_call_probe(self.0),
            ProbeKind::GeneratedVcfDemography => vcf::generate_vcf_demography_probe(self.0),
            ProbeKind::GeneratedVcfIbd => vcf::generate_vcf_ibd_probe(self.0),
            ProbeKind::GeneratedVcfImpute => vcf::generate_vcf_impute_probe(self.0),
            ProbeKind::GeneratedVcfPhasing => vcf::generate_vcf_phasing_probe(self.0),
            ProbeKind::GeneratedVcfPopulationStructure => {
                vcf::generate_vcf_population_structure_probe(self.0)
            }
            ProbeKind::GeneratedVcfQc => vcf::generate_vcf_qc_probe(self.0),
        }
    }
}

pub(super) fn collect_active_families(repo_root: &Path) -> Result<Vec<(String, String)>> {
    let config_path = repo_root.join(DEFAULT_TOOL_FAMILIES_PATH);
    let family_report = validate_tool_families_path(repo_root, &config_path)?;
    let active_tool_ids = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| row.tool_id)
        .chain(
            collect_vcf_expected_benchmark_result_rows(repo_root)?
                .into_iter()
                .map(|row| row.tool_id),
        )
        .collect::<BTreeSet<_>>();

    let mut families = BTreeMap::<String, String>::new();
    for row in family_report.rows {
        if active_tool_ids.contains(&row.tool_id) {
            families.entry(row.family_id).or_insert(row.family_summary);
        }
    }
    Ok(families.into_iter().collect())
}
