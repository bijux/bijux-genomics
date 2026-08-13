use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_external_artifact_tool_bindings, canonical_retained_tool_bindings,
    canonical_tool_bindings, project_tool_parameters, render_tool_command, validate_tool_binding,
    BindingInputV1, BindingOutputV1, BindingParameterV1, EffectiveToolParameterSourceV1,
    OutputDiscoveryV1, ToolArtifactSourceV1, ToolBindingRegistry, ToolBindingRenderContext,
    ToolBindingV1, ToolParameterProjectionRequest, EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1,
    TOOL_BINDING_SCHEMA_V1,
};

const ALIGN_STAGE: &str = "bam.align";
const ALIGN_BUNDLE: &str = "bijux.benchmark.bam.align.bundle.v1";
const UNSORTED_SAM: &str = "bijux.sam.artifact.unsorted.v1";
const FASTQ_PAIRED: &str = "bijux.fastq.artifact.paired_end.v1";
const GENOTYPING_STAGE: &str = "bam.genotyping";
const COVERAGE_STAGE: &str = "bam.coverage";
const ENDOGENOUS_CONTENT_STAGE: &str = "bam.endogenous_content";
const MAPPING_SUMMARY_STAGE: &str = "bam.mapping_summary";
const DUPLICATION_METRICS_STAGE: &str = "bam.duplication_metrics";
const INSERT_SIZE_STAGE: &str = "bam.insert_size";
const GC_BIAS_STAGE: &str = "bam.gc_bias";
const SEX_STAGE: &str = "bam.sex";
const DAMAGE_STAGE: &str = "bam.damage";
const AUTHENTICITY_STAGE: &str = "bam.authenticity";
const EXTRACT_UMIS_STAGE: &str = "fastq.extract_umis";
const CORRECT_ERRORS_STAGE: &str = "fastq.correct_errors";
const TRIM_READS_STAGE: &str = "fastq.trim_reads";
const CLUSTER_OTUS_STAGE: &str = "fastq.cluster_otus";
const MERGE_PAIRS_STAGE: &str = "fastq.merge_pairs";
const SCREEN_TAXONOMY_STAGE: &str = "fastq.screen_taxonomy";
const DETECT_ADAPTERS_STAGE: &str = "fastq.detect_adapters";
const INFER_ASVS_STAGE: &str = "fastq.infer_asvs";
const FILTER_LOW_COMPLEXITY_STAGE: &str = "fastq.filter_low_complexity";
const FILTER_READS_STAGE: &str = "fastq.filter_reads";
const INDEX_REFERENCE_STAGE: &str = "fastq.index_reference";
const VALIDATE_READS_STAGE: &str = "fastq.validate_reads";
const PROFILE_READ_LENGTHS_STAGE: &str = "fastq.profile_read_lengths";
const PROFILE_READS_STAGE: &str = "fastq.profile_reads";

#[test]
fn canonical_retained_registry_owns_bam_align_tools_in_stable_order() -> Result<()> {
    let retained = canonical_retained_tool_bindings()?;
    let align = retained.bindings_for_stage(&StageId::new(ALIGN_STAGE));

    assert_eq!(retained.bindings().len(), 151);
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
        assert_eq!(binding.resources.runtime, "apptainer");
        assert!(!binding.command_template.iter().any(|token| [
            "bash", "sh", "python", "samtools", "gatk"
        ]
        .iter()
        .any(|name| token.contains(name))));
    }

    let all = canonical_tool_bindings()?;
    assert_eq!(all.bindings().len(), 159);
    assert_eq!(all.bindings_for_stage(&StageId::new(ALIGN_STAGE)).len(), 2);
    Ok(())
}

#[test]
fn canonical_registry_owns_complete_supported_bam_closure() -> Result<()> {
    let registry = canonical_tool_bindings()?;
    let expected = [
        ("bam.bias_mitigation", "mapdamage2"),
        ("bam.complexity", "preseq"),
        ("bam.contamination", "contammix"),
        ("bam.contamination", "schmutzi"),
        ("bam.contamination", "verifybamid2"),
        ("bam.filter", "bamtools"),
        ("bam.filter", "bedtools"),
        ("bam.filter", "samtools"),
        ("bam.haplogroups", "yleaf"),
        ("bam.kinship", "king"),
        ("bam.length_filter", "picard"),
        ("bam.length_filter", "samtools"),
        ("bam.mapq_filter", "bamtools"),
        ("bam.mapq_filter", "samtools"),
        ("bam.markdup", "picard"),
        ("bam.markdup", "samtools"),
        ("bam.overlap_correction", "bamutil"),
        ("bam.qc_pre", "multiqc"),
        ("bam.qc_pre", "samtools"),
        ("bam.recalibration", "gatk"),
        ("bam.sex", "yleaf"),
        ("bam.validate", "bamtools"),
        ("bam.validate", "bedtools"),
        ("bam.validate", "samtools"),
    ];
    assert_eq!(expected.len(), 24);
    for (stage_id, tool_id) in expected {
        let binding = registry
            .binding(&StageId::new(stage_id), &ToolId::new(tool_id))
            .ok_or_else(|| anyhow::anyhow!("canonical {stage_id}/{tool_id} binding is missing"))?;
        assert_eq!(
            binding.normalized_output_contract_id,
            format!("bijux.benchmark.{stage_id}.bundle.v1")
        );
        assert!(binding.normalizer_id.contains(stage_id.trim_start_matches("bam.")));
    }
    assert_eq!(
        registry.bindings().filter(|binding| binding.stage_id.as_str().starts_with("bam.")).count(),
        49
    );
    Ok(())
}

#[test]
fn canonical_bam_portfolio_bindings_render_native_commands() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let cases = [
        ("bam.complexity", "preseq", ["lc_extrap", "complexity_curve"]),
        ("bam.contamination", "verifybamid2", ["--BamFile", "--SVDPrefix"]),
        ("bam.filter", "samtools", ["view -b -q 20", "flagstat_before"]),
        ("bam.length_filter", "picard", ["FilterSamReads", "getReadLength() >= 30"]),
        ("bam.markdup", "samtools", ["markdup", "markdup_bam"]),
        ("bam.overlap_correction", "bamutil", ["clipOverlap", "--stats"]),
        ("bam.recalibration", "gatk", ["BaseRecalibrator", "ApplyBQSR"]),
        ("bam.sex", "yleaf", ["-bam", "--reference_genome"]),
        ("bam.validate", "bedtools", ["bamtobed", "validation_report"]),
    ];

    for (stage_id, tool_id, fragments) in cases {
        let binding = registry
            .binding(&StageId::new(stage_id), &ToolId::new(tool_id))
            .ok_or_else(|| anyhow::anyhow!("canonical {stage_id}/{tool_id} binding is missing"))?;
        let inputs = binding
            .inputs
            .iter()
            .map(|value| (value.name.clone(), format!("/input/{}", value.name)))
            .collect();
        let outputs = binding
            .outputs
            .iter()
            .map(|value| (value.name.clone(), format!("/output/{}", value.name)))
            .collect();
        let rendered = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: format!("/images/{tool_id}"),
                inputs,
                outputs,
                ..ToolBindingRenderContext::default()
            },
        )?
        .join(" ");
        assert!(!rendered.contains("${"), "{stage_id}/{tool_id} retains a projection");
        for fragment in fragments {
            assert!(rendered.contains(fragment), "{stage_id}/{tool_id} misses {fragment}");
        }
    }
    Ok(())
}

