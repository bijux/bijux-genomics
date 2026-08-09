use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_retained_tool_bindings, canonical_tool_bindings, project_tool_parameters,
    render_tool_command, validate_tool_binding, BindingInputV1, BindingOutputV1,
    BindingParameterV1, EffectiveToolParameterSourceV1, OutputDiscoveryV1, ToolArtifactSourceV1,
    ToolBindingRegistry, ToolBindingRenderContext, ToolBindingV1, ToolParameterProjectionRequest,
    EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1, TOOL_BINDING_SCHEMA_V1,
};

const ALIGN_STAGE: &str = "bam.align";
const ALIGN_BUNDLE: &str = "bijux.benchmark.bam.align.bundle.v1";
const UNSORTED_SAM: &str = "bijux.sam.artifact.unsorted.v1";
const GENOTYPING_STAGE: &str = "bam.genotyping";
const COVERAGE_STAGE: &str = "bam.coverage";
const ENDOGENOUS_CONTENT_STAGE: &str = "bam.endogenous_content";
const MAPPING_SUMMARY_STAGE: &str = "bam.mapping_summary";
const DUPLICATION_METRICS_STAGE: &str = "bam.duplication_metrics";
const INSERT_SIZE_STAGE: &str = "bam.insert_size";
const GC_BIAS_STAGE: &str = "bam.gc_bias";
const SEX_STAGE: &str = "bam.sex";
const EXTRACT_UMIS_STAGE: &str = "fastq.extract_umis";
const CORRECT_ERRORS_STAGE: &str = "fastq.correct_errors";

#[test]
fn canonical_retained_registry_owns_bam_align_tools_in_stable_order() -> Result<()> {
    let retained = canonical_retained_tool_bindings()?;
    let align = retained.bindings_for_stage(&StageId::new(ALIGN_STAGE));

    assert_eq!(retained.bindings().len(), 16);
    assert_eq!(
        align.iter().map(|binding| binding.tool_id.to_string()).collect::<Vec<_>>(),
        ["bowtie2", "bwa"]
    );
    for binding in align {
        assert_eq!(binding.outputs.len(), 1);
        assert_eq!(binding.outputs[0].artifact_contract_id, UNSORTED_SAM);
        assert_eq!(binding.normalized_output_contract_id, ALIGN_BUNDLE);
        assert_eq!(binding.normalizer_id, "bijux_dna_planner_bam::tool_adapters::bam::align");
        assert_eq!(binding.downstream_probe_id, "benchmark.bam.align.compare");
        assert_eq!(
            binding.artifact_source,
            ToolArtifactSourceV1::CatalogImage { catalog_tool_id: binding.tool_id.clone() }
        );
        assert!(!binding.command_template.iter().any(|token| [
            "bash", "sh", "python", "samtools", "gatk"
        ]
        .iter()
        .any(|name| token.contains(name))));
    }

    let all = canonical_tool_bindings()?;
    assert_eq!(all.bindings().len(), 24);
    assert_eq!(all.bindings_for_stage(&StageId::new(ALIGN_STAGE)).len(), 2);
    Ok(())
}

#[test]
fn canonical_rcorrector_binding_refuses_ambiguous_paired_outputs() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(CORRECT_ERRORS_STAGE), &ToolId::new("rcorrector"))
        .ok_or_else(|| anyhow::anyhow!("canonical Rcorrector binding is missing"))?;

    assert_eq!(binding.resources.threads, 4);
    assert_eq!(
        binding.normalized_output_contract_id,
        "bijux.benchmark.fastq.correct_errors.bundle.v1"
    );
    assert_eq!(
        binding.normalizer_id,
        "bijux_dna_planner_fastq::tool_adapters::stages::transform::correct_errors"
    );
    assert_eq!(binding.downstream_probe_id, "bijux.stage_probe.fastq.correct_errors.v1");
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["corrected_reads_r1", "corrected_reads_r2"]
    );

    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "rcorrector".to_owned(),
            inputs: BTreeMap::from([
                ("reads_r1".to_owned(), "/input/sample_R1.fastq.gz".to_owned()),
                ("reads_r2".to_owned(), "/input/sample_R2.fastq.gz".to_owned()),
            ]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::new(),
            outputs: BTreeMap::from([
                ("corrected_reads_r1".to_owned(), "/result/corrected_R1.fastq.gz".to_owned()),
                ("corrected_reads_r2".to_owned(), "/result/corrected_R2.fastq.gz".to_owned()),
            ]),
        },
    )?;
    assert_eq!(command[0..2], ["/bin/sh", "-c"]);
    assert!(command[2].contains("work=\"$TMPDIR/bijux-rcorrector-work\""));
    assert!(!command[2].contains("work=/tmp/"));
    assert!(command[2].contains("'rcorrector' -t 4"));
    assert!(command[2].contains("-1 '/input/sample_R1.fastq.gz'"));
    assert!(command[2].contains("-2 '/input/sample_R2.fastq.gz'"));
    assert!(command[2].contains("-eq 2"));
    assert!(command[2].contains("'/result/corrected_R1.fastq.gz'"));
    assert!(command[2].contains("'/result/corrected_R2.fastq.gz'"));
    Ok(())
}

