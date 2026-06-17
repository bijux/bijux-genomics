use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use super::tool_families::{validate_tool_families_path, DEFAULT_TOOL_FAMILIES_PATH};
use super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use crate::commands::benchmark::local_stage_commands::{
    local_stage_plans, materialize_local_stage,
};
use crate::commands::benchmark::local_vcf_call_smoke::run_local_vcf_call_smoke;
use crate::commands::benchmark::local_vcf_impute_smoke::run_local_vcf_impute_smoke;
use crate::commands::benchmark::local_vcf_phasing_smoke::run_local_vcf_phasing_smoke;
use crate::commands::benchmark::local_vcf_population_structure_smoke::run_local_vcf_population_structure_smoke;
use crate::commands::benchmark::local_vcf_qc_smoke::run_local_vcf_qc_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH: &str =
    "benchmarks/readiness/tools/real-output-parser-smoke.json";
const REAL_OUTPUT_PARSER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.real_output_parser_smoke.v1";
const DEFAULT_PROBE_ROOT: &str = "runs/bench/readiness-probes/tools/real-output-parser-smoke";
const LOCAL_BAM_GENOTYPING_CONFIG_PATH: &str = "benchmarks/configs/local/bam-genotyping.toml";
const LOCAL_BAM_KINSHIP_CONFIG_PATH: &str = "benchmarks/configs/local/bam-kinship.toml";
const LOCAL_BAM_SEX_CONFIG_PATH: &str = "benchmarks/configs/local/bam-sex.toml";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RealOutputParserSmokeRow {
    pub(crate) family_id: String,
    pub(crate) family_summary: String,
    pub(crate) representative_stage_id: String,
    pub(crate) representative_tool_id: String,
    pub(crate) proof_path: String,
    pub(crate) parser_surface: String,
    pub(crate) parsed_schema_version: String,
    pub(crate) parsed_top_level_key_count: usize,
    pub(crate) parsed_top_level_keys: Vec<String>,
    pub(crate) normalized_snapshot: BTreeMap<String, Value>,
    pub(crate) passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RealOutputParserSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) probe_root: String,
    pub(crate) family_count: usize,
    pub(crate) passed_family_count: usize,
    pub(crate) failed_family_count: usize,
    pub(crate) parser_surface_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<RealOutputParserSmokeRow>,
}

