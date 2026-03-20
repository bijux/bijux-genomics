use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::FastqPipelineMode;
use bijux_dna_domain_fastq::{STAGE_TRIM_READS, STAGE_VALIDATE_READS};
use bijux_dna_stage_contract::StagePlanV1;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-planner-fastq__{group}__{name}")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn tool_registry() -> &'static bijux_dna_core::contract::ToolRegistry {
    static REGISTRY: OnceLock<bijux_dna_core::contract::ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        bijux_dna_runtime::manifests::load_manifests(&workspace_root())
            .expect("load domain tool registry")
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
        image: ContainerImageRefV1 {
            image: format!("bijuxdna/{tool}"),
            digest: None,
        },
        command: CommandSpecV1 {
            template: manifest.command_template.clone(),
        },
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
    assert_snapshot("stage__fastq__fastq.detect_adapters", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::profile_read_lengths::plan(
        &domain_tool("fastq.profile_read_lengths", "seqkit_stats"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.profile_read_lengths", &plan)?;

    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::profile_overrepresented_sequences::plan(
            &domain_tool("fastq.profile_overrepresented_sequences", "fastqc"),
            r1,
            Some(r2),
            out_dir,
        )?;
    assert_snapshot("stage__fastq__fastq.profile_overrepresented_sequences", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &domain_tool("fastq.trim_reads", "fastp"),
        r1,
        None,
        out_dir,
        None,
        None,
        None,
    )?;
    assert_snapshot("stage__fastq__fastq.trim_reads", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails(
        &domain_tool("fastq.trim_polyg_tails", "fastp"),
        r1,
        Some(r2),
        out_dir,
    )?;
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
    assert_snapshot("stage__fastq__fastq.trim_terminal_damage", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::plan_filter(
        &domain_tool("fastq.filter_reads", "seqkit"),
        r1,
        None,
        out_dir,
        &bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::FilterPlanOptions::default(),
    )?;
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
    assert_eq!(plan.command.template[0], "bowtie2-build");
    assert!(plan.command.template.iter().any(|part| part == "out/reference_index/bowtie2/reference"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::plan(
        &domain_tool("fastq.validate_reads", "fastqvalidator"),
        r1,
        None,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.validate_reads", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::screen_taxonomy::plan_screen(
        &domain_tool("fastq.screen_taxonomy", "kraken2"),
        r1,
        Some(r2),
        out_dir,
    )?;
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
    assert!(plan.command.template.iter().any(|part| part == "--un-conc-gz"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::deplete_reference_contaminants::plan_contaminant_screen(
        &domain_tool("fastq.deplete_reference_contaminants", "bowtie2"),
        r1,
        Some(r2),
        Path::new("contaminant_reference_index"),
        out_dir,
    )?;
    assert_snapshot(
        "stage__fastq__fastq.deplete_reference_contaminants",
        &plan,
    )?;
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
        r2,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.correct_errors", &plan)?;

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
                resources: Default::default(),
            },
        )?;
    assert_snapshot("internal__fastq__preprocess_summary", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::report_qc::plan_qc_post(
        &domain_tool("fastq.report_qc", "multiqc"),
        r1,
        Some(r2),
        out_dir,
        std::collections::BTreeMap::new(),
        None,
        None,
    )?;
    assert_snapshot("stage__fastq__fastq.report_qc", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::profile_reads::plan_stats_neutral(
        &domain_tool("fastq.profile_reads", "seqkit_stats"),
        r1,
        None,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.profile_reads", &plan)?;

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::normalize_primers::plan(
        &domain_tool("fastq.normalize_primers", "cutadapt"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(plan.io.outputs.len(), 4);
    assert_eq!(plan.io.inputs[1].name.as_str(), "reads_r2");
    assert_eq!(plan.io.outputs[1].name.as_str(), "normalized_reads_r2");
    assert_eq!(plan.command.template[0], "cutadapt");
    assert!(plan.command.template.iter().any(|part| part == "--info-file"));
    assert!(plan.command.template.iter().any(|part| part == "--json"));
    assert!(plan.command.template.iter().any(|part| part == "out/R2.primer_normalized.fastq.gz"));

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::remove_chimeras::plan(
        &domain_tool("fastq.remove_chimeras", "vsearch"),
        r1,
        Some(r2),
        out_dir,
    )?;
    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(plan.io.outputs.len(), 4);
    assert_eq!(plan.io.inputs[1].name.as_str(), "reads_r2");
    assert_eq!(plan.io.outputs[1].name.as_str(), "chimera_filtered_reads_r2");
    assert_eq!(
        plan.io.outputs[3].role.as_str(),
        "reads",
        "chimera sequence FASTA must be typed as a sequence artifact",
    );

    let infer_asvs_error = bijux_dna_planner_fastq::tool_adapters::fastq::infer_asvs::plan(
        &ToolExecutionSpecV1 {
            tool_id: ToolId::new("dada2"),
            tool_version: "domain-manifest".to_string(),
            image: ContainerImageRefV1 {
                image: "bijuxdna/dada2".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["Rscript".to_string(), "run_dada2.R".to_string()],
            },
            resources: Default::default(),
        },
        r1,
        Some(r2),
        out_dir,
    )
    .expect_err("declared-only ASV inference must not plan through the FASTQ adapter");
    assert!(
        infer_asvs_error.to_string().contains("declared-only"),
        "infer_asvs failure must explain that the runtime contract is not closed",
    );

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan(
        &domain_tool("fastq.cluster_otus", "vsearch"),
        r1,
        None,
        out_dir,
    )?;
    assert_eq!(plan.io.inputs.len(), 1);
    assert_eq!(plan.io.outputs.len(), 4);
    assert_eq!(plan.io.inputs[0].name.as_str(), "reads");
    assert_eq!(plan.command.template[0], "vsearch");
    assert_eq!(plan.command.template[1], "--cluster_fast");
    assert!(plan.command.template.iter().any(|part| part == "out/otu_abundance.tsv"));
    assert_eq!(plan.io.outputs[2].name.as_str(), "taxonomy_ready_fasta");
    assert!(
        bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan(
            &domain_tool("fastq.cluster_otus", "vsearch"),
            r1,
            Some(r2),
            out_dir,
        )
        .is_err()
    );
    Ok(())
}