#[test]
fn canonical_umi_tools_binding_preserves_paired_reads_and_native_log() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(EXTRACT_UMIS_STAGE), &ToolId::new("umi_tools"))
        .ok_or_else(|| anyhow::anyhow!("canonical UMI-tools extraction binding is missing"))?;

    assert_eq!(binding.resources.threads, 2);
    assert_eq!(
        binding.normalized_output_contract_id,
        "bijux.benchmark.fastq.extract_umis.bundle.v1"
    );
    assert_eq!(
        binding.normalizer_id,
        "bijux_dna_planner_fastq::tool_adapters::stages::transform::extract_umis"
    );
    assert_eq!(binding.downstream_probe_id, "bijux.stage_probe.fastq.extract_umis.v1");
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["umi_reads_r1", "umi_reads_r2", "extraction_log"]
    );

    assert_eq!(
        render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: "/images/umi_tools.sif".to_owned(),
                inputs: BTreeMap::from([
                    ("reads_r1".to_owned(), "sample_R1.fastq.gz".to_owned()),
                    ("reads_r2".to_owned(), "sample_R2.fastq.gz".to_owned()),
                ]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([("umi_pattern".to_owned(), "NNNNNNNN".to_owned())]),
                outputs: BTreeMap::from([
                    ("umi_reads_r1".to_owned(), "umi_R1.fastq.gz".to_owned()),
                    ("umi_reads_r2".to_owned(), "umi_R2.fastq.gz".to_owned()),
                    ("extraction_log".to_owned(), "umi_tools.extract.log".to_owned()),
                ]),
            },
        )?,
        [
            "/images/umi_tools.sif",
            "extract",
            "--stdin",
            "sample_R1.fastq.gz",
            "--stdout",
            "umi_R1.fastq.gz",
            "--read2-in",
            "sample_R2.fastq.gz",
            "--read2-out",
            "umi_R2.fastq.gz",
            "--bc-pattern",
            "NNNNNNNN",
            "--log",
            "umi_tools.extract.log",
        ]
    );
    Ok(())
}

#[test]
fn canonical_rxy_binding_requires_explicit_interpretation_context() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(SEX_STAGE), &ToolId::new("rxy"))
        .ok_or_else(|| anyhow::anyhow!("canonical Rxy sex-inference binding is missing"))?;

    assert_eq!(binding.resources.threads, 1);
    assert_eq!(binding.normalized_output_contract_id, "bijux.benchmark.bam.sex.bundle.v1");
    assert_eq!(binding.normalizer_id, "parse_sex_json");
    assert_eq!(binding.downstream_probe_id, "bijux.stage_probe.bam.sex.v1");
    assert_eq!(
        binding.parameters.iter().map(|parameter| parameter.name.as_str()).collect::<Vec<_>>(),
        ["classification_profile", "minimum_sex_reads"]
    );
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["sex_report", "sex_summary"]
    );

    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "/images/rxy.sif".to_owned(),
            inputs: BTreeMap::from([("bam".to_owned(), "/input/sample.bam".to_owned())]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("classification_profile".to_owned(), "human-skoglund-2013".to_owned()),
                ("minimum_sex_reads".to_owned(), "100".to_owned()),
            ]),
            outputs: BTreeMap::from([
                ("sex_report".to_owned(), "/result/sex.json".to_owned()),
                ("sex_summary".to_owned(), "/result/sex.summary.json".to_owned()),
            ]),
        },
    )?;
    assert_eq!(command[0..2], ["/bin/sh", "-c"]);
    assert!(command[2].contains("'/images/rxy.sif' --input '/input/sample.bam'"));
    assert!(command[2].contains("--classification-profile 'human-skoglund-2013'"));
    assert!(command[2].contains("--minimum-sex-reads '100'"));
    assert!(command[2].contains("> '/result/sex.json' && cp '/result/sex.json'"));
    Ok(())
}