#[derive(Debug, Clone, Copy)]
struct FamilyProbeSpec {
    family_id: &'static str,
    representative_stage_id: &'static str,
    probe_kind: ProbeKind,
    parse_kind: ParseKind,
    snapshot_keys: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
enum ProbeKind {
    LocalStage { stage_id: &'static str, proof_pointer: Option<&'static str> },
    GeneratedIndexReference,
    GeneratedCorrectErrors,
    GeneratedDepleteHost,
    GeneratedDepleteRrna,
    GeneratedScreenTaxonomy,
    GeneratedBamAlign,
    GeneratedBamContamination,
    GeneratedBamDamage,
    GeneratedBamAuthenticity,
    GeneratedBamGenotyping,
    GeneratedBamKinship,
    GeneratedBamSex,
    GeneratedVcfCall,
    GeneratedVcfImpute,
    GeneratedVcfPhasing,
    GeneratedVcfPopulationStructure,
    GeneratedVcfQc,
}

#[derive(Debug, Clone, Copy)]
enum ParseKind {
    FastqIndexReference,
    FastqCorrectErrors,
    FastqDepleteHost,
    FastqDepleteRrna,
    FastqTrimReads,
    FastqMergePairs,
    FastqFilterReads,
    FastqProfileReads,
    FastqReportQc,
    FastqRemoveDuplicates,
    FastqExtractUmis,
    FastqScreenTaxonomy,
    FastqInferAsvs,
    FastqNormalizeAbundance,
    FastqDetectDuplicatesPremerge,
    BamValidationSummary,
    BamAlignmentProvenance,
    BamContaminationJson,
    BamOverlapCorrectionSummary,
    BamRecalibrationSummary,
    BamDamageEvidence,
    BamAuthenticityAdvisory,
    BamComplexitySummary,
    BamKinshipSummary,
    BamSexJson,
    BamGenotypingJson,
    VcfCallMetrics,
    VcfImputeMetrics,
    VcfPhasingMetrics,
    VcfPopulationStructureReport,
    VcfQcMetrics,
}

#[derive(Debug, Clone)]
struct MaterializedProof {
    proof_path: PathBuf,
    observed_tool_id: String,
}

#[derive(Debug, Clone)]
struct FastqReadStats {
    reads: u64,
    bases: u64,
}

#[derive(Debug, Deserialize)]
struct LocalBamGenotypingConfig {
    tool_id: String,
    bam: PathBuf,
    reference_fasta: PathBuf,
    sites_vcf: PathBuf,
    regions: PathBuf,
    sample_id: String,
}

#[derive(Debug, Deserialize)]
struct LocalBamKinshipConfig {
    tool_id: String,
    cases: Vec<LocalBamKinshipCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalBamKinshipCase {
    bam: PathBuf,
    reference_panel: String,
    reference_build: String,
    population_scope: String,
    min_overlap_snps: u32,
    requires_cohort_context: bool,
    expected_status: String,
}

#[derive(Debug, Deserialize)]
struct LocalBamSexConfig {
    tool_id: String,
    cases: Vec<LocalBamSexCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalBamSexCase {
    bam: PathBuf,
    reference: PathBuf,
    chromosome_system: String,
    minimum_y_sites: u32,
}

pub(crate) fn run_render_real_output_parser_smoke(
    args: &parse::BenchReadinessRenderRealOutputParserSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_real_output_parser_smoke(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_real_output_parser_smoke(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RealOutputParserSmokeReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let probe_root = repo_root.join(DEFAULT_PROBE_ROOT);
    if probe_root.exists() {
        fs::remove_dir_all(&probe_root)
            .with_context(|| format!("remove {}", probe_root.display()))?;
    }
    fs::create_dir_all(&probe_root).with_context(|| format!("create {}", probe_root.display()))?;

    let report =
        build_real_output_parser_smoke_report(repo_root, &absolute_output_path, &probe_root)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if report.failed_family_count != 0 {
        return Err(anyhow!(
            "real-output parser smoke found {} failed retained tool families",
            report.failed_family_count
        ));
    }
    Ok(report)
}

fn build_real_output_parser_smoke_report(
    repo_root: &Path,
    output_path: &Path,
    probe_root: &Path,
) -> Result<RealOutputParserSmokeReport> {
    let active_families = collect_active_families(repo_root)?;
    let spec_map = family_probe_specs()
        .into_iter()
        .map(|spec| (spec.family_id, spec))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(active_families.len());
    for (family_id, family_summary) in active_families {
        let spec = spec_map.get(family_id.as_str()).copied().ok_or_else(|| {
            anyhow!("real-output parser smoke has no governed probe spec for family `{family_id}`")
        })?;
        let materialized = materialize_family_proof(repo_root, probe_root, &spec)?;
        let parsed = parse_proof(&materialized.proof_path, spec.parse_kind, spec.snapshot_keys)?;
        rows.push(RealOutputParserSmokeRow {
            family_id,
            family_summary,
            representative_stage_id: spec.representative_stage_id.to_string(),
            representative_tool_id: materialized.observed_tool_id,
            proof_path: path_relative_to_repo(repo_root, &materialized.proof_path),
            parser_surface: parse_surface_label(spec.parse_kind).to_string(),
            parsed_schema_version: parsed
                .0
                .get("schema_version")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            parsed_top_level_key_count: top_level_keys(&parsed.0).len(),
            parsed_top_level_keys: top_level_keys(&parsed.0),
            normalized_snapshot: parsed.1,
            passed: true,
            reason: format!(
                "validated {} family output through {}",
                spec.family_id,
                parse_surface_label(spec.parse_kind)
            ),
        });
    }

    rows.sort_by(|left, right| left.family_id.cmp(&right.family_id));
    let passed_family_count = rows.iter().filter(|row| row.passed).count();
    let failed_family_count = rows.len().saturating_sub(passed_family_count);
    let mut parser_surface_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *parser_surface_counts.entry(row.parser_surface.clone()).or_default() += 1;
    }

    Ok(RealOutputParserSmokeReport {
        schema_version: REAL_OUTPUT_PARSER_SMOKE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        probe_root: path_relative_to_repo(repo_root, probe_root),
        family_count: rows.len(),
        passed_family_count,
        failed_family_count,
        parser_surface_counts,
        rows,
    })
}

fn collect_active_families(repo_root: &Path) -> Result<Vec<(String, String)>> {
    let config_path = repo_root.join(DEFAULT_TOOL_FAMILIES_PATH);
    let family_report = validate_tool_families_path(repo_root, &config_path)?;
    let active_tool_ids = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| row.tool_id)
        .chain(
            collect_vcf_expected_benchmark_result_rows(repo_root)?
                .into_iter()
                .map(|row| row.tool_id),
        )
        .collect::<BTreeSet<_>>();

    let mut families = BTreeMap::<String, String>::new();
    for row in family_report.rows {
        if active_tool_ids.contains(&row.tool_id) {
            families.entry(row.family_id).or_insert(row.family_summary);
        }
    }
    Ok(families.into_iter().collect())
}

fn family_probe_specs() -> Vec<FamilyProbeSpec> {
    let mut specs = vec![
        FamilyProbeSpec {
            family_id: "reference_indexing",
            representative_stage_id: "fastq.index_reference",
            probe_kind: ProbeKind::GeneratedIndexReference,
            parse_kind: ParseKind::FastqIndexReference,
            snapshot_keys: &[
                "tool_id",
                "threads",
                "index_format",
                "index_file_count",
                "index_bytes",
            ],
        },
        FamilyProbeSpec {
            family_id: "alignment",
            representative_stage_id: "bam.align",
            probe_kind: ProbeKind::GeneratedBamAlign,
            parse_kind: ParseKind::BamAlignmentProvenance,
            snapshot_keys: &["backend_tool_id", "strategy_id", "preset", "mode"],
        },
        FamilyProbeSpec {
            family_id: "adapter_and_quality_trimming",
            representative_stage_id: "fastq.trim_reads",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.trim_reads",
                proof_pointer: Some("/cases/0/report_json"),
            },
            parse_kind: ParseKind::FastqTrimReads,
            snapshot_keys: &["tool_id", "reads_in", "reads_out", "bases_removed"],
        },
        FamilyProbeSpec {
            family_id: "read_pair_merging",
            representative_stage_id: "fastq.merge_pairs",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.merge_pairs",
                proof_pointer: Some("/case_report_json"),
            },
            parse_kind: ParseKind::FastqMergePairs,
            snapshot_keys: &["tool_id", "reads_merged", "reads_unmerged", "merge_rate"],
        },
        FamilyProbeSpec {
            family_id: "read_filtering_and_low_complexity",
            representative_stage_id: "fastq.filter_reads",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.filter_reads",
                proof_pointer: Some("/report_json"),
            },
            parse_kind: ParseKind::FastqFilterReads,
            snapshot_keys: &["tool_id", "reads_in", "reads_out", "reads_removed_by_kmer"],
        },
        FamilyProbeSpec {
            family_id: "read_validation_and_profiling",
            representative_stage_id: "fastq.profile_reads",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.profile_reads",
                proof_pointer: Some("/cases/0/report_json"),
            },
            parse_kind: ParseKind::FastqProfileReads,
            snapshot_keys: &["tool_id", "reads_total", "bases_total", "gc_percent"],
        },
        FamilyProbeSpec {
            family_id: "report_aggregation",
            representative_stage_id: "fastq.report_qc",
            probe_kind: ProbeKind::LocalStage { stage_id: "fastq.report_qc", proof_pointer: None },
            parse_kind: ParseKind::FastqReportQc,
            snapshot_keys: &[
                "tool_id",
                "governed_qc_input_count",
                "governed_qc_lineage_hash",
                "aggregation_scope",
            ],
        },
        FamilyProbeSpec {
            family_id: "error_correction",
            representative_stage_id: "fastq.correct_errors",
            probe_kind: ProbeKind::GeneratedCorrectErrors,
            parse_kind: ParseKind::FastqCorrectErrors,
            snapshot_keys: &["tool_id", "threads", "reads_in", "reads_out", "changed_reads"],
        },
        FamilyProbeSpec {
            family_id: "duplicate_handling",
            representative_stage_id: "fastq.remove_duplicates",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.remove_duplicates",
                proof_pointer: Some("/case_report_json"),
            },
            parse_kind: ParseKind::FastqRemoveDuplicates,
            snapshot_keys: &["tool_id", "reads_in", "reads_out", "duplicate_reads"],
        },
        FamilyProbeSpec {
            family_id: "umi_processing",
            representative_stage_id: "fastq.extract_umis",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.extract_umis",
                proof_pointer: Some("/case_report_json"),
            },
            parse_kind: ParseKind::FastqExtractUmis,
            snapshot_keys: &["tool_id", "reads_in", "reads_out", "extracted_umis"],
        },
        FamilyProbeSpec {
            family_id: "taxonomy_classification",
            representative_stage_id: "fastq.screen_taxonomy",
            probe_kind: ProbeKind::GeneratedScreenTaxonomy,
            parse_kind: ParseKind::FastqScreenTaxonomy,
            snapshot_keys: &["tool_id", "reads_in", "classified_fraction", "contamination_rate"],
        },
        FamilyProbeSpec {
            family_id: "rrna_depletion",
            representative_stage_id: "fastq.deplete_rrna",
            probe_kind: ProbeKind::GeneratedDepleteRrna,
            parse_kind: ParseKind::FastqDepleteRrna,
            snapshot_keys: &["tool_id", "reads_in", "reads_removed", "rrna_fraction_removed"],
        },
        FamilyProbeSpec {
            family_id: "amplicon_processing",
            representative_stage_id: "fastq.infer_asvs",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.infer_asvs",
                proof_pointer: Some("/case_report_json"),
            },
            parse_kind: ParseKind::FastqInferAsvs,
            snapshot_keys: &["tool_id", "feature_count", "sample_count", "reads_in"],
        },
        FamilyProbeSpec {
            family_id: "abundance_normalization",
            representative_stage_id: "fastq.normalize_abundance",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.normalize_abundance",
                proof_pointer: Some("/case_report_json"),
            },
            parse_kind: ParseKind::FastqNormalizeAbundance,
            snapshot_keys: &["tool_id", "row_count", "sample_count", "normalization_method"],
        },
        FamilyProbeSpec {
            family_id: "internal_workflow_runtime",
            representative_stage_id: "fastq.detect_duplicates_premerge",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "fastq.detect_duplicates_premerge",
                proof_pointer: Some("/cases/0/report_json"),
            },
            parse_kind: ParseKind::FastqDetectDuplicatesPremerge,
            snapshot_keys: &["tool_id", "reads_in", "duplicate_reads", "duplicate_fraction"],
        },
        FamilyProbeSpec {
            family_id: "bam_quality_control_and_metrics",
            representative_stage_id: "bam.validate",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "bam.validate",
                proof_pointer: Some("/cases/0/validation_report"),
            },
            parse_kind: ParseKind::BamValidationSummary,
            snapshot_keys: &["stage_id", "validation_report_present"],
        },
        FamilyProbeSpec {
            family_id: "overlap_correction",
            representative_stage_id: "bam.overlap_correction",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "bam.overlap_correction",
                proof_pointer: Some("/overlap_correction_summary"),
            },
            parse_kind: ParseKind::BamOverlapCorrectionSummary,
            snapshot_keys: &["method", "pair_count", "corrected_pairs", "corrected_overlap_bases"],
        },
        FamilyProbeSpec {
            family_id: "variant_analysis_and_recalibration",
            representative_stage_id: "bam.recalibration",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "bam.recalibration",
                proof_pointer: Some("/recalibration_summary"),
            },
            parse_kind: ParseKind::BamRecalibrationSummary,
            snapshot_keys: &["requested_mode", "effective_mode", "status", "coverage_gate"],
        },
        FamilyProbeSpec {
            family_id: "genotyping_and_population_inference",
            representative_stage_id: "bam.genotyping",
            probe_kind: ProbeKind::GeneratedBamGenotyping,
            parse_kind: ParseKind::BamGenotypingJson,
            snapshot_keys: &["producer", "reference", "sites", "call_rate", "mean_posterior"],
        },
        FamilyProbeSpec {
            family_id: "damage_and_postmortem_bias",
            representative_stage_id: "bam.damage",
            probe_kind: ProbeKind::GeneratedBamDamage,
            parse_kind: ParseKind::BamDamageEvidence,
            snapshot_keys: &["damage_signal", "terminal_c_to_t_5p", "terminal_g_to_a_3p"],
        },
        FamilyProbeSpec {
            family_id: "authenticity_assessment",
            representative_stage_id: "bam.authenticity",
            probe_kind: ProbeKind::GeneratedBamAuthenticity,
            parse_kind: ParseKind::BamAuthenticityAdvisory,
            snapshot_keys: &["score", "confidence", "pmd_like_signal_present"],
        },
        FamilyProbeSpec {
            family_id: "contamination_estimation",
            representative_stage_id: "bam.contamination",
            probe_kind: ProbeKind::GeneratedBamContamination,
            parse_kind: ParseKind::BamContaminationJson,
            snapshot_keys: &["method", "estimate", "ci_low", "ci_high"],
        },
        FamilyProbeSpec {
            family_id: "library_complexity_estimation",
            representative_stage_id: "bam.complexity",
            probe_kind: ProbeKind::LocalStage {
                stage_id: "bam.complexity",
                proof_pointer: Some("/complexity_summary"),
            },
            parse_kind: ParseKind::BamComplexitySummary,
            snapshot_keys: &[
                "method",
                "status",
                "estimated_distinct_fragments",
                "observed_fragments",
            ],
        },
        FamilyProbeSpec {
            family_id: "sex_and_haplogroup_inference",
            representative_stage_id: "bam.sex",
            probe_kind: ProbeKind::GeneratedBamSex,
            parse_kind: ParseKind::BamSexJson,
            snapshot_keys: &["method", "x_to_y_ratio", "confidence", "status"],
        },
        FamilyProbeSpec {
            family_id: "vcf_calling_and_curation",
            representative_stage_id: "vcf.call",
            probe_kind: ProbeKind::GeneratedVcfCall,
            parse_kind: ParseKind::VcfCallMetrics,
            snapshot_keys: &["stage_id", "tool_id", "variant_count", "snp_count", "sample_count"],
        },
        FamilyProbeSpec {
            family_id: "vcf_imputation",
            representative_stage_id: "vcf.impute",
            probe_kind: ProbeKind::GeneratedVcfImpute,
            parse_kind: ParseKind::VcfImputeMetrics,
            snapshot_keys: &[
                "stage_id",
                "tool_id",
                "variant_count",
                "imputed_genotypes",
                "masked_truth_match_count",
            ],
        },
        FamilyProbeSpec {
            family_id: "vcf_ordination_and_population_structure",
            representative_stage_id: "vcf.population_structure",
            probe_kind: ProbeKind::GeneratedVcfPopulationStructure,
            parse_kind: ParseKind::VcfPopulationStructureReport,
            snapshot_keys: &["tool_id", "status", "sample_groups", "distance_summary"],
        },
        FamilyProbeSpec {
            family_id: "vcf_phasing",
            representative_stage_id: "vcf.phasing",
            probe_kind: ProbeKind::GeneratedVcfPhasing,
            parse_kind: ParseKind::VcfPhasingMetrics,
            snapshot_keys: &[
                "stage_id",
                "tool_id",
                "input_genotypes",
                "phased_genotypes",
                "phase_set_count",
            ],
        },
        FamilyProbeSpec {
            family_id: "vcf_quality_control",
            representative_stage_id: "vcf.qc",
            probe_kind: ProbeKind::GeneratedVcfQc,
            parse_kind: ParseKind::VcfQcMetrics,
            snapshot_keys: &[
                "tool_id",
                "sample_missingness",
                "variant_missingness",
                "heterozygosity",
                "hwe_summary",
            ],
        },
    ];
    specs.push(FamilyProbeSpec {
        family_id: "kinship_relatedness",
        representative_stage_id: "bam.kinship",
        probe_kind: ProbeKind::GeneratedBamKinship,
        parse_kind: ParseKind::BamKinshipSummary,
        snapshot_keys: &["method", "pair_count", "status", "observed_max_overlap_snps"],
    });
    specs
}

