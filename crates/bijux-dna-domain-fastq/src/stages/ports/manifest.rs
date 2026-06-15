use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub(super) struct StageManifestPort {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct StageManifestParameter {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
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
    static MANIFESTS: OnceLock<BTreeMap<&'static str, StageManifestShape>> = OnceLock::new();
    MANIFESTS
        .get_or_init(|| {
            [
                ("fastq.cluster_otus", stage_manifest!("cluster_otus.yaml")),
                ("fastq.correct_errors", stage_manifest!("correct_errors.yaml")),
                ("fastq.deplete_host", stage_manifest!("deplete_host.yaml")),
                (
                    "fastq.deplete_reference_contaminants",
                    stage_manifest!("deplete_reference_contaminants.yaml"),
                ),
                ("fastq.deplete_rrna", stage_manifest!("deplete_rrna.yaml")),
                ("fastq.detect_adapters", stage_manifest!("detect_adapters.yaml")),
                (
                    "fastq.detect_duplicates_premerge",
                    stage_manifest!("detect_duplicates_premerge.yaml"),
                ),
                (
                    "fastq.estimate_library_complexity_prealign",
                    stage_manifest!("estimate_library_complexity_prealign.yaml"),
                ),
                ("fastq.extract_umis", stage_manifest!("extract_umis.yaml")),
                ("fastq.filter_low_complexity", stage_manifest!("filter_low_complexity.yaml")),
                ("fastq.filter_reads", stage_manifest!("filter_reads.yaml")),
                ("fastq.index_reference", stage_manifest!("index_reference.yaml")),
                ("fastq.infer_asvs", stage_manifest!("infer_asvs.yaml")),
                ("fastq.merge_pairs", stage_manifest!("merge_pairs.yaml")),
                ("fastq.normalize_abundance", stage_manifest!("normalize_abundance.yaml")),
                ("fastq.normalize_primers", stage_manifest!("normalize_primers.yaml")),
                (
                    "fastq.profile_overrepresented_sequences",
                    stage_manifest!("profile_overrepresented_sequences.yaml"),
                ),
                ("fastq.profile_read_lengths", stage_manifest!("profile_read_lengths.yaml")),
                ("fastq.profile_reads", stage_manifest!("profile_reads.yaml")),
                ("fastq.remove_chimeras", stage_manifest!("remove_chimeras.yaml")),
                ("fastq.remove_duplicates", stage_manifest!("remove_duplicates.yaml")),
                ("fastq.report_qc", stage_manifest!("report_qc.yaml")),
                ("fastq.screen_taxonomy", stage_manifest!("screen_taxonomy.yaml")),
                ("fastq.trim_polyg_tails", stage_manifest!("trim_polyg_tails.yaml")),
                ("fastq.trim_reads", stage_manifest!("trim_reads.yaml")),
                ("fastq.trim_terminal_damage", stage_manifest!("trim_terminal_damage.yaml")),
                ("fastq.validate_reads", stage_manifest!("validate_reads.yaml")),
            ]
            .into_iter()
            .map(|(stage_id, raw)| {
                let manifest = bijux_dna_infra::formats::parse_yaml(raw)
                    .unwrap_or_else(|err| panic!("parse fastq stage manifest `{stage_id}`: {err}"));
                (stage_id, manifest)
            })
            .collect()
        })
        .get(stage_id)
        .cloned()
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
