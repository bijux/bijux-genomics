use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy;
use bijux_dna_domain_fastq::params::DamageMode;

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
        Some("seqpurge.fastq.gz")
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
fn plan_trim_supports_filter_style_manifest_placeholders_for_governed_tools() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &templated_tool(
            "seqkit",
            &[
                "seqkit",
                "seq",
                "-o",
                "{{filtered_reads_r1}}",
                "{{reads_r1}}",
            ],
        ),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    assert!(plan.command.template[2].contains("out/seqkit.fastq.gz"));
    assert!(!plan.command.template[2].contains("{{filtered_reads_r1}}"));
    Ok(())
}

#[test]
fn plan_from_config_preserves_layout_without_enabling_bank_policies() -> Result<()> {
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
    assert_eq!(plan.effective_params["paired_mode"], "paired_end");
    assert_eq!(plan.effective_params["adapter_policy"], "none");
    assert_eq!(plan.effective_params["polyx_policy"], "none");
    assert_eq!(plan.effective_params["contaminant_policy"], "none");
    assert!(plan.params.get("adapter_bank").is_none());
    assert!(plan.params.get("polyx_bank").is_none());
    assert!(plan.params.get("contaminant_bank").is_none());
    assert!(plan.command.template.iter().any(|part| part == "--in2"));
    assert!(plan.command.template.iter().any(|part| part == "--out2"));
    assert!(!plan
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
    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("--in2"));
    assert!(script.contains("--out2"));
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
fn plan_trim_with_options_maps_min_length_for_seqkit() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("seqkit"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: Some(75),
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        },
    )?;

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    assert!(plan.command.template[2].contains("seqkit seq -m 75"));
    assert!(plan.command.template[2].contains("out/seqkit.fastq.gz"));
    Ok(())
}

#[test]
fn plan_trim_seqpurge_supports_single_end_and_paired_layouts() -> Result<()> {
    let single_end = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("seqpurge"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;
    assert_eq!(single_end.command.template[0], "sh");
    assert_eq!(single_end.command.template[1], "-lc");
    assert!(single_end.command.template[2].contains("seqpurge"));
    assert!(!single_end.command.template[2].contains("{{reads_r2}}"));

    let paired = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("seqpurge"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;
    assert!(paired.command.template[2].contains("-in2"));
    assert!(paired.command.template[2].contains("reads_R2.fastq.gz"));
    Ok(())
}

#[test]
fn plan_trim_with_options_maps_min_length_for_seqpurge() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("seqpurge"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: Some(80),
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        },
    )?;

    assert!(plan.command.template[2].contains("-min_len"));
    assert!(plan.command.template[2].contains("80"));
    Ok(())
}

#[test]
fn plan_trim_with_drop_n_policy_maps_backend_specific_n_filters() -> Result<()> {
    let fastp_plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: None,
            quality_cutoff: None,
            n_policy: Some("drop".to_string()),
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        },
    )?;
    assert!(fastp_plan
        .command
        .template
        .windows(2)
        .any(|window| window == ["--n_base_limit", "0"]));

    let cutadapt_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
            &dummy_tool("cutadapt"),
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("out"),
            None,
            None,
            None,
            &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
                min_length: None,
                quality_cutoff: None,
                n_policy: Some("drop".to_string()),
                adapter_policy: None,
                polyx_policy: None,
                contaminant_policy: None,
            },
        )?;
    assert!(cutadapt_plan
        .command
        .template[2]
        .contains("--max-n"));
    assert!(cutadapt_plan
        .command
        .template[2]
        .contains("'0'"));
    Ok(())
}

#[test]
fn plan_trim_with_options_maps_length_and_quality_for_atropos() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("atropos"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
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

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("'atropos' 'trim'"));
    assert!(script.contains("'-q' '18'"));
    assert!(script.contains("'-m' '42'"));
    assert!(script.contains("'-pe1' 'reads_R1.fastq.gz'"));
    assert!(script.contains("'-pe2' 'reads_R2.fastq.gz'"));
    assert!(script.contains("'-o' 'out/R1.atropos.fastq.gz'"));
    assert!(script.contains("'-p' 'out/R2.atropos.fastq.gz'"));
    Ok(())
}

#[test]
fn plan_trim_with_options_maps_length_and_quality_for_adapterremoval() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("adapterremoval"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
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

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("'AdapterRemoval'"));
    assert!(script.contains("'--file1' 'reads_R1.fastq.gz'"));
    assert!(script.contains("'--file2' 'reads_R2.fastq.gz'"));
    assert!(script.contains("'--output1' 'out/R1.adapterremoval.fastq.gz'"));
    assert!(script.contains("'--output2' 'out/R2.adapterremoval.fastq.gz'"));
    assert!(script.contains("'--minlength' '42'"));
    assert!(script.contains("'--trimqualities'"));
    assert!(script.contains("'--minquality' '18'"));
    Ok(())
}