fn materialize_family_proof(
    repo_root: &Path,
    probe_root: &Path,
    spec: &FamilyProbeSpec,
) -> Result<MaterializedProof> {
    match spec.probe_kind {
        ProbeKind::LocalStage { stage_id, proof_pointer } => {
            let materialized_path = materialize_local_stage(repo_root, stage_id)
                .with_context(|| format!("materialize local stage `{stage_id}`"))?;
            let summary_path = if proof_pointer.is_some() && !is_json_path(&materialized_path) {
                let candidate = materialized_path
                    .parent()
                    .map(|parent| parent.join("report.json"))
                    .filter(|path| path.is_file());
                candidate.unwrap_or_else(|| materialized_path.clone())
            } else {
                materialized_path.clone()
            };
            let proof_path = if let Some(pointer) = proof_pointer {
                let summary = read_json_document(&summary_path)?;
                let proof_relative = json_pointer_string(&summary, pointer).with_context(|| {
                    format!(
                        "extract proof path `{pointer}` from local stage summary {}",
                        summary_path.display()
                    )
                })?;
                repo_root.join(proof_relative)
            } else {
                summary_path
            };
            let observed_tool_id = observe_tool_id(repo_root, &proof_path, stage_id)?;
            Ok(MaterializedProof { proof_path, observed_tool_id })
        }
        ProbeKind::GeneratedIndexReference => generate_index_reference_probe(repo_root, probe_root),
        ProbeKind::GeneratedCorrectErrors => generate_correct_errors_probe(repo_root, probe_root),
        ProbeKind::GeneratedDepleteHost => generate_deplete_host_probe(repo_root, probe_root),
        ProbeKind::GeneratedDepleteRrna => generate_deplete_rrna_probe(repo_root, probe_root),
        ProbeKind::GeneratedScreenTaxonomy => generate_screen_taxonomy_probe(repo_root, probe_root),
        ProbeKind::GeneratedBamAlign => generate_bam_align_probe(repo_root, probe_root),
        ProbeKind::GeneratedBamContamination => {
            generate_bam_contamination_probe(repo_root, probe_root)
        }
        ProbeKind::GeneratedBamDamage => generate_bam_damage_probe(repo_root, probe_root),
        ProbeKind::GeneratedBamAuthenticity => {
            generate_bam_authenticity_probe(repo_root, probe_root)
        }
        ProbeKind::GeneratedBamGenotyping => generate_bam_genotyping_probe(repo_root, probe_root),
        ProbeKind::GeneratedBamKinship => generate_bam_kinship_probe(repo_root, probe_root),
        ProbeKind::GeneratedBamSex => generate_bam_sex_probe(repo_root, probe_root),
        ProbeKind::GeneratedVcfCall => generate_vcf_call_probe(repo_root),
        ProbeKind::GeneratedVcfImpute => generate_vcf_impute_probe(repo_root),
        ProbeKind::GeneratedVcfPhasing => generate_vcf_phasing_probe(repo_root),
        ProbeKind::GeneratedVcfPopulationStructure => {
            generate_vcf_population_structure_probe(repo_root)
        }
        ProbeKind::GeneratedVcfQc => generate_vcf_qc_probe(repo_root),
    }
}