#[test]
fn canonical_gc_bias_binding_closes_run_2013_native_reports() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(GC_BIAS_STAGE), &ToolId::new("picard"))
        .ok_or_else(|| anyhow::anyhow!("canonical Picard GC-bias binding is missing"))?;

    assert_eq!(binding.resources.threads, 1);
    assert_eq!(binding.normalized_output_contract_id, "bijux.benchmark.bam.gc_bias.bundle.v1");
    assert_eq!(binding.normalizer_id, "bijux_dna_planner_bam::tool_adapters::bam::gc_bias");
    assert_eq!(binding.downstream_probe_id, "benchmark.bam.gc_bias.consume");
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["gc_bias_metrics", "gc_bias_summary", "gc_bias_chart"]
    );
    assert!(binding.command_template.iter().any(|token| token == "CollectGcBiasMetrics"));
    Ok(())
}

#[test]
fn canonical_insert_size_binding_closes_run_2015_native_reports() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(INSERT_SIZE_STAGE), &ToolId::new("picard"))
        .ok_or_else(|| anyhow::anyhow!("canonical Picard insert-size binding is missing"))?;

    assert_eq!(binding.resources.threads, 1);
    assert_eq!(binding.normalized_output_contract_id, "bijux.benchmark.bam.insert_size.bundle.v1");
    assert_eq!(binding.normalizer_id, "bijux_dna_planner_bam::tool_adapters::bam::insert_size");
    assert_eq!(binding.downstream_probe_id, "benchmark.bam.insert_size.consume");
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["insert_size_metrics", "insert_size_histogram"]
    );
    assert!(binding.command_template.iter().any(|token| token == "CollectInsertSizeMetrics"));
    Ok(())
}

#[test]
fn canonical_duplication_metrics_bindings_close_run_2011_native_reports() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(DUPLICATION_METRICS_STAGE));

    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["picard", "samtools"]
    );
    assert_eq!(bindings[0].resources.threads, 1);
    assert_eq!(bindings[1].resources.threads, 4);
    for binding in bindings {
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.bam.duplication_metrics.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_bam::tool_adapters::bam::duplication_metrics"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.bam.duplication_metrics.compare");
        assert!(binding.outputs.iter().any(|output| output.name == "marked_bam"));
    }
    Ok(())
}

#[test]
fn canonical_mapping_summary_bindings_close_run_2004_native_reports() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(MAPPING_SUMMARY_STAGE));

    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["picard", "samtools"]
    );
    for binding in &bindings {
        assert_eq!(binding.resources.threads, 1);
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.bam.mapping_summary.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_bam::tool_adapters::bam::mapping_summary"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.bam.mapping_summary.compare");
        assert_eq!(binding.outputs.len(), 1);
        assert_eq!(binding.outputs[0].artifact_contract_id, "bijux.auxiliary.artifact.table.v1");
    }
    assert_eq!(bindings[0].outputs[0].name, "alignment_summary_metrics");
    assert!(bindings[0]
        .command_template
        .iter()
        .any(|token| token == "CollectAlignmentSummaryMetrics"));
    assert_eq!(bindings[1].outputs[0].name, "flagstat");
    assert!(bindings[1].command_template[2].contains("flagstat -@ 1"));
    Ok(())
}

#[test]
fn canonical_endogenous_content_binding_closes_run_2003_native_counts() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(ENDOGENOUS_CONTENT_STAGE), &ToolId::new("samtools"))
        .ok_or_else(|| {
            anyhow::anyhow!("canonical Samtools endogenous-content binding is missing")
        })?;

    assert_eq!(binding.resources.threads, 4);
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["total_primary_reads", "mapped_primary_reads", "mapped_nonduplicate_primary_reads"]
    );
    assert_eq!(
        binding.normalized_output_contract_id,
        "bijux.benchmark.bam.endogenous_content.bundle.v1"
    );
    assert_eq!(
        binding.normalizer_id,
        "bijux_dna_planner_bam::tool_adapters::bam::endogenous_content"
    );
    assert_eq!(binding.downstream_probe_id, "benchmark.bam.endogenous_content.consume");
    assert!(binding.command_template[2].contains("-F 0x900"));
    assert!(binding.command_template[2].contains("-F 0x904"));
    assert!(binding.command_template[2].contains("-F 0xD04"));
    assert_eq!(binding.command_template[2].matches(" && ").count(), 2);
    Ok(())
}

