use anyhow::{anyhow, Result};
use bijux_dna_stage_contract::StagePlanV1;
use std::path::Path;

pub(crate) fn local_stage_plans(repo_root: &Path, stage_id: &str) -> Result<Vec<StagePlanV1>> {
    macro_rules! smoke_plans {
        ($expr:expr) => {
            Ok($expr?.into_iter().map(|case| case.plan).collect::<Vec<_>>())
        };
    }

    match stage_id {
        "fastq.cluster_otus" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_cluster_otus_smoke_plans(repo_root))
        }
        "fastq.correct_errors" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_correct_errors_smoke_plans(repo_root))
        }
        "fastq.deplete_host" => {
            Ok(vec![bijux_dna_api::v1::api::fastq::local_deplete_host_plan(repo_root)?])
        }
        "fastq.deplete_reference_contaminants" => {
            Ok(vec![bijux_dna_api::v1::api::fastq::local_deplete_reference_contaminants_plan(
                repo_root,
            )?])
        }
        "fastq.deplete_rrna" => {
            Ok(vec![bijux_dna_api::v1::api::fastq::local_deplete_rrna_plan(repo_root)?])
        }
        "fastq.detect_adapters" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_detect_adapters_smoke_plans(
                repo_root
            ))
        }
        "fastq.detect_duplicates_premerge" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_detect_duplicates_premerge_smoke_plans(repo_root)
        ),
        "fastq.estimate_library_complexity_prealign" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_estimate_library_complexity_prealign_smoke_plans(
                repo_root,
            )
        ),
        "fastq.extract_umis" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_extract_umis_smoke_plans(repo_root))
        }
        "fastq.filter_low_complexity" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_filter_low_complexity_smoke_plans(repo_root)
        ),
        "fastq.filter_reads" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_filter_reads_smoke_plans(repo_root))
        }
        "fastq.index_reference" => {
            Ok(vec![bijux_dna_api::v1::api::fastq::local_index_reference_plan(repo_root)?])
        }
        "fastq.infer_asvs" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_infer_asvs_smoke_plans(repo_root))
        }
        "fastq.merge_pairs" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_merge_pairs_smoke_plans(repo_root))
        }
        "fastq.normalize_abundance" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_normalize_abundance_smoke_plans(repo_root)
        ),
        "fastq.normalize_primers" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_normalize_primers_smoke_plans(repo_root)
        ),
        "fastq.profile_overrepresented_sequences" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_profile_overrepresented_sequences_smoke_plans(
                repo_root,
            )
        ),
        "fastq.profile_read_lengths" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_profile_read_lengths_smoke_plans(repo_root)
        ),
        "fastq.profile_reads" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_profile_reads_smoke_plans(repo_root))
        }
        "fastq.remove_chimeras" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_remove_chimeras_smoke_plans(
                repo_root
            ))
        }
        "fastq.remove_duplicates" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_remove_duplicates_smoke_plans(repo_root)
        ),
        "fastq.report_qc" => {
            Ok(vec![bijux_dna_api::v1::api::fastq::local_report_qc_smoke_plan(repo_root)?])
        }
        "fastq.screen_taxonomy" => {
            Ok(vec![bijux_dna_api::v1::api::fastq::local_screen_taxonomy_plan(repo_root)?])
        }
        "fastq.trim_polyg_tails" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_trim_polyg_tails_smoke_plans(repo_root)
        ),
        "fastq.trim_reads" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_trim_reads_smoke_plans(repo_root))
        }
        "fastq.trim_terminal_damage" => smoke_plans!(
            bijux_dna_api::v1::api::fastq::local_trim_terminal_damage_smoke_plans(repo_root)
        ),
        "fastq.validate_reads" => {
            smoke_plans!(bijux_dna_api::v1::api::fastq::local_validate_reads_smoke_plans(repo_root))
        }
        "bam.align" => {
            Ok(vec![bijux_dna_api::v1::api::bam::bam_banks::local_align_plan(repo_root)?])
        }
        "bam.authenticity" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_authenticity_smoke_plans(
                repo_root,
            ))
        }
        "bam.complexity" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_complexity_smoke_plans(
                repo_root,
            ))
        }
        "bam.contamination" => {
            Ok(vec![bijux_dna_api::v1::api::bam::bam_banks::local_contamination_plan(repo_root)?])
        }
        "bam.coverage" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_coverage_smoke_plans(
                repo_root,
            ))
        }
        "bam.damage" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_damage_smoke_plans(
                repo_root,
            ))
        }
        "bam.duplication_metrics" => smoke_plans!(
            bijux_dna_api::v1::api::bam::bam_banks::local_duplication_metrics_smoke_plans(
                repo_root,
            )
        ),
        "bam.endogenous_content" => smoke_plans!(
            bijux_dna_api::v1::api::bam::bam_banks::local_endogenous_content_smoke_plans(repo_root,)
        ),
        "bam.filter" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_filter_smoke_plans(
                repo_root,
            ))
        }
        "bam.gc_bias" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_gc_bias_smoke_plans(
                repo_root,
            ))
        }
        "bam.insert_size" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_insert_size_smoke_plans(
                repo_root,
            ))
        }
        "bam.length_filter" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_length_filter_smoke_plans(
                repo_root,
            ))
        }
        "bam.mapping_summary" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_mapping_summary_smoke_plans(
                repo_root,
            ))
        }
        "bam.mapq_filter" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_mapq_filter_smoke_plans(
                repo_root,
            ))
        }
        "bam.markdup" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_markdup_smoke_plans(
                repo_root,
            ))
        }
        "bam.overlap_correction" => smoke_plans!(
            bijux_dna_api::v1::api::bam::bam_banks::local_overlap_correction_smoke_plans(repo_root,)
        ),
        "bam.qc_pre" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_qc_pre_smoke_plans(
                repo_root,
            ))
        }
        "bam.recalibration" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_recalibration_smoke_plans(
                repo_root,
            ))
        }
        "bam.sex" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_sex_smoke_plans(repo_root,))
        }
        "bam.validate" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_validate_smoke_plans(
                repo_root,
            ))
        }
        other => feature_gated_local_stage_plans(repo_root, other),
    }
}

