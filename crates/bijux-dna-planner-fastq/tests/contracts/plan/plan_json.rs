use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId,
    ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::FastqPipelineMode;
use bijux_dna_domain_fastq::{STAGE_TRIM_READS, STAGE_VALIDATE_READS};
use bijux_dna_stage_contract::StagePlanV1;

#[allow(clippy::duplicate_mod)]
#[path = "../../support/tool_registry.rs"]
mod tool_registry_support;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-planner-fastq__{group}__{name}")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn tool_registry() -> &'static bijux_dna_core::contract::ToolRegistry {
    static REGISTRY: OnceLock<bijux_dna_core::contract::ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        tool_registry_support::load_domain_tool_registry(&workspace_root())
            .unwrap_or_else(|error| panic!("load domain tool registry: {error}"))
    })
}

fn domain_tool(stage: &str, tool: &str) -> ToolExecutionSpecV1 {
    let stage_id = StageId::new(stage);
    let tool_id = ToolId::new(tool);
    let manifest = tool_registry()
        .tool_by_id(&stage_id, &tool_id)
        .unwrap_or_else(|| panic!("missing tool manifest {tool} for {stage}"));
    ToolExecutionSpecV1 {
        tool_id,
        tool_version: "domain-manifest".to_string(),
        image: ContainerImageRefV1 { image: format!("bijuxdna/{tool}"), digest: None },
        command: CommandSpecV1 { template: manifest.command_template.clone() },
        resources: manifest.constraints.clone(),
    }
}

fn assert_snapshot(name: &str, plan: &StagePlanV1) -> Result<()> {
    let payload = serde_json::to_string_pretty(&plan)?;
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{}.json", snapshot_name("contracts", name)));
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(&snapshot_path, payload)?;
        return Ok(());
    }
    let expected = fs::read_to_string(&snapshot_path)?;
    assert_eq!(payload, expected);
    Ok(())
}

fn assert_command_is_concrete(plan: &StagePlanV1) {
    assert!(
        plan.command.template.iter().all(|part| !part.contains("{{") && !part.contains("}}")),
        "stage plan command must not leak unresolved template placeholders: {:?}",
        plan.command.template
    );
}

