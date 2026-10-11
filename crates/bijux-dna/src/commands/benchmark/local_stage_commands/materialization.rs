use super::{planning::local_stage_plans, qc::materialize_local_report_qc_smoke_report};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub(crate) fn materialize_local_stage(repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
    match stage_id {
        "fastq.cluster_otus" => {
            bijux_dna_api::v1::api::fastq::write_local_cluster_otus_smoke_report()
        }
        "fastq.correct_errors" => materialize_local_stage_from_plans(repo_root, stage_id),
        "fastq.deplete_host" => materialize_local_stage_from_plans(repo_root, stage_id),
        "fastq.deplete_reference_contaminants" => {
            materialize_local_stage_from_plans(repo_root, stage_id)
        }
        "fastq.deplete_rrna" => materialize_local_stage_from_plans(repo_root, stage_id),
        "fastq.detect_adapters" => {
            bijux_dna_api::v1::api::fastq::write_local_detect_adapters_smoke_report()
        }
        "fastq.detect_duplicates_premerge" => {
            bijux_dna_api::v1::api::fastq::write_local_detect_duplicates_premerge_smoke_report()
        }
        "fastq.estimate_library_complexity_prealign" => {
            bijux_dna_api::v1::api::fastq::write_local_estimate_library_complexity_prealign_smoke_report(
            )
        }
        "fastq.extract_umis" => {
            bijux_dna_api::v1::api::fastq::write_local_extract_umis_smoke_report()
        }
        "fastq.filter_low_complexity" => {
            bijux_dna_api::v1::api::fastq::write_local_filter_low_complexity_smoke_report()
        }
        "fastq.filter_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_filter_reads_smoke_report()
        }
        "fastq.index_reference" => materialize_local_stage_from_plans(repo_root, stage_id),
        "fastq.infer_asvs" => {
            bijux_dna_api::v1::api::fastq::write_local_infer_asvs_smoke_report()
        }
        "fastq.merge_pairs" => {
            bijux_dna_api::v1::api::fastq::write_local_merge_pairs_smoke_report()
        }
        "fastq.normalize_abundance" => {
            bijux_dna_api::v1::api::fastq::write_local_normalize_abundance_smoke_report()
        }
        "fastq.normalize_primers" => {
            bijux_dna_api::v1::api::fastq::write_local_normalize_primers_smoke_report()
        }
        "fastq.profile_overrepresented_sequences" => bijux_dna_api::v1::api::fastq::write_local_profile_overrepresented_sequences_smoke_summary(),
        "fastq.profile_read_lengths" => {
            bijux_dna_api::v1::api::fastq::write_local_profile_read_lengths_smoke_summary()
        }
        "fastq.profile_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_profile_reads_smoke_report()
        }
        "fastq.remove_chimeras" => {
            bijux_dna_api::v1::api::fastq::write_local_remove_chimeras_smoke_report()
        }
        "fastq.remove_duplicates" => {
            bijux_dna_api::v1::api::fastq::write_local_remove_duplicates_smoke_report()
        }
        "fastq.report_qc" => materialize_local_report_qc_smoke_report(repo_root),
        "fastq.screen_taxonomy" => materialize_local_stage_from_plans(repo_root, stage_id),
        "fastq.trim_polyg_tails" => {
            bijux_dna_api::v1::api::fastq::write_local_trim_polyg_tails_smoke_report()
        }
        "fastq.trim_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_trim_reads_smoke_report()
        }
        "fastq.trim_terminal_damage" => {
            bijux_dna_api::v1::api::fastq::write_local_trim_terminal_damage_smoke_report()
        }
        "fastq.validate_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_validate_reads_smoke_report()
        }
        "bam.align" => materialize_local_stage_from_plans(repo_root, stage_id),
        "bam.authenticity" => {
            bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()
        }
        "bam.complexity" => {
            bijux_dna_api::v1::api::bam::write_local_complexity_smoke_report()
        }
        "bam.contamination" => materialize_local_stage_from_plans(repo_root, stage_id),
        "bam.coverage" => bijux_dna_api::v1::api::bam::write_local_coverage_smoke_summary(),
        "bam.damage" => bijux_dna_api::v1::api::bam::write_local_damage_smoke_report(),
        "bam.duplication_metrics" => {
            bijux_dna_api::v1::api::bam::write_local_duplication_metrics_smoke_report()
        }
        "bam.endogenous_content" => {
            bijux_dna_api::v1::api::bam::write_local_endogenous_content_smoke_report()
        }
        "bam.filter" => bijux_dna_api::v1::api::bam::write_local_filter_smoke_report(),
        "bam.gc_bias" => bijux_dna_api::v1::api::bam::write_local_gc_bias_smoke_summary(),
        "bam.insert_size" => {
            bijux_dna_api::v1::api::bam::write_local_insert_size_smoke_report()
        }
        "bam.length_filter" => {
            bijux_dna_api::v1::api::bam::write_local_length_filter_smoke_report()
        }
        "bam.mapping_summary" => {
            bijux_dna_api::v1::api::bam::write_local_mapping_summary_smoke_summary()
        }
        "bam.mapq_filter" => {
            bijux_dna_api::v1::api::bam::write_local_mapq_filter_smoke_report()
        }
        "bam.markdup" => bijux_dna_api::v1::api::bam::write_local_markdup_smoke_report(),
        "bam.overlap_correction" => {
            bijux_dna_api::v1::api::bam::write_local_overlap_correction_smoke_report()
        }
        "bam.qc_pre" => bijux_dna_api::v1::api::bam::write_local_qc_pre_smoke_report(),
        "bam.recalibration" => {
            bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()
        }
        "bam.sex" => bijux_dna_api::v1::api::bam::write_local_sex_smoke_report(),
        "bam.validate" => bijux_dna_api::v1::api::bam::write_local_validate_smoke_report(),
        other => materialize_feature_gated_stage(repo_root, other),
    }
}

#[cfg(feature = "bam_downstream")]
fn materialize_feature_gated_stage(_repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
    match stage_id {
        "bam.bias_mitigation" => {
            bijux_dna_api::v1::api::bam::write_local_bias_mitigation_smoke_report()
        }
        "bam.genotyping" => bijux_dna_api::v1::api::bam::write_local_genotyping_plan(),
        "bam.haplogroups" => bijux_dna_api::v1::api::bam::write_local_haplogroups_plan(),
        "bam.kinship" => bijux_dna_api::v1::api::bam::write_local_kinship_smoke_report(),
        other => Err(anyhow!("unsupported local benchmark stage `{other}`")),
    }
}

#[cfg(not(feature = "bam_downstream"))]
fn materialize_feature_gated_stage(_repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
    match stage_id {
        "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship" => {
            Err(anyhow!(
                "stage `{stage_id}` requires the `bam_downstream` feature; rerun with `cargo run -p bijux-dna --features bam_downstream -- ...`"
            ))
        }
        other => Err(anyhow!("unsupported local benchmark stage `{other}`")),
    }
}

fn materialize_local_stage_from_plans(repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
    bijux_dna_bench_stage::execution::materialize_plan_fixtures(
        repo_root,
        stage_id,
        &local_stage_plans(repo_root, stage_id)?,
    )
}