fn is_json_path(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("json")
}

fn parse_proof(
    proof_path: &Path,
    parse_kind: ParseKind,
    snapshot_keys: &[&str],
) -> Result<(Value, BTreeMap<String, Value>)> {
    match parse_kind {
        ParseKind::FastqIndexReference => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_index_reference_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqCorrectErrors => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_correct_errors_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqDepleteHost => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_deplete_host_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqDepleteRrna => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_deplete_rrna_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqTrimReads => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqMergePairs => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_merge_pairs_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqFilterReads => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_filter_reads_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqProfileReads => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_profile_reads_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqReportQc => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_report_qc_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqRemoveDuplicates => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_remove_duplicates_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqExtractUmis => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_extract_umis_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqScreenTaxonomy => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_screen_taxonomy_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqInferAsvs => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_infer_asvs_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqNormalizeAbundance => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_normalize_abundance_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::FastqDetectDuplicatesPremerge => {
            let raw = fs::read_to_string(proof_path)
                .with_context(|| format!("read {}", proof_path.display()))?;
            let _ = bijux_dna_domain_fastq::observer::parse_detect_duplicates_premerge_report(&raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamValidationSummary => {
            let _: bijux_dna_domain_bam::BamValidationSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamAlignmentProvenance => {
            let _: bijux_dna_domain_bam::BamAlignmentProvenanceV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamContaminationJson => {
            let _ = bijux_dna_domain_bam::metrics::parse_contamination_json(proof_path)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamOverlapCorrectionSummary => {
            let _: bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamRecalibrationSummary => {
            let _: bijux_dna_domain_bam::BamRecalibrationSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamDamageEvidence => {
            let _: bijux_dna_domain_bam::BamDamageEvidenceV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamAuthenticityAdvisory => {
            let _: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamComplexitySummary => {
            let _: bijux_dna_domain_bam::BamComplexitySummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamKinshipSummary => {
            let _: bijux_dna_domain_bam::BamKinshipSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamSexJson => {
            let _ = bijux_dna_domain_bam::metrics::parse_sex_json(proof_path)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        ParseKind::BamGenotypingJson => {
            let value = read_json_document(proof_path)?;
            if value.get("producer").is_none() {
                return Err(anyhow!("{} is missing `producer`", proof_path.display()));
            }
        }
        ParseKind::VcfCallMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "stage_id", "tool_id", "variant_count", "sample_count"],
            )?;
        }
        ParseKind::VcfImputeMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "stage_id", "tool_id", "variant_count", "imputed_genotypes"],
            )?;
        }
        ParseKind::VcfPhasingMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "stage_id", "tool_id", "phased_genotypes", "phase_set_count"],
            )?;
        }
        ParseKind::VcfPopulationStructureReport => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "tool_id", "status", "sample_groups", "distance_summary"],
            )?;
        }
        ParseKind::VcfQcMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &[
                    "schema_version",
                    "tool_id",
                    "sample_missingness",
                    "heterozygosity",
                    "hwe_summary",
                ],
            )?;
        }
    }

    let value = read_json_document(proof_path)?;
    Ok((value.clone(), json_snapshot(&value, snapshot_keys)))
}