#[test]
fn canonical_registry_owns_complete_supported_fastq_closure() -> Result<()> {
    let registry = canonical_tool_bindings()?;
    let expected = [
        ("fastq.deplete_host", "bowtie2"),
        ("fastq.deplete_reference_contaminants", "bowtie2"),
        ("fastq.deplete_rrna", "sortmerna"),
        ("fastq.detect_duplicates_premerge", "bijux_dna"),
        ("fastq.estimate_library_complexity_prealign", "bijux_dna"),
        ("fastq.filter_reads", "bbduk"),
        ("fastq.filter_reads", "fastp"),
        ("fastq.filter_reads", "prinseq"),
        ("fastq.normalize_abundance", "seqkit"),
        ("fastq.normalize_primers", "cutadapt"),
        ("fastq.profile_overrepresented_sequences", "fastq_scan"),
        ("fastq.profile_overrepresented_sequences", "fastqc"),
        ("fastq.profile_overrepresented_sequences", "seqkit"),
        ("fastq.remove_chimeras", "vsearch"),
        ("fastq.remove_duplicates", "clumpify"),
        ("fastq.report_qc", "multiqc"),
        ("fastq.trim_polyg_tails", "bbduk"),
        ("fastq.trim_polyg_tails", "fastp"),
        ("fastq.trim_reads", "adapterremoval"),
        ("fastq.trim_reads", "bbduk"),
        ("fastq.trim_reads", "cutadapt"),
        ("fastq.trim_reads", "fastp"),
        ("fastq.trim_reads", "leehom"),
        ("fastq.trim_reads", "prinseq"),
        ("fastq.trim_reads", "seqkit"),
        ("fastq.trim_reads", "seqprep"),
        ("fastq.trim_terminal_damage", "adapterremoval"),
        ("fastq.trim_terminal_damage", "cutadapt"),
        ("fastq.trim_terminal_damage", "seqkit"),
        ("fastq.validate_reads", "fastq_scan"),
        ("fastq.validate_reads", "fastqc"),
    ];
    assert_eq!(expected.len(), 31);
    for (stage_id, tool_id) in expected {
        let binding = registry
            .binding(&StageId::new(stage_id), &ToolId::new(tool_id))
            .ok_or_else(|| anyhow::anyhow!("canonical {stage_id}/{tool_id} binding is missing"))?;
        assert_eq!(
            binding.normalized_output_contract_id,
            format!("bijux.benchmark.{stage_id}.bundle.v1")
        );
        assert!(binding.normalizer_id.contains(stage_id.trim_start_matches("fastq.")));
    }
    assert_eq!(
        registry
            .bindings()
            .filter(|binding| binding.stage_id.as_str().starts_with("fastq."))
            .count(),
        71
    );
    Ok(())
}

#[test]
fn canonical_fastq_portfolio_bindings_render_native_commands() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let cases = [
        (
            "fastq.deplete_host",
            "bowtie2",
            BTreeMap::from([
                ("reads_r1".to_owned(), "/input/reads.fastq.gz".to_owned()),
                ("reference_index".to_owned(), "/input/host-index".to_owned()),
            ]),
            BTreeMap::from([
                ("host_depleted_reads_r1".to_owned(), "/result/depleted.fastq.gz".to_owned()),
                ("removed_host_reads_r1".to_owned(), "/result/host.fastq.gz".to_owned()),
                ("host_mapping_metrics".to_owned(), "/result/host.metrics".to_owned()),
            ]),
            ["--un-gz", "--al-gz", "--met-file"],
        ),
        (
            "fastq.remove_chimeras",
            "vsearch",
            BTreeMap::from([(
                "representative_sequences".to_owned(),
                "/input/representatives.fasta".to_owned(),
            )]),
            BTreeMap::from([
                ("nonchimeric_sequences".to_owned(), "/result/nonchimeras.fasta".to_owned()),
                ("chimeric_sequences".to_owned(), "/result/chimeras.fasta".to_owned()),
                ("chimera_report".to_owned(), "/result/chimeras.tsv".to_owned()),
            ]),
            ["--uchime_denovo", "--nonchimeras", "--uchimeout"],
        ),
        (
            "fastq.trim_terminal_damage",
            "cutadapt",
            BTreeMap::from([("reads_r1".to_owned(), "/input/reads.fastq.gz".to_owned())]),
            BTreeMap::from([(
                "damage_trimmed_reads_r1".to_owned(),
                "/result/trimmed.fastq.gz".to_owned(),
            )]),
            ["--cores", "--cut", "--output"],
        ),
        (
            "fastq.report_qc",
            "multiqc",
            BTreeMap::from([("qc_inputs".to_owned(), "/input/qc".to_owned())]),
            BTreeMap::from([("multiqc_data".to_owned(), "/result/multiqc".to_owned())]),
            ["--outdir", "--filename", "multiqc_report.html"],
        ),
    ];

    for (stage_id, tool_id, inputs, outputs, fragments) in cases {
        let binding = registry
            .binding(&StageId::new(stage_id), &ToolId::new(tool_id))
            .ok_or_else(|| anyhow::anyhow!("canonical {stage_id}/{tool_id} binding is missing"))?;
        let rendered = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool_id.to_owned(),
                inputs,
                outputs,
                ..ToolBindingRenderContext::default()
            },
        )?
        .join(" ");
        for fragment in fragments {
            assert!(rendered.contains(fragment), "{stage_id}/{tool_id} misses {fragment}");
        }
    }
    Ok(())
}

#[test]
fn canonical_registry_owns_every_supported_vcf_pair() -> Result<()> {
    let registry = canonical_tool_bindings()?;
    for binding in
        registry.bindings().filter(|binding| binding.stage_id.as_str().starts_with("vcf."))
    {
        assert_eq!(
            binding.normalized_output_contract_id,
            format!("bijux.benchmark.{}.bundle.v1", binding.stage_id)
        );
        assert!(
            binding.normalizer_id.starts_with("parse_")
                && binding.normalizer_id.ends_with("metrics"),
            "{}/{} names a non-canonical VCF normalizer: {}",
            binding.stage_id,
            binding.tool_id,
            binding.normalizer_id
        );
    }
    let observed = registry
        .bindings()
        .filter(|binding| binding.stage_id.as_str().starts_with("vcf."))
        .map(|binding| (binding.stage_id.to_string(), binding.tool_id.to_string()))
        .collect::<BTreeSet<_>>();
    let expected = [
        ("vcf.admixture", "plink"),
        ("vcf.admixture", "plink2"),
        ("vcf.call", "bcftools"),
        ("vcf.call_diploid", "bcftools"),
        ("vcf.call_gl", "angsd"),
        ("vcf.call_gl", "bcftools"),
        ("vcf.call_pseudohaploid", "angsd"),
        ("vcf.call_pseudohaploid", "bcftools"),
        ("vcf.damage_filter", "bcftools"),
        ("vcf.demography", "ibdne"),
        ("vcf.filter", "bcftools"),
        ("vcf.gl_propagation", "bcftools"),
        ("vcf.ibd", "germline"),
        ("vcf.ibd", "ibdhap"),
        ("vcf.ibd", "ibdseq"),
        ("vcf.imputation_metrics", "beagle"),
        ("vcf.imputation_metrics", "glimpse"),
        ("vcf.imputation_metrics", "impute5"),
        ("vcf.imputation_metrics", "minimac4"),
        ("vcf.impute", "beagle"),
        ("vcf.impute", "glimpse"),
        ("vcf.impute", "impute5"),
        ("vcf.impute", "minimac4"),
        ("vcf.pca", "eigensoft"),
        ("vcf.pca", "plink2"),
        ("vcf.phasing", "beagle"),
        ("vcf.phasing", "eagle"),
        ("vcf.phasing", "shapeit"),
        ("vcf.phasing", "shapeit5"),
        ("vcf.population_structure", "eigensoft"),
        ("vcf.population_structure", "plink"),
        ("vcf.population_structure", "plink2"),
        ("vcf.postprocess", "bcftools"),
        ("vcf.prepare_reference_panel", "bcftools"),
        ("vcf.qc", "bcftools"),
        ("vcf.qc", "plink"),
        ("vcf.qc", "plink2"),
        ("vcf.roh", "plink2"),
        ("vcf.stats", "bcftools"),
    ]
    .into_iter()
    .map(|(stage_id, tool_id)| (stage_id.to_string(), tool_id.to_string()))
    .collect::<BTreeSet<_>>();

    assert_eq!(observed, expected);
    Ok(())
}

