use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids;
use serde::Serialize;

use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::{
    build_vcf_stage_matrix_rows, VcfStageMatrixRow,
};
use crate::commands::benchmark::readiness::vcf_readiness_inputs::materialize_reference_fasta_with_index;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH: &str =
    "benchmarks/readiness/adapters/bcftools.vcf.json";
const VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_bcftools_adapter.v1";
const GOVERNED_BCFTOOLS_TOOL_ID: &str = "bcftools";
const GOVERNED_BAM_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam";
const GOVERNED_BAM_INDEX_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam.bai";
const GOVERNED_REFERENCE_FASTA_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";
const GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf";
const GOVERNED_FILTERED_SINGLE_SAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_filtered_single_sample.vcf";
const GOVERNED_MULTISAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";
const GOVERNED_REFERENCE_PANEL_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_reference_panel.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfBcftoolsAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfBcftoolsAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfBcftoolsAdapterRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) support_status: String,
    pub(crate) benchmark_status: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) output_root: String,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfBcftoolsAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfBcftoolsAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfBcftoolsAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfBcftoolsAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) tool_id: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) supported_row_count: usize,
    pub(crate) planned_row_count: usize,
    pub(crate) argv_valid_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) indexed_row_count: usize,
    pub(crate) rows: Vec<VcfBcftoolsAdapterRow>,
}