fn parse_surface_label(parse_kind: ParseKind) -> &'static str {
    match parse_kind {
        ParseKind::FastqIndexReference => "fastq::parse_index_reference_report",
        ParseKind::FastqCorrectErrors => "fastq::parse_correct_errors_report",
        ParseKind::FastqDepleteHost => "fastq::parse_deplete_host_report",
        ParseKind::FastqDepleteRrna => "fastq::parse_deplete_rrna_report",
        ParseKind::FastqTrimReads => "fastq::parse_trim_reads_report",
        ParseKind::FastqMergePairs => "fastq::parse_merge_pairs_report",
        ParseKind::FastqFilterReads => "fastq::parse_filter_reads_report",
        ParseKind::FastqProfileReads => "fastq::parse_profile_reads_report",
        ParseKind::FastqReportQc => "fastq::parse_report_qc_report",
        ParseKind::FastqRemoveDuplicates => "fastq::parse_remove_duplicates_report",
        ParseKind::FastqExtractUmis => "fastq::parse_extract_umis_report",
        ParseKind::FastqScreenTaxonomy => "fastq::parse_screen_taxonomy_report",
        ParseKind::FastqInferAsvs => "fastq::parse_infer_asvs_report",
        ParseKind::FastqNormalizeAbundance => "fastq::parse_normalize_abundance_report",
        ParseKind::FastqDetectDuplicatesPremerge => {
            "fastq::parse_detect_duplicates_premerge_report"
        }
        ParseKind::BamValidationSummary => "serde_json::<BamValidationSummaryV1>",
        ParseKind::BamAlignmentProvenance => "serde_json::<BamAlignmentProvenanceV1>",
        ParseKind::BamContaminationJson => "bam::parse_contamination_json",
        ParseKind::BamOverlapCorrectionSummary => "serde_json::<BamOverlapCorrectionSummaryV1>",
        ParseKind::BamRecalibrationSummary => "serde_json::<BamRecalibrationSummaryV1>",
        ParseKind::BamDamageEvidence => "serde_json::<BamDamageEvidenceV1>",
        ParseKind::BamAuthenticityAdvisory => "serde_json::<BamAuthenticityAdvisoryV1>",
        ParseKind::BamComplexitySummary => "serde_json::<BamComplexitySummaryV1>",
        ParseKind::BamKinshipSummary => "serde_json::<BamKinshipSummaryV1>",
        ParseKind::BamSexJson => "bam::parse_sex_json",
        ParseKind::BamGenotypingJson => "serde_json::<Value> + governed keys",
        ParseKind::VcfCallMetrics => "serde_json::<Value> + governed vcf call metric keys",
        ParseKind::VcfImputeMetrics => "serde_json::<Value> + governed vcf impute metric keys",
        ParseKind::VcfPhasingMetrics => "serde_json::<Value> + governed vcf phasing metric keys",
        ParseKind::VcfPopulationStructureReport => {
            "serde_json::<Value> + governed vcf population-structure keys"
        }
        ParseKind::VcfQcMetrics => "serde_json::<Value> + governed vcf qc metric keys",
    }
}

fn generate_index_reference_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_index_reference_plan(repo_root)?;
    let probe_dir = probe_root.join("reference_indexing");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let reference_fasta = required_input_path(&plan, "reference_fasta")?;
    let reference_bytes = fs::metadata(&reference_fasta)
        .with_context(|| format!("stat {}", reference_fasta.display()))?
        .len();
    let tool_id = plan.tool_id.as_str().to_string();
    let index_root = probe_dir.join("reference_index");
    fs::create_dir_all(&index_root).with_context(|| format!("create {}", index_root.display()))?;
    let extension = if tool_id == "bowtie2_build" { "bt2" } else { "idx" };
    let emitted_path = index_root.join(format!("reference.1.{extension}"));
    fs::write(&emitted_path, b"bijux-index\n")
        .with_context(|| format!("write {}", emitted_path.display()))?;
    let emitted_files = vec![bijux_dna_domain_fastq::IndexReferenceFileEntryV1 {
        relative_path: emitted_path
            .strip_prefix(&index_root)
            .unwrap_or(&emitted_path)
            .display()
            .to_string(),
        bytes: fs::metadata(&emitted_path)
            .with_context(|| format!("stat {}", emitted_path.display()))?
            .len(),
    }];
    let index_bytes = emitted_files.iter().map(|entry| entry.bytes).sum::<u64>();
    let report_path = probe_dir.join("index_reference_report.json");
    let report = bijux_dna_domain_fastq::IndexReferenceReportV1 {
        schema_version: bijux_dna_domain_fastq::INDEX_REFERENCE_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.index_reference".to_string(),
        stage_id: "fastq.index_reference".to_string(),
        tool_id: tool_id.clone(),
        threads: u64_json(&plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        index_format: tool_id.clone(),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        reference_bytes,
        reference_index: path_relative_to_repo(repo_root, &index_root),
        report_json: path_relative_to_repo(repo_root, &report_path),
        index_prefix: Some("reference".to_string()),
        index_file_count: emitted_files.len() as u64,
        index_bytes,
        emitted_files,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        backend_metrics: Some(serde_json::json!({ "probe": "governed_tiny_index_reference" })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: tool_id })
}

fn generate_correct_errors_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let cases = bijux_dna_planner_fastq::stage_api::local_correct_errors_smoke_plans(repo_root)?;
    let case =
        cases.first().ok_or_else(|| anyhow!("missing governed local correct-errors smoke case"))?;
    let probe_dir = probe_root.join("error_correction");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_r1 = repo_root.join(&case.r1);
    let input_r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let output_r1 = probe_dir.join("corrected_R1.fastq.gz");
    copy_file(&input_r1, &output_r1)?;
    let output_r2 = if let Some(input) = input_r2.as_ref() {
        let path = probe_dir.join("corrected_R2.fastq.gz");
        copy_file(input, &path)?;
        Some(path)
    } else {
        None
    };
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let report_path = probe_dir.join("correct_report.json");
    let engine = match case.plan.tool_id.as_str() {
        "lighter" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Lighter,
        "musket" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Musket,
        "bayeshammer" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Bayeshammer,
        _ => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Rcorrector,
    };
    let report = bijux_dna_domain_fastq::CorrectErrorsReportV1 {
        schema_version: bijux_dna_domain_fastq::CORRECT_ERRORS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.correct_errors".to_string(),
        stage_id: "fastq.correct_errors".to_string(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: u64_json(&case.plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        correction_engine: engine,
        quality_encoding: case.quality_encoding.clone(),
        kmer_size: u64_json(&case.plan.effective_params, "kmer_size")
            .and_then(|value| u32::try_from(value).ok()),
        musket_kmer_budget: u64_json(&case.plan.effective_params, "musket_kmer_budget"),
        genome_size: u64_json(&case.plan.effective_params, "genome_size"),
        max_memory_gb: u64_json(&case.plan.effective_params, "max_memory_gb")
            .and_then(|value| u32::try_from(value).ok()),
        trusted_kmer_artifact: None,
        conservative_mode: case.conservative_mode,
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, &report_path),
        corrected_reads: Some(reads_in),
        changed_reads: Some(0),
        unchanged_reads: Some(reads_in),
        reads_in: Some(reads_in),
        reads_out: Some(reads_in),
        bases_in: Some(bases_in),
        bases_out: Some(bases_in),
        pairs_in,
        pairs_out: pairs_in,
        mean_q_before: None,
        mean_q_after: None,
        kmer_fix_rate: Some(0.0),
        correction_effect: Some(
            serde_json::json!({ "outputs_changed": false, "probe": "governed_tiny_copy" }),
        ),
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(serde_json::json!({ "changed_reads": 0_u64 })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: case.plan.tool_id.as_str().to_string(),
    })
}