#[test]
fn canonical_vcf_bindings_render_native_command_profiles() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let cases = [
        (
            "vcf.call",
            "bcftools",
            BTreeMap::from([
                ("bam".to_owned(), "/input/sample.bam".to_owned()),
                ("reference".to_owned(), "/input/reference.fa".to_owned()),
            ]),
            BTreeMap::from([("called_vcf".to_owned(), "/result/called.vcf.gz".to_owned())]),
            ["mpileup", " call ", "index -f"],
        ),
        (
            "vcf.phasing",
            "shapeit",
            BTreeMap::from([
                ("cohort_vcf".to_owned(), "/input/cohort.vcf.gz".to_owned()),
                ("reference_panel_vcf".to_owned(), "/input/panel.vcf.gz".to_owned()),
                ("genetic_map".to_owned(), "/input/map.txt".to_owned()),
            ]),
            BTreeMap::from([("phased_vcf".to_owned(), "/result/phased.vcf.gz".to_owned())]),
            ["--input", "--reference", "--output"],
        ),
        (
            "vcf.ibd",
            "ibdhap",
            BTreeMap::from([
                ("cohort_vcf".to_owned(), "/input/cohort.vcf.gz".to_owned()),
                ("genetic_map".to_owned(), "/input/map.txt".to_owned()),
            ]),
            BTreeMap::from([("ibd_segments".to_owned(), "/result/segments.tsv".to_owned())]),
            ["--vcf", "--genetic-map", "--out"],
        ),
        (
            "vcf.pca",
            "eigensoft",
            BTreeMap::from([
                ("cohort_vcf".to_owned(), "/input/cohort.vcf.gz".to_owned()),
                ("sample_metadata".to_owned(), "/input/populations.txt".to_owned()),
            ]),
            BTreeMap::from([("pca_report".to_owned(), "/result/pca.evec".to_owned())]),
            ["convertf", "smartpca", "poplistname"],
        ),
    ];

    for (stage_id, tool_id, inputs, outputs, fragments) in cases {
        let binding = registry
            .binding(&StageId::new(stage_id), &ToolId::new(tool_id))
            .ok_or_else(|| anyhow::anyhow!("canonical {stage_id}/{tool_id} binding is missing"))?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool_id.to_owned(),
                inputs,
                outputs,
                ..ToolBindingRenderContext::default()
            },
        )?
        .join(" ");
        for fragment in fragments {
            assert!(
                command.contains(fragment),
                "{stage_id}/{tool_id} command is missing native fragment {fragment}: {command}"
            );
        }
    }
    Ok(())
}

#[test]
fn canonical_damage_bindings_own_representative_qualification_pairs() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    assert_eq!(
        registry
            .bindings_for_stage(&StageId::new(DAMAGE_STAGE))
            .iter()
            .map(|binding| binding.tool_id.to_string())
            .collect::<Vec<_>>(),
        ["addeam", "damageprofiler", "mapdamage2", "pmdtools", "pydamage"]
    );
    assert_eq!(
        registry
            .bindings_for_stage(&StageId::new(AUTHENTICITY_STAGE))
            .iter()
            .map(|binding| binding.tool_id.to_string())
            .collect::<Vec<_>>(),
        ["authenticct", "damageprofiler", "pmdtools"]
    );
    let all = canonical_tool_bindings()?;
    assert_eq!(
        all.bindings_for_stage(&StageId::new(DAMAGE_STAGE))
            .iter()
            .map(|binding| binding.tool_id.to_string())
            .collect::<Vec<_>>(),
        ["addeam", "damageprofiler", "mapdamage2", "ngsbriggs", "pmdtools", "pydamage"]
    );
    Ok(())
}

#[test]
fn canonical_addeam_and_mapdamage2_bindings_render_native_profiles() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let addeam = registry
        .binding(&StageId::new(DAMAGE_STAGE), &ToolId::new("addeam"))
        .ok_or_else(|| anyhow::anyhow!("canonical AdDeam damage binding is missing"))?;
    let addeam_command = render_tool_command(
        addeam,
        &ToolBindingRenderContext {
            executable: "addeam".to_owned(),
            inputs: BTreeMap::from([("bam".to_owned(), "/input/sample.bam".to_owned())]),
            outputs: BTreeMap::from([(
                "damage_profiles".to_owned(),
                "/result/addeam-profiles".to_owned(),
            )]),
            ..ToolBindingRenderContext::default()
        },
    )?;
    assert!(addeam_command[2].contains("addeam' profile"));
    assert!(addeam_command[2].contains("/input/sample.bam"));
    assert!(addeam_command[2].contains("-both"));

    let mapdamage2 = registry
        .binding(&StageId::new(DAMAGE_STAGE), &ToolId::new("mapdamage2"))
        .ok_or_else(|| anyhow::anyhow!("canonical mapDamage2 damage binding is missing"))?;
    let command = render_tool_command(
        mapdamage2,
        &ToolBindingRenderContext {
            executable: "mapdamage2".to_owned(),
            inputs: BTreeMap::from([
                ("bam".to_owned(), "/input/sample.bam".to_owned()),
                ("reference".to_owned(), "/input/reference.fa".to_owned()),
            ]),
            outputs: BTreeMap::from([(
                "mapdamage2_reports".to_owned(),
                "/result/mapdamage2".to_owned(),
            )]),
            ..ToolBindingRenderContext::default()
        },
    )?;
    assert_eq!(
        command,
        [
            "mapdamage2",
            "analyze",
            "--bam",
            "/input/sample.bam",
            "--reference",
            "/input/reference.fa",
            "--out-dir",
            "/result/mapdamage2",
        ]
    );
    Ok(())
}

#[test]
fn canonical_damageprofiler_and_pydamage_bindings_render_native_directories() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    for (stage, tool, input, input_path, output, directory, fragments) in [
        (
            AUTHENTICITY_STAGE,
            "damageprofiler",
            "sam",
            "/input/sample.calmd.sam",
            "damageprofiler_reports",
            "/result/damageprofiler",
            ["damageprofiler' -i", "-title bijux-sample"],
        ),
        (
            DAMAGE_STAGE,
            "pydamage",
            "bam",
            "/input/sample.bam",
            "pydamage_results",
            "/result/pydamage",
            ["--outdir", "--group"],
        ),
    ] {
        let binding = registry
            .binding(&StageId::new(stage), &ToolId::new(tool))
            .ok_or_else(|| anyhow::anyhow!("canonical {tool} binding is missing"))?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool.to_owned(),
                inputs: BTreeMap::from([(input.to_owned(), input_path.to_owned())]),
                outputs: BTreeMap::from([(output.to_owned(), directory.to_owned())]),
                ..ToolBindingRenderContext::default()
            },
        )?;
        let rendered = command.join(" ");
        assert!(rendered.contains(input_path));
        assert!(rendered.contains(directory));
        for fragment in fragments {
            assert!(rendered.contains(fragment), "{tool} command omitted `{fragment}`");
        }
    }
    Ok(())
}

#[test]
fn canonical_authenticct_binding_renders_admitted_deamination_interface() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(AUTHENTICITY_STAGE), &ToolId::new("authenticct"))
        .ok_or_else(|| anyhow::anyhow!("canonical AuthentiCT binding is missing"))?;
    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "authenticct".to_owned(),
            inputs: BTreeMap::from([("sam".to_owned(), "/input/sample.calmd.sam".to_owned())]),
            outputs: BTreeMap::from([(
                "deamination_table".to_owned(),
                "/result/deamination.tsv".to_owned(),
            )]),
            ..ToolBindingRenderContext::default()
        },
    )?;
    assert_eq!(
        command,
        [
            "authenticct",
            "deamination",
            "-m",
            "20",
            "-l",
            "30",
            "-b",
            "20",
            "-o",
            "/result/deamination.tsv",
            "/input/sample.calmd.sam",
        ]
    );
    Ok(())
}