#[test]
fn canonical_coverage_bindings_close_run_2002_native_outputs() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(COVERAGE_STAGE));

    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["bedtools", "mosdepth", "samtools"]
    );
    for binding in bindings {
        let expected_threads = if binding.tool_id.as_str() == "mosdepth" { 4 } else { 1 };
        assert_eq!(binding.resources.threads, expected_threads);
        assert_eq!(binding.normalized_output_contract_id, "bijux.benchmark.bam.coverage.bundle.v1");
        assert_eq!(binding.normalizer_id, "bijux_dna_planner_bam::tool_adapters::bam::coverage");
        assert_eq!(binding.downstream_probe_id, "benchmark.bam.coverage.compare");
        assert!(binding.outputs.iter().all(|output| {
            output.artifact_contract_id == "bijux.auxiliary.artifact.table.v1"
                && output.discovery == OutputDiscoveryV1::ExactPath
        }));
        let outputs = binding
            .outputs
            .iter()
            .map(|output| {
                (output.name.clone(), format!("/result/{}.tsv", output.name.replace('_', "-")))
            })
            .collect();
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: format!("/images/{}.sif", binding.tool_id),
                inputs: BTreeMap::from([("bam".to_string(), "/input/sample.bam".to_string())]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::new(),
                outputs,
            },
        )?;
        assert!(command.iter().all(|token| !token.contains("${")));
        assert!(command.iter().any(|token| token.contains("/input/sample.bam")));
    }
    Ok(())
}

#[test]
fn canonical_genotyping_binding_closes_run_2001_native_execution() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(GENOTYPING_STAGE), &ToolId::new("bcftools"))
        .ok_or_else(|| anyhow::anyhow!("canonical BCFtools genotyping binding is missing"))?;

    assert_eq!(binding.resources.threads, 4);
    assert_eq!(binding.normalized_output_contract_id, "bijux.benchmark.bam.genotyping.bundle.v1");
    assert_eq!(binding.normalizer_id, "bijux_dna_planner_bam::tool_adapters::bam::genotyping");
    assert_eq!(binding.downstream_probe_id, "benchmark.bam.genotyping.consume");
    assert_eq!(binding.outputs.len(), 1);
    assert_eq!(
        binding.outputs[0].artifact_contract_id,
        "bijux.vcf.artifact.coordinate_sorted_indexed.v1"
    );
    assert!(binding.command_template[2].contains("mpileup"));
    assert!(binding.command_template[2].contains("index -f"));
    Ok(())
}