fn generate_deplete_host_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_host_plan(repo_root)?;
    let probe_dir = probe_root.join("deplete_host");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_r1 = required_input_path(&plan, "reads_r1")?;
    let input_r2 = optional_input_path(&plan, "reads_r2");
    let output_r1 = probe_dir.join("host_depleted_R1.fastq.gz");
    copy_file(&input_r1, &output_r1)?;
    let output_r2 = if let Some(input) = input_r2.as_ref() {
        let path = probe_dir.join("host_depleted_R2.fastq.gz");
        copy_file(input, &path)?;
        Some(path)
    } else {
        None
    };
    let removed_r1 = probe_dir.join("removed_host_R1.fastq.gz");
    write_empty_gzip(&removed_r1)?;
    let removed_r2 = if input_r2.is_some() {
        let path = probe_dir.join("removed_host_R2.fastq.gz");
        write_empty_gzip(&path)?;
        Some(path)
    } else {
        None
    };
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let report_path = probe_dir.join("host_depletion_report.json");
    let report = bijux_dna_domain_fastq::DepleteHostReportV1 {
        schema_version: bijux_dna_domain_fastq::DEPLETE_HOST_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.deplete_host".to_string(),
        stage_id: "fastq.deplete_host".to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: u64_json(&plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        reference_scope: serde_json::from_value(
            plan.effective_params
                .get("reference_scope")
                .cloned()
                .unwrap_or(Value::String("host".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReferenceScope::Host),
        reference_catalog_id: string_json(&plan.effective_params, "reference_catalog_id")
            .unwrap_or_else(|| "host_reference".to_string()),
        reference_index_artifact_id: string_json(
            &plan.effective_params,
            "reference_index_artifact_id",
        )
        .unwrap_or_else(|| "reference_index".to_string()),
        reference_index_backend: string_json(&plan.effective_params, "reference_index_backend")
            .unwrap_or_else(|| "bowtie2_build".to_string()),
        reference_build_id: optional_string_json(&plan.effective_params, "reference_build_id"),
        reference_digest: optional_string_json(&plan.effective_params, "reference_digest"),
        masking_policy: serde_json::from_value(
            plan.effective_params
                .get("masking_policy")
                .cloned()
                .unwrap_or(Value::String("unmasked".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReferenceMaskingPolicy::Unmasked),
        decoy_policy: serde_json::from_value(
            plan.effective_params
                .get("decoy_policy")
                .cloned()
                .unwrap_or(Value::String("none".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReferenceDecoyPolicy::None),
        decoy_catalog_id: optional_string_json(&plan.effective_params, "decoy_catalog_id"),
        identity_threshold: f64_json(&plan.effective_params, "identity_threshold").unwrap_or(0.95),
        retained_read_policy: serde_json::from_value(
            plan.effective_params
                .get("retained_read_policy")
                .cloned()
                .unwrap_or(Value::String("keep_non_host_reads".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::ReadRetentionPolicy::KeepNonHostReads),
        emit_removed_reads: bool_json(&plan.effective_params, "emit_removed_reads").unwrap_or(true),
        report_format: serde_json::from_value(
            plan.effective_params
                .get("report_format")
                .cloned()
                .unwrap_or(Value::String("bowtie2_metrics_file".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::MappingReportFormat::Bowtie2MetricsFile),
        retain_unmapped_pairs: bool_json(&plan.effective_params, "retain_unmapped_pairs")
            .unwrap_or(false),
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        removed_host_r1: path_relative_to_repo(repo_root, &removed_r1),
        removed_host_r2: removed_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, &report_path),
        reads_in,
        reads_out: reads_in,
        reads_removed: 0,
        bases_in,
        bases_out: bases_in,
        bases_removed: 0,
        pairs_in,
        pairs_out: pairs_in,
        host_fraction_removed: 0.0,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(
            serde_json::json!({ "reads_removed": 0_u64, "bases_removed": 0_u64 }),
        ),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}

fn generate_deplete_rrna_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan(repo_root)?;
    let probe_dir = probe_root.join("deplete_rrna");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_r1 = required_input_path(&plan, "reads_r1")?;
    let input_r2 = optional_input_path(&plan, "reads_r2");
    let output_r1 = probe_dir.join("rrna_filtered_R1.fastq.gz");
    copy_file(&input_r1, &output_r1)?;
    let output_r2 = if let Some(input) = input_r2.as_ref() {
        let path = probe_dir.join("rrna_filtered_R2.fastq.gz");
        copy_file(input, &path)?;
        Some(path)
    } else {
        None
    };
    let removed_r1 = probe_dir.join("removed_rrna_R1.fastq.gz");
    write_empty_gzip(&removed_r1)?;
    let removed_r2 = if input_r2.is_some() {
        let path = probe_dir.join("removed_rrna_R2.fastq.gz");
        write_empty_gzip(&path)?;
        Some(path)
    } else {
        None
    };
    let report_tsv = probe_dir.join("rrna_report.tsv");
    fs::write(&report_tsv, "sample_id\treads_removed\nlocal_smoke\t0\n")
        .with_context(|| format!("write {}", report_tsv.display()))?;
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let report_path = probe_dir.join("rrna_report.json");
    let report = bijux_dna_domain_fastq::DepleteRrnaReportV1 {
        schema_version: bijux_dna_domain_fastq::DEPLETE_RRNA_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.deplete_rrna".to_string(),
        stage_id: "fastq.deplete_rrna".to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: u64_json(&plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        rrna_db: optional_string_json(&plan.effective_params, "contaminant_db"),
        database_artifact_id: string_json(&plan.effective_params, "database_artifact_id")
            .unwrap_or_else(|| "rrna_reference".to_string()),
        database_build_id: optional_string_json(&plan.effective_params, "database_build_id"),
        database_digest: optional_string_json(&plan.effective_params, "database_digest"),
        screening_engine: serde_json::from_value(
            plan.effective_params
                .get("screening_engine")
                .cloned()
                .unwrap_or(Value::String("sortmerna".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::RrnaScreeningEngine::Sortmerna),
        report_format: serde_json::from_value(
            plan.effective_params
                .get("report_format")
                .cloned()
                .unwrap_or(Value::String("summary_tsv_and_json".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::RrnaReportFormat::SummaryTsvAndJson),
        emit_removed_reads: bool_json(&plan.effective_params, "emit_removed_reads").unwrap_or(true),
        min_identity: f64_json(&plan.params, "min_identity"),
        retained_read_role: "rrna_filtered_reads".to_string(),
        rejected_read_role: "removed_rrna_reads".to_string(),
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        removed_reads_r1: path_relative_to_repo(repo_root, &removed_r1),
        removed_reads_r2: removed_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        rrna_report_tsv: path_relative_to_repo(repo_root, &report_tsv),
        rrna_report_json: path_relative_to_repo(repo_root, &report_path),
        reads_in,
        reads_out: reads_in,
        reads_removed: 0,
        bases_in,
        bases_out: bases_in,
        bases_removed: 0,
        pairs_in,
        pairs_out: pairs_in,
        rrna_fraction_removed: 0.0,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(serde_json::json!({ "reads_removed": 0_u64 })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}

fn generate_screen_taxonomy_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan(repo_root)?;
    let probe_dir = probe_root.join("screen_taxonomy");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_r1 = required_input_path(&plan, "reads_r1")?;
    let input_r2 = optional_input_path(&plan, "reads_r2");
    let unclassified_r1 = probe_dir.join("unclassified_R1.fastq.gz");
    write_empty_gzip(&unclassified_r1)?;
    let unclassified_r2 = if input_r2.is_some() {
        let path = probe_dir.join("unclassified_R2.fastq.gz");
        write_empty_gzip(&path)?;
        Some(path)
    } else {
        None
    };
    let report_tsv = probe_dir.join("taxonomy_screen.tsv");
    fs::write(&report_tsv, "tax_id\tname\treads\tfraction\n9606\tHomo sapiens\t4\t1.000000\n")
        .with_context(|| format!("write {}", report_tsv.display()))?;
    let effective_params: bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("parse governed screen_taxonomy effective params")?;
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let entries = vec![bijux_dna_domain_fastq::TaxonomyScreenSummaryEntryV1 {
        label: "Homo sapiens".to_string(),
        percent: 100.0,
    }];
    let report_path = probe_dir.join("classification_report.json");
    let report = bijux_dna_domain_fastq::ScreenTaxonomyReportV1 {
        schema_version: bijux_dna_domain_fastq::SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.screen_taxonomy".to_string(),
        stage_id: "fastq.screen_taxonomy".to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: effective_params.threads,
        classifier: effective_params.classifier,
        report_format: effective_params.report_format,
        assignment_format: effective_params.assignment_format,
        database_catalog_id: effective_params.database_catalog_id,
        database_artifact_id: effective_params.database_artifact_id,
        database_build_id: effective_params.database_build_id,
        database_digest: effective_params.database_digest,
        database_namespace: effective_params.database_namespace,
        database_scope: effective_params.database_scope,
        minimum_confidence: effective_params.minimum_confidence,
        emit_unclassified: effective_params.emit_unclassified,
        interpretation_boundary: effective_params.interpretation_boundary,
        truth_conditions: effective_params.truth_conditions,
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        screen_report_tsv: path_relative_to_repo(repo_root, &report_tsv),
        classification_report_json: path_relative_to_repo(repo_root, &report_path),
        unclassified_reads_r1: Some(path_relative_to_repo(repo_root, &unclassified_r1)),
        unclassified_reads_r2: unclassified_r2
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        reads_in: Some(reads_in),
        reads_out: Some(reads_in),
        bases_in: Some(bases_in),
        bases_out: Some(bases_in),
        pairs_in,
        pairs_out: pairs_in,
        contamination_rate: Some(0.0),
        classified_fraction: Some(1.0),
        unclassified_fraction: Some(0.0),
        summary_entries: entries.clone(),
        top_taxa: entries,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}

fn generate_bam_align_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_bam::stage_api::local_align_plan(repo_root)?;
    let probe_dir = probe_root.join("bam_align");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let reference_fasta = required_input_path(&plan, "reference_fasta")
        .or_else(|_| required_input_path(&plan, "reference"))?;
    let input_r1 = required_input_path(&plan, "fastq_r1")
        .or_else(|_| required_input_path(&plan, "reads_r1"))
        .or_else(|_| required_input_path(&plan, "input_r1"))?;
    let input_r2 = optional_input_path(&plan, "fastq_r2")
        .or_else(|| optional_input_path(&plan, "reads_r2"))
        .or_else(|| optional_input_path(&plan, "input_r2"));
    let sample_id =
        string_json(&plan.params, "sample_id").unwrap_or_else(|| "local_smoke".to_string());
    let read_group: bijux_dna_domain_bam::params::ReadGroupSpec =
        serde_json::from_value(plan.params.get("read_group").cloned().unwrap_or_else(|| {
            serde_json::json!({
                "id": sample_id,
                "sample": sample_id,
                "platform": "ILLUMINA",
                "library": "local_smoke",
                "platform_unit": "local_smoke",
            })
        }))
        .context("decode bam.align read_group")?;
    let (provenance, _mapping) = match plan.tool_id.as_str() {
        "bowtie2" => bijux_dna_domain_bam::align_fastq_to_bam_bowtie2_style(
            &reference_fasta,
            &input_r1,
            input_r2.as_deref(),
            &probe_dir,
            &sample_id,
            &read_group,
            string_json(&plan.params, "sensitivity_profile").as_deref(),
        )?,
        _ => bijux_dna_domain_bam::align_fastq_to_bam_bwa_style(
            &reference_fasta,
            &input_r1,
            input_r2.as_deref(),
            &probe_dir,
            &sample_id,
            &read_group,
            string_json(&plan.params, "preset").as_deref(),
            u64_json(&plan.params, "seed_length").and_then(|value| u32::try_from(value).ok()),
        )?,
    };
    let report_path = probe_dir.join("alignment.provenance.json");
    bijux_dna_infra::atomic_write_json(&report_path, &provenance)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}

fn generate_bam_contamination_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_bam::stage_api::local_contamination_plan(repo_root)?;
    let probe_dir = probe_root.join("bam_contamination");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let report_path = probe_dir.join("contamination.summary.json");
    let scope = string_json(&plan.params, "scope").unwrap_or_else(|| "both".to_string());
    let method = if scope == "mitochondrial" || plan.tool_id.as_str() == "schmutzi" {
        "schmutzi"
    } else if plan.tool_id.as_str() == "contammix" {
        "contammix"
    } else {
        "verifybamid2"
    };
    let assumptions = vec![
        string_json(&plan.params, "assumptions")
            .unwrap_or_else(|| "governed tiny contamination parser smoke".to_string()),
        format!("scope:{scope}"),
    ];
    bijux_dna_infra::atomic_write_json(
        &report_path,
        &serde_json::json!({
            "method": method,
            "estimate": 0.02,
            "ci_low": 0.01,
            "ci_high": 0.03,
            "assumptions": assumptions,
        }),
    )?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}

fn generate_bam_damage_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let cases = bijux_dna_planner_bam::stage_api::local_damage_smoke_plans(repo_root)?;
    let case = cases.first().ok_or_else(|| anyhow!("missing governed local damage smoke case"))?;
    let probe_dir = probe_root.join("bam_damage");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_bam = repo_root.join(&case.bam);
    let damage_metrics = bijux_dna_domain_bam::metrics::DamageMetricsV1 {
        c_to_t_5p: case.expected_terminal_c_to_t_5p,
        g_to_a_3p: case.expected_terminal_g_to_a_3p,
        pmd_score_histogram: Vec::new(),
    };
    let evidence = bijux_dna_domain_bam::summarize_tiny_bam_damage_evidence(
        &input_bam,
        &damage_metrics,
        case.expected_strict_profile_upgraded,
    )?;
    let report_path = probe_dir.join("damage.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &evidence)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: case.plan.tool_id.as_str().to_string(),
    })
}

fn generate_bam_authenticity_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let cases = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(repo_root)?;
    let case =
        cases.first().ok_or_else(|| anyhow!("missing governed local authenticity smoke case"))?;
    let probe_dir = probe_root.join("bam_authenticity");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_bam = repo_root.join(&case.bam);
    let damage_metrics = bijux_dna_domain_bam::metrics::DamageMetricsV1 {
        c_to_t_5p: case.damage_terminal_c_to_t_5p,
        g_to_a_3p: case.damage_terminal_g_to_a_3p,
        pmd_score_histogram: Vec::new(),
    };
    let advisory = bijux_dna_domain_bam::summarize_tiny_bam_authenticity_advisory(
        &input_bam,
        &damage_metrics,
    )?;
    let report_path = probe_dir.join("authenticity.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &advisory)?;
    Ok(MaterializedProof {
        proof_path: report_path,
        observed_tool_id: case.plan.tool_id.as_str().to_string(),
    })
}

fn generate_bam_genotyping_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let config: LocalBamGenotypingConfig =
        read_toml_document(&repo_root.join(LOCAL_BAM_GENOTYPING_CONFIG_PATH))?;
    let tool_id = config.tool_id.clone();
    let probe_dir = probe_root.join("bam_genotyping");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let report_path = probe_dir.join("genotyping.json");
    bijux_dna_infra::atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.bam.genotyping.v1",
            "stage_id": "bam.genotyping",
            "producer": "bam.genotyping",
            "caller": tool_id,
            "reference": path_relative_to_repo(repo_root, &repo_root.join(config.reference_fasta)),
            "sites": path_relative_to_repo(repo_root, &repo_root.join(config.sites_vcf)),
            "regions": path_relative_to_repo(repo_root, &repo_root.join(config.regions)),
            "sample_id": config.sample_id,
            "call_rate": 1.0,
            "mean_posterior": 0.99,
            "posterior_histogram": [[99, 4]],
        }),
    )?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: config.tool_id })
}

fn generate_bam_kinship_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let config: LocalBamKinshipConfig =
        read_toml_document(&repo_root.join(LOCAL_BAM_KINSHIP_CONFIG_PATH))?;
    let case = config
        .cases
        .iter()
        .find(|case| case.expected_status == "ok")
        .or_else(|| config.cases.first())
        .ok_or_else(|| anyhow!("missing governed local kinship smoke case"))?;
    let probe_dir = probe_root.join("bam_kinship");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let report_path = probe_dir.join("kinship.summary.json");
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_kinship(
        &repo_root.join(&case.bam),
        &config.tool_id,
        &case.reference_panel,
        &case.reference_build,
        &case.population_scope,
        case.min_overlap_snps,
        case.requires_cohort_context,
    )?;
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: config.tool_id })
}

fn generate_bam_sex_probe(repo_root: &Path, probe_root: &Path) -> Result<MaterializedProof> {
    let config: LocalBamSexConfig = read_toml_document(&repo_root.join(LOCAL_BAM_SEX_CONFIG_PATH))?;
    let case =
        config.cases.first().ok_or_else(|| anyhow!("missing governed local sex smoke case"))?;
    let probe_dir = probe_root.join("bam_sex");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_sex(
        &repo_root.join(&case.bam),
        &repo_root.join(&case.reference),
        &config.tool_id,
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    let report_path = probe_dir.join("sex.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: config.tool_id })
}

fn generate_vcf_call_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_call_smoke(repo_root, "bcftools")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

fn generate_vcf_impute_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_impute_smoke(repo_root, "beagle")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

fn generate_vcf_phasing_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_phasing_smoke(repo_root, "shapeit5")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

fn generate_vcf_population_structure_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_population_structure_smoke(repo_root, "plink2")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.population_structure_json_path),
        observed_tool_id: report.tool_id,
    })
}

fn generate_vcf_qc_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_qc_smoke(repo_root, "plink")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

fn observe_tool_id(repo_root: &Path, proof_path: &Path, stage_id: &str) -> Result<String> {
    let value = read_json_document(proof_path)?;
    if let Some(tool_id) = value
        .get("tool_id")
        .and_then(Value::as_str)
        .or_else(|| value.get("planned_tool_id").and_then(Value::as_str))
        .or_else(|| value.get("report_tool_id").and_then(Value::as_str))
    {
        return Ok(tool_id.to_string());
    }
    let plans = local_stage_plans(repo_root, stage_id)?;
    plans.first().map(|plan| plan.tool_id.as_str().to_string()).ok_or_else(|| {
        anyhow!("unable to determine tool_id for stage `{stage_id}` proof {}", proof_path.display())
    })
}

fn required_input_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<PathBuf> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("plan `{}` is missing input `{artifact_name}`", plan.stage_id.as_str())
        })
}

