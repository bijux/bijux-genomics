use bijux_dna_core::prelude::id_catalog;

pub(super) fn append_stage_once(stages: &mut Vec<String>, stage_id: &str) {
    if !stages.iter().any(|stage| stage == stage_id) {
        stages.push(stage_id.to_string());
    }
}

pub(super) fn default_shotgun_required_stages() -> Vec<String> {
    bijux_dna_domain_fastq::default_shotgun_preprocess_stage_order()
        .into_iter()
        .map(|stage| match stage.as_str() {
            "fastq.validate_reads" => id_catalog::FASTQ_VALIDATE_PRE.to_string(),
            "fastq.profile_read_lengths" => "fastq.profile_read_lengths".to_string(),
            "fastq.detect_adapters" => id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
            "fastq.trim_polyg_tails" => "fastq.trim_polyg_tails".to_string(),
            "fastq.trim_terminal_damage" => "fastq.trim_terminal_damage".to_string(),
            "fastq.trim_reads" => id_catalog::FASTQ_TRIM.to_string(),
            "fastq.filter_reads" => id_catalog::FASTQ_FILTER.to_string(),
            "fastq.profile_reads" => id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
            "fastq.profile_overrepresented_sequences" => {
                "fastq.profile_overrepresented_sequences".to_string()
            }
            "fastq.report_qc" => id_catalog::FASTQ_QC_POST.to_string(),
            other => other.to_string(),
        })
        .collect()
}
