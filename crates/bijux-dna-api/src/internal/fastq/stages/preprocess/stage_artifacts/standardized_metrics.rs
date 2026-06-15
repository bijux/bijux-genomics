use super::super::stage_backend_policy::{
    parse_cluster_otus_metrics, parse_correct_errors_metrics, parse_deplete_host_metrics,
    parse_deplete_reference_contaminants_metrics, parse_deplete_rrna_metrics,
    parse_detect_adapters_metrics, parse_detect_duplicates_premerge_metrics,
    parse_estimate_library_complexity_prealign_metrics, parse_extract_umis_metrics,
    parse_filter_low_complexity_metrics, parse_filter_reads_metrics, parse_index_reference_metrics,
    parse_infer_asvs_metrics, parse_merge_pairs_metrics, parse_normalize_abundance_metrics,
    parse_normalize_primers_metrics, parse_profile_overrepresented_metrics,
    parse_profile_read_lengths_metrics, parse_profile_reads_metrics, parse_remove_chimeras_metrics,
    parse_remove_duplicates_metrics, parse_report_qc_metrics, parse_screen_taxonomy_metrics,
    parse_trim_polyg_metrics, parse_trim_reads_metrics, parse_trim_terminal_damage_metrics,
    parse_validate_reads_metrics,
};
use super::{Context, Result, StageResultV1};

pub(super) fn write_stage_standardized_metrics(
    stage_root: &std::path::Path,
    stage_id: &str,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<()> {
    let metrics = match stage_id {
        "fastq.index_reference" => parse_index_reference_metrics(out_dir),
        "fastq.validate_reads" => parse_validate_reads_metrics(out_dir, execution),
        "fastq.detect_duplicates_premerge" => parse_detect_duplicates_premerge_metrics(out_dir),
        "fastq.estimate_library_complexity_prealign" => {
            parse_estimate_library_complexity_prealign_metrics(out_dir)
        }
        "fastq.detect_adapters" => parse_detect_adapters_metrics(out_dir),
        "fastq.profile_read_lengths" => parse_profile_read_lengths_metrics(out_dir),
        "fastq.profile_overrepresented_sequences" => parse_profile_overrepresented_metrics(out_dir),
        "fastq.trim_polyg_tails" => parse_trim_polyg_metrics(out_dir),
        "fastq.screen_taxonomy" => parse_screen_taxonomy_metrics(out_dir),
        "fastq.filter_low_complexity" => parse_filter_low_complexity_metrics(out_dir),
        "fastq.trim_reads" => parse_trim_reads_metrics(out_dir),
        "fastq.filter_reads" => parse_filter_reads_metrics(out_dir),
        "fastq.correct_errors" => parse_correct_errors_metrics(out_dir),
        "fastq.merge_pairs" => parse_merge_pairs_metrics(out_dir),
        "fastq.remove_duplicates" => parse_remove_duplicates_metrics(out_dir),
        "fastq.extract_umis" => parse_extract_umis_metrics(out_dir),
        "fastq.deplete_host" => parse_deplete_host_metrics(out_dir),
        "fastq.deplete_reference_contaminants" => {
            parse_deplete_reference_contaminants_metrics(out_dir)
        }
        "fastq.deplete_rrna" => parse_deplete_rrna_metrics(out_dir),
        "fastq.profile_reads" => parse_profile_reads_metrics(out_dir),
        "fastq.report_qc" => parse_report_qc_metrics(out_dir),
        "fastq.normalize_primers" => parse_normalize_primers_metrics(out_dir),
        "fastq.normalize_abundance" => parse_normalize_abundance_metrics(out_dir),
        "fastq.trim_terminal_damage" => parse_trim_terminal_damage_metrics(out_dir),
        "fastq.remove_chimeras" => parse_remove_chimeras_metrics(out_dir),
        "fastq.infer_asvs" => parse_infer_asvs_metrics(out_dir),
        "fastq.cluster_otus" => parse_cluster_otus_metrics(out_dir),
        _ => return Ok(()),
    };
    bijux_dna_infra::atomic_write_json(
        &stage_root.join("stage.metrics.standardized.json"),
        &metrics,
    )
    .context("write standardized stage metrics")
}

pub(super) fn discover_screen_taxonomy_report_path(
    stage_root: &std::path::Path,
    outputs: &[std::path::PathBuf],
) -> Option<std::path::PathBuf> {
    outputs
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".classifications.json"))
        })
        .cloned()
        .or_else(|| {
            [
                "kraken2.classifications.json",
                "krakenuniq.classifications.json",
                "centrifuge.classifications.json",
                "kaiju.classifications.json",
                "classification_report.json",
            ]
            .into_iter()
            .map(|name| stage_root.join(name))
            .find(|path| path.exists())
        })
}