#[test]
fn plan_trim_with_options_maps_length_and_quality_for_trimmomatic() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("trimmomatic"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
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

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("'trimmomatic' 'PE' '-phred33'"));
    assert!(script.contains("'reads_R1.fastq.gz' 'reads_R2.fastq.gz'"));
    assert!(script.contains("'out/R1.trimmomatic.fastq.gz'"));
    assert!(script.contains("'out/R1.trimmomatic.unpaired.fastq.gz'"));
    assert!(script.contains("'out/R2.trimmomatic.fastq.gz'"));
    assert!(script.contains("'out/R2.trimmomatic.unpaired.fastq.gz'"));
    assert!(script.contains("'SLIDINGWINDOW:4:18'"));
    assert!(script.contains("'MINLEN:42'"));
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
fn plan_trim_with_options_rejects_contaminant_handoffs_without_execution_support() {
    let error = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
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
    )
    .expect_err("trim_reads should reject contaminant handoffs that do not drive execution");

    assert!(error
        .to_string()
        .contains("use fastq.deplete_reference_contaminants"));
}

#[test]
fn plan_trim_with_bank_contaminant_policy_maps_bbduk_reference_filter() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("bbduk"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        Some(&serde_json::json!({
            "enabled_entries": [
                {"id": "phix-motif", "sequence": "ACGTACGT"}
            ],
            "references": [
                {"id": "phix-ref", "fasta": ">phix-ref\nTTTTAAAA"}
            ]
        })),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: None,
            quality_cutoff: None,
            n_policy: Some("drop".to_string()),
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: Some("bank".to_string()),
        },
    )?;

    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("bbduk_contaminants.fa"));
    assert!(script.contains(">phix-motif"));
    assert!(script.contains(">phix-ref"));
    assert!(script.contains("ref=out/bbduk_contaminants.fa"));
    assert!(script.contains("maxns=0"));
    assert!(script.contains("k=31"));
    Ok(())
}

#[test]
fn plan_trim_with_bank_policy_maps_explicit_adapters_for_fastp() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        Some(&serde_json::json!({
            "enabled_entries": [
                {"sequence": "ACGTACGT"},
                {"sequence": "TGCATGCA"}
            ]
        })),
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: None,
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: Some("bank".to_string()),
            polyx_policy: None,
            contaminant_policy: None,
        },
    )?;

    assert_eq!(plan.params["adapter_policy"], serde_json::json!("bank"));
    assert_eq!(plan.params["adapter_bank"]["enabled_entries"][0]["sequence"], "ACGTACGT");
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--adapter_sequence"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "ACGTACGT"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--adapter_sequence_r2"));
    assert!(!plan
        .command
        .template
        .iter()
        .any(|part| part == "--detect_adapter_for_pe"));
    Ok(())
}

#[test]
fn plan_trim_with_polyx_trim_enables_fastp_polyx_mode() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: None,
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: Some("trim".to_string()),
            contaminant_policy: None,
        },
    )?;

    assert_eq!(plan.params["polyx_policy"], serde_json::json!("trim"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--trim_poly_x"));
    Ok(())
}

#[test]
fn plan_trim_rejects_contaminant_policy_without_a_contaminant_stage() {
    let error = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_with_options(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        Some(&serde_json::json!({"preset": "host"})),
        &bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimPlanOptions {
            min_length: None,
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: Some("bank".to_string()),
        },
    )
    .expect_err("trim_reads should refuse contaminant_policy handoffs that do not change execution");

    assert!(error
        .to_string()
        .contains("use fastq.deplete_reference_contaminants"));
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
    assert_eq!(fastp_plan.command.template[0], "sh");
    assert_eq!(fastp_plan.command.template[1], "-lc");
    let fastp_script = &fastp_plan.command.template[2];
    assert!(fastp_script.contains("--poly_g_min_len"));
    assert!(fastp_script.contains("'14'"));
    assert!(fastp_script.contains("trim_polyg_tails_report.fastp.json"));
    assert!(fastp_script.contains("\"raw_report_format\":\"fastp_json\""));
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
    assert!(script.contains("\"raw_report_format\":\"bbduk_stats\""));
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
    assert_eq!(
        serde_json::from_value::<DamageMode>(plan.effective_params["damage_mode"].clone())?,
        DamageMode::Ancient
    );
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
    assert!(script.contains("\"execution_policy\":\"explicit_terminal_trim\""));
    Ok(())
}

#[test]
fn plan_trim_terminal_damage_udg_defaults_preserve_terminal_ends() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage(
        &dummy_tool("cutadapt"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        "udg_trimmed",
        2,
        2,
    )?;

    assert_eq!(
        serde_json::from_value::<TerminalDamageExecutionPolicy>(
            plan.effective_params["execution_policy"].clone()
        )?,
        TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds
    );
    assert_eq!(plan.effective_params["trim_5p_bases"], 0);
    assert_eq!(plan.effective_params["trim_3p_bases"], 0);
    assert_eq!(plan.effective_params["requested_trim_5p_bases"], 2);
    assert_eq!(plan.effective_params["requested_trim_3p_bases"], 2);
    assert!(!plan.command.template.iter().any(|part| part == "-3"));
    Ok(())
}

#[test]
fn plan_trim_terminal_damage_rejects_unknown_damage_mode() {
    let error = bijux_dna_planner_fastq::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage(
        &dummy_tool("cutadapt"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        "partial_udg",
        2,
        2,
    )
    .expect_err("unknown damage_mode must fail fast");

    assert!(error.to_string().contains("invalid fastq.trim_terminal_damage damage_mode"));
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