#[test]
fn canonical_pmdtools_bindings_share_native_score_evidence_across_stages() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    for stage in [DAMAGE_STAGE, AUTHENTICITY_STAGE] {
        let binding = registry
            .binding(&StageId::new(stage), &ToolId::new("pmdtools"))
            .ok_or_else(|| anyhow::anyhow!("canonical PMDtools {stage} binding is missing"))?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: "pmdtools".to_owned(),
                inputs: BTreeMap::from([("bam".to_owned(), "/input/sample.bam".to_owned())]),
                outputs: BTreeMap::from([
                    ("pmd_scores".to_owned(), "/result/pmd-scores.tsv".to_owned()),
                    ("pmd_statistics".to_owned(), "/result/pmd-stats.txt".to_owned()),
                ]),
                ..ToolBindingRenderContext::default()
            },
        )?;
        let rendered = command.join(" ");
        assert!(rendered.contains("samtools view -h '/input/sample.bam'"));
        assert!(rendered.contains("'pmdtools' --printDS --stats"));
        assert!(rendered.contains("/result/pmd-scores.tsv"));
        assert!(rendered.contains("/result/pmd-stats.txt"));
    }
    Ok(())
}

#[test]
fn canonical_trim_read_bindings_preserve_representative_native_shapes() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let stage = StageId::new(TRIM_READS_STAGE);
    assert_eq!(
        registry
            .bindings_for_stage(&stage)
            .iter()
            .map(|binding| binding.tool_id.to_string())
            .collect::<Vec<_>>(),
        [
            "adapterremoval",
            "alientrimmer",
            "atropos",
            "bbduk",
            "cutadapt",
            "fastp",
            "fastx_clipper",
            "leehom",
            "prinseq",
            "seqkit",
            "seqprep",
            "skewer",
            "trim_galore",
            "trimmomatic",
        ]
    );
    Ok(())
}

#[test]
fn canonical_paired_trim_bindings_render_native_output_shapes() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let stage = StageId::new(TRIM_READS_STAGE);
    let paired_inputs = paired_merge_inputs();
    let paired_outputs = BTreeMap::from([
        ("trimmed_reads_r1".to_owned(), "/result/trimmed-R1.fastq.gz".to_owned()),
        ("trimmed_reads_r2".to_owned(), "/result/trimmed-R2.fastq.gz".to_owned()),
    ]);
    for (tool, inputs, parameters, required_fragments) in [
        (
            "atropos",
            paired_inputs.clone(),
            BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("adapter_r1".to_owned(), "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA".to_owned()),
                ("adapter_r2".to_owned(), "AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT".to_owned()),
                ("quality_cutoff".to_owned(), "20".to_owned()),
                ("min_length".to_owned(), "30".to_owned()),
            ]),
            ["-pe1", "-pe2", "/result/trimmed-R2.fastq.gz"],
        ),
        (
            "trim_galore",
            paired_inputs.clone(),
            BTreeMap::from([
                ("threads".to_owned(), "2".to_owned()),
                ("adapter_sequence".to_owned(), "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA".to_owned()),
                ("quality_cutoff".to_owned(), "20".to_owned()),
                ("min_length".to_owned(), "30".to_owned()),
            ]),
            ["--paired", "r1.list", "r2.list"],
        ),
        (
            "trimmomatic",
            paired_inputs.clone(),
            BTreeMap::from([
                ("threads".to_owned(), "2".to_owned()),
                ("quality_cutoff".to_owned(), "20".to_owned()),
                ("min_length".to_owned(), "30".to_owned()),
            ]),
            ["PE", "SLIDINGWINDOW:4:20", "MINLEN:30"],
        ),
    ] {
        let binding = registry
            .binding(&stage, &ToolId::new(tool))
            .ok_or_else(|| anyhow::anyhow!("canonical {tool} trim binding is missing"))?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool.to_owned(),
                inputs,
                input_sets: BTreeMap::new(),
                parameters,
                outputs: paired_outputs.clone(),
            },
        )?;
        let rendered = command.join(" ");
        for fragment in required_fragments {
            assert!(rendered.contains(fragment), "{tool} command omitted {fragment}");
        }
        assert_eq!(binding.outputs.len(), 2);
    }

    let alientrimmer = registry
        .binding(&stage, &ToolId::new("alientrimmer"))
        .ok_or_else(|| anyhow::anyhow!("canonical alientrimmer trim binding is missing"))?;
    let alientrimmer_command = render_tool_command(
        alientrimmer,
        &ToolBindingRenderContext {
            executable: "alientrimmer".to_owned(),
            inputs: paired_inputs
                .into_iter()
                .chain([("adapter_reference".to_owned(), "/input/adapters.fasta".to_owned())])
                .collect(),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::new(),
            outputs: paired_outputs,
        },
    )?;
    assert!(alientrimmer_command[2].contains("-a '/input/adapters.fasta'"));
    assert!(alientrimmer_command[2].contains("$prefix.1.fastq.gz"));
    assert!(alientrimmer_command[2].contains("$prefix.2.fastq.gz"));
    Ok(())
}

#[test]
fn canonical_single_end_trim_bindings_render_native_output_shapes() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let stage = StageId::new(TRIM_READS_STAGE);
    for (tool, parameters, required_fragment) in [
        (
            "fastx_clipper",
            BTreeMap::from([
                ("adapter_sequence".to_owned(), "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA".to_owned()),
                ("adapter_match_length".to_owned(), "34".to_owned()),
                ("min_length".to_owned(), "30".to_owned()),
            ]),
            "-M 34",
        ),
        (
            "skewer",
            BTreeMap::from([
                ("threads".to_owned(), "2".to_owned()),
                ("adapter_sequence".to_owned(), "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA".to_owned()),
                ("quality_cutoff".to_owned(), "20".to_owned()),
                ("min_length".to_owned(), "30".to_owned()),
            ]),
            "skewer-trimmed.fastq.gz",
        ),
    ] {
        let binding = registry
            .binding(&stage, &ToolId::new(tool))
            .ok_or_else(|| anyhow::anyhow!("canonical {tool} trim binding is missing"))?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool.to_owned(),
                inputs: BTreeMap::from([("reads_r1".to_owned(), "/input/reads.fastq".to_owned())]),
                input_sets: BTreeMap::new(),
                parameters,
                outputs: BTreeMap::from([(
                    "trimmed_reads_r1".to_owned(),
                    "/result/trimmed.fastq.gz".to_owned(),
                )]),
            },
        )?;
        assert!(command.join(" ").contains(required_fragment));
        assert_eq!(binding.inputs.len(), 1);
        assert_eq!(binding.outputs.len(), 1);
    }
    Ok(())
}

#[test]
fn canonical_merge_pair_bindings_preserve_each_native_output_shape() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let stage = StageId::new(MERGE_PAIRS_STAGE);
    assert_eq!(
        registry
            .bindings_for_stage(&stage)
            .iter()
            .map(|binding| binding.tool_id.to_string())
            .collect::<Vec<_>>(),
        ["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "seqprep", "vsearch"]
    );
    assert_adapterremoval_merge_binding(&registry, &stage)?;
    assert_bbmerge_merge_binding(&registry, &stage)?;
    assert_flash2_merge_binding(&registry, &stage)?;
    assert_directory_merge_bindings(&registry, &stage)?;
    assert_direct_merge_bindings(&registry, &stage)?;
    assert!(registry.bindings_for_stage(&stage).iter().all(|binding| {
        binding.downstream_probe_id == "benchmark.fastq.merge_pairs.consume"
            && binding.normalizer_id == "bijux_dna_planner_fastq::tool_adapters::fastq::merge_pairs"
    }));
    Ok(())
}

