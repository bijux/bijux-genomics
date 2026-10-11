use super::super::vcf_qc_ready::{render_vcf_qc_ready, DEFAULT_VCF_QC_READY_PATH};
use super::super::vcf_stats_ready::{render_vcf_stats_ready, DEFAULT_VCF_STATS_READY_PATH};
use anyhow::Result;
use bijux_dna_bench_bam_proofs::downstream::{
    observations::{
        PublishedVariantCounts, PublishedVariantMissingness, VariantCounts, VariantMissingness,
    },
    ports::VariantEvidence,
};
use std::path::Path;
pub(super) struct VariantAdapter;
impl VariantEvidence for VariantAdapter {
    fn counts(&self, root: &Path) -> Result<PublishedVariantCounts> {
        let report = render_vcf_stats_ready(root, DEFAULT_VCF_STATS_READY_PATH.into())?;
        Ok(PublishedVariantCounts {
            output_path: report.output_path,
            rows: report
                .rows
                .into_iter()
                .map(|row| VariantCounts {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    coverage_status: row.coverage_status,
                    variant_count: row.smoke_variant_count,
                    sample_count: row.smoke_sample_count,
                })
                .collect(),
        })
    }
    fn missingness(&self, root: &Path) -> Result<PublishedVariantMissingness> {
        let report = render_vcf_qc_ready(root, DEFAULT_VCF_QC_READY_PATH.into())?;
        Ok(PublishedVariantMissingness {
            output_path: report.output_path,
            rows: report
                .rows
                .into_iter()
                .map(|row| VariantMissingness {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    coverage_status: row.coverage_status,
                    sample_missingness_count: row.smoke_sample_missingness.len(),
                    variant_missingness_count: row.smoke_variant_missingness.len(),
                    sample_exclusion_threshold: row.smoke_sample_missingness_exclusion_threshold,
                    variant_exclusion_threshold: row.smoke_variant_missingness_exclusion_threshold,
                })
                .collect(),
        })
    }
}