#[cfg(feature = "bam_downstream")]
fn feature_gated_local_stage_plans(repo_root: &Path, stage_id: &str) -> Result<Vec<StagePlanV1>> {
    macro_rules! smoke_plans {
        ($expr:expr) => {
            Ok($expr?.into_iter().map(|case| case.plan).collect::<Vec<_>>())
        };
    }

    match stage_id {
        "bam.bias_mitigation" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_bias_mitigation_smoke_plans(
                repo_root,
            ))
        }
        "bam.genotyping" => {
            Ok(vec![bijux_dna_api::v1::api::bam::bam_banks::local_genotyping_plan(repo_root)?])
        }
        "bam.haplogroups" => {
            Ok(vec![bijux_dna_api::v1::api::bam::bam_banks::local_haplogroups_plan(repo_root)?])
        }
        "bam.kinship" => {
            smoke_plans!(bijux_dna_api::v1::api::bam::bam_banks::local_kinship_smoke_plans(
                repo_root,
            ))
        }
        other => Err(anyhow!("unsupported local benchmark stage `{other}`")),
    }
}

#[cfg(not(feature = "bam_downstream"))]
fn feature_gated_local_stage_plans(_repo_root: &Path, stage_id: &str) -> Result<Vec<StagePlanV1>> {
    match stage_id {
        "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship" => {
            Err(anyhow!(
                "stage `{stage_id}` requires the `bam_downstream` feature for rendered local benchmark metadata; rerun with `cargo run -p bijux-dna --features bam_downstream -- bench local render-stage-commands`"
            ))
        }
        other => Err(anyhow!("unsupported local benchmark stage `{other}`")),
    }
}