#[test]
fn canonical_error_correction_bindings_preserve_paired_native_outputs() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let stage = StageId::new(CORRECT_ERRORS_STAGE);
    assert_eq!(
        registry
            .bindings_for_stage(&stage)
            .iter()
            .map(|binding| binding.tool_id.to_string())
            .collect::<Vec<_>>(),
        ["bayeshammer", "lighter", "musket", "rcorrector"]
    );
    let inputs = paired_merge_inputs();
    let outputs = BTreeMap::from([
        ("corrected_reads_r1".to_owned(), "/result/corrected-R1.fastq.gz".to_owned()),
        ("corrected_reads_r2".to_owned(), "/result/corrected-R2.fastq.gz".to_owned()),
    ]);
    for (tool, parameters, required_fragments) in [
        (
            "bayeshammer",
            BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("phred_offset".to_owned(), "33".to_owned()),
                ("max_memory_gb".to_owned(), "16".to_owned()),
            ]),
            ["--phred-offset '33'", "r1.list", "r2.list"],
        ),
        (
            "lighter",
            BTreeMap::from([
                ("threads".to_owned(), "16".to_owned()),
                ("kmer_size".to_owned(), "31".to_owned()),
                ("genome_size".to_owned(), "2506966135".to_owned()),
            ]),
            ["-K '31' '2506966135'", "outputs.list", "normalize"],
        ),
        (
            "musket",
            BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("kmer_size".to_owned(), "21".to_owned()),
                ("musket_kmer_budget".to_owned(), "134217728".to_owned()),
            ]),
            ["-k '21' '134217728'", "-omulti", "-inorder"],
        ),
    ] {
        let binding = registry.binding(&stage, &ToolId::new(tool)).ok_or_else(|| {
            anyhow::anyhow!("canonical {tool} error-correction binding is missing")
        })?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool.to_owned(),
                inputs: inputs.clone(),
                input_sets: BTreeMap::new(),
                parameters,
                outputs: outputs.clone(),
            },
        )?;
        assert_eq!(command[..2], ["/bin/sh", "-c"]);
        for fragment in required_fragments {
            assert!(command[2].contains(fragment), "{tool} command omitted {fragment}");
        }
        assert!(binding.outputs.iter().all(|output| output.artifact_contract_id == FASTQ_PAIRED));
    }
    Ok(())
}

fn paired_merge_inputs() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("reads_r1".to_owned(), "/input/R1.fastq.gz".to_owned()),
        ("reads_r2".to_owned(), "/input/R2.fastq.gz".to_owned()),
    ])
}

fn assert_directory_merge_bindings(registry: &ToolBindingRegistry, stage: &StageId) -> Result<()> {
    for (tool, output_name, output_path, required_fragment) in [
        ("leehom", "leehom_result_directory", "/result/leehom", "-fqo '/result/leehom/leehom'"),
        ("pear", "pear_result_directory", "/result/pear", "-o '/result/pear/pear'"),
    ] {
        let binding = registry
            .binding(stage, &ToolId::new(tool))
            .ok_or_else(|| anyhow::anyhow!("canonical {tool} merge binding is missing"))?;
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: tool.to_owned(),
                inputs: paired_merge_inputs(),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([
                    ("threads".to_owned(), "4".to_owned()),
                    ("merge_overlap".to_owned(), "11".to_owned()),
                    ("min_len".to_owned(), "30".to_owned()),
                ]),
                outputs: BTreeMap::from([(output_name.to_owned(), output_path.to_owned())]),
            },
        )?;
        assert_eq!(command[..2], ["sh", "-lc"]);
        assert!(command[2].contains(required_fragment));
        assert_eq!(
            binding.outputs[0].artifact_contract_id,
            "bijux.auxiliary.artifact.directory.v1"
        );
    }
    Ok(())
}

fn assert_direct_merge_bindings(registry: &ToolBindingRegistry, stage: &StageId) -> Result<()> {
    let outputs = BTreeMap::from([
        ("merged_reads".to_owned(), "/result/merged.fastq.gz".to_owned()),
        ("unmerged_reads_r1".to_owned(), "/result/unmerged-R1.fastq.gz".to_owned()),
        ("unmerged_reads_r2".to_owned(), "/result/unmerged-R2.fastq.gz".to_owned()),
    ]);
    let seqprep = registry
        .binding(stage, &ToolId::new("seqprep"))
        .ok_or_else(|| anyhow::anyhow!("canonical SeqPrep merge binding is missing"))?;
    let seqprep_command = render_tool_command(
        seqprep,
        &ToolBindingRenderContext {
            executable: "seqprep".to_owned(),
            inputs: paired_merge_inputs(),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::new(),
            outputs: outputs.clone(),
        },
    )?;
    assert_eq!(seqprep_command[0], "seqprep");
    assert!(seqprep_command.windows(2).any(|pair| pair == ["-s", "/result/merged.fastq.gz"]));

    let vsearch = registry
        .binding(stage, &ToolId::new("vsearch"))
        .ok_or_else(|| anyhow::anyhow!("canonical VSEARCH merge binding is missing"))?;
    let vsearch_command = render_tool_command(
        vsearch,
        &ToolBindingRenderContext {
            executable: "vsearch".to_owned(),
            inputs: paired_merge_inputs(),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("merge_overlap".to_owned(), "11".to_owned()),
                ("min_len".to_owned(), "30".to_owned()),
            ]),
            outputs,
        },
    )?;
    assert!(vsearch_command.contains(&"--fastqout_notmerged_fwd".to_owned()));
    assert!(vsearch_command.contains(&"--fastq_minmergelen".to_owned()));
    Ok(())
}

fn assert_adapterremoval_merge_binding(
    registry: &ToolBindingRegistry,
    stage: &StageId,
) -> Result<()> {
    let adapterremoval = registry
        .binding(stage, &ToolId::new("adapterremoval"))
        .ok_or_else(|| anyhow::anyhow!("canonical AdapterRemoval merge binding is missing"))?;
    let adapterremoval_command = render_tool_command(
        adapterremoval,
        &ToolBindingRenderContext {
            executable: "adapterremoval".to_owned(),
            inputs: BTreeMap::from([
                ("reads_r1".to_owned(), "/input/R1.fastq.gz".to_owned()),
                ("reads_r2".to_owned(), "/input/R2.fastq.gz".to_owned()),
            ]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("merge_overlap".to_owned(), "11".to_owned()),
                ("min_len".to_owned(), "30".to_owned()),
            ]),
            outputs: BTreeMap::from([
                ("merged_reads".to_owned(), "/result/merged.fastq.gz".to_owned()),
                ("unmerged_reads_r1".to_owned(), "/result/unmerged-R1.fastq.gz".to_owned()),
                ("unmerged_reads_r2".to_owned(), "/result/unmerged-R2.fastq.gz".to_owned()),
                ("adapterremoval_settings".to_owned(), "/result/settings.txt".to_owned()),
            ]),
        },
    )?;
    assert!(adapterremoval_command
        .windows(2)
        .any(|pair| pair == ["--outputcollapsed", "/result/merged.fastq.gz"]));
    assert!(adapterremoval_command
        .windows(2)
        .any(|pair| pair == ["--settings", "/result/settings.txt"]));
    assert_eq!(
        adapterremoval.normalized_output_contract_id,
        "bijux.benchmark.fastq.merge_pairs.bundle.v1"
    );
    Ok(())
}

