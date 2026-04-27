#![allow(clippy::too_many_arguments)]

use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::{AsvInferenceEffectiveParams, EDNA_SCHEMA_VERSION};
use bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS;
use bijux_dna_domain_fastq::{ExecutionStatus, PairedMode};
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_INFER_ASVS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
pub type InferAsvsPlanOptions = crate::InferAsvsStageParams;
const DEFAULT_INFER_ASVS_THREADS: u32 = 1;

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, r1, r2, out_dir, &InferAsvsPlanOptions::baseline())
}

pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &InferAsvsPlanOptions,
) -> Result<StagePlanV1> {
    match bijux_dna_domain_fastq::execution_support_for_stage(&STAGE_ID) {
        Some(support) if support.execution_status == ExecutionStatus::Closed => {}
        _ => {
            return Err(anyhow!(
                "{} is declared-only and cannot be planned until its runtime contract closes",
                STAGE_ID.as_str()
            ));
        }
    }
    validate_infer_asvs_options(&tool.tool_id.0, options)?;
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    let asv_table = out_dir.join("asv_abundance.tsv");
    let asv_sequences = out_dir.join("asv_sequences.fasta");
    let taxonomy_ready_fasta_path = out_dir.join("taxonomy_ready.fasta");
    let taxonomy_ready_fastq_path = out_dir.join("taxonomy_ready.fastq");
    let report_json = out_dir.join("infer_asvs_report.json");
    let threads = options.threads.unwrap_or(DEFAULT_INFER_ASVS_THREADS).max(1);
    let mut resources = tool.resources.clone();
    resources.threads = threads;
    let effective_params = AsvInferenceEffectiveParams {
        schema_version: EDNA_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        denoising_method: options.denoising_method.clone(),
        pooling_mode: options.pooling_mode.clone(),
        chimera_policy: options.chimera_policy.clone(),
        threads: Some(threads),
        requires_r_runtime: true,
        output_table_kind: "asv_abundance_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("report_json".to_string()),
        raw_backend_report_format: Some("infer_asvs_governed_report_json".to_string()),
    };
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
            template: infer_asvs_command(
                &tool.tool_id.0,
                r1,
                r2,
                &asv_table,
                &asv_sequences,
                &taxonomy_ready_fasta_path,
                &taxonomy_ready_fastq_path,
                &report_json,
                threads,
                options,
            )?,
        },
        resources,
        io: StageIO {
            inputs,
            outputs: infer_asvs_output_refs(
                &asv_table,
                &asv_sequences,
                &taxonomy_ready_fasta_path,
                &taxonomy_ready_fastq_path,
                &report_json,
            ),
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "denoising_method": options.denoising_method,
            "pooling_mode": options.pooling_mode,
            "chimera_policy": options.chimera_policy,
            "threads": threads,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize infer_asvs effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "amplicon ASV inference"),
    })
}

fn validate_infer_asvs_options(tool_id: &str, options: &InferAsvsPlanOptions) -> Result<()> {
    if tool_id != "dada2" {
        return Err(anyhow!("unsupported ASV inference tool for stage planning: {tool_id}"));
    }
    if options.denoising_method != "dada2" {
        return Err(anyhow!("dada2 infer-asvs planning supports only denoising_method=dada2"));
    }
    if !matches!(options.pooling_mode.as_str(), "independent" | "pseudo_pool" | "pooled") {
        return Err(anyhow!(
            "infer-asvs planning requires pooling_mode to be one of: independent, pseudo_pool, pooled"
        ));
    }
    if !matches!(options.chimera_policy.as_str(), "remove_bimera_denovo" | "keep_candidates") {
        return Err(anyhow!(
            "infer-asvs planning requires chimera_policy to be one of: remove_bimera_denovo, keep_candidates"
        ));
    }
    Ok(())
}

