use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{vcf_domain_stage_adapter_id, vcf_domain_stage_parser_id};
use serde::Serialize;

use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ANGSD_ADAPTER_PATH: &str =
    "target/bench-readiness/adapters/angsd.vcf.json";
const VCF_ANGSD_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_angsd_adapter.v1";
const GOVERNED_ANGSD_TOOL_ID: &str = "angsd";
const GOVERNED_ANGSD_TOOL_STATUS: &str = "planned";
const GOVERNED_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_BAM_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam";
const GOVERNED_REFERENCE_FASTA_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";
const GOVERNED_SITES_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf";
const GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAngsdAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAngsdAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAngsdAdapterRow {
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) stage_id: String,
    pub(crate) stage_status: String,
    pub(crate) benchmark_status: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_contract_source: String,
    pub(crate) output_root: String,
    pub(crate) output_prefix: String,
    pub(crate) bam_member_paths: Vec<String>,
    pub(crate) bam_list_path: Option<String>,
    pub(crate) reference_path: Option<String>,
    pub(crate) sites_path: Option<String>,
    pub(crate) likelihood_model: Option<String>,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfAngsdAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfAngsdAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfAngsdAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfAngsdAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) tool_id: &'static str,
    pub(crate) tool_status: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) supported_stage_row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) argv_valid_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) bam_list_row_count: usize,
    pub(crate) parser_output_row_count: usize,
    pub(crate) rows: Vec<VcfAngsdAdapterRow>,
}