fn assert_bbmerge_merge_binding(registry: &ToolBindingRegistry, stage: &StageId) -> Result<()> {
    let bbmerge = registry
        .binding(stage, &ToolId::new("bbmerge"))
        .ok_or_else(|| anyhow::anyhow!("canonical BBMerge binding is missing"))?;
    let bbmerge_command = render_tool_command(
        bbmerge,
        &ToolBindingRenderContext {
            executable: "bbmerge".to_owned(),
            inputs: BTreeMap::from([
                ("reads_r1".to_owned(), "/input/R1.fastq.gz".to_owned()),
                ("reads_r2".to_owned(), "/input/R2.fastq.gz".to_owned()),
            ]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("merge_overlap".to_owned(), "11".to_owned()),
                ("min_len".to_owned(), "30".to_owned()),
            ]),
            outputs: BTreeMap::from([
                ("merged_reads".to_owned(), "/result/merged.fastq".to_owned()),
                ("unmerged_reads_r1".to_owned(), "/result/unmerged-R1.fastq".to_owned()),
                ("unmerged_reads_r2".to_owned(), "/result/unmerged-R2.fastq".to_owned()),
            ]),
        },
    )?;
    assert_eq!(
        bbmerge_command,
        [
            "bbmerge",
            "in1=/input/R1.fastq.gz",
            "in2=/input/R2.fastq.gz",
            "out=/result/merged.fastq",
            "outu1=/result/unmerged-R1.fastq",
            "outu2=/result/unmerged-R2.fastq",
            "threads=4",
            "minoverlap=11",
            "mininsert=30",
        ]
    );
    Ok(())
}

fn assert_flash2_merge_binding(registry: &ToolBindingRegistry, stage: &StageId) -> Result<()> {
    let flash2 = registry
        .binding(stage, &ToolId::new("flash2"))
        .ok_or_else(|| anyhow::anyhow!("canonical FLASH2 binding is missing"))?;
    let flash2_command = render_tool_command(
        flash2,
        &ToolBindingRenderContext {
            executable: "flash2".to_owned(),
            inputs: BTreeMap::from([
                ("reads_r1".to_owned(), "/input/R1.fastq.gz".to_owned()),
                ("reads_r2".to_owned(), "/input/R2.fastq.gz".to_owned()),
            ]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("threads".to_owned(), "4".to_owned()),
                ("merge_overlap".to_owned(), "11".to_owned()),
            ]),
            outputs: BTreeMap::from([(
                "flash2_result_directory".to_owned(),
                "/result/flash2".to_owned(),
            )]),
        },
    )?;
    assert_eq!(
        flash2_command,
        [
            "flash2",
            "-o",
            "flash2",
            "-d",
            "/result/flash2",
            "-t",
            "4",
            "-m",
            "11",
            "/input/R1.fastq.gz",
            "/input/R2.fastq.gz",
        ]
    );
    assert_eq!(flash2.outputs[0].artifact_contract_id, "bijux.auxiliary.artifact.directory.v1");
    Ok(())
}

#[test]
fn canonical_vsearch_cluster_binding_preserves_native_clustering_outputs() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(CLUSTER_OTUS_STAGE), &ToolId::new("vsearch"))
        .ok_or_else(|| anyhow::anyhow!("canonical VSEARCH cluster-OTUs binding is missing"))?;

    assert_eq!(binding.inputs[0].artifact_contract_id, "bijux.fastq.artifact.single_end.v1");
    assert_eq!(
        binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
        ["otu_representatives", "otu_clusters", "otu_table"]
    );
    assert_eq!(binding.outputs[0].artifact_contract_id, "bijux.reference.artifact.fasta.v1");
    assert!(binding
        .outputs
        .iter()
        .skip(1)
        .all(|output| output.artifact_contract_id == "bijux.auxiliary.artifact.table.v1"));
    assert_eq!(
        binding.normalized_output_contract_id,
        "bijux.benchmark.fastq.cluster_otus.bundle.v1"
    );
    assert_eq!(binding.downstream_probe_id, "benchmark.fastq.cluster_otus.consume");

    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "vsearch".to_owned(),
            inputs: BTreeMap::from([("reads".to_owned(), "/input/amplicons.fastq".to_owned())]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("identity_threshold".to_owned(), "0.97".to_owned()),
                ("threads".to_owned(), "4".to_owned()),
            ]),
            outputs: BTreeMap::from([
                ("otu_representatives".to_owned(), "/result/centroids.fasta".to_owned()),
                ("otu_clusters".to_owned(), "/result/clusters.uc".to_owned()),
                ("otu_table".to_owned(), "/result/otu-table.tsv".to_owned()),
            ]),
        },
    )?;
    assert_eq!(
        command,
        [
            "vsearch",
            "--cluster_fast",
            "/input/amplicons.fastq",
            "--id",
            "0.97",
            "--sizein",
            "--sizeout",
            "--relabel",
            "OTU_",
            "--threads",
            "4",
            "--centroids",
            "/result/centroids.fasta",
            "--uc",
            "/result/clusters.uc",
            "--otutabout",
            "/result/otu-table.tsv",
        ]
    );
    Ok(())
}

#[test]
fn canonical_dada2_binding_collects_the_complete_native_asv_bundle() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new(INFER_ASVS_STAGE), &ToolId::new("dada2"))
        .ok_or_else(|| anyhow::anyhow!("canonical DADA2 infer-ASVs binding is missing"))?;

    assert_eq!(
        binding.inputs.iter().map(|input| input.name.as_str()).collect::<Vec<_>>(),
        ["reads_r1", "reads_r2"]
    );
    assert!(binding
        .inputs
        .iter()
        .all(|input| input.artifact_contract_id == "bijux.fastq.artifact.paired_end.v1"));
    assert_eq!(binding.outputs.len(), 1);
    assert_eq!(binding.outputs[0].name, "asv_result_directory");
    assert_eq!(binding.outputs[0].artifact_contract_id, "bijux.auxiliary.artifact.directory.v1");
    assert_eq!(binding.outputs[0].discovery, OutputDiscoveryV1::ExactPath);
    assert_eq!(binding.normalized_output_contract_id, "bijux.benchmark.fastq.infer_asvs.bundle.v1");
    assert_eq!(binding.downstream_probe_id, "benchmark.fastq.infer_asvs.consume");

    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "dada2".to_owned(),
            inputs: BTreeMap::from([
                ("reads_r1".to_owned(), "/input/F3D0_R1.fastq".to_owned()),
                ("reads_r2".to_owned(), "/input/F3D0_R2.fastq".to_owned()),
            ]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([
                ("sample_id".to_owned(), "F3D0".to_owned()),
                ("threads".to_owned(), "4".to_owned()),
            ]),
            outputs: BTreeMap::from([(
                "asv_result_directory".to_owned(),
                "/result/asv-bundle".to_owned(),
            )]),
        },
    )?;
    assert_eq!(
        command,
        [
            "dada2",
            "infer-asvs",
            "--input",
            "/input/F3D0_R1.fastq",
            "--input-r2",
            "/input/F3D0_R2.fastq",
            "--output-dir",
            "/result/asv-bundle",
            "--sample-id",
            "F3D0",
            "--threads",
            "4",
        ]
    );
    Ok(())
}

