use bijux_dna_core::prelude::id_catalog;

pub(crate) fn append_stage_once(stages: &mut Vec<String>, stage_id: &str) {
    if !stages.iter().any(|stage| stage == stage_id) {
        stages.push(stage_id.to_string());
    }
}

pub(crate) fn default_shotgun_required_stages() -> Vec<String> {
    bijux_dna_domain_fastq::default_shotgun_preprocess_stage_order()
        .into_iter()
        .map(|stage| match stage.as_str() {
            "fastq.validate_reads" => id_catalog::FASTQ_VALIDATE_PRE.to_string(),
            "fastq.profile_read_lengths" => "fastq.profile_read_lengths".to_string(),
            "fastq.detect_adapters" => id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
            "fastq.trim_polyg_tails" => "fastq.trim_polyg_tails".to_string(),
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

pub(crate) fn minimal_shotgun_required_stages() -> Vec<String> {
    vec![
        id_catalog::FASTQ_VALIDATE_PRE.to_string(),
        id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
        id_catalog::FASTQ_TRIM.to_string(),
        id_catalog::FASTQ_FILTER.to_string(),
        id_catalog::FASTQ_QC_POST.to_string(),
    ]
}

pub(crate) fn qc_only_required_stages() -> Vec<String> {
    vec![
        id_catalog::FASTQ_VALIDATE_PRE.to_string(),
        "fastq.profile_read_lengths".to_string(),
        id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
        id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
        "fastq.profile_overrepresented_sequences".to_string(),
        id_catalog::FASTQ_QC_POST.to_string(),
    ]
}

pub(crate) fn umi_required_stages() -> Vec<String> {
    let mut stages = qc_only_required_stages();
    append_stage_once(&mut stages, id_catalog::FASTQ_UMI);
    append_stage_once(&mut stages, id_catalog::FASTQ_TRIM);
    append_stage_once(&mut stages, id_catalog::FASTQ_FILTER);
    stages
}

fn depletion_required_stages(stage_id: &str) -> Vec<String> {
    let mut stages = default_shotgun_required_stages();
    append_stage_once(&mut stages, stage_id);
    stages
}

pub(crate) fn host_depletion_required_stages() -> Vec<String> {
    depletion_required_stages("fastq.deplete_host")
}

pub(crate) fn rrna_depletion_required_stages() -> Vec<String> {
    depletion_required_stages("fastq.deplete_rrna")
}

pub(crate) fn contaminant_depletion_required_stages() -> Vec<String> {
    depletion_required_stages("fastq.deplete_reference_contaminants")
}

pub(crate) fn amplicon_standard_required_stages() -> Vec<String> {
    vec![
        id_catalog::FASTQ_VALIDATE_PRE.to_string(),
        "fastq.normalize_primers".to_string(),
        id_catalog::FASTQ_FILTER.to_string(),
        id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
        "fastq.remove_chimeras".to_string(),
        "fastq.infer_asvs".to_string(),
        "fastq.normalize_abundance".to_string(),
        id_catalog::FASTQ_QC_POST.to_string(),
    ]
}

pub(crate) fn amplicon_umi_required_stages() -> Vec<String> {
    let mut stages = amplicon_standard_required_stages();
    stages.insert(1, id_catalog::FASTQ_UMI.to_string());
    stages
}

pub(crate) fn edna_metabarcoding_required_stages() -> Vec<String> {
    vec![
        id_catalog::FASTQ_VALIDATE_PRE.to_string(),
        "fastq.normalize_primers".to_string(),
        id_catalog::FASTQ_FILTER.to_string(),
        id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
        "fastq.remove_chimeras".to_string(),
        "fastq.cluster_otus".to_string(),
        "fastq.normalize_abundance".to_string(),
        id_catalog::FASTQ_SCREEN.to_string(),
        id_catalog::FASTQ_QC_POST.to_string(),
    ]
}