pub(crate) fn run_render_vcf_angsd_adapter(
    args: &parse::BenchReadinessRenderVcfAngsdAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_angsd_adapter(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ANGSD_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_angsd_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAngsdAdapterReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_angsd_adapter_rows(repo_root)?;
    let supported_stage_row_count =
        rows.iter().filter(|row| row.stage_status == "supported").count();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let argv_valid_row_count = rows.iter().filter(|row| row.argv_validation_passed).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();
    let bam_list_row_count = rows.iter().filter(|row| row.bam_list_path.is_some()).count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::to_string_pretty(&VcfAngsdAdapterReport {
        schema_version: VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: GOVERNED_ANGSD_TOOL_ID,
        tool_status: GOVERNED_ANGSD_TOOL_STATUS,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        supported_stage_row_count,
        benchmark_ready_row_count,
        argv_valid_row_count,
        missing_input_test_passed_row_count,
        bam_list_row_count,
        parser_output_row_count,
        rows: rows.clone(),
    })
    .context("render VCF angsd adapter report")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;

    Ok(VcfAngsdAdapterReport {
        schema_version: VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: GOVERNED_ANGSD_TOOL_ID,
        tool_status: GOVERNED_ANGSD_TOOL_STATUS,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        supported_stage_row_count,
        benchmark_ready_row_count,
        argv_valid_row_count,
        missing_input_test_passed_row_count,
        bam_list_row_count,
        parser_output_row_count,
        rows,
    })
}

pub(crate) fn collect_vcf_angsd_adapter_rows(repo_root: &Path) -> Result<Vec<VcfAngsdAdapterRow>> {
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let benchmark_ready_stage_ids = build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.tool_id == GOVERNED_ANGSD_TOOL_ID)
        .map(|row| row.stage_id)
        .collect::<BTreeSet<_>>();
    let admitted_stage_ids = load_admitted_angsd_stage_ids(repo_root)?;

    let mut rows = Vec::new();
    for stage_id in admitted_stage_ids {
        let catalog_row = catalog_by_stage.get(stage_id.as_str()).ok_or_else(|| {
            anyhow!("VCF angsd adapter report is missing catalog coverage for `{stage_id}`")
        })?;
        rows.push(build_angsd_row(
            repo_root,
            &stage_id,
            catalog_row,
            benchmark_ready_stage_ids.contains(stage_id.as_str()),
        )?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_angsd_adapter_contract(&rows)?;
    Ok(rows)
}

fn build_angsd_row(
    repo_root: &Path,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    benchmark_ready: bool,
) -> Result<VcfAngsdAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let adapter_id = vcf_domain_stage_adapter_id(stage)
        .ok_or_else(|| anyhow!("VCF angsd adapter row `{stage_id}` is missing adapter id"))?;
    let parser_id = vcf_domain_stage_parser_id(stage)
        .ok_or_else(|| anyhow!("VCF angsd adapter row `{stage_id}` is missing parser id"))?;
    let output_root =
        format!("target/bench-readiness/adapters/{}/{}", GOVERNED_ANGSD_TOOL_ID, stage_id);
    let bam_member_paths =
        if uses_bam_inputs(stage) { vec![GOVERNED_BAM_PATH.to_string()] } else { Vec::new() };
    let bam_list_path = if uses_bam_inputs(stage) {
        Some(materialize_bam_list(repo_root, &output_root, &bam_member_paths)?)
    } else {
        None
    };
    let required_inputs = governed_inputs_for_stage(stage, bam_list_path.as_deref());
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let (
        command_contract_source,
        reference_path,
        sites_path,
        likelihood_model,
        output_prefix,
        stage_output_ids,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    ) = build_stage_adapter_contract(stage, &output_root, &required_inputs)?;
    let argv_validation_passed = validate_command_steps(stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, stage_id, &required_inputs);

    let reason = format!(
        "row `{stage_id}` / `{}` renders concrete angsd argv with {} declared command step(s), {} required input(s), {} raw output id(s), and {} parser output id(s)",
        GOVERNED_ANGSD_TOOL_ID,
        command_steps.len(),
        required_inputs.len(),
        raw_output_ids.len(),
        parser_output_ids.len(),
    );

    Ok(VcfAngsdAdapterRow {
        tool_id: GOVERNED_ANGSD_TOOL_ID.to_string(),
        tool_status: GOVERNED_ANGSD_TOOL_STATUS.to_string(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status: if benchmark_ready {
            "benchmark_ready".to_string()
        } else {
            "not_benchmark_ready".to_string()
        },
        adapter_id: adapter_id.to_string(),
        parser_id: parser_id.to_string(),
        corpus_id: GOVERNED_CORPUS_ID.to_string(),
        asset_profile_id: asset_profile_id_for_stage(stage).to_string(),
        command_contract_source,
        output_root,
        output_prefix,
        bam_member_paths,
        bam_list_path,
        reference_path,
        sites_path,
        likelihood_model,
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

fn load_admitted_angsd_stage_ids(repo_root: &Path) -> Result<Vec<String>> {
    let registry_path = repo_root.join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let parsed: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parse {}", registry_path.display()))?;
    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing tools in {}", registry_path.display()))?;
    let angsd = tools
        .iter()
        .find(|entry| {
            entry
                .get("tool_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|tool_id| tool_id == GOVERNED_ANGSD_TOOL_ID)
        })
        .ok_or_else(|| anyhow!("missing angsd VCF registry row in {}", registry_path.display()))?;
    let stage_ids = angsd
        .get("stage_ids")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("angsd VCF registry row is missing stage_ids"))?
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    Ok(stage_ids)
}

fn governed_inputs_for_stage(
    stage: VcfDomainStage,
    bam_list_path: Option<&str>,
) -> Vec<VcfAngsdAdapterArtifact> {
    match stage {
        VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid
        | VcfDomainStage::DamageFilter => vec![
            artifact("bam_list", "bam_list", bam_list_path.expect("bam list path")),
            artifact("reference_fasta", "reference", GOVERNED_REFERENCE_FASTA_PATH),
            artifact("sites_vcf", "sites_vcf", GOVERNED_SITES_VCF_PATH),
        ],
        VcfDomainStage::GlPropagation => vec![
            artifact("vcf", "variant", GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH),
            artifact("sites_vcf", "sites_vcf", GOVERNED_SITES_VCF_PATH),
        ],
        other => {
            panic!("VCF angsd adapter does not govern stage `{}`", other.as_str());
        }
    }
}

#[allow(clippy::type_complexity)]
fn build_stage_adapter_contract(
    stage: VcfDomainStage,
    output_root: &str,
    inputs: &[VcfAngsdAdapterArtifact],
) -> Result<(
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<VcfAngsdAdapterArtifact>,
    Vec<VcfAngsdAdapterCommandStep>,
)> {
    let vcf_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
    let json_output_path = |name: &str| format!("{output_root}/{name}.json");
    let text_output_path = |name: &str| format!("{output_root}/{name}.txt");
    let arg_output_path = |prefix: &str| format!("{prefix}.arg");

    let contract = match stage {
        VcfDomainStage::CallGl => {
            let prefix = format!("{output_root}/gl_sites");
            (
                "domain/vcf/fixtures/vcf.call_gl/angsd.txt".to_string(),
                Some(input_by_id(inputs, "reference_fasta")?.to_string()),
                Some(input_by_id(inputs, "sites_vcf")?.to_string()),
                Some("GL2_doGlf2_doMajorMinor1_doMaf1".to_string()),
                prefix.clone(),
                vec!["gl_sites_vcf".to_string(), "gl_provenance".to_string()],
                vec!["gl_sites_vcf".to_string(), "angsd_arg".to_string()],
                vec!["gl_provenance".to_string()],
                vec![
                    artifact("gl_sites_vcf", "variant", &vcf_output_path("gl_sites")),
                    artifact("angsd_arg", "report", &arg_output_path(&prefix)),
                    artifact("gl_provenance", "report_json", &json_output_path("gl_provenance")),
                ],
                vec![step(
                    "call_gl",
                    "call",
                    vec![
                        "angsd",
                        "-b",
                        input_by_id(inputs, "bam_list")?,
                        "-ref",
                        input_by_id(inputs, "reference_fasta")?,
                        "-sites",
                        input_by_id(inputs, "sites_vcf")?,
                        "-GL",
                        "2",
                        "-doGlf",
                        "2",
                        "-doMajorMinor",
                        "1",
                        "-doMaf",
                        "1",
                        "-minMapQ",
                        "30",
                        "-minQ",
                        "20",
                        "-setMinDepth",
                        "3",
                        "-setMaxDepth",
                        "100",
                        "-doVcf",
                        "1",
                        "-out",
                        &prefix,
                    ],
                    &["gl_sites_vcf", "angsd_arg"],
                )],
            )
        }
        VcfDomainStage::CallPseudohaploid => {
            let prefix = format!("{output_root}/pseudo");
            (
                "domain/vcf/fixtures/vcf.call_pseudohaploid/angsd.txt".to_string(),
                Some(input_by_id(inputs, "reference_fasta")?.to_string()),
                Some(input_by_id(inputs, "sites_vcf")?.to_string()),
                Some("doHaploCall1_seed42".to_string()),
                prefix.clone(),
                vec!["pseudohaploid_vcf".to_string(), "pseudohaploid_sampling_log".to_string()],
                vec!["pseudohaploid_vcf".to_string(), "angsd_arg".to_string()],
                vec!["pseudohaploid_sampling_log".to_string()],
                vec![
                    artifact("pseudohaploid_vcf", "variant", &vcf_output_path("pseudo")),
                    artifact("angsd_arg", "report", &arg_output_path(&prefix)),
                    artifact(
                        "pseudohaploid_sampling_log",
                        "report_json",
                        &json_output_path("pseudohaploid_sampling_log"),
                    ),
                ],
                vec![step(
                    "call_pseudohaploid",
                    "call",
                    vec![
                        "angsd",
                        "-b",
                        input_by_id(inputs, "bam_list")?,
                        "-ref",
                        input_by_id(inputs, "reference_fasta")?,
                        "-sites",
                        input_by_id(inputs, "sites_vcf")?,
                        "-doHaploCall",
                        "1",
                        "-doCounts",
                        "1",
                        "-seed",
                        "42",
                        "-doVcf",
                        "1",
                        "-out",
                        &prefix,
                    ],
                    &["pseudohaploid_vcf", "angsd_arg"],
                )],
            )
        }
        VcfDomainStage::DamageFilter => {
            let prefix = format!("{output_root}/damage");
            (
                "domain/vcf/fixtures/vcf.damage_filter/angsd.txt".to_string(),
                Some(input_by_id(inputs, "reference_fasta")?.to_string()),
                Some(input_by_id(inputs, "sites_vcf")?.to_string()),
                Some("doDamage1_pmd1".to_string()),
                prefix.clone(),
                vec!["damage_report_txt".to_string(), "damage_bias_audit_report".to_string()],
                vec!["damage_report_txt".to_string(), "angsd_arg".to_string()],
                vec!["damage_bias_audit_report".to_string()],
                vec![
                    artifact("damage_report_txt", "report", &text_output_path("damage_report")),
                    artifact("angsd_arg", "report", &arg_output_path(&prefix)),
                    artifact(
                        "damage_bias_audit_report",
                        "report_json",
                        &json_output_path("damage_bias_audit_report"),
                    ),
                ],
                vec![step(
                    "damage_filter",
                    "damage_aware",
                    vec![
                        "angsd",
                        "-b",
                        input_by_id(inputs, "bam_list")?,
                        "-ref",
                        input_by_id(inputs, "reference_fasta")?,
                        "-sites",
                        input_by_id(inputs, "sites_vcf")?,
                        "-out",
                        &prefix,
                        "-doDamage",
                        "1",
                        "-minMapQ",
                        "30",
                        "-minQ",
                        "20",
                        "-pmd",
                        "1",
                    ],
                    &["damage_report_txt", "angsd_arg"],
                )],
            )
        }
        VcfDomainStage::GlPropagation => {
            let prefix = format!("{output_root}/gl_propagation");
            (
                "registry:configs/ci/registry/tool_registry_vcf_downstream.toml".to_string(),
                None,
                Some(input_by_id(inputs, "sites_vcf")?.to_string()),
                Some("vcf_gl_input_doPost1_doVcf1".to_string()),
                prefix.clone(),
                vec!["gl_propagated_vcf".to_string(), "gl_propagation_report".to_string()],
                vec!["gl_propagated_vcf".to_string(), "angsd_arg".to_string()],
                vec!["gl_propagation_report".to_string()],
                vec![
                    artifact("gl_propagated_vcf", "variant", &vcf_output_path("gl_propagation")),
                    artifact("angsd_arg", "report", &arg_output_path(&prefix)),
                    artifact(
                        "gl_propagation_report",
                        "report_json",
                        &json_output_path("gl_propagation_report"),
                    ),
                ],
                vec![step(
                    "gl_propagation",
                    "normalize_gl",
                    vec![
                        "angsd",
                        "-vcf-gl",
                        input_by_id(inputs, "vcf")?,
                        "-sites",
                        input_by_id(inputs, "sites_vcf")?,
                        "-doMajorMinor",
                        "1",
                        "-doMaf",
                        "1",
                        "-doPost",
                        "1",
                        "-doVcf",
                        "1",
                        "-out",
                        &prefix,
                    ],
                    &["gl_propagated_vcf", "angsd_arg"],
                )],
            )
        }
        other => {
            return Err(anyhow!(
                "VCF angsd adapter does not govern stage `{}` for Goal 236",
                other.as_str()
            ));
        }
    };

    Ok(contract)
}

fn asset_profile_id_for_stage(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid
        | VcfDomainStage::DamageFilter => "bam_bundle",
        VcfDomainStage::GlPropagation => "vcf_single_sample",
        other => panic!("unsupported angsd asset profile stage `{}`", other.as_str()),
    }
}

fn uses_bam_inputs(stage: VcfDomainStage) -> bool {
    matches!(
        stage,
        VcfDomainStage::CallGl | VcfDomainStage::CallPseudohaploid | VcfDomainStage::DamageFilter
    )
}

fn materialize_bam_list(
    repo_root: &Path,
    output_root: &str,
    bam_member_paths: &[String],
) -> Result<String> {
    let bam_list_path = repo_root.join(output_root).join("angsd-inputs.bam.list");
    if let Some(parent) = bam_list_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = format!("{}\n", bam_member_paths.join("\n"));
    bijux_dna_infra::atomic_write_bytes(&bam_list_path, payload.as_bytes())?;
    Ok(path_relative_to_repo(repo_root, &bam_list_path))
}

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfAngsdAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF angsd adapter for `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            ));
        }
    }
    Ok(())
}

fn validate_command_steps(stage_id: &str, steps: &[VcfAngsdAdapterCommandStep]) -> Result<()> {
    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF angsd adapter step `{}` for `{stage_id}` has empty argv",
                step.step_id
            ));
        }
        if step.argv[0] != GOVERNED_ANGSD_TOOL_ID {
            return Err(anyhow!(
                "VCF angsd adapter step `{}` for `{stage_id}` must execute `{}` first, found `{}`",
                step.step_id,
                GOVERNED_ANGSD_TOOL_ID,
                step.argv[0]
            ));
        }
        if step.argv.iter().any(|part| {
            let lowered = part.to_ascii_lowercase();
            lowered.contains("placeholder") || lowered == "--help" || lowered.contains("todo")
        }) {
            return Err(anyhow!(
                "VCF angsd adapter step `{}` for `{stage_id}` still contains placeholder argv: {:?}",
                step.step_id,
                step.argv
            ));
        }
        if !step.argv.iter().any(|part| part == "-out") {
            return Err(anyhow!(
                "VCF angsd adapter step `{}` for `{stage_id}` is missing `-out`",
                step.step_id
            ));
        }
    }

    let argv = &steps[0].argv;
    match stage_id {
        "vcf.call_gl" => {
            ensure_arg_present(stage_id, argv, "-b")?;
            ensure_arg_present(stage_id, argv, "-ref")?;
            ensure_arg_present(stage_id, argv, "-sites")?;
            ensure_arg_present(stage_id, argv, "-GL")?;
            ensure_arg_present(stage_id, argv, "-doGlf")?;
        }
        "vcf.call_pseudohaploid" => {
            ensure_arg_present(stage_id, argv, "-b")?;
            ensure_arg_present(stage_id, argv, "-ref")?;
            ensure_arg_present(stage_id, argv, "-sites")?;
            ensure_arg_present(stage_id, argv, "-doHaploCall")?;
            ensure_arg_present(stage_id, argv, "-seed")?;
        }
        "vcf.damage_filter" => {
            ensure_arg_present(stage_id, argv, "-b")?;
            ensure_arg_present(stage_id, argv, "-ref")?;
            ensure_arg_present(stage_id, argv, "-sites")?;
            ensure_arg_present(stage_id, argv, "-doDamage")?;
        }
        "vcf.gl_propagation" => {
            ensure_arg_present(stage_id, argv, "-vcf-gl")?;
            ensure_arg_present(stage_id, argv, "-sites")?;
            ensure_arg_present(stage_id, argv, "-doPost")?;
            ensure_arg_present(stage_id, argv, "-doVcf")?;
        }
        _ => {}
    }
    Ok(())
}

fn ensure_arg_present(stage_id: &str, argv: &[String], needle: &str) -> Result<()> {
    if argv.iter().any(|part| part == needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "VCF angsd adapter row `{stage_id}` is missing required argv flag `{needle}`: {argv:?}"
        ))
    }
}

fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfAngsdAdapterArtifact],
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
            "VCF angsd adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

fn ensure_vcf_angsd_adapter_contract(rows: &[VcfAngsdAdapterRow]) -> Result<()> {
    if rows.len() != 4 {
        return Err(anyhow!(
            "VCF angsd adapter must cover exactly 4 admitted registry rows, found {}",
            rows.len()
        ));
    }
    let expected_stages = BTreeSet::from([
        "vcf.call_gl",
        "vcf.call_pseudohaploid",
        "vcf.damage_filter",
        "vcf.gl_propagation",
    ]);
    let observed_stages = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed_stages != expected_stages {
        return Err(anyhow!(
            "VCF angsd adapter stage set drifted: expected {:?}, found {:?}",
            expected_stages,
            observed_stages
        ));
    }
    for row in rows {
        if row.tool_status != GOVERNED_ANGSD_TOOL_STATUS {
            return Err(anyhow!(
                "VCF angsd adapter row `{}` drifted from planned tool status: {}",
                row.stage_id,
                row.tool_status
            ));
        }
        if !row.argv_validation_passed {
            return Err(anyhow!("VCF angsd adapter row `{}` failed argv validation", row.stage_id));
        }
        if !row.missing_input_test_passed {
            return Err(anyhow!(
                "VCF angsd adapter row `{}` failed missing-input validation: {}",
                row.stage_id,
                row.missing_input_observed_error
            ));
        }
    }
    Ok(())
}

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfAngsdAdapterArtifact {
    VcfAngsdAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<&str>,
    declared_output_artifact_ids: &[&str],
) -> VcfAngsdAdapterCommandStep {
    VcfAngsdAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv: argv.into_iter().map(str::to_string).collect(),
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
}

fn input_by_id<'a>(inputs: &'a [VcfAngsdAdapterArtifact], artifact_id: &str) -> Result<&'a str> {
    inputs
        .iter()
        .find(|input| input.artifact_id == artifact_id)
        .map(|input| input.path.as_str())
        .ok_or_else(|| anyhow!("VCF angsd adapter input `{artifact_id}` is missing"))
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
        render_vcf_angsd_adapter, DEFAULT_VCF_ANGSD_ADAPTER_PATH, VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_angsd_adapter_tracks_governed_rows() {
        let repo_root = repo_root();
        let report =
            render_vcf_angsd_adapter(&repo_root, PathBuf::from(DEFAULT_VCF_ANGSD_ADAPTER_PATH))
                .expect("render VCF angsd adapter");

        assert_eq!(report.schema_version, VCF_ANGSD_ADAPTER_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert_eq!(report.tool_id, "angsd");
        assert_eq!(report.tool_status, "planned");
        assert_eq!(report.row_count, 4);
        assert_eq!(report.supported_stage_row_count, 4);
        assert_eq!(report.benchmark_ready_row_count, 0);
        assert_eq!(report.argv_valid_row_count, 4);
        assert_eq!(report.missing_input_test_passed_row_count, 4);
        assert_eq!(report.bam_list_row_count, 3);
        assert_eq!(report.parser_output_row_count, 4);

        let call_gl =
            report.rows.iter().find(|row| row.stage_id == "vcf.call_gl").expect("call_gl row");
        assert_eq!(call_gl.command_steps.len(), 1);
        assert_eq!(call_gl.command_steps[0].argv[0], "angsd");
        assert!(
            call_gl.command_steps[0].argv.iter().any(|part| part == "-GL"),
            "call_gl row must keep ANGSD genotype-likelihood flags"
        );

        let gl_propagation = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.gl_propagation")
            .expect("gl_propagation row");
        assert_eq!(gl_propagation.bam_list_path, None);
        assert!(
            gl_propagation.command_steps[0].argv.iter().any(|part| part == "-vcf-gl"),
            "gl_propagation row must keep the ANGSD VCF-GL input contract"
        );
    }
}
