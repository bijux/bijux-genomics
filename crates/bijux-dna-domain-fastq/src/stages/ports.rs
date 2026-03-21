use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StageManifestPort {
    name: String,
}

#[derive(Debug, Deserialize)]
struct StageManifestParameter {
    name: String,
}

#[derive(Debug, Deserialize)]
struct StageManifestShape {
    #[serde(default)]
    inputs: Vec<StageManifestPort>,
    #[serde(default)]
    outputs: Vec<StageManifestPort>,
    #[serde(default)]
    parameters: Vec<StageManifestParameter>,
    #[serde(default)]
    compatible_tools: Vec<String>,
}

macro_rules! stage_manifest {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../domain/fastq/stages/",
            $path
        ))
    };
}

fn manifest_yaml(stage_id: &str) -> Option<&'static str> {
    match stage_id {
        "fastq.cluster_otus" => Some(stage_manifest!("cluster_otus.yaml")),
        "fastq.correct_errors" => Some(stage_manifest!("correct_errors.yaml")),
        "fastq.deplete_host" => Some(stage_manifest!("deplete_host.yaml")),
        "fastq.deplete_reference_contaminants" => {
            Some(stage_manifest!("deplete_reference_contaminants.yaml"))
        }
        "fastq.deplete_rrna" => Some(stage_manifest!("deplete_rrna.yaml")),
        "fastq.detect_adapters" => Some(stage_manifest!("detect_adapters.yaml")),
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
    }
}

fn parse_manifest(stage_id: &str) -> Option<StageManifestShape> {
    let raw = manifest_yaml(stage_id)?;
    serde_yaml::from_str(raw).ok()
}

#[must_use]
pub fn stage_input_ids(stage_id: &str) -> Option<BTreeSet<String>> {
    parse_manifest(stage_id).map(|manifest| {
        manifest
            .inputs
            .into_iter()
            .map(|port| port.name)
            .collect::<BTreeSet<_>>()
    })
}

#[must_use]
pub fn stage_output_ids(stage_id: &str) -> Option<BTreeSet<String>> {
    stage_output_ids_in_manifest_order(stage_id).map(|outputs| outputs.into_iter().collect())
}

#[must_use]
pub fn stage_output_ids_in_manifest_order(stage_id: &str) -> Option<Vec<String>> {
    parse_manifest(stage_id).map(|manifest| {
        manifest
            .outputs
            .into_iter()
            .map(|port| port.name)
            .collect::<Vec<_>>()
    })
}

#[must_use]
pub fn stage_parameter_ids(stage_id: &str) -> Option<BTreeSet<String>> {
    parse_manifest(stage_id).map(|manifest| {
        manifest
            .parameters
            .into_iter()
            .map(|parameter| parameter.name)
            .collect::<BTreeSet<_>>()
    })
}

#[must_use]
pub fn stage_compatible_tool_ids(stage_id: &str) -> Option<Vec<String>> {
    parse_manifest(stage_id).map(|manifest| manifest.compatible_tools)
}

#[cfg(test)]
mod tests {
    use super::{
        stage_compatible_tool_ids, stage_input_ids, stage_output_ids,
        stage_output_ids_in_manifest_order, stage_parameter_ids,
    };

    #[test]
    fn stage_ports_follow_governed_manifest_names() {
        assert_eq!(
            stage_input_ids("fastq.report_qc"),
            Some(["qc_artifacts"].into_iter().map(str::to_string).collect())
        );
        assert_eq!(
            stage_output_ids("fastq.trim_reads"),
            Some(
                ["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            )
        );
        assert_eq!(
            stage_output_ids_in_manifest_order("fastq.report_qc"),
            Some(vec![
                "multiqc_report".to_string(),
                "multiqc_data".to_string(),
                "governed_qc_inputs_manifest".to_string()
            ])
        );
        assert_eq!(
            stage_parameter_ids("fastq.trim_reads"),
            Some(
                [
                    "min_length",
                    "quality_cutoff",
                    "adapter_policy",
                    "polyx_policy",
                    "n_policy",
                    "contaminant_policy",
                ]
                .into_iter()
                .map(str::to_string)
                .collect()
            )
        );
        assert_eq!(
            stage_compatible_tool_ids("fastq.remove_duplicates"),
            Some(vec!["fastuniq".to_string(), "clumpify".to_string()])
        );
    }
}