#[test]
fn stage_plan_snapshots_are_stable() -> Result<()> {
    let r1 = Path::new("reads_R1.fastq.gz");
    let r2 = Path::new("reads_R2.fastq.gz");
    let out_dir = Path::new("out");

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::detect_adapters::plan(
        &domain_tool("fastq.detect_adapters", "fastqc"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_command_is_concrete(&plan);
    assert_eq!(plan.io.outputs.len(), 3);
    assert_eq!(plan.io.outputs[0].name.as_str(), "report_json");
    assert_eq!(plan.io.outputs[1].name.as_str(), "adapter_report");
    assert_eq!(plan.io.outputs[2].name.as_str(), "adapter_evidence_dir");
    assert_eq!(plan.io.outputs[2].role.as_str(), "stage_report");
    assert_eq!(plan.params["adapter_evidence_dir"], serde_json::json!("out/fastqc"));
    assert_snapshot("stage__fastq__fastq.detect_adapters", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::profile_read_lengths::plan(
        &domain_tool("fastq.profile_read_lengths", "seqkit_stats"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.profile_read_lengths", &plan)?;

    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::profile_overrepresented_sequences::plan(
            &domain_tool("fastq.profile_overrepresented_sequences", "fastqc"),
            r1,
            Some(r2),
            out_dir,
        )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.profile_overrepresented_sequences", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::filter_low_complexity::plan_low_complexity(
        &domain_tool("fastq.filter_low_complexity", "bbduk"),
        r1,
        Some(r2),
        out_dir,
        &bijux_dna_planner_fastq::tool_adapters::fastq::filter_low_complexity::LowComplexityPlanOptions::default(),
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.filter_low_complexity", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &domain_tool("fastq.trim_reads", "fastp"),
        r1,
        None,
        out_dir,
        None,
        None,
        None,
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.trim_reads", &plan)?;

    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails(
            &domain_tool("fastq.trim_polyg_tails", "fastp"),
            r1,
            Some(r2),
            out_dir,
        )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.trim_polyg_tails", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage(
        &domain_tool("fastq.trim_terminal_damage", "cutadapt"),
        r1,
        Some(r2),
        out_dir,
        "ancient",
        2,
        2,
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.trim_terminal_damage", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::plan_filter(
        &domain_tool("fastq.filter_reads", "fastp"),
        r1,
        None,
        out_dir,
        &bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::FilterPlanOptions::default(),
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.filter_reads", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::merge_pairs::plan_merge(
        &domain_tool("fastq.merge_pairs", "pear"),
        r1,
        r2,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.merge_pairs", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::index_reference::plan(
        &domain_tool("fastq.index_reference", "bowtie2_build"),
        Path::new("reference.fa"),
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.index_reference", &plan)?;
    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    assert!(plan.command.template[2].contains("out/reference_index/bowtie2/reference"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::plan(
        &domain_tool("fastq.validate_reads", "fastqvalidator"),
        r1,
        None,
        out_dir,
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.validate_reads", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::screen_taxonomy::plan_screen(
        &domain_tool("fastq.screen_taxonomy", "kraken2"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.screen_taxonomy", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::deplete_rrna::plan_rrna(
        &domain_tool("fastq.deplete_rrna", "sortmerna"),
        r1,
        None,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.deplete_rrna", &plan)?;
    assert!(
        !plan.command.template.is_empty(),
        "deplete_rrna must plan a concrete command template"
    );

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::deplete_host::plan_host_depletion(
        &domain_tool("fastq.deplete_host", "bowtie2"),
        r1,
        Some(r2),
        Path::new("host_reference_index"),
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.deplete_host", &plan)?;
    assert_eq!(plan.io.inputs[1].name.as_str(), "reference_index");
    assert_eq!(plan.command.template[0], "bowtie2");
    assert!(plan.command.template.iter().any(|part| part == "host_reference_index"));
    assert!(plan.command.template.iter().any(|part| part == &plan.resources.threads.to_string()));
    assert!(plan.command.template.iter().any(|part| part == "--un-conc-gz"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::deplete_reference_contaminants::plan_contaminant_screen(
        &domain_tool("fastq.deplete_reference_contaminants", "bowtie2"),
        r1,
        Some(r2),
        Path::new("contaminant_reference_index"),
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.deplete_reference_contaminants", &plan)?;
    assert_eq!(plan.io.inputs[1].name.as_str(), "reference_index");
    assert_eq!(plan.command.template[0], "bowtie2");
    assert!(plan.command.template.iter().any(|part| part == "contaminant_reference_index"));
    assert!(plan.command.template.iter().any(|part| part == "--un-conc-gz"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::extract_umis::plan_umi(
        &domain_tool("fastq.extract_umis", "umi_tools"),
        r1,
        r2,
        out_dir,
        Some("NNNNNNNN"),
    )?;
    assert_snapshot("stage__fastq__fastq.extract_umis", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
        &domain_tool("fastq.correct_errors", "rcorrector"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.correct_errors", &plan)?;

    let single_end_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
            &domain_tool("fastq.correct_errors", "rcorrector"),
            r1,
            None,
            out_dir,
        )
        .expect("correction planning must admit single-end inputs");
    assert_eq!(single_end_plan.io.inputs.len(), 1);
    assert_eq!(single_end_plan.io.outputs.len(), 2);

    let preprocess_plan =
        bijux_dna_planner_fastq::tool_adapters::stages::pre::plan_preprocess::PreprocessPlan {
            r1: r1.to_path_buf(),
            r2: Some(r2.to_path_buf()),
            stages: vec![
                STAGE_TRIM_READS.as_str().to_string(),
                STAGE_VALIDATE_READS.as_str().to_string(),
            ],
            enable_contaminant_removal: false,
            pipeline_mode: FastqPipelineMode::Shotgun,
        };
    let plan =
        bijux_dna_planner_fastq::tool_adapters::stages::pre::plan_preprocess::plan_preprocess_stage(
            &preprocess_plan,
            &ToolExecutionSpecV1 {
                tool_id: ToolId::new("planner"),
                tool_version: "domain-manifest".to_string(),
                image: ContainerImageRefV1 {
                    image: "bijuxdna/planner".to_string(),
                    digest: None,
                },
                command: CommandSpecV1 {
                    template: vec!["planner".to_string()],
                },
                resources: ToolConstraints::default(),
            },
        )?;
    assert_snapshot("internal__fastq__preprocess_summary", &plan)?;

    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
            &domain_tool("fastq.report_qc", "multiqc"),
            &[
                ArtifactRef::required(
                    ArtifactId::from_static("fastq.detect_adapters.tool.fastqc.adapter_report"),
                    out_dir.join("detect_adapters/fastqc/adapter_report.json"),
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::optional(
                    ArtifactId::from_static(
                        "fastq.detect_adapters.tool.fastqc.adapter_evidence_dir",
                    ),
                    out_dir.join("detect_adapters/fastqc/fastqc"),
                    ArtifactRole::StageReport,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("fastq.profile_reads.tool.seqkit_stats.qc_json"),
                    out_dir.join("profile_reads/seqkit_stats/qc.json"),
                    ArtifactRole::MetricsJson,
                ),
            ],
            out_dir,
            std::collections::BTreeMap::new(),
            bijux_dna_domain_fastq::params::PairedMode::PairedEnd,
            bijux_dna_domain_fastq::params::qc_post::QcAggregationEngine::Multiqc,
            bijux_dna_domain_fastq::params::qc_post::QcAggregationScope::GovernedQcArtifacts,
            Some(r1),
            Some(r2),
        )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.report_qc", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::profile_reads::plan_stats_neutral(
        &domain_tool("fastq.profile_reads", "seqkit_stats"),
        r1,
        None,
        out_dir,
    )?;
    assert_command_is_concrete(&plan);
    assert_snapshot("stage__fastq__fastq.profile_reads", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::normalize_primers::plan(
        &domain_tool("fastq.normalize_primers", "cutadapt"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(plan.io.outputs.len(), 5);
    assert_eq!(plan.io.inputs[1].name.as_str(), "reads_r2");
    assert_eq!(plan.io.outputs[1].name.as_str(), "normalized_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert_eq!(plan.io.outputs[2].role.as_str(), "report_json");
    assert_eq!(plan.command.template[0], "bash");
    assert!(plan.command.template.iter().any(|part| part.contains("cutadapt")));
    assert!(plan.command.template.iter().any(|part| part.contains("--json")));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part.contains("R2.primer_normalized.fastq.gz")));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::remove_chimeras::plan(
        &domain_tool("fastq.remove_chimeras", "vsearch"),
        r1,
        None,
        out_dir,
    )?;
    assert_eq!(plan.io.inputs.len(), 1);
    assert_eq!(plan.io.outputs.len(), 5);
    assert_eq!(plan.io.inputs[0].name.as_str(), "reads");
    assert_eq!(plan.io.outputs[0].name.as_str(), "chimera_filtered_reads");
    assert_eq!(plan.io.outputs[1].name.as_str(), "report_json");
    assert_eq!(plan.io.outputs[1].role.as_str(), "report_json");
    assert_eq!(plan.io.outputs[2].name.as_str(), "chimera_metrics_json");
    assert_eq!(
        plan.io.outputs[3].role.as_str(),
        "reads",
        "chimera sequence FASTA must stay typed as a read-derived artifact",
    );
    assert_eq!(plan.io.outputs[4].name.as_str(), "uchime_report_tsv");
    assert_eq!(plan.io.outputs[4].role.as_str(), "summary_tsv");
    assert!(
        bijux_dna_planner_fastq::tool_adapters::fastq::remove_chimeras::plan(
            &domain_tool("fastq.remove_chimeras", "vsearch"),
            r1,
            Some(r2),
            out_dir,
        )
        .is_err(),
        "remove_chimeras must reject paired inputs until a paired-aware runtime exists",
    );

    let infer_asvs_plan = bijux_dna_planner_fastq::tool_adapters::fastq::infer_asvs::plan(
        &ToolExecutionSpecV1 {
            tool_id: ToolId::new("dada2"),
            tool_version: "domain-manifest".to_string(),
            image: ContainerImageRefV1 { image: "bijuxdna/dada2".to_string(), digest: None },
            command: CommandSpecV1 {
                template: vec!["Rscript".to_string(), "run_dada2.R".to_string()],
            },
            resources: ToolConstraints::default(),
        },
        r1,
        Some(r2),
        out_dir,
    )
    .expect("governed ASV inference must plan through the FASTQ adapter");
    assert!(
        infer_asvs_plan.io.outputs.iter().any(|artifact| artifact.name.as_str() == "report_json"),
        "infer_asvs planning must emit the canonical governed report artifact",
    );
    assert_eq!(infer_asvs_plan.command.template[0], "dada2");
    assert!(infer_asvs_plan.command.template.iter().any(|part| part == "--report-json"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan(
        &domain_tool("fastq.cluster_otus", "vsearch"),
        r1,
        None,
        out_dir,
    )?;
    assert_eq!(plan.io.inputs.len(), 1);
    assert_eq!(plan.io.outputs.len(), 5);
    assert_eq!(plan.io.inputs[0].name.as_str(), "reads");
    assert_eq!(plan.command.template[0], "vsearch");
    assert_eq!(plan.command.template[1], "--cluster_fast");
    assert!(plan.command.template.iter().any(|part| part == "out/otu_abundance.tsv"));
    assert!(plan.io.outputs.iter().any(|artifact| artifact.name.as_str() == "report_json"));
    assert!(plan.command.template.iter().any(|part| part
        == &bijux_dna_domain_fastq::params::edna::DEFAULT_OTU_IDENTITY_THRESHOLD.to_string()));
    assert!(plan.command.template.iter().any(|part| part == "--threads"));
    assert!(plan.command.template.iter().any(|part| part == "--uc"));
    assert_eq!(
        plan.io.outputs[1].role.as_str(),
        "reference",
        "OTU representatives must be typed as reference-style sequence outputs",
    );
    assert_eq!(plan.io.outputs[2].name.as_str(), "taxonomy_ready_fasta");
    assert_eq!(
        plan.io.outputs[2].role.as_str(),
        "reference",
        "taxonomy-ready FASTA must be typed as a reference-style sequence output",
    );
    assert_eq!(
        plan.io.outputs[3].role.as_str(),
        "reference",
        "taxonomy-ready FASTQ must be typed as a reference-style sequence output",
    );
    assert_eq!(plan.io.outputs[4].role.as_str(), "report_json");
    assert_eq!(plan.effective_params["report_artifact"], "report_json");
    assert_eq!(plan.effective_params["threads"], serde_json::json!(4));
    assert!(bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan(
        &domain_tool("fastq.cluster_otus", "vsearch"),
        r1,
        Some(r2),
        out_dir,
    )
    .is_err());
    let custom_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan_with_options(
            &domain_tool("fastq.cluster_otus", "vsearch"),
            r1,
            None,
            out_dir,
            &bijux_dna_planner_fastq::ClusterOtusStageParams {
                otu_identity: 0.99,
                threads: Some(8),
            },
        )?;
    assert_eq!(custom_plan.params["otu_identity"], serde_json::json!(0.99));
    assert_eq!(custom_plan.params["threads"], serde_json::json!(8));
    assert!(custom_plan.command.template.iter().any(|part| part == "0.99"));
    assert!(custom_plan.command.template.iter().any(|part| part == "8"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::normalize_abundance::plan(
        &domain_tool("fastq.normalize_abundance", "seqkit"),
        Path::new("otu_abundance.tsv"),
        out_dir,
    )?;
    assert_eq!(plan.io.inputs[0].role.as_str(), "summary_tsv");
    assert_eq!(plan.io.outputs[0].name.as_str(), "normalized_abundance_tsv");
    assert_eq!(plan.io.outputs[1].name.as_str(), "report_json");
    assert_eq!(plan.command.template[0], "bash");
    assert_eq!(plan.effective_params["method"], "relative_abundance");
    assert_eq!(plan.effective_params["input_value_column"], "abundance");
    assert_eq!(plan.effective_params["normalized_value_column"], "normalized_abundance");
    assert_eq!(plan.effective_params["compositional_rule"], "per_sample_sum_to_one");
    assert_eq!(plan.effective_params["report_artifact"], "report_json");
    Ok(())
}
