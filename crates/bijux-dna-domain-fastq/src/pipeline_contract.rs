#![allow(dead_code)]

use std::collections::BTreeSet;

use bijux_dna_core::{
    contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec},
    ids::StageId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FastqPipelineMode {
    Shotgun,
    Amplicon,
}

impl FastqPipelineMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shotgun => "shotgun",
            Self::Amplicon => "amplicon",
        }
    }
}

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_polyg_tails"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.correct_errors"),
        StageId::from_static("fastq.extract_umis"),
        StageId::from_static("fastq.index_reference"),
        StageId::from_static("fastq.deplete_host"),
        StageId::from_static("fastq.deplete_reference_contaminants"),
        StageId::from_static("fastq.deplete_rrna"),
        StageId::from_static("fastq.merge_pairs"),
        StageId::from_static("fastq.remove_duplicates"),
        StageId::from_static("fastq.filter_low_complexity"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.screen_taxonomy"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn canonical_amplicon_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.normalize_primers"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.correct_errors"),
        StageId::from_static("fastq.extract_umis"),
        StageId::from_static("fastq.remove_chimeras"),
        StageId::from_static("fastq.cluster_otus"),
        StageId::from_static("fastq.normalize_abundance"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.screen_taxonomy"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn default_shotgun_preprocess_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_polyg_tails"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn default_amplicon_preprocess_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.normalize_primers"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.remove_chimeras"),
        StageId::from_static("fastq.cluster_otus"),
        StageId::from_static("fastq.normalize_abundance"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    vec![
        (
            StageId::from_static("fastq.merge_pairs"),
            vec![
                StageId::from_static("fastq.trim_reads"),
                StageId::from_static("fastq.filter_reads"),
            ],
        ),
        (
            StageId::from_static("fastq.correct_errors"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.extract_umis"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.report_qc"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
    ]
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(StageId, StageId)> {
    vec![
        (
            StageId::from_static("fastq.validate_reads"),
            StageId::from_static("fastq.merge_pairs"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.trim_reads"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.filter_reads"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.merge_pairs"),
        ),
        (
            StageId::from_static("fastq.infer_asvs"),
            StageId::from_static("fastq.cluster_otus"),
        ),
        (
            StageId::from_static("fastq.cluster_otus"),
            StageId::from_static("fastq.infer_asvs"),
        ),
    ]
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    match stage_id.as_str() {
        "fastq.validate_reads"
        | "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.filter_reads"
        | "fastq.normalize_primers"
        | "fastq.remove_chimeras"
        | "fastq.cluster_otus"
        | "fastq.normalize_abundance"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.report_qc" => Some(StageCriticality::Essential),
        "fastq.index_reference"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.filter_low_complexity"
        | "fastq.correct_errors"
        | "fastq.extract_umis"
        | "fastq.screen_taxonomy" => Some(StageCriticality::Optional),
        "fastq.infer_asvs" => Some(StageCriticality::Experimental),
        _ => None,
    }
}

#[must_use]
pub fn preprocess_pipeline() -> PipelineSpec {
    preprocess_pipeline_graph_for_stage_order(&canonical_stage_order())
}

#[must_use]
pub fn preprocess_pipeline_for_mode(mode: FastqPipelineMode) -> PipelineSpec {
    preprocess_pipeline_graph_for_stage_order(&match mode {
        FastqPipelineMode::Shotgun => canonical_stage_order(),
        FastqPipelineMode::Amplicon => canonical_amplicon_stage_order(),
    })
}

#[must_use]
pub fn preprocess_pipeline_graph_for_stage_order(stages: &[StageId]) -> PipelineSpec {
    let nodes = stages
        .iter()
        .map(|stage| PipelineNodeSpec {
            stage_id: stage.to_string(),
            stage_instance_id: None,
        })
        .collect::<Vec<_>>();
    let present = stages
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();

    for stage in stages {
        match stage.as_str() {
            "fastq.validate_reads" | "fastq.index_reference" | "fastq.report_qc" => {}
            "fastq.deplete_host" | "fastq.deplete_reference_contaminants" => {
                if let Some(reads_upstream) = first_present_stage(
                    &present,
                    &[
                        "fastq.filter_reads",
                        "fastq.trim_reads",
                        "fastq.validate_reads",
                    ],
                ) {
                    edges.push(pipeline_edge(reads_upstream, stage.as_str()));
                }
                if present.contains("fastq.index_reference") {
                    edges.push(pipeline_edge("fastq.index_reference", stage.as_str()));
                }
            }
            stage_id => {
                let dependency =
                    first_present_stage(&present, primary_upstream_candidates(stage_id));
                if let Some(upstream) = dependency {
                    edges.push(pipeline_edge(upstream, stage_id));
                }
            }
        }
    }

    if present.contains("fastq.report_qc") {
        let mut contributors = crate::governed_qc_producer_stage_ids()
            .into_iter()
            .map(|stage| stage.to_string())
            .filter(|stage_id| present.contains(stage_id))
            .collect::<Vec<_>>();
        contributors.sort();
        contributors.dedup();
        for contributor in contributors {
            for output_id in
                crate::governed_qc_output_ids_for_stage(&StageId::new(contributor.clone()))
            {
                edges.push(pipeline_artifact_edge(
                    &contributor,
                    "fastq.report_qc",
                    &output_id,
                    "qc_artifacts",
                ));
            }
        }
    }

    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from
            && left.to == right.to
            && left.from_output_id == right.from_output_id
            && left.to_input_id == right.to_input_id
    });

    PipelineSpec::graph(nodes, edges)
}

fn pipeline_edge(from: &str, to: &str) -> PipelineEdgeSpec {
    PipelineEdgeSpec {
        from: from.to_string(),
        to: to.to_string(),
        from_output_id: None,
        to_input_id: None,
    }
}

fn pipeline_artifact_edge(
    from: &str,
    to: &str,
    from_output_id: &str,
    to_input_id: &str,
) -> PipelineEdgeSpec {
    PipelineEdgeSpec {
        from: from.to_string(),
        to: to.to_string(),
        from_output_id: Some(from_output_id.to_string()),
        to_input_id: Some(to_input_id.to_string()),
    }
}

fn first_present_stage<'a>(
    present: &BTreeSet<String>,
    candidates: &'a [&'a str],
) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|stage_id| present.contains(*stage_id))
}

