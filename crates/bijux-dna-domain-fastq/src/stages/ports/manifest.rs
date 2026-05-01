use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct StageManifestPort {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct StageManifestParameter {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct StageManifestShape {
    #[serde(default)]
    pub inputs: Vec<StageManifestPort>,
    #[serde(default)]
    pub outputs: Vec<StageManifestPort>,
    #[serde(default)]
    pub parameters: Vec<StageManifestParameter>,
    #[serde(default)]
    pub compatible_tools: Vec<String>,
}

macro_rules! stage_manifest {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../domain/fastq/stages/", $path))
    };
}

pub(super) fn parse_manifest(stage_id: &str) -> Option<StageManifestShape> {
    let raw = match stage_id {
        "fastq.cluster_otus" => Some(stage_manifest!("cluster_otus.yaml")),
        "fastq.correct_errors" => Some(stage_manifest!("correct_errors.yaml")),
        "fastq.deplete_host" => Some(stage_manifest!("deplete_host.yaml")),
        "fastq.deplete_reference_contaminants" => {
            Some(stage_manifest!("deplete_reference_contaminants.yaml"))
        }
        "fastq.deplete_rrna" => Some(stage_manifest!("deplete_rrna.yaml")),
        "fastq.detect_adapters" => Some(stage_manifest!("detect_adapters.yaml")),
        "fastq.detect_duplicates_premerge" => {
            Some(stage_manifest!("detect_duplicates_premerge.yaml"))
        }
        "fastq.estimate_library_complexity_prealign" => {
            Some(stage_manifest!("estimate_library_complexity_prealign.yaml"))
        }
        "fastq.extract_umis" => Some(stage_manifest!("extract_umis.yaml")),
        "fastq.filter_low_complexity" => Some(stage_manifest!("filter_low_complexity.yaml")),
        "fastq.filter_reads" => Some(stage_manifest!("filter_reads.yaml")),
        "fastq.index_reference" => Some(stage_manifest!("index_reference.yaml")),
        "fastq.infer_asvs" => Some(stage_manifest!("infer_asvs.yaml")),
        "fastq.merge_pairs" => Some(stage_manifest!("merge_pairs.yaml")),
        "fastq.normalize_abundance" => Some(stage_manifest!("normalize_abundance.yaml")),
        "fastq.normalize_primers" => Some(stage_manifest!("normalize_primers.yaml")),
        "fastq.profile_overrepresented_sequences" => {
            Some(stage_manifest!("profile_overrepresented_sequences.yaml"))
        }
        "fastq.profile_read_lengths" => Some(stage_manifest!("profile_read_lengths.yaml")),
        "fastq.profile_reads" => Some(stage_manifest!("profile_reads.yaml")),
        "fastq.remove_chimeras" => Some(stage_manifest!("remove_chimeras.yaml")),
        "fastq.remove_duplicates" => Some(stage_manifest!("remove_duplicates.yaml")),
        "fastq.report_qc" => Some(stage_manifest!("report_qc.yaml")),
        "fastq.screen_taxonomy" => Some(stage_manifest!("screen_taxonomy.yaml")),
        "fastq.trim_polyg_tails" => Some(stage_manifest!("trim_polyg_tails.yaml")),
        "fastq.trim_reads" => Some(stage_manifest!("trim_reads.yaml")),
        "fastq.trim_terminal_damage" => Some(stage_manifest!("trim_terminal_damage.yaml")),
        "fastq.validate_reads" => Some(stage_manifest!("validate_reads.yaml")),
        _ => None,
    }?;

    bijux_dna_infra::formats::parse_yaml(raw).ok()
}

#[cfg(test)]
mod tests {
    use super::parse_manifest;

    #[test]
    fn planned_duplicate_and_complexity_manifests_are_loadable() {
        for stage_id in
            ["fastq.detect_duplicates_premerge", "fastq.estimate_library_complexity_prealign"]
        {
            assert!(parse_manifest(stage_id).is_some(), "missing manifest for {stage_id}");
        }
    }
}