#[test]
fn canonical_index_reference_bindings_collect_complete_index_directories() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(INDEX_REFERENCE_STAGE));
    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["bowtie2_build", "star"]
    );

    let expected_commands = BTreeMap::from([
        (
            "bowtie2_build",
            "set -eu\nmkdir -p '/result/index'\n'/images/bowtie2-build' --threads '4' '/input/reference.fa' '/result/index/reference'",
        ),
        (
            "star",
            "set -eu\nmkdir -p '/result/index'\n'/images/star' --runMode genomeGenerate --runThreadN '4' --genomeDir '/result/index' --genomeFastaFiles '/input/reference.fa'",
        ),
    ]);
    for binding in bindings {
        assert_eq!(binding.inputs[0].artifact_contract_id, "bijux.reference.artifact.fasta.v1");
        assert_eq!(
            binding.outputs[0].artifact_contract_id,
            "bijux.auxiliary.artifact.directory.v1"
        );
        assert_eq!(binding.outputs[0].discovery, OutputDiscoveryV1::ExactPath);
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.fastq.index_reference.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_fastq::tool_adapters::fastq::index_reference"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.fastq.index_reference.consume");
        assert!(binding
            .limitations
            .iter()
            .any(|limitation| limitation.contains("compatible_alignment_consumer")));

        let executable = if binding.tool_id.as_str() == "bowtie2_build" {
            "/images/bowtie2-build"
        } else {
            "/images/star"
        };
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: executable.to_owned(),
                inputs: BTreeMap::from([(
                    "reference_fasta".to_owned(),
                    "/input/reference.fa".to_owned(),
                )]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
                outputs: BTreeMap::from([(
                    "reference_index".to_owned(),
                    "/result/index".to_owned(),
                )]),
            },
        )?;
        assert_eq!(command, ["sh", "-lc", expected_commands[binding.tool_id.as_str()]]);
    }
    Ok(())
}

#[test]
fn canonical_low_complexity_bindings_project_declared_single_end_qualification() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(FILTER_LOW_COMPLEXITY_STAGE));
    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["bbduk", "fastp", "prinseq"]
    );

    let expected_commands = low_complexity_expected_commands();

    for binding in bindings {
        assert_eq!(binding.inputs[0].artifact_contract_id, "bijux.fastq.artifact.single_end.v1");
        assert_eq!(binding.outputs[0].artifact_contract_id, "bijux.fastq.artifact.single_end.v1");
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.fastq.filter_low_complexity.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_fastq::tool_adapters::fastq::filter_low_complexity"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.fastq.filter_low_complexity.compare");
        assert!(binding
            .limitations
            .iter()
            .any(|limitation| limitation.contains("comparative_case_admission_requires")));
        if binding.tool_id.as_str() == "fastp" {
            assert!(binding.limitations.iter().any(|limitation| limitation
                == "fastp_adjacent_base_change_complexity_is_not_shannon_entropy"));
        }

        let report_name = if binding.tool_id.as_str() == "fastp" {
            "native-report.json"
        } else {
            "native-report.txt"
        };
        let mut outputs = BTreeMap::from([(
            "filtered_reads_r1".to_owned(),
            "/result/filtered.fastq.gz".to_owned(),
        )]);
        if binding.tool_id.as_str() == "bbduk" {
            outputs.insert("native_filter_statistics".to_owned(), format!("/result/{report_name}"));
        } else if binding.tool_id.as_str() == "fastp" {
            outputs.insert("native_filter_report".to_owned(), format!("/result/{report_name}"));
        }
        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: binding.tool_id.to_string(),
                inputs: BTreeMap::from([(
                    "reads_r1".to_owned(),
                    "/input/sample.fastq.gz".to_owned(),
                )]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
                outputs,
            },
        )?;
        assert_eq!(command, expected_commands[binding.tool_id.as_str()]);
    }
    Ok(())
}

fn low_complexity_expected_commands() -> BTreeMap<&'static str, Vec<&'static str>> {
    BTreeMap::from([
        (
            "bbduk",
            vec![
                "bbduk",
                "in=/input/sample.fastq.gz",
                "out=/result/filtered.fastq.gz",
                "stats=/result/native-report.txt",
                "entropy=0.5",
                "threads=4",
                "overwrite=t",
            ],
        ),
        (
            "fastp",
            vec![
                "fastp",
                "--in1",
                "/input/sample.fastq.gz",
                "--out1",
                "/result/filtered.fastq.gz",
                "--thread",
                "4",
                "--disable_adapter_trimming",
                "--disable_quality_filtering",
                "--low_complexity_filter",
                "--complexity_threshold",
                "50",
                "--json",
                "/result/native-report.json",
                "--html",
                "/dev/null",
            ],
        ),
        (
            "prinseq",
            vec![
                "prinseq",
                "-threads",
                "4",
                "-fastq",
                "/input/sample.fastq.gz",
                "-out_good",
                "/result/filtered.fastq.gz",
                "-out_bad",
                "/dev/null",
                "-lc_entropy",
                "50",
            ],
        ),
    ])
}

#[test]
fn canonical_seqkit_filter_binding_projects_one_shot_qualification_io() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(FILTER_READS_STAGE));
    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["bbduk", "fastp", "prinseq", "seqkit"]
    );
    let binding = registry
        .binding(&StageId::new(FILTER_READS_STAGE), &ToolId::new("seqkit"))
        .ok_or_else(|| anyhow::anyhow!("canonical seqkit filter binding is missing"))?;

    assert_eq!(binding.tool_id.as_str(), "seqkit");
    assert_eq!(binding.inputs[0].name, "reads_r1");
    assert_eq!(binding.inputs[0].artifact_contract_id, "bijux.fastq.artifact.single_end.v1");
    assert_eq!(binding.outputs[0].name, "filtered_reads_r1");
    assert_eq!(binding.outputs[0].artifact_contract_id, "bijux.fastq.artifact.single_end.v1");
    assert_eq!(
        binding.normalized_output_contract_id,
        "bijux.benchmark.fastq.filter_reads.bundle.v1"
    );
    assert_eq!(
        binding.normalizer_id,
        "bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads"
    );
    assert_eq!(binding.downstream_probe_id, "benchmark.fastq.filter_reads.compare");

    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "seqkit".to_owned(),
            inputs: BTreeMap::from([("reads_r1".to_owned(), "/input/sample.fastq.gz".to_owned())]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
            outputs: BTreeMap::from([(
                "filtered_reads_r1".to_owned(),
                "/result/filtered.fastq.gz".to_owned(),
            )]),
        },
    )?;
    assert_eq!(
        command,
        [
            "seqkit",
            "seq",
            "-j",
            "4",
            "-m",
            "1",
            "-o",
            "/result/filtered.fastq.gz",
            "/input/sample.fastq.gz",
        ]
    );
    Ok(())
}

#[test]
fn canonical_fastqc_binding_preserves_directory_evidence_for_shared_normalization() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(DETECT_ADAPTERS_STAGE));
    assert_eq!(bindings.len(), 1);
    let binding = bindings[0];

    assert_eq!(binding.tool_id.as_str(), "fastqc");
    assert_eq!(binding.outputs.len(), 1);
    assert_eq!(binding.outputs[0].name, "adapter_evidence_dir");
    assert_eq!(binding.outputs[0].artifact_contract_id, "bijux.auxiliary.artifact.directory.v1");
    assert_eq!(
        binding.normalized_output_contract_id,
        "bijux.benchmark.fastq.detect_adapters.bundle.v1"
    );
    assert_eq!(
        binding.normalizer_id,
        "bijux_dna_planner_fastq::tool_adapters::fastq::detect_adapters"
    );
    assert_eq!(binding.downstream_probe_id, "benchmark.fastq.detect_adapters.compare");
    assert_eq!(binding.resources.threads, 4);

    let command = render_tool_command(
        binding,
        &ToolBindingRenderContext {
            executable: "fastqc".to_owned(),
            inputs: BTreeMap::from([("reads_r1".to_owned(), "/input/sample.fastq.gz".to_owned())]),
            input_sets: BTreeMap::new(),
            parameters: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
            outputs: BTreeMap::from([(
                "adapter_evidence_dir".to_owned(),
                "/result/fastqc".to_owned(),
            )]),
        },
    )?;
    assert_eq!(command[0..2], ["/bin/sh", "-c"]);
    assert_eq!(
        command[2],
        "mkdir -p '/result/fastqc' && 'fastqc' --outdir '/result/fastqc' --threads '4' '/input/sample.fastq.gz'"
    );
    assert!(!command[2].contains("fastq_detect_adapters.sh"));
    Ok(())
}