fn infer_asvs_output_refs(
    asv_table: &Path,
    asv_sequences: &Path,
    taxonomy_ready_fasta_path: &Path,
    taxonomy_ready_fastq_path: &Path,
    report_json: &Path,
) -> Vec<ArtifactRef> {
    vec![
        ArtifactRef::required(
            ArtifactId::from_static("asv_table_tsv"),
            asv_table.to_path_buf(),
            ArtifactRole::SummaryTsv,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("asv_sequences_fasta"),
            asv_sequences.to_path_buf(),
            ArtifactRole::Reference,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("taxonomy_ready_fasta"),
            taxonomy_ready_fasta_path.to_path_buf(),
            ArtifactRole::Reference,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("taxonomy_ready_fastq"),
            taxonomy_ready_fastq_path.to_path_buf(),
            ArtifactRole::Reference,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("report_json"),
            report_json.to_path_buf(),
            ArtifactRole::ReportJson,
        ),
    ]
}

fn infer_asvs_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    asv_table: &Path,
    asv_sequences: &Path,
    taxonomy_ready_fasta_path: &Path,
    taxonomy_ready_fastq_path: &Path,
    report_json: &Path,
    threads: u32,
    options: &InferAsvsPlanOptions,
) -> Result<Vec<String>> {
    match tool_id {
        "dada2" => {
            let mut command = vec![
                "dada2".to_string(),
                "run_dada2.R".to_string(),
                "--input-r1".to_string(),
                r1.display().to_string(),
            ];
            if let Some(r2) = r2 {
                command.push("--input-r2".to_string());
                command.push(r2.display().to_string());
            }
            command.extend([
                "--asv-table".to_string(),
                asv_table.display().to_string(),
                "--asv-fasta".to_string(),
                asv_sequences.display().to_string(),
                "--taxonomy-ready-fasta".to_string(),
                taxonomy_ready_fasta_path.display().to_string(),
                "--taxonomy-ready-fastq".to_string(),
                taxonomy_ready_fastq_path.display().to_string(),
                "--report-json".to_string(),
                report_json.display().to_string(),
                "--denoising-method".to_string(),
                options.denoising_method.clone(),
                "--pooling-mode".to_string(),
                options.pooling_mode.clone(),
                "--chimera-policy".to_string(),
                options.chimera_policy.clone(),
                "--threads".to_string(),
                threads.to_string(),
            ]);
            Ok(command)
        }
        _ => Err(anyhow!("unsupported ASV inference tool for stage planning: {tool_id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{infer_asvs_output_refs, plan_with_options, InferAsvsPlanOptions};
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };

    #[test]
    fn infer_asvs_outputs_use_reference_roles_for_representative_sequences() {
        let outputs = infer_asvs_output_refs(
            std::path::Path::new("out/asv_abundance.tsv"),
            std::path::Path::new("out/asv_sequences.fasta"),
            std::path::Path::new("out/taxonomy_ready.fasta"),
            std::path::Path::new("out/taxonomy_ready.fastq"),
            std::path::Path::new("out/infer_asvs_report.json"),
        );

        assert_eq!(outputs[0].name.as_str(), "asv_table_tsv");
        assert_eq!(outputs[0].role.as_str(), "summary_tsv");
        assert_eq!(outputs[1].role.as_str(), "reference");
        assert_eq!(outputs[2].role.as_str(), "reference");
        assert_eq!(outputs[3].role.as_str(), "reference");
        assert_eq!(outputs[4].role.as_str(), "report_json");
    }

    #[test]
    fn infer_asvs_plan_records_governed_report_and_execution_policies() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("dada2"),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 { image: "bijuxdna/dada2".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["dada2".to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 16,
                tmp_gb: 8,
                threads: 4,
            },
        };
        let plan = plan_with_options(
            &tool,
            std::path::Path::new("reads_R1.fastq.gz"),
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out"),
            &InferAsvsPlanOptions {
                denoising_method: "dada2".to_string(),
                pooling_mode: "pseudo_pool".to_string(),
                chimera_policy: "remove_bimera_denovo".to_string(),
                threads: Some(6),
            },
        )
        .expect("infer_asvs plan");
        assert_eq!(plan.io.outputs.len(), 5);
        assert_eq!(plan.io.outputs[4].name.as_str(), "report_json");
        assert_eq!(plan.command.template[0], "dada2");
        assert!(plan.command.template.iter().any(|part| part == "--pooling-mode"));
        assert_eq!(plan.effective_params["pooling_mode"], "pseudo_pool");
        assert_eq!(plan.effective_params["threads"], 6);
    }
}