fn primary_upstream_candidates(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "fastq.profile_read_lengths" | "fastq.detect_adapters" => &["fastq.validate_reads"],
        "fastq.trim_polyg_tails" => &["fastq.detect_adapters", "fastq.validate_reads"],
        "fastq.trim_terminal_damage" => &[
            "fastq.trim_polyg_tails",
            "fastq.detect_adapters",
            "fastq.validate_reads",
        ],
        "fastq.normalize_primers" => &[
            "fastq.trim_terminal_damage",
            "fastq.detect_adapters",
            "fastq.validate_reads",
        ],
        "fastq.trim_reads" => &[
            "fastq.normalize_primers",
            "fastq.trim_terminal_damage",
            "fastq.trim_polyg_tails",
            "fastq.detect_adapters",
            "fastq.validate_reads",
        ],
        "fastq.filter_reads" => &[
            "fastq.trim_reads",
            "fastq.normalize_primers",
            "fastq.trim_terminal_damage",
            "fastq.validate_reads",
        ],
        "fastq.correct_errors"
        | "fastq.extract_umis"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.screen_taxonomy"
        | "fastq.deplete_rrna" => &[
            "fastq.filter_reads",
            "fastq.trim_reads",
            "fastq.validate_reads",
        ],
        "fastq.merge_pairs" | "fastq.remove_chimeras" => {
            &["fastq.filter_reads", "fastq.trim_reads"]
        }
        "fastq.remove_duplicates" => &[
            "fastq.merge_pairs",
            "fastq.filter_reads",
            "fastq.trim_reads",
        ],
        "fastq.filter_low_complexity" => &[
            "fastq.remove_duplicates",
            "fastq.filter_reads",
            "fastq.trim_reads",
        ],
        "fastq.cluster_otus" => &["fastq.remove_chimeras", "fastq.filter_reads"],
        "fastq.normalize_abundance" => &["fastq.cluster_otus"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::preprocess_pipeline_graph_for_stage_order;
    use bijux_dna_core::ids::StageId;

    #[test]
    fn report_qc_edges_bind_governed_artifacts_instead_of_generic_stage_edges() {
        let pipeline = preprocess_pipeline_graph_for_stage_order(&vec![
            StageId::from_static("fastq.validate_reads"),
            StageId::from_static("fastq.report_qc"),
        ]);

        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("validation_report")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("validated_reads_manifest")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
        assert!(!pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.is_none()
                && edge.to_input_id.is_none()
        }));
    }

    #[test]
    fn report_qc_keeps_parallel_artifact_joins_for_multiple_contributors() {
        let pipeline = preprocess_pipeline_graph_for_stage_order(&vec![
            StageId::from_static("fastq.detect_adapters"),
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.report_qc"),
        ]);

        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.detect_adapters"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("adapter_evidence_dir")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.profile_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("qc_json")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
    }
}