#[test]
fn canonical_fastq_profile_bindings_preserve_native_observations_for_shared_normalizers(
) -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let length_bindings = registry.bindings_for_stage(&StageId::new(PROFILE_READ_LENGTHS_STAGE));
    assert_eq!(
        length_bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["seqfu", "seqkit"]
    );
    for binding in length_bindings {
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.fastq.profile_read_lengths.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_fastq::tool_adapters::stages::pre::profile_read_lengths"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.fastq.profile_read_lengths.compare");
        assert_eq!(binding.outputs[0].name, "length_distribution_tsv");
        assert_eq!(binding.resources.threads, 2);

        let length_command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: binding.tool_id.to_string(),
                inputs: BTreeMap::from([(
                    "reads_r1".to_owned(),
                    "/input/sample.fastq.gz".to_owned(),
                )]),
                input_sets: BTreeMap::new(),
                parameters: binding
                    .parameters
                    .iter()
                    .map(|parameter| {
                        (parameter.name.clone(), binding.resources.threads.to_string())
                    })
                    .collect(),
                outputs: BTreeMap::from([(
                    "length_distribution_tsv".to_owned(),
                    "/result/read-lengths.tsv".to_owned(),
                )]),
            },
        )?;
        assert_eq!(length_command[0..2], ["/bin/sh", "-c"]);
        assert!(length_command[2].contains("'/input/sample.fastq.gz'"));
        assert!(length_command[2].ends_with("> '/result/read-lengths.tsv'"));
        if binding.tool_id.as_str() == "seqfu" {
            assert!(length_command[2].contains("'seqfu' tab"));
            assert!(length_command[2].contains("length($3)"));
        } else {
            assert!(length_command[2].contains("'seqkit' fx2tab"));
            assert!(length_command[2].contains("$NF"));
        }
    }

    let profile_bindings = registry.bindings_for_stage(&StageId::new(PROFILE_READS_STAGE));
    assert_eq!(
        profile_bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["seqfu", "seqkit", "seqkit_stats"]
    );
    for binding in profile_bindings {
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.fastq.profile_reads.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_fastq::tool_adapters::stages::qc::profile_reads"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.fastq.profile_reads.compare");
        assert_eq!(binding.outputs.len(), 1);
        assert_eq!(binding.outputs[0].name, "qc_tsv");
        assert_eq!(binding.resources.threads, 2);

        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: binding.tool_id.to_string(),
                inputs: BTreeMap::from([(
                    "reads_r1".to_owned(),
                    "/input/sample.fastq.gz".to_owned(),
                )]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([("threads".to_owned(), "2".to_owned())]),
                outputs: BTreeMap::from([("qc_tsv".to_owned(), "/result/profile.tsv".to_owned())]),
            },
        )?;
        assert_eq!(command[0..2], ["/bin/sh", "-c"]);
        assert!(command[2].contains("'/input/sample.fastq.gz'"));
        assert!(command[2].contains("'/result/profile.tsv'"));
        assert!(command[2].contains("'2'"));
    }
    Ok(())
}

#[test]
fn canonical_validate_reads_bindings_close_governed_single_stream_qualification() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let bindings = registry.bindings_for_stage(&StageId::new(VALIDATE_READS_STAGE));

    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"]
    );
    for binding in bindings.iter().copied().filter(|binding| {
        matches!(binding.tool_id.as_str(), "fastqvalidator" | "fqtools" | "seqtk")
    }) {
        assert_eq!(binding.resources.runtime, "apptainer");
        assert_eq!(binding.resources.threads, 4);
        assert_eq!(binding.inputs.len(), 1);
        assert_eq!(binding.inputs[0].name, "reads_r1");
        assert_eq!(
            binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
            ["validation_report", "validated_reads_manifest"]
        );
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.fastq.validate_reads.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_fastq::tool_adapters::stages::pre::validate_reads"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.fastq.validate_reads.compare");

        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: binding.tool_id.to_string(),
                inputs: BTreeMap::from([(
                    "reads_r1".to_owned(),
                    "/input/sample.fastq.gz".to_owned(),
                )]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::new(),
                outputs: BTreeMap::from([
                    ("validation_report".to_owned(), "/result/validation.json".to_owned()),
                    (
                        "validated_reads_manifest".to_owned(),
                        "/result/validated-reads.json".to_owned(),
                    ),
                ]),
            },
        )?;
        assert_eq!(command[0..2], ["/bin/sh", "-c"]);
        assert!(command[2].contains("/input/sample.fastq.gz"));
        assert!(command[2].contains("/result/validation.json"));
        assert!(command[2].contains("/result/validated-reads.json"));
        assert!(command[2].contains("bijux.fastq.validate.report.v1"));
        assert!(command[2].contains("bijux.fastq.validate.lineage.v1"));
        assert!(command[2].contains(&format!("\"tool_id\":\"{}\"", binding.tool_id)));
    }
    Ok(())
}

#[test]
fn canonical_taxonomy_bindings_close_all_governed_read_classifiers() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let stage_id = StageId::new(SCREEN_TAXONOMY_STAGE);
    let bindings = registry.bindings_for_stage(&stage_id);

    assert_eq!(
        bindings.iter().map(|binding| binding.tool_id.as_str()).collect::<Vec<_>>(),
        ["centrifuge", "kaiju", "kraken2", "krakenuniq"]
    );
    for binding in bindings {
        assert_eq!(binding.resources.runtime, "apptainer");
        assert_eq!(
            binding.normalized_output_contract_id,
            "bijux.benchmark.fastq.screen_taxonomy.bundle.v1"
        );
        assert_eq!(
            binding.normalizer_id,
            "bijux_dna_planner_fastq::tool_adapters::stages::qc::screen_taxonomy"
        );
        assert_eq!(binding.downstream_probe_id, "benchmark.fastq.screen_taxonomy.compare");
        assert_eq!(
            binding.inputs.iter().map(|input| input.name.as_str()).collect::<Vec<_>>(),
            ["reads_r1", "taxonomy_database"]
        );
        assert_eq!(
            binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>(),
            ["screen_report_tsv", "native_assignments"]
        );

        let command = render_tool_command(
            binding,
            &ToolBindingRenderContext {
                executable: binding.tool_id.to_string(),
                inputs: BTreeMap::from([
                    ("reads_r1".to_owned(), "/input/sample_R1.fastq.gz".to_owned()),
                    ("taxonomy_database".to_owned(), "/input/taxonomy".to_owned()),
                ]),
                input_sets: BTreeMap::new(),
                parameters: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
                outputs: BTreeMap::from([
                    ("screen_report_tsv".to_owned(), "/result/screen.tsv".to_owned()),
                    ("native_assignments".to_owned(), "/result/assignments.tsv".to_owned()),
                ]),
            },
        )?;
        let rendered = command.join(" ");
        assert!(rendered.contains("/input/sample_R1.fastq.gz"));
        assert!(rendered.contains("/input/taxonomy"));
        assert!(rendered.contains("/result/screen.tsv"));
        assert!(rendered.contains("/result/assignments.tsv"));
    }
    Ok(())
}

#[test]
fn canonical_external_bindings_preserve_external_runtime_identity() -> Result<()> {
    let registry = canonical_external_artifact_tool_bindings()?;
    assert_eq!(registry.bindings().len(), 8);
    assert!(registry.bindings().all(|binding| binding.resources.runtime == "external_artifact"));
    assert!(registry.bindings().all(|binding| {
        binding.normalized_output_contract_id
            == format!("bijux.benchmark.{}.bundle.v1", binding.stage_id)
    }));
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
    assert!(command[2].contains("-tmpd \"$work\" -od \"$work\""));
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
    assert_eq!(binding.downstream_probe_id, "benchmark.bam.sex.compare");
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