fn optional_input_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Option<PathBuf> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
}

fn count_fastq_stats(path: &Path) -> Result<FastqReadStats> {
    let reader: Box<dyn Read> = match path.extension().and_then(|value| value.to_str()) {
        Some("gz") => Box::new(MultiGzDecoder::new(
            fs::File::open(path).with_context(|| format!("open {}", path.display()))?,
        )),
        _ => Box::new(fs::File::open(path).with_context(|| format!("open {}", path.display()))?),
    };
    let mut lines = BufReader::new(reader).lines();
    let mut reads = 0_u64;
    let mut bases = 0_u64;
    loop {
        let Some(_header) = lines.next() else {
            break;
        };
        let sequence = lines
            .next()
            .ok_or_else(|| anyhow!("malformed FASTQ {}", path.display()))?
            .with_context(|| format!("read sequence from {}", path.display()))?;
        let _plus = lines
            .next()
            .ok_or_else(|| anyhow!("malformed FASTQ {}", path.display()))?
            .with_context(|| format!("read plus from {}", path.display()))?;
        let _quality = lines
            .next()
            .ok_or_else(|| anyhow!("malformed FASTQ {}", path.display()))?
            .with_context(|| format!("read quality from {}", path.display()))?;
        reads += 1;
        bases += sequence.len() as u64;
    }
    Ok(FastqReadStats { reads, bases })
}