#[test]
fn canonical_bam_align_bindings_render_direct_native_commands() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bowtie2 = registry
        .binding(&StageId::new(ALIGN_STAGE), &ToolId::new("bowtie2"))
        .ok_or_else(|| anyhow::anyhow!("canonical bowtie2 binding is missing"))?;
    let bwa = registry
        .binding(&StageId::new(ALIGN_STAGE), &ToolId::new("bwa"))
        .ok_or_else(|| anyhow::anyhow!("canonical bwa binding is missing"))?;

    assert_eq!(
        render_tool_command(
            bowtie2,
            &ToolBindingRenderContext {
                executable: "/images/bowtie2.sif".to_owned(),
                inputs: BTreeMap::from([
                    ("reads_r1".to_owned(), "sample_R1.fastq.gz".to_owned()),
                    ("reads_r2".to_owned(), "sample_R2.fastq.gz".to_owned()),
                    ("reference_index".to_owned(), "indexes/grch38".to_owned()),
                ]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([
                    ("threads".to_owned(), "8".to_owned()),
                    ("read_group_id".to_owned(), "sample-01".to_owned()),
                    ("read_group_sample".to_owned(), "sample-01".to_owned()),
                ]),
                outputs: BTreeMap::from([("raw_sam".to_owned(), "bowtie2.sam".to_owned())]),
            },
        )?,
        [
            "/images/bowtie2.sif",
            "-x",
            "indexes/grch38",
            "-1",
            "sample_R1.fastq.gz",
            "-2",
            "sample_R2.fastq.gz",
            "--very-sensitive",
            "--rg-id",
            "sample-01",
            "--rg",
            "SM:sample-01",
            "-p",
            "8",
            "-S",
            "bowtie2.sam",
        ]
    );
    assert_eq!(
        render_tool_command(
            bwa,
            &ToolBindingRenderContext {
                executable: "/images/bwa.sif".to_owned(),
                inputs: BTreeMap::from([
                    ("reads_r1".to_owned(), "sample_R1.fastq.gz".to_owned()),
                    ("reads_r2".to_owned(), "sample_R2.fastq.gz".to_owned()),
                    ("reference".to_owned(), "grch38.fa".to_owned()),
                ]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([
                    ("threads".to_owned(), "8".to_owned()),
                    ("read_group".to_owned(), "@RG\\tID:sample-01\\tSM:sample-01".to_owned()),
                ]),
                outputs: BTreeMap::from([("raw_sam".to_owned(), "bwa.sam".to_owned())]),
            },
        )?,
        [
            "/images/bwa.sif",
            "mem",
            "-t",
            "8",
            "-R",
            "@RG\\tID:sample-01\\tSM:sample-01",
            "-o",
            "bwa.sam",
            "grch38.fa",
            "sample_R1.fastq.gz",
            "sample_R2.fastq.gz",
        ]
    );
    Ok(())
}

#[test]
fn canonical_bam_align_bindings_project_equal_scheduler_threads() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    for binding in registry.bindings_for_stage(&StageId::new(ALIGN_STAGE)) {
        let mut defaults = BTreeMap::from([("threads".to_owned(), "8".to_owned())]);
        for parameter in &binding.parameters {
            defaults.entry(parameter.name.clone()).or_insert_with(|| "admitted-value".to_owned());
        }
        let effective = project_tool_parameters(&ToolParameterProjectionRequest {
            binding,
            scientific_defaults: &defaults,
            input_derived: &BTreeMap::new(),
            operator_overrides: &BTreeMap::new(),
            immutable_parameters: &BTreeSet::new(),
            allocated_cpus: 8,
        })?;

        assert_eq!(effective.resources.threads, 8);
        assert_eq!(effective.parameters["threads"].value, "8");
        assert_eq!(
            effective.parameters["threads"].source,
            EffectiveToolParameterSourceV1::ScientificDefault
        );
    }
    Ok(())
}

#[test]
fn binding_renders_declared_projections_without_tool_specific_code() -> Result<()> {
    let binding = binding(
        "fastq.trim_reads",
        ToolArtifactSourceV1::CatalogImage { catalog_tool_id: ToolId::new("fastp") },
    );
    validate_tool_binding(&binding)?;
    let context = ToolBindingRenderContext {
        executable: "/images/fastp.sif".to_string(),
        inputs: BTreeMap::from([("reads".to_string(), "reads.fastq.gz".to_string())]),
        input_sets: BTreeMap::new(),
        parameters: BTreeMap::from([("threads".to_string(), "8".to_string())]),
        outputs: BTreeMap::from([("trimmed".to_string(), "trimmed.fastq.gz".to_string())]),
    };

    assert_eq!(
        render_tool_command(&binding, &context)?,
        ["/images/fastp.sif", "--in=reads.fastq.gz", "--out", "trimmed.fastq.gz", "--thread", "8",]
    );
    Ok(())
}

