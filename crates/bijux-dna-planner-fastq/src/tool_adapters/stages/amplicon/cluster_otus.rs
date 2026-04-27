use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::{OtuClusteringEffectiveParams, EDNA_SCHEMA_VERSION};
use bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_CLUSTER_OTUS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
pub type ClusterOtusPlanOptions = crate::ClusterOtusStageParams;
const DEFAULT_CLUSTER_OTUS_THREADS: u32 = 4;

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, r1, r2, out_dir, &ClusterOtusPlanOptions::baseline())
}

pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &ClusterOtusPlanOptions,
) -> Result<StagePlanV1> {
    if r2.is_some() {
        return Err(anyhow!(
            "vsearch OTU clustering planning requires a single merged or single-end input"
        ));
    }
    validate_cluster_otus_options(&tool.tool_id.0, options)?;
    let inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    let otu_table = out_dir.join("otu_abundance.tsv");
    let otu_representatives = out_dir.join("otu_representatives.fasta");
    let taxonomy_ready_fasta_path = out_dir.join("taxonomy_ready.fasta");
    let taxonomy_ready_fastq_path = out_dir.join("taxonomy_ready.fastq");
    let report_json = out_dir.join("cluster_otus_report.json");
    let otu_clusters_uc = out_dir.join("otu_clusters.uc");
    let threads = options.threads.unwrap_or(DEFAULT_CLUSTER_OTUS_THREADS).max(1);
    let effective_params = OtuClusteringEffectiveParams {
        schema_version: EDNA_SCHEMA_VERSION.to_string(),
        identity_threshold: options.otu_identity,
        threads,
        output_table_kind: "otu_abundance_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("otu_clusters_uc".to_string()),
        raw_backend_report_format: Some("vsearch_uc".to_string()),
    };
    let mut resources = tool.resources.clone();
    resources.threads = threads;
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
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
                &otu_clusters_uc,
                &options.otu_identity.to_string(),
                threads,
            )?,
        },
        resources,
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
                    ArtifactRole::Reference,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fasta"),
                    taxonomy_ready_fasta_path.clone(),
                    ArtifactRole::Reference,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fastq"),
                    taxonomy_ready_fastq_path.clone(),
                    ArtifactRole::Reference,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report_json.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "otu_identity": options.otu_identity,
            "threads": threads,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize cluster_otus effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "amplicon OTU clustering"),
    })
}

fn validate_cluster_otus_options(tool_id: &str, options: &ClusterOtusPlanOptions) -> Result<()> {
    if tool_id != "vsearch" {
        return Err(anyhow!("unsupported OTU clustering tool for stage planning: {tool_id}"));
    }
    if !(0.0 < options.otu_identity && options.otu_identity <= 1.0) {
        return Err(anyhow!(
            "cluster-otus planning requires otu_identity to be in the range (0, 1]"
        ));
    }
    Ok(())
}

fn cluster_otus_command(
    tool_id: &str,
    reads: &Path,
    otu_table: &Path,
    otu_representatives: &Path,
    otu_clusters_uc: &Path,
    otu_identity_arg: &str,
    threads: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "vsearch" => Ok(vec![
            "vsearch".to_string(),
            "--cluster_fast".to_string(),
            reads.display().to_string(),
            "--id".to_string(),
            otu_identity_arg.to_string(),
            "--sizein".to_string(),
            "--sizeout".to_string(),
            "--relabel".to_string(),
            "OTU_".to_string(),
            "--threads".to_string(),
            threads.to_string(),
            "--centroids".to_string(),
            otu_representatives.display().to_string(),
            "--uc".to_string(),
            otu_clusters_uc.display().to_string(),
            "--otutabout".to_string(),
            otu_table.display().to_string(),
        ]),
        _ => Err(anyhow!("unsupported OTU clustering tool for stage planning: {tool_id}")),
    }
}
