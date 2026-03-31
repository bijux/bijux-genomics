use std::collections::BTreeSet;

use bijux_dna_core::{
    contract::{PipelineNodeSpec, PipelineSpec},
    ids::StageId,
};

use super::dependencies::{first_present_stage, primary_upstream_candidates};
use super::edges::{pipeline_artifact_edge, pipeline_edge};
use crate::pipeline_contract::{
    canonical_amplicon_stage_order, canonical_stage_order, FastqPipelineMode,
};

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
                        "fastq.correct_errors",
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
                if let Some(upstream) =
                    first_present_stage(&present, primary_upstream_candidates(stage_id))
                {
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
