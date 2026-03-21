use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn templated_tool(tool: &str, template: &[&str]) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: template.iter().map(|part| (*part).to_string()).collect(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[test]
fn trim_output_names_are_defined_for_known_tools() {
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("fastp"),
        Some("fastp.fastq.gz")
    );
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("trimmomatic"),
        Some("trimmomatic.fastq.gz")
    );
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("unknown"),
        None
    );
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("seqpurge"),
        None
    );
}

#[test]
fn plan_trim_builds_expected_paths() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;
    assert_eq!(
        plan.io.outputs[0].path.to_string_lossy(),
        "out/fastp.fastq.gz"
    );
    assert_eq!(plan.io.outputs[0].name.as_str(), "trimmed_reads_r1");
    assert_eq!(plan.io.outputs[1].name.as_str(), "report_json");
    assert_eq!(plan.params["min_length"], serde_json::json!(30));
    assert_eq!(plan.params["n_policy"], serde_json::json!("retain"));
    assert_eq!(plan.effective_params["min_len"], serde_json::json!(30));
    assert_eq!(plan.effective_params["n_policy"], serde_json::json!("retain"));
    Ok(())
}

#[test]
fn plan_trim_rejects_unknown_tool() {
    match bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("mystery"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    ) {
        Ok(_) => panic!("expected unsupported trim tool"),
        Err(err) => assert!(err.to_string().contains("unsupported trim tool")),
    }
}

#[test]
fn plan_from_config_preserves_layout_and_bank_policies() -> Result<()> {
    let config = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::resolve_config(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimUserConfig {
            tool: "fastp".to_string(),
            r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(std::path::PathBuf::from("reads_R2.fastq.gz")),
            out_dir: std::path::PathBuf::from("out"),
            adapter_bank: Some(serde_json::json!({"preset": "illumina"})),
            polyx_bank: Some(serde_json::json!({"enabled": true})),
            contaminant_bank: Some(serde_json::json!({"catalog": "decoys"})),
        },
    );

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_from_config(
        &dummy_tool("fastp"),
        &config,
    )?;

    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(plan.io.outputs[1].name.as_str(), "trimmed_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert_eq!(plan.params["adapter_bank"]["preset"], "illumina");
    assert_eq!(plan.params["polyx_bank"]["enabled"], true);
    assert_eq!(plan.params["contaminant_bank"]["catalog"], "decoys");
    assert_eq!(plan.effective_params["paired_mode"], "paired_end");
    assert_eq!(plan.effective_params["adapter_policy"], "bank");
    assert_eq!(plan.effective_params["polyx_policy"], "bank");
    assert_eq!(plan.effective_params["contaminant_policy"], "bank");
    assert!(plan.command.template.iter().any(|part| part == "--in2"));
    assert!(plan.command.template.iter().any(|part| part == "--out2"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--detect_adapter_for_pe"));
    Ok(())
}

#[test]
fn plan_trim_polyg_preserves_paired_output_names() -> Result<()> {
    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails(
            &dummy_tool("fastp"),
            std::path::Path::new("reads_R1.fastq.gz"),
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out"),
        )?;

    assert_eq!(plan.io.outputs[0].name.as_str(), "trimmed_reads_r1");
    assert_eq!(plan.io.outputs[1].name.as_str(), "trimmed_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert!(plan.command.template.iter().any(|part| part == "--in2"));
    assert!(plan.command.template.iter().any(|part| part == "--out2"));
    assert_eq!(
        plan.effective_params["schema_version"],
        "bijux.fastq.params.trim_polyg_tails.v1"
    );
    assert_eq!(plan.effective_params["trim_polyg"], true);
    assert_eq!(plan.effective_params["min_polyg_run"], 10);
    Ok(())
}

#[test]
fn plan_trim_with_options_maps_length_and_quality_for_fastp() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: Some(42),
            quality_cutoff: Some(18),
            n_policy: Some("retain".to_string()),
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        },
    )?;

    assert_eq!(plan.params["min_length"], serde_json::json!(42));
    assert_eq!(plan.params["quality_cutoff"], serde_json::json!(18));
    assert_eq!(plan.effective_params["min_len"], serde_json::json!(42));
    assert_eq!(plan.effective_params["q_cutoff"], serde_json::json!(18));
    assert_eq!(plan.effective_params["n_policy"], serde_json::json!("retain"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--length_required"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--qualified_quality_phred"));
    Ok(())
}

#[test]
fn plan_trim_rejects_nondefault_quality_controls_for_unmapped_backends() {
    let error = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &templated_tool("seqkit", &["seqkit", "seq", "{{reads_r1}}"]),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: Some(42),
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        },
    )
    .expect_err("seqkit trim planning should reject unmapped non-default stage controls");

    assert!(error
        .to_string()
        .contains("does not yet map min_length/quality_cutoff"));
}