pub(crate) fn run_render_vcf_bcftools_adapter(
    args: &parse::BenchReadinessRenderVcfBcftoolsAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_bcftools_adapter(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_bcftools_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfBcftoolsAdapterReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_bcftools_adapter_rows(repo_root)?;
    let supported_row_count = rows.iter().filter(|row| row.support_status == "supported").count();
    let planned_row_count = rows.len().saturating_sub(supported_row_count);
    let argv_valid_row_count = rows.iter().filter(|row| row.argv_validation_passed).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();
    let indexed_row_count = rows
        .iter()
        .filter(|row| row.command_steps.iter().any(|step| step.step_kind == "index"))
        .count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::to_string_pretty(&VcfBcftoolsAdapterReport {
        schema_version: VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: GOVERNED_BCFTOOLS_TOOL_ID,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        supported_row_count,
        planned_row_count,
        argv_valid_row_count,
        missing_input_test_passed_row_count,
        indexed_row_count,
        rows: rows.clone(),
    })
    .context("render VCF bcftools adapter report")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;

    Ok(VcfBcftoolsAdapterReport {
        schema_version: VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: GOVERNED_BCFTOOLS_TOOL_ID,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        supported_row_count,
        planned_row_count,
        argv_valid_row_count,
        missing_input_test_passed_row_count,
        indexed_row_count,
        rows,
    })
}

pub(crate) fn collect_vcf_bcftools_adapter_rows(
    repo_root: &Path,
) -> Result<Vec<VcfBcftoolsAdapterRow>> {
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    for matrix_row in build_vcf_stage_matrix_rows()? {
        if matrix_row.tool_id != GOVERNED_BCFTOOLS_TOOL_ID {
            continue;
        }
        let catalog_row = catalog_by_stage.get(matrix_row.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF bcftools adapter report is missing catalog coverage for `{}`",
                matrix_row.stage_id
            )
        })?;
        rows.push(build_bcftools_row(repo_root, &matrix_row, catalog_row)?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_bcftools_adapter_contract(&rows)?;
    Ok(rows)
}

fn build_bcftools_row(
    repo_root: &Path,
    matrix_row: &VcfStageMatrixRow,
    catalog_row: &VcfStageCatalogRow,
) -> Result<VcfBcftoolsAdapterRow> {
    let stage = VcfDomainStage::try_from(matrix_row.stage_id.as_str())
        .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", matrix_row.stage_id))?;
    let output_root =
        format!("benchmarks/readiness/adapters/{}/{}", matrix_row.tool_id, matrix_row.stage_id);
    let required_inputs = governed_inputs_for_stage(repo_root, stage, &output_root)?;
    validate_required_inputs(repo_root, &matrix_row.stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| {
            anyhow!("VCF stage `{}` is missing expected output ids", matrix_row.stage_id)
        })?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let (raw_output_ids, parser_output_ids, declared_outputs, command_steps) =
        build_stage_adapter_contract(stage, &matrix_row.stage_id, &output_root, &required_inputs)?;
    let argv_validation_passed =
        validate_command_steps(&matrix_row.stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, &matrix_row.stage_id, &required_inputs);
    let benchmark_status = match catalog_row.support_status.as_str() {
        "supported" => "benchmark_ready",
        _ => "not_benchmark_ready",
    }
    .to_string();

    let reason = format!(
        "row `{}` / `{}` renders concrete bcftools argv with {} declared command step(s), {} required input(s), and {} stage output id(s)",
        matrix_row.stage_id,
        matrix_row.tool_id,
        command_steps.len(),
        required_inputs.len(),
        stage_output_ids.len()
    );

    Ok(VcfBcftoolsAdapterRow {
        tool_id: matrix_row.tool_id.clone(),
        stage_id: matrix_row.stage_id.clone(),
        support_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id: matrix_row.adapter_id.clone(),
        parser_id: matrix_row.parser_id.clone(),
        corpus_id: matrix_row.corpus_id.clone(),
        asset_profile_id: matrix_row.asset_profile_id.clone(),
        output_root,
        stage_output_ids,
        raw_output_ids,
        parser_output_ids,
        required_inputs,
        declared_outputs,
        command_steps,
        argv_validation_passed,
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
        reason,
    })
}

fn governed_inputs_for_stage(
    repo_root: &Path,
    stage: VcfDomainStage,
    output_root: &str,
) -> Result<Vec<VcfBcftoolsAdapterArtifact>> {
    Ok(match stage {
        VcfDomainStage::PrepareReferencePanel => {
            vec![artifact("reference_panel_vcf", "variant", GOVERNED_REFERENCE_PANEL_VCF_PATH)]
        }
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => {
            let (reference_fasta, reference_fai) = materialize_reference_fasta_with_index(
                repo_root,
                GOVERNED_REFERENCE_FASTA_PATH,
                &PathBuf::from(output_root).join("artifacts/reference"),
            )?;
            vec![
                artifact("input_bam", "bam", GOVERNED_BAM_PATH),
                artifact("input_bam_index", "index", GOVERNED_BAM_INDEX_PATH),
                artifact("reference_fasta", "reference", &reference_fasta),
                artifact("reference_fai", "index", &reference_fai),
            ]
        }
        VcfDomainStage::Stats => vec![artifact("vcf", "variant", GOVERNED_MULTISAMPLE_VCF_PATH)],
        VcfDomainStage::Postprocess => {
            vec![artifact("vcf", "variant", GOVERNED_FILTERED_SINGLE_SAMPLE_VCF_PATH)]
        }
        _ => vec![artifact("vcf", "variant", GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH)],
    })
}

fn build_stage_adapter_contract(
    stage: VcfDomainStage,
    stage_id: &str,
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<(
    Vec<String>,
    Vec<String>,
    Vec<VcfBcftoolsAdapterArtifact>,
    Vec<VcfBcftoolsAdapterCommandStep>,
)> {
    let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
    let json_output_path = |name: &str| format!("{output_root}/{name}.json");
    let text_output_path = |name: &str| format!("{output_root}/{name}.txt");

    let contract = match stage {
        VcfDomainStage::PrepareReferencePanel => {
            let panel_output = variant_output_path("prepared_panel");
            (
                vec!["prepared_panel".to_string(), "prepared_panel_tbi".to_string()],
                vec!["chunks_json".to_string()],
                vec![
                    artifact("prepared_panel", "variant", &panel_output),
                    artifact("prepared_panel_tbi", "index", &format!("{panel_output}.tbi")),
                    artifact("chunks_json", "report_json", &json_output_path("chunks")),
                ],
                vec![
                    step(
                        "normalize_panel",
                        "transform",
                        vec![
                            "bcftools",
                            "norm",
                            "-m-any",
                            input_by_id(inputs, "reference_panel_vcf")?,
                            "-Oz",
                            "-o",
                            &panel_output,
                        ],
                        false,
                        &["prepared_panel"],
                    ),
                    step(
                        "index_panel",
                        "index",
                        vec!["bcftools", "index", "-t", &panel_output],
                        false,
                        &["prepared_panel_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::Call => {
            let output = variant_output_path("called_vcf");
            (
                vec!["called_vcf".to_string(), "called_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("called_vcf", "variant", &output),
                    artifact("called_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "mpileup",
                        "pipeline_source",
                        vec![
                            "bcftools",
                            "mpileup",
                            "-Ou",
                            "-f",
                            input_by_id(inputs, "reference_fasta")?,
                            input_by_id(inputs, "input_bam")?,
                        ],
                        false,
                        &[],
                    ),
                    step(
                        "call",
                        "pipeline_sink",
                        vec!["bcftools", "call", "-c", "-Oz", "-o", &output],
                        true,
                        &["called_vcf"],
                    ),
                    step(
                        "index_called_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["called_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::CallDiploid => {
            let output = variant_output_path("diploid_vcf");
            (
                vec!["diploid_vcf".to_string(), "diploid_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("diploid_vcf", "variant", &output),
                    artifact("diploid_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "mpileup",
                        "pipeline_source",
                        vec![
                            "bcftools",
                            "mpileup",
                            "-Ou",
                            "-f",
                            input_by_id(inputs, "reference_fasta")?,
                            input_by_id(inputs, "input_bam")?,
                        ],
                        false,
                        &[],
                    ),
                    step(
                        "call_diploid",
                        "pipeline_sink",
                        vec!["bcftools", "call", "-mv", "-Oz", "-o", &output],
                        true,
                        &["diploid_vcf"],
                    ),
                    step(
                        "index_diploid_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["diploid_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::CallGl => {
            let output = variant_output_path("gl_sites_vcf");
            (
                vec!["gl_sites_vcf".to_string(), "gl_sites_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("gl_sites_vcf", "variant", &output),
                    artifact("gl_sites_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "mpileup",
                        "pipeline_source",
                        vec![
                            "bcftools",
                            "mpileup",
                            "-Ou",
                            "-f",
                            input_by_id(inputs, "reference_fasta")?,
                            input_by_id(inputs, "input_bam")?,
                        ],
                        false,
                        &[],
                    ),
                    step(
                        "call_gl",
                        "pipeline_sink",
                        vec!["bcftools", "call", "-Aim", "-Oz", "-o", &output],
                        true,
                        &["gl_sites_vcf"],
                    ),
                    step(
                        "index_gl_sites_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["gl_sites_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::CallPseudohaploid => {
            let output = variant_output_path("pseudohaploid_vcf");
            (
                vec!["pseudohaploid_vcf".to_string(), "pseudohaploid_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("pseudohaploid_vcf", "variant", &output),
                    artifact("pseudohaploid_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "mpileup",
                        "pipeline_source",
                        vec![
                            "bcftools",
                            "mpileup",
                            "-Ou",
                            "-f",
                            input_by_id(inputs, "reference_fasta")?,
                            input_by_id(inputs, "input_bam")?,
                        ],
                        false,
                        &[],
                    ),
                    step(
                        "call_pseudohaploid",
                        "pipeline_sink",
                        vec!["bcftools", "call", "--ploidy", "1", "-mv", "-Oz", "-o", &output],
                        true,
                        &["pseudohaploid_vcf"],
                    ),
                    step(
                        "index_pseudohaploid_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["pseudohaploid_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::DamageFilter => {
            let output = variant_output_path("damage_filtered_vcf");
            (
                vec!["damage_filtered_vcf".to_string(), "damage_filtered_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("damage_filtered_vcf", "variant", &output),
                    artifact("damage_filtered_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "damage_filter",
                        "transform",
                        vec![
                            "bcftools",
                            "filter",
                            "-e",
                            "((REF=\"C\" && ALT=\"T\") || (REF=\"G\" && ALT=\"A\")) && INFO/PMD>3",
                            input_by_id(inputs, "vcf")?,
                            "-Oz",
                            "-o",
                            &output,
                        ],
                        false,
                        &["damage_filtered_vcf"],
                    ),
                    step(
                        "index_damage_filtered_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["damage_filtered_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::Filter => {
            let output = variant_output_path("filtered_vcf");
            (
                vec!["filtered_vcf".to_string(), "filtered_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("filtered_vcf", "variant", &output),
                    artifact("filtered_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "filter_sites",
                        "transform",
                        vec![
                            "bcftools",
                            "filter",
                            "-s",
                            "LOWQUAL",
                            "-e",
                            "QUAL<30",
                            input_by_id(inputs, "vcf")?,
                            "-Oz",
                            "-o",
                            &output,
                        ],
                        false,
                        &["filtered_vcf"],
                    ),
                    step(
                        "index_filtered_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["filtered_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::GlPropagation => {
            let output = variant_output_path("gl_propagated_vcf");
            (
                vec!["gl_propagated_vcf".to_string(), "gl_propagated_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("gl_propagated_vcf", "variant", &output),
                    artifact("gl_propagated_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "propagate_gl_fields",
                        "transform",
                        vec![
                            "bcftools",
                            "annotate",
                            "-x",
                            "INFO,^FORMAT/GL,^FORMAT/PL,^FORMAT/GP",
                            input_by_id(inputs, "vcf")?,
                            "-Oz",
                            "-o",
                            &output,
                        ],
                        false,
                        &["gl_propagated_vcf"],
                    ),
                    step(
                        "index_gl_propagated_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["gl_propagated_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::Postprocess => {
            let output = variant_output_path("postprocess_vcf");
            (
                vec!["postprocess_vcf".to_string(), "postprocess_vcf_tbi".to_string()],
                Vec::new(),
                vec![
                    artifact("postprocess_vcf", "variant", &output),
                    artifact("postprocess_vcf_tbi", "index", &format!("{output}.tbi")),
                ],
                vec![
                    step(
                        "fill_tags",
                        "transform",
                        vec![
                            "bcftools",
                            "+fill-tags",
                            input_by_id(inputs, "vcf")?,
                            "-Oz",
                            "-o",
                            &output,
                            "--",
                            "-t",
                            "AC,AN,AF",
                        ],
                        false,
                        &["postprocess_vcf"],
                    ),
                    step(
                        "index_postprocess_vcf",
                        "index",
                        vec!["bcftools", "index", "-t", &output],
                        false,
                        &["postprocess_vcf_tbi"],
                    ),
                ],
            )
        }
        VcfDomainStage::Stats => {
            let output = text_output_path("bcftools_stats");
            (
                vec!["bcftools_stats_txt".to_string()],
                vec!["stats_json".to_string()],
                vec![
                    artifact("bcftools_stats_txt", "report", &output),
                    artifact("stats_json", "metrics_json", &json_output_path("stats_json")),
                ],
                vec![step(
                    "render_stats",
                    "report",
                    vec![
                        "bcftools",
                        "stats",
                        "-s",
                        "-",
                        "-o",
                        &output,
                        input_by_id(inputs, "vcf")?,
                    ],
                    false,
                    &["bcftools_stats_txt"],
                )],
            )
        }
        other => {
            return Err(anyhow!(
                "VCF bcftools adapter does not govern stage `{}` for Goal 235",
                other.as_str()
            ));
        }
    };

    let (raw_output_ids, parser_output_ids, declared_outputs, command_steps) = contract;
    if command_steps.is_empty() {
        return Err(anyhow!(
            "VCF bcftools adapter row `{stage_id}` must keep at least one executable command step"
        ));
    }
    Ok((raw_output_ids, parser_output_ids, declared_outputs, command_steps))
}

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF bcftools adapter for `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            ));
        }
    }
    Ok(())
}

fn validate_command_steps(stage_id: &str, steps: &[VcfBcftoolsAdapterCommandStep]) -> Result<()> {
    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF bcftools adapter step `{}` for `{stage_id}` has empty argv",
                step.step_id
            ));
        }
        if step.argv[0] != GOVERNED_BCFTOOLS_TOOL_ID {
            return Err(anyhow!(
                "VCF bcftools adapter step `{}` for `{stage_id}` must execute `{}` first, found `{}`",
                step.step_id,
                GOVERNED_BCFTOOLS_TOOL_ID,
                step.argv[0]
            ));
        }
        if step.argv.iter().any(|part| {
            let lowered = part.to_ascii_lowercase();
            lowered.contains("placeholder") || lowered == "--help" || lowered.contains("todo")
        }) {
            return Err(anyhow!(
                "VCF bcftools adapter step `{}` for `{stage_id}` still contains placeholder argv: {:?}",
                step.step_id,
                step.argv
            ));
        }
    }
    Ok(())
}

fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> (String, String, String, bool) {
    let mut mutated_inputs = inputs.to_vec();
    let probe = mutated_inputs.first().cloned().unwrap_or_else(|| {
        artifact("missing_input", "unknown", "artifacts/bench-readiness/adapters/none.missing")
    });
    if let Some(first) = mutated_inputs.first_mut() {
        first.path = format!(
            "artifacts/bench-readiness/adapters/probes/{stage_id}/{}.missing",
            first.artifact_id
        );
    }
    let expected_error_fragment = format!("required input `{}`", probe.artifact_id);
    let observed_error = match validate_required_inputs(repo_root, stage_id, &mutated_inputs) {
        Ok(()) => format!(
            "VCF bcftools adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

fn ensure_vcf_bcftools_adapter_contract(rows: &[VcfBcftoolsAdapterRow]) -> Result<()> {
    if rows.len() != 10 {
        return Err(anyhow!(
            "VCF bcftools adapter must cover exactly 10 governed matrix rows, found {}",
            rows.len()
        ));
    }
    let expected_stages = BTreeSet::from([
        "vcf.prepare_reference_panel",
        "vcf.call",
        "vcf.call_gl",
        "vcf.call_diploid",
        "vcf.call_pseudohaploid",
        "vcf.damage_filter",
        "vcf.filter",
        "vcf.gl_propagation",
        "vcf.postprocess",
        "vcf.stats",
    ]);
    let observed_stages = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed_stages != expected_stages {
        return Err(anyhow!(
            "VCF bcftools adapter stage set drifted: expected {:?}, found {:?}",
            expected_stages,
            observed_stages
        ));
    }
    for row in rows {
        if !row.argv_validation_passed {
            return Err(anyhow!(
                "VCF bcftools adapter row `{}` failed argv validation",
                row.stage_id
            ));
        }
        if !row.missing_input_test_passed {
            return Err(anyhow!(
                "VCF bcftools adapter row `{}` failed missing-input validation: {}",
                row.stage_id,
                row.missing_input_observed_error
            ));
        }
    }
    Ok(())
}

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfBcftoolsAdapterArtifact {
    VcfBcftoolsAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<&str>,
    consumes_previous_stdout: bool,
    declared_output_artifact_ids: &[&str],
) -> VcfBcftoolsAdapterCommandStep {
    VcfBcftoolsAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv: argv.into_iter().map(str::to_string).collect(),
        consumes_previous_stdout,
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
}

fn input_by_id<'a>(inputs: &'a [VcfBcftoolsAdapterArtifact], artifact_id: &str) -> Result<&'a str> {
    inputs
        .iter()
        .find(|input| input.artifact_id == artifact_id)
        .map(|input| input.path.as_str())
        .ok_or_else(|| anyhow!("VCF bcftools adapter input `{artifact_id}` is missing"))
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_bcftools_adapter, DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH,
        VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_bcftools_adapter_tracks_governed_rows() {
        let root = repo_root();
        let report =
            render_vcf_bcftools_adapter(&root, PathBuf::from(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH))
                .expect("render VCF bcftools adapter");

        assert_eq!(report.schema_version, VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert_eq!(report.tool_id, "bcftools");
        assert_eq!(report.row_count, 10);
        assert_eq!(report.supported_row_count, 9);
        assert_eq!(report.planned_row_count, 1);
        assert_eq!(report.argv_valid_row_count, 10);
        assert_eq!(report.missing_input_test_passed_row_count, 10);
        assert_eq!(report.indexed_row_count, 9);
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call_diploid"
                && row.command_steps.iter().any(|step| {
                    step.step_id == "call_diploid"
                        && step.argv
                            == vec!["bcftools", "call", "-mv", "-Oz", "-o",
                                "benchmarks/readiness/adapters/bcftools/vcf.call_diploid/diploid_vcf.vcf.gz"]
                })
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.prepare_reference_panel"
                && row.parser_output_ids == vec!["chunks_json".to_string()]
                && row.command_steps.iter().any(|step| step.step_kind == "index")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.stats"
                && row.raw_output_ids == vec!["bcftools_stats_txt".to_string()]
                && row.parser_output_ids == vec!["stats_json".to_string()]
        }));
    }
}