#[test]
fn binding_expands_multi_path_inputs_as_distinct_argv_tokens() -> Result<()> {
    let mut binding = binding("fastq.profile_reads", catalog_source());
    binding.command_template = vec![
        "${executable}".to_string(),
        "stats".to_string(),
        "${inputs:reads}".to_string(),
        "--out".to_string(),
        "${output:trimmed}".to_string(),
        "--threads".to_string(),
        "${parameter:threads}".to_string(),
    ];
    validate_tool_binding(&binding)?;

    let command = render_tool_command(
        &binding,
        &ToolBindingRenderContext {
            executable: "/images/seqkit.sif".to_string(),
            inputs: BTreeMap::new(),
            input_sets: BTreeMap::from([(
                "reads".to_string(),
                vec!["sample_R1.fastq.gz".to_string(), "sample_R2.fastq.gz".to_string()],
            )]),
            parameters: BTreeMap::from([("threads".to_string(), "8".to_string())]),
            outputs: BTreeMap::from([("trimmed".to_string(), "summary.tsv".to_string())]),
        },
    )?;

    assert_eq!(
        command,
        [
            "/images/seqkit.sif",
            "stats",
            "sample_R1.fastq.gz",
            "sample_R2.fastq.gz",
            "--out",
            "summary.tsv",
            "--threads",
            "8",
        ]
    );
    Ok(())
}

#[test]
fn binding_rejects_empty_multi_path_input_sets() -> Result<()> {
    let mut binding = binding("fastq.profile_reads", catalog_source());
    binding.command_template[1] = "${inputs:reads}".to_string();
    validate_tool_binding(&binding)?;

    let error = render_tool_command(
        &binding,
        &ToolBindingRenderContext {
            executable: "/images/seqkit.sif".to_string(),
            inputs: BTreeMap::new(),
            input_sets: BTreeMap::from([("reads".to_string(), Vec::new())]),
            parameters: BTreeMap::from([("threads".to_string(), "8".to_string())]),
            outputs: BTreeMap::from([("trimmed".to_string(), "summary.tsv".to_string())]),
        },
    )
    .err()
    .ok_or_else(|| anyhow::anyhow!("empty multi-path input set was accepted"))?;

    assert!(error.to_string().contains("requires non-empty paths"));
    Ok(())
}

#[test]
fn binding_rejects_embedded_multi_path_input_projections() -> Result<()> {
    let mut binding = binding("fastq.profile_reads", catalog_source());
    binding.command_template[1] = "--reads=${inputs:reads}".to_string();

    let error = validate_tool_binding(&binding)
        .err()
        .ok_or_else(|| anyhow::anyhow!("embedded multi-path input projection was accepted"))?;
    assert!(error.to_string().contains("must occupy one complete command token"));
    Ok(())
}

#[test]
fn registry_allows_one_tool_to_bind_multiple_stages() -> Result<()> {
    let mut registry = ToolBindingRegistry::default();
    registry.insert(binding("fastq.trim_reads", catalog_source()))?;
    registry.insert(binding("fastq.quality_filter", catalog_source()))?;

    assert_eq!(registry.bindings_for_tool(&ToolId::new("fastp")).len(), 2);
    assert_eq!(registry.bindings().len(), 2);
    Ok(())
}

#[test]
fn external_artifacts_use_the_same_binding_interface() -> Result<()> {
    let mut external = binding(
        "bam.ancient_dna_damage",
        ToolArtifactSourceV1::ExternalArtifact {
            artifact_kind: "operator_supplied_sif".to_string(),
            version_probe: vec!["${executable}".to_string(), "--version".to_string()],
            checksum_required: true,
        },
    );
    external.binding_id = "bam.ancient_dna_damage__angsd".to_string();
    external.tool_id = ToolId::new("angsd");
    external.inputs = vec![BindingInputV1 {
        name: "alignment".to_string(),
        artifact_contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_string(),
        required: true,
    }];
    external.outputs = vec![BindingOutputV1 {
        name: "damage_report".to_string(),
        artifact_contract_id: "bijux.auxiliary.artifact.table.v1".to_string(),
        discovery: OutputDiscoveryV1::ExactPath,
    }];
    external.command_template = vec![
        "${executable}".to_string(),
        "-i".to_string(),
        "${input:alignment}".to_string(),
        "-out".to_string(),
        "${output:damage_report}".to_string(),
        "-P".to_string(),
        "${parameter:threads}".to_string(),
    ];
    external.normalizer_id = "bam.ancient_dna_damage.v1".to_string();
    external.downstream_probe_id = "report.ancient_dna_damage-readable.v1".to_string();

    validate_tool_binding(&external)?;
    let context = ToolBindingRenderContext {
        executable: "/operator/artifacts/angsd.sif".to_string(),
        inputs: BTreeMap::from([("alignment".to_string(), "sample.bam".to_string())]),
        input_sets: BTreeMap::new(),
        parameters: BTreeMap::from([("threads".to_string(), "8".to_string())]),
        outputs: BTreeMap::from([("damage_report".to_string(), "damage.tsv".to_string())]),
    };
    assert_eq!(render_tool_command(&external, &context)?[0], "/operator/artifacts/angsd.sif");

    let mut registry = ToolBindingRegistry::default();
    registry.insert(external)?;
    assert!(registry
        .binding(&StageId::new("bam.ancient_dna_damage"), &ToolId::new("angsd"))
        .is_some());
    Ok(())
}