#[test]
fn plan_trim_with_options_preserves_explicit_bank_policy_bindings() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        Some(&serde_json::json!({"preset": "illumina"})),
        None,
        Some(&serde_json::json!({"preset": "host"})),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: None,
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: Some("none".to_string()),
            polyx_policy: Some("none".to_string()),
            contaminant_policy: Some("bank".to_string()),
        },
    )?;

    assert_eq!(plan.params["adapter_policy"], serde_json::json!("none"));
    assert_eq!(plan.params["polyx_policy"], serde_json::json!("none"));
    assert_eq!(plan.params["contaminant_policy"], serde_json::json!("bank"));
    assert_eq!(plan.effective_params["adapter_policy"], serde_json::json!("none"));
    assert_eq!(plan.effective_params["polyx_policy"], serde_json::json!("none"));
    assert_eq!(
        plan.effective_params["contaminant_policy"],
        serde_json::json!("bank")
    );
    Ok(())
}

#[test]
fn plan_trim_polyg_uses_configured_min_run_for_backends() -> Result<()> {
    let fastp_plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads_R1.fastq.gz"),
        None,
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions {
            trim_polyg: true,
            min_polyg_run: 14,
        },
    )?;
    assert!(fastp_plan
        .command
        .template
        .iter()
        .any(|part| part == "--poly_g_min_len"));
    assert!(fastp_plan.command.template.iter().any(|part| part == "14"));
    assert_eq!(fastp_plan.effective_params["min_polyg_run"], 14);

    let bbduk_plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
        &dummy_tool("bbduk"),
        std::path::Path::new("reads_R1.fastq.gz"),
        None,
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions {
            trim_polyg: true,
            min_polyg_run: 14,
        },
    )?;
    assert!(bbduk_plan
        .command
        .template
        .iter()
        .any(|part| part.contains("trimpolygright=14")));
    assert_eq!(bbduk_plan.command.template[0], "sh");
    assert_eq!(bbduk_plan.command.template[1], "-lc");
    let script = &bbduk_plan.command.template[2];
    assert!(script.contains("trim_polyg_tails_report.stats.txt"));
    assert!(script.contains("\"tool_id\":\"bbduk\""));
    assert!(script.contains("\"stage_id\":\"fastq.trim_polyg_tails\""));
    Ok(())
}

#[test]
fn plan_trim_polyg_can_disable_polyg_flag_for_bench_comparisons() -> Result<()> {
    let fastp_plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads_R1.fastq.gz"),
        None,
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions {
            trim_polyg: false,
            min_polyg_run: 14,
        },
    )?;
    assert_eq!(fastp_plan.params["trim_polyg"], false);
    assert_eq!(fastp_plan.effective_params["trim_polyg"], false);
    assert!(!fastp_plan
        .command
        .template
        .iter()
        .any(|part| part == "--trim_poly_g"));

    let bbduk_plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
        &dummy_tool("bbduk"),
        std::path::Path::new("reads_R1.fastq.gz"),
        None,
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions {
            trim_polyg: false,
            min_polyg_run: 14,
        },
    )?;
    assert_eq!(bbduk_plan.params["trim_polyg"], false);
    assert!(!bbduk_plan.command.template[2].contains("trimpolygright="));
    Ok(())
}

#[test]
fn plan_trim_terminal_damage_preserves_paired_output_names() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage(
        &dummy_tool("cutadapt"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        "ancient",
        2,
        2,
    )?;

    assert_eq!(plan.io.outputs[0].name.as_str(), "trimmed_reads_r1");
    assert_eq!(plan.io.outputs[1].name.as_str(), "trimmed_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert!(plan.command.template.iter().any(|part| part == "-p"));
    assert_eq!(
        plan.effective_params["schema_version"],
        "bijux.fastq.params.trim_terminal_damage.v1"
    );
    assert_eq!(plan.effective_params["damage_mode"], "ancient");
    assert_eq!(plan.effective_params["trim_5p_bases"], 2);
    assert_eq!(plan.effective_params["trim_3p_bases"], 2);
    Ok(())
}

#[test]
fn plan_trim_terminal_damage_seqkit_respects_terminal_trim_settings() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage(
        &dummy_tool("seqkit"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        "udg_trimmed",
        5,
        3,
    )?;

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("seqkit subseq -r '6:-4'"));
    assert!(script.contains("reads_R1.fastq.gz"));
    assert!(script.contains("reads_R2.fastq.gz"));
    assert!(script.contains("trim_terminal_damage_report.json"));
    assert!(script.contains("\"damage_mode\":\"udg_trimmed\""));
    Ok(())
}

#[test]
fn plan_trim_wraps_generic_backends_with_normalized_report_json() -> Result<()> {
    let tool = templated_tool(
        "cutadapt",
        &["cutadapt", "-o", "{{trimmed_reads_r1}}", "{{reads_r1}}"],
    );
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &tool,
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("cutadapt"));
    assert!(script.contains("trim_report.json"));
    assert!(script.contains("\"tool_id\":\"cutadapt\""));
    Ok(())
}

#[test]
fn plan_trim_galore_uses_output_directory_and_moves_governed_outputs() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("trim_galore"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("trim_galore --output_dir"));
    assert!(script.contains("--paired"));
    assert!(script.contains("reads_R1_trimmed.fq.gz"));
    assert!(script.contains("reads_R2_trimmed.fq.gz"));
    assert!(script.contains("trim_report.json"));
    Ok(())
}