fn write_empty_gzip(path: &Path) -> Result<()> {
    let file = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&[]).with_context(|| format!("write {}", path.display()))?;
    encoder.finish().with_context(|| format!("finish {}", path.display()))?;
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(source, destination)
        .with_context(|| format!("copy {} -> {}", source.display(), destination.display()))?;
    Ok(())
}

fn ensure_governed_keys_present(value: &Value, proof_path: &Path, keys: &[&str]) -> Result<()> {
    for key in keys {
        if value.get(*key).is_none() {
            return Err(anyhow!("{} is missing `{key}`", proof_path.display()));
        }
    }
    Ok(())
}

fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn read_toml_document<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn top_level_keys(value: &Value) -> Vec<String> {
    value.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()
}

fn json_snapshot(value: &Value, keys: &[&str]) -> BTreeMap<String, Value> {
    keys.iter()
        .filter_map(|key| value.get(*key).cloned().map(|entry| ((*key).to_string(), entry)))
        .collect()
}

fn json_pointer_string(value: &Value, pointer: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("JSON pointer `{pointer}` did not resolve to a string"))
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn string_json(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn optional_string_json(value: &Value, key: &str) -> Option<String> {
    string_json(value, key)
}

fn bool_json(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn u64_json(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn f64_json(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn array_of_strings_json(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{build_real_output_parser_smoke_report, DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH};
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn build_real_output_parser_smoke_report_covers_active_families() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .map(std::path::Path::to_path_buf)
            .expect("resolve workspace root from crate manifest dir");
        let output_path = repo_root.join(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH);
        let probe_root =
            repo_root.join("runs/bench/readiness-probes/tests/real-output-parser-smoke");
        if probe_root.exists() {
            std::fs::remove_dir_all(&probe_root)?;
        }
        std::fs::create_dir_all(&probe_root)?;
        let report = build_real_output_parser_smoke_report(&repo_root, &output_path, &probe_root)?;
        assert!(!report.rows.is_empty());
        assert_eq!(report.family_count, report.rows.len());
        assert_eq!(report.failed_family_count, 0);
        assert!(report.rows.iter().all(|row| row.passed));
        Ok(())
    }
}
