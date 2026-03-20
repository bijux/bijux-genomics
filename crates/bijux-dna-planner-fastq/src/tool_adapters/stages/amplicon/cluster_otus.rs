use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::{
    OtuClusteringEffectiveParams, DEFAULT_OTU_IDENTITY_THRESHOLD,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_CLUSTER_OTUS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
const DEFAULT_OTU_IDENTITY_THRESHOLD_ARG: &str = "0.97";

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    if r2.is_some() {
        return Err(anyhow!(
            "vsearch OTU clustering planning requires a single merged or single-end input"
        ));
    }
    let inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    let otu_table = out_dir.join("otu_abundance.tsv");
    let otu_representatives = out_dir.join("otu_representatives.fasta");
    let taxonomy_ready_fasta = out_dir.join("taxonomy_ready.fasta");
    let taxonomy_ready_fastq = out_dir.join("taxonomy_ready.fastq");
    let effective_params = OtuClusteringEffectiveParams {
        identity_threshold: DEFAULT_OTU_IDENTITY_THRESHOLD,
        output_table_kind: "otu_abundance_table".to_string(),
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: cluster_otus_command(
                &tool.tool_id.0,
                r1,
                &otu_table,
                &otu_representatives,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("otu_table"),
                    otu_table.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("otu_representatives"),
                    otu_representatives.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fasta"),
                    taxonomy_ready_fasta.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fastq"),
                    taxonomy_ready_fastq.clone(),
                    ArtifactRole::Reads,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "otu_identity": DEFAULT_OTU_IDENTITY_THRESHOLD,
        }),
        effective_params: serde_json::json!({
            "identity_threshold": effective_params.identity_threshold,
            "output_table_kind": effective_params.output_table_kind,
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "output_naming": "deterministic"
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon OTU clustering",
        ),
    })
}

fn cluster_otus_command(
    tool_id: &str,
    reads: &Path,
    otu_table: &Path,
    otu_representatives: &Path,
) -> Result<Vec<String>> {
    match tool_id {
        "vsearch" => Ok(vec![
            "vsearch".to_string(),
            "--cluster_fast".to_string(),
            reads.display().to_string(),
            "--id".to_string(),
            DEFAULT_OTU_IDENTITY_THRESHOLD_ARG.to_string(),
            "--sizein".to_string(),
            "--sizeout".to_string(),
            "--relabel".to_string(),
            "OTU_".to_string(),
            "--centroids".to_string(),
            otu_representatives.display().to_string(),
            "--otutabout".to_string(),
            otu_table.display().to_string(),
        ]),
        _ => Err(anyhow!(
            "unsupported OTU clustering tool for stage planning: {tool_id}"
        )),
    }
}