#[test]
fn registry_rejects_duplicate_stage_tool_pairs() -> Result<()> {
    let mut registry = ToolBindingRegistry::default();
    registry.insert(binding("fastq.trim_reads", catalog_source()))?;

    let Err(error) = registry.insert(binding("fastq.trim_reads", catalog_source())) else {
        anyhow::bail!("duplicate stage/tool binding was accepted");
    };
    assert!(error.to_string().contains("duplicate tool binding"));
    Ok(())
}

#[test]
fn undeclared_command_projection_fails_conformance() -> Result<()> {
    let mut invalid = binding("fastq.trim_reads", catalog_source());
    invalid.command_template.push("${input:reference}".to_string());

    let Err(error) = validate_tool_binding(&invalid) else {
        anyhow::bail!("undeclared projection was accepted");
    };
    assert!(error.to_string().contains("undeclared input `reference`"));
    Ok(())
}

#[test]
fn unknown_artifact_contracts_fail_conformance() -> Result<()> {
    let mut invalid_input = binding("fastq.trim_reads", catalog_source());
    invalid_input.inputs[0].artifact_contract_id = "bijux.fastq.artifact.unknown.v1".to_string();
    let error = validate_tool_binding(&invalid_input)
        .err()
        .ok_or_else(|| anyhow::anyhow!("unknown input artifact contract was accepted"))?;
    assert!(error.to_string().contains("unknown artifact contract"));

    let mut invalid_output = binding("fastq.trim_reads", catalog_source());
    invalid_output.outputs[0].artifact_contract_id = "bijux.fastq.artifact.unknown.v1".to_string();
    let error = validate_tool_binding(&invalid_output)
        .err()
        .ok_or_else(|| anyhow::anyhow!("unknown output artifact contract was accepted"))?;
    assert!(error.to_string().contains("unknown artifact contract"));

    let mut invalid_normalized = binding("fastq.trim_reads", catalog_source());
    invalid_normalized.normalized_output_contract_id =
        "bijux.benchmark.fastq.other_stage.bundle.v1".to_string();
    let error = validate_tool_binding(&invalid_normalized)
        .err()
        .ok_or_else(|| anyhow::anyhow!("unowned normalized output contract was accepted"))?;
    assert!(error
        .to_string()
        .contains("neither a canonical artifact nor the stage benchmark bundle"));
    Ok(())
}

#[test]
fn parameter_projection_records_provenance_and_cpu_fairness() -> Result<()> {
    let binding = binding("fastq.trim_reads", catalog_source());
    let defaults = BTreeMap::from([("threads".to_owned(), "8".to_owned())]);
    let input_derived = BTreeMap::new();
    let overrides = BTreeMap::new();
    let immutable = BTreeSet::new();
    let effective = project_tool_parameters(&ToolParameterProjectionRequest {
        binding: &binding,
        scientific_defaults: &defaults,
        input_derived: &input_derived,
        operator_overrides: &overrides,
        immutable_parameters: &immutable,
        allocated_cpus: 8,
    })?;

    assert_eq!(effective.schema_version, EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1);
    assert_eq!(effective.resources.threads, effective.allocated_cpus);
    assert_eq!(
        effective.parameters["threads"].source,
        EffectiveToolParameterSourceV1::ScientificDefault
    );
    let command = render_tool_command(
        &binding,
        &ToolBindingRenderContext {
            executable: "/images/fastp.sif".to_owned(),
            inputs: BTreeMap::from([("reads".to_owned(), "reads.fastq.gz".to_owned())]),
            input_sets: BTreeMap::new(),
            parameters: effective.render_values(),
            outputs: BTreeMap::from([("trimmed".to_owned(), "trimmed.fastq.gz".to_owned())]),
        },
    )?;
    assert_eq!(command.last().map(String::as_str), Some("8"));
    Ok(())
}

#[test]
fn parameter_projection_records_precedence() -> Result<()> {
    let binding = binding("fastq.trim_reads", catalog_source());
    let defaults = BTreeMap::from([("threads".to_owned(), "2".to_owned())]);
    let input_derived = BTreeMap::from([("threads".to_owned(), "4".to_owned())]);
    let overrides = BTreeMap::from([("threads".to_owned(), "8".to_owned())]);
    let immutable = BTreeSet::new();
    let effective = project_tool_parameters(&ToolParameterProjectionRequest {
        binding: &binding,
        scientific_defaults: &defaults,
        input_derived: &input_derived,
        operator_overrides: &overrides,
        immutable_parameters: &immutable,
        allocated_cpus: 8,
    })?;

    assert_eq!(effective.parameters["threads"].value, "8");
    assert_eq!(
        effective.parameters["threads"].source,
        EffectiveToolParameterSourceV1::OperatorOverride
    );
    Ok(())
}

#[test]
fn parameter_projection_rejects_immutable_overrides_and_thread_drift() -> Result<()> {
    let binding = binding("fastq.trim_reads", catalog_source());
    let defaults = BTreeMap::from([("threads".to_owned(), "8".to_owned())]);
    let input_derived = BTreeMap::new();
    let overrides = BTreeMap::from([("threads".to_owned(), "4".to_owned())]);
    let immutable = BTreeSet::from(["threads".to_owned()]);
    let request = ToolParameterProjectionRequest {
        binding: &binding,
        scientific_defaults: &defaults,
        input_derived: &input_derived,
        operator_overrides: &overrides,
        immutable_parameters: &immutable,
        allocated_cpus: 8,
    };
    let error = project_tool_parameters(&request)
        .err()
        .ok_or_else(|| anyhow::anyhow!("immutable override was accepted"))?;
    assert!(error.to_string().contains("immutable parameter"));

    let no_overrides = BTreeMap::new();
    let no_immutable = BTreeSet::new();
    let cpu_drift = ToolParameterProjectionRequest {
        operator_overrides: &no_overrides,
        immutable_parameters: &no_immutable,
        allocated_cpus: 4,
        ..request
    };
    let error = project_tool_parameters(&cpu_drift)
        .err()
        .ok_or_else(|| anyhow::anyhow!("CPU drift was accepted"))?;
    assert!(error.to_string().contains("binding threads"));
    Ok(())
}

fn catalog_source() -> ToolArtifactSourceV1 {
    ToolArtifactSourceV1::CatalogImage { catalog_tool_id: ToolId::new("fastp") }
}

fn binding(stage_id: &str, artifact_source: ToolArtifactSourceV1) -> ToolBindingV1 {
    ToolBindingV1 {
        schema_version: TOOL_BINDING_SCHEMA_V1.to_string(),
        binding_id: format!("{stage_id}__fastp"),
        stage_id: StageId::new(stage_id),
        tool_id: ToolId::new("fastp"),
        artifact_source,
        capabilities: vec!["paired_or_single_reads".to_string()],
        inputs: vec![BindingInputV1 {
            name: "reads".to_string(),
            artifact_contract_id: "bijux.fastq.artifact.reads.v1".to_string(),
            required: true,
        }],
        parameters: vec![BindingParameterV1 { name: "threads".to_string(), required: true }],
        outputs: vec![BindingOutputV1 {
            name: "trimmed".to_string(),
            artifact_contract_id: "bijux.fastq.artifact.reads.v1".to_string(),
            discovery: OutputDiscoveryV1::ExactPath,
        }],
        normalized_output_contract_id: "bijux.fastq.artifact.reads.v1".to_string(),
        command_template: vec![
            "${executable}".to_string(),
            "--in=${input:reads}".to_string(),
            "--out".to_string(),
            "${output:trimmed}".to_string(),
            "--thread".to_string(),
            "${parameter:threads}".to_string(),
        ],
        resources: ToolConstraints {
            runtime: "apptainer".to_string(),
            mem_gb: 4,
            tmp_gb: 2,
            threads: 8,
        },
        normalizer_id: "fastq.trim_reads.v1".to_string(),
        downstream_probe_id: "fastq.reads-readable.v1".to_string(),
        limitations: vec!["interleaved_reads_not_supported".to_string()],
    }
}
