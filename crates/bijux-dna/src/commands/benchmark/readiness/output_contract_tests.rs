use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus,
};
use super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use crate::commands::benchmark::local_stage_commands::{
    local_stage_plans, materialize_local_stage,
};
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, BenchStageResultManifestV1,
};
use crate::commands::benchmark::local_vcf_admixture_smoke::run_local_vcf_admixture_smoke;
use crate::commands::benchmark::local_vcf_call_diploid_smoke::run_local_vcf_call_diploid_smoke;
use crate::commands::benchmark::local_vcf_call_gl_smoke::run_local_vcf_call_gl_smoke;
use crate::commands::benchmark::local_vcf_call_pseudohaploid_smoke::run_local_vcf_call_pseudohaploid_smoke;
use crate::commands::benchmark::local_vcf_call_smoke::run_local_vcf_call_smoke;
use crate::commands::benchmark::local_vcf_damage_filter_smoke::run_local_vcf_damage_filter_smoke;
use crate::commands::benchmark::local_vcf_demography_smoke::run_local_vcf_demography_smoke;
use crate::commands::benchmark::local_vcf_filter_smoke::run_local_vcf_filter_smoke;
use crate::commands::benchmark::local_vcf_gl_propagation_smoke::run_local_vcf_gl_propagation_smoke;
use crate::commands::benchmark::local_vcf_ibd_smoke::run_local_vcf_ibd_smoke;
use crate::commands::benchmark::local_vcf_imputation_metrics_smoke::run_local_vcf_imputation_metrics_smoke;
use crate::commands::benchmark::local_vcf_impute_smoke::run_local_vcf_impute_smoke;
use crate::commands::benchmark::local_vcf_pca_smoke::run_local_vcf_pca_smoke;
use crate::commands::benchmark::local_vcf_phasing_smoke::run_local_vcf_phasing_smoke;
use crate::commands::benchmark::local_vcf_population_structure_smoke::run_local_vcf_population_structure_smoke;
use crate::commands::benchmark::local_vcf_prepare_reference_panel_smoke::run_local_vcf_prepare_reference_panel_smoke;
use crate::commands::benchmark::local_vcf_qc_smoke::run_local_vcf_qc_smoke;
use crate::commands::benchmark::local_vcf_roh_smoke::run_local_vcf_roh_smoke;
use crate::commands::benchmark::local_vcf_stats_smoke::run_local_vcf_stats_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_OUTPUT_CONTRACT_TESTS_PATH: &str =
    "benchmarks/readiness/tools/output-contract-tests.json";
const OUTPUT_CONTRACT_TESTS_SCHEMA_VERSION: &str = "bijux.bench.readiness.output_contract_tests.v1";
const PROOF_SURFACE_SHARED_STAGE_SMOKE: &str = "shared_stage_smoke";
const PROOF_SURFACE_DIRECT_TOOL_SMOKE: &str = "direct_tool_smoke";
const PROOF_SURFACE_PLAN_ONLY: &str = "local_ready_plan_only";
const PROOF_SURFACE_MATERIALIZATION_BLOCKED: &str = "local_stage_materialization_blocked";
const PROOF_SURFACE_MISSING_DIRECT_SMOKE: &str = "missing_direct_tool_smoke";
const RUNTIME_PROOF_SURFACE_EXPECTED_RESULT_PATHS: &str = "expected_benchmark_result_paths";
const RUNTIME_PROOF_SURFACE_DECLARED_VCF_PATHS: &str = "declared_vcf_result_paths";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OutputContractTestRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) output_proof_surface: String,
    pub(crate) output_proof_path: String,
    pub(crate) proof_tool_id: Option<String>,
    pub(crate) runtime_path_proof_surface: String,
    pub(crate) runtime_path_proof_path: String,
    pub(crate) declared_raw_output_ids: Vec<String>,
    pub(crate) observed_raw_output_ids: Vec<String>,
    pub(crate) declared_normalized_metric_ids: Vec<String>,
    pub(crate) observed_normalized_metric_ids: Vec<String>,
    pub(crate) declared_index_output_ids: Vec<String>,
    pub(crate) observed_index_output_ids: Vec<String>,
    pub(crate) observed_output_paths: Vec<String>,
    pub(crate) stage_undeclared_output_ids: Vec<String>,
    pub(crate) declared_logs: Vec<String>,
    pub(crate) observed_logs: Vec<String>,
    pub(crate) declared_manifest: String,
    pub(crate) observed_manifest: String,
    pub(crate) raw_outputs_matched: bool,
    pub(crate) normalized_metrics_matched: bool,
    pub(crate) index_outputs_matched: bool,
    pub(crate) logs_matched: bool,
    pub(crate) manifest_matched: bool,
    pub(crate) no_undeclared_outputs: bool,
    pub(crate) independent_execution_proven: bool,
    pub(crate) passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OutputContractTestsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) passed_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) output_proof_surface_counts: BTreeMap<String, usize>,
    pub(crate) failed_stage_ids: Vec<String>,
    pub(crate) failed_tool_ids: Vec<String>,
    pub(crate) rows: Vec<OutputContractTestRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StageKey {
    domain: String,
    stage_id: String,
}

#[derive(Debug, Clone)]
struct LocalStageProof {
    proof_surface: &'static str,
    proof_path: String,
    proof_tool_id: Option<String>,
    observed_ids: BTreeSet<String>,
    observed_paths_by_id: BTreeMap<String, Vec<String>>,
    undeclared_stage_output_ids: Vec<String>,
    independent_execution_proven: bool,
}

#[derive(Debug, Clone)]
struct VcfToolProof {
    proof_surface: &'static str,
    proof_path: String,
    proof_tool_id: Option<String>,
    observed_ids: BTreeSet<String>,
    observed_paths_by_id: BTreeMap<String, Vec<String>>,
    independent_execution_proven: bool,
}

#[derive(Debug, Clone)]
struct RuntimePathProof {
    runtime_path_proof_surface: &'static str,
    runtime_path_proof_path: String,
    stdout_path: String,
    stderr_path: String,
    stage_result_path: String,
    output_ids: BTreeSet<String>,
}

pub(crate) fn run_render_output_contract_tests(
    args: &parse::BenchReadinessRenderOutputContractTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_output_contract_tests(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_CONTRACT_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_output_contract_tests(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<OutputContractTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_output_contract_tests_report(repo_root, &output_path)?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if report.failed_row_count != 0 {
        return Err(anyhow!(
            "retained-tool output contract tests found {} failed rows; independent execution proof is still missing for at least one active binding",
            report.failed_row_count
        ));
    }
    Ok(report)
}

fn build_output_contract_tests_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<OutputContractTestsReport> {
    let output_rows = collect_all_domain_output_declaration_rows(repo_root)?;
    let mut runtime_path_proofs = output_rows
        .iter()
        .map(|row| (row.result_id.clone(), fallback_runtime_path_proof_for_row(row)))
        .collect::<BTreeMap<_, _>>();
    for expected_row in collect_expected_benchmark_result_rows(repo_root)? {
        runtime_path_proofs.insert(
            expected_row.result_row_id.clone(),
            expected_runtime_path_proof_for_row(&expected_row),
        );
    }

    let declared_stage_output_ids = collect_declared_stage_output_ids(&output_rows);
    let fastq_bam_proofs = collect_fastq_bam_stage_proofs(repo_root, &declared_stage_output_ids)?;
    let vcf_proofs = collect_vcf_tool_proofs(repo_root, &output_rows)?;

    let mut rows = Vec::with_capacity(output_rows.len());
    for output_row in output_rows {
        let runtime_path_proof =
            runtime_path_proofs.get(&output_row.result_id).ok_or_else(|| {
                anyhow!(
                    "output contract tests are missing runtime-path coverage for `{}`",
                    output_row.result_id
                )
            })?;
        rows.push(build_row(&output_row, runtime_path_proof, &fastq_bam_proofs, &vcf_proofs)?);
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut output_proof_surface_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *output_proof_surface_counts.entry(row.output_proof_surface.clone()).or_default() += 1;
    }
    let failed_stage_ids = rows
        .iter()
        .filter(|row| !row.passed)
        .map(|row| row.stage_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let failed_tool_ids = rows
        .iter()
        .filter(|row| !row.passed)
        .map(|row| row.tool_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Ok(OutputContractTestsReport {
        schema_version: OUTPUT_CONTRACT_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        domain_counts,
        output_proof_surface_counts,
        failed_stage_ids,
        failed_tool_ids,
        rows,
    })
}

fn collect_declared_stage_output_ids(
    rows: &[AllDomainOutputDeclarationRow],
) -> BTreeMap<StageKey, BTreeSet<String>> {
    let mut declared = BTreeMap::<StageKey, BTreeSet<String>>::new();
    for row in rows {
        let stage_key = StageKey { domain: row.domain.clone(), stage_id: row.stage_id.clone() };
        let stage_ids = declared.entry(stage_key).or_default();
        for artifact_id in row
            .raw_outputs
            .iter()
            .chain(row.normalized_metrics.iter())
            .chain(row.index_outputs.iter())
        {
            stage_ids.insert(artifact_id.clone());
        }
    }
    declared
}

fn collect_fastq_bam_stage_proofs(
    repo_root: &Path,
    declared_stage_output_ids: &BTreeMap<StageKey, BTreeSet<String>>,
) -> Result<BTreeMap<StageKey, LocalStageProof>> {
    let mut proofs = BTreeMap::<StageKey, LocalStageProof>::new();
    for stage_key in declared_stage_output_ids.keys() {
        if !matches!(stage_key.domain.as_str(), "fastq" | "bam") {
            continue;
        }
        let proof_path = match materialize_local_stage(repo_root, &stage_key.stage_id) {
            Ok(proof_path) => proof_path,
            Err(err) => {
                proofs.insert(
                    stage_key.clone(),
                    LocalStageProof {
                        proof_surface: PROOF_SURFACE_MATERIALIZATION_BLOCKED,
                        proof_path: format!(
                            "materialize {} blocked: {:#}",
                            stage_key.stage_id, err
                        ),
                        proof_tool_id: None,
                        observed_ids: BTreeSet::new(),
                        observed_paths_by_id: BTreeMap::new(),
                        undeclared_stage_output_ids: Vec::new(),
                        independent_execution_proven: false,
                    },
                );
                continue;
            }
        };
        let plans = local_stage_plans(repo_root, &stage_key.stage_id)
            .with_context(|| format!("collect local stage plans for `{}`", stage_key.stage_id))?;
        let Some(first_plan) = plans.first() else {
            return Err(anyhow!(
                "local stage proof for `{}` yielded no governed stage plans",
                stage_key.stage_id
            ));
        };

        let mut observed_ids = BTreeSet::<String>::new();
        let mut observed_paths_by_id = BTreeMap::<String, Vec<String>>::new();
        let mut missing_required_count = 0usize;
        for plan in &plans {
            for artifact in &plan.io.outputs {
                let resolved = repo_relative_path(repo_root, &artifact.path);
                if resolved.exists() {
                    observed_ids.insert(artifact.name.to_string());
                    observed_paths_by_id
                        .entry(artifact.name.to_string())
                        .or_default()
                        .push(path_relative_to_repo(repo_root, &resolved));
                } else if !artifact.optional {
                    missing_required_count += 1;
                }
            }
        }
        for paths in observed_paths_by_id.values_mut() {
            paths.sort();
            paths.dedup();
        }

        let proof_surface = if proof_path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.ends_with("plan.json"))
        {
            PROOF_SURFACE_PLAN_ONLY
        } else {
            PROOF_SURFACE_SHARED_STAGE_SMOKE
        };
        let undeclared_stage_output_ids = observed_ids
            .difference(
                declared_stage_output_ids
                    .get(stage_key)
                    .expect("declared stage ids exist for collected proof"),
            )
            .cloned()
            .collect::<Vec<_>>();
        let independent_execution_proven =
            proof_surface == PROOF_SURFACE_SHARED_STAGE_SMOKE && missing_required_count == 0;

        proofs.insert(
            stage_key.clone(),
            LocalStageProof {
                proof_surface,
                proof_path: path_relative_to_repo(repo_root, &proof_path),
                proof_tool_id: Some(first_plan.tool_id.to_string()),
                observed_ids,
                observed_paths_by_id,
                undeclared_stage_output_ids,
                independent_execution_proven,
            },
        );
    }
    Ok(proofs)
}

fn collect_vcf_tool_proofs(
    repo_root: &Path,
    rows: &[AllDomainOutputDeclarationRow],
) -> Result<BTreeMap<String, VcfToolProof>> {
    let mut proofs = BTreeMap::<String, VcfToolProof>::new();
    for row in rows {
        if row.domain != "vcf" {
            continue;
        }
        if proofs.contains_key(&row.result_id) {
            continue;
        }
        let Some(manifest_path) =
            local_vcf_stage_result_manifest_path(repo_root, &row.stage_id, &row.tool_id)?
        else {
            proofs.insert(
                row.result_id.clone(),
                VcfToolProof {
                    proof_surface: PROOF_SURFACE_MISSING_DIRECT_SMOKE,
                    proof_path: format!(
                        "missing direct smoke for {} / {}",
                        row.stage_id, row.tool_id
                    ),
                    proof_tool_id: Some(row.tool_id.clone()),
                    observed_ids: BTreeSet::new(),
                    observed_paths_by_id: BTreeMap::new(),
                    independent_execution_proven: false,
                },
            );
            continue;
        };
        let manifest = load_validated_stage_result_manifest_path(&repo_root.join(&manifest_path))
            .with_context(|| format!("load `{manifest_path}`"))?;
        proofs
            .insert(row.result_id.clone(), vcf_tool_proof_from_manifest(&manifest_path, manifest));
    }
    Ok(proofs)
}

fn vcf_tool_proof_from_manifest(
    manifest_path: &str,
    manifest: BenchStageResultManifestV1,
) -> VcfToolProof {
    let mut observed_ids = BTreeSet::<String>::new();
    let mut observed_paths_by_id = BTreeMap::<String, Vec<String>>::new();
    for output in manifest.outputs {
        if output.exists {
            observed_ids.insert(output.artifact_id.clone());
            observed_paths_by_id.entry(output.artifact_id).or_default().push(output.realized_path);
        }
    }
    for paths in observed_paths_by_id.values_mut() {
        paths.sort();
        paths.dedup();
    }
    VcfToolProof {
        proof_surface: PROOF_SURFACE_DIRECT_TOOL_SMOKE,
        proof_path: manifest_path.to_string(),
        proof_tool_id: Some(manifest.tool.id),
        observed_ids,
        observed_paths_by_id,
        independent_execution_proven: true,
    }
}

fn build_row(
    output_row: &AllDomainOutputDeclarationRow,
    runtime_path_proof: &RuntimePathProof,
    fastq_bam_proofs: &BTreeMap<StageKey, LocalStageProof>,
    vcf_proofs: &BTreeMap<String, VcfToolProof>,
) -> Result<OutputContractTestRow> {
    let stage_key =
        StageKey { domain: output_row.domain.clone(), stage_id: output_row.stage_id.clone() };
    let declared_raw = output_row.raw_outputs.clone();
    let declared_norm = output_row.normalized_metrics.clone();
    let declared_index = output_row.index_outputs.clone();

    let (
        output_proof_surface,
        output_proof_path,
        proof_tool_id,
        observed_ids,
        observed_paths_by_id,
        stage_undeclared_output_ids,
        independent_execution_proven,
    ) = match output_row.domain.as_str() {
        "fastq" | "bam" => {
            let proof = fastq_bam_proofs.get(&stage_key).ok_or_else(|| {
                anyhow!(
                    "output contract tests are missing local stage proof for `{}`",
                    output_row.stage_id
                )
            })?;
            (
                proof.proof_surface.to_string(),
                proof.proof_path.clone(),
                proof.proof_tool_id.clone(),
                proof.observed_ids.clone(),
                proof.observed_paths_by_id.clone(),
                proof.undeclared_stage_output_ids.clone(),
                proof.independent_execution_proven,
            )
        }
        "vcf" => {
            let proof = vcf_proofs.get(&output_row.result_id).ok_or_else(|| {
                anyhow!(
                    "output contract tests are missing VCF proof for `{}`",
                    output_row.result_id
                )
            })?;
            (
                proof.proof_surface.to_string(),
                proof.proof_path.clone(),
                proof.proof_tool_id.clone(),
                proof.observed_ids.clone(),
                proof.observed_paths_by_id.clone(),
                Vec::new(),
                proof.independent_execution_proven,
            )
        }
        other => {
            return Err(anyhow!(
                "output contract tests do not support unexpected domain `{other}`"
            ));
        }
    };

    let declared_union = declared_raw
        .iter()
        .chain(declared_norm.iter())
        .chain(declared_index.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let runtime_layout_output_ids_matched = runtime_path_proof.output_ids == declared_union;

    let raw_outputs_matched = declared_raw.iter().all(|id| observed_ids.contains(id));
    let normalized_metrics_matched = declared_norm.iter().all(|id| observed_ids.contains(id));
    let index_outputs_matched = declared_index.iter().all(|id| observed_ids.contains(id));
    let logs_matched = string_set(&output_row.logs)
        == string_set(&[
            format!("stdout={}", runtime_path_proof.stdout_path),
            format!("stderr={}", runtime_path_proof.stderr_path),
        ]);
    let manifest_matched = output_row.manifest == runtime_path_proof.stage_result_path;
    let no_undeclared_outputs =
        stage_undeclared_output_ids.is_empty() && runtime_layout_output_ids_matched;

    let observed_raw_output_ids = filter_declared_ids(&observed_ids, &declared_raw);
    let observed_normalized_metric_ids = filter_declared_ids(&observed_ids, &declared_norm);
    let observed_index_output_ids = filter_declared_ids(&observed_ids, &declared_index);
    let observed_output_paths = declared_raw
        .iter()
        .chain(declared_norm.iter())
        .chain(declared_index.iter())
        .flat_map(|artifact_id| {
            observed_paths_by_id.get(artifact_id).cloned().unwrap_or_default().into_iter()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let observed_logs = vec![
        format!("stdout={}", runtime_path_proof.stdout_path),
        format!("stderr={}", runtime_path_proof.stderr_path),
    ];
    let observed_manifest = runtime_path_proof.stage_result_path.clone();
    let passed = output_row.status == AllDomainOutputDeclarationStatus::Complete
        && independent_execution_proven
        && raw_outputs_matched
        && normalized_metrics_matched
        && index_outputs_matched
        && logs_matched
        && manifest_matched
        && no_undeclared_outputs;

    let reason = if passed {
        format!(
            "binding `{}` / `{}` keeps governed raw outputs, normalized metrics, index outputs, logs, and manifest paths under `{}` plus `{}`",
            output_row.stage_id,
            output_row.tool_id,
            output_proof_surface,
            runtime_path_proof.runtime_path_proof_surface
        )
    } else {
        let mut failures = Vec::<String>::new();
        if output_row.status != AllDomainOutputDeclarationStatus::Complete {
            failures.push("output declaration is not complete".to_string());
        }
        if !independent_execution_proven {
            failures.push(format!(
                "independent execution proof is missing from `{}`",
                output_proof_surface
            ));
        }
        if !raw_outputs_matched {
            failures.push("declared raw outputs are not fully observed".to_string());
        }
        if !normalized_metrics_matched {
            failures.push("declared normalized metrics are not fully observed".to_string());
        }
        if !index_outputs_matched {
            failures.push("declared index outputs are not fully observed".to_string());
        }
        if !logs_matched {
            failures.push(
                "declared stdout/stderr paths drifted from the fake-run runtime paths".to_string(),
            );
        }
        if !manifest_matched {
            failures.push(
                "declared stage-result manifest path drifted from the fake-run runtime path"
                    .to_string(),
            );
        }
        if !no_undeclared_outputs {
            failures.push("observed outputs include undeclared artifact ids".to_string());
        }
        failures.join("; ")
    };

    Ok(OutputContractTestRow {
        result_id: output_row.result_id.clone(),
        domain: output_row.domain.clone(),
        stage_id: output_row.stage_id.clone(),
        tool_id: output_row.tool_id.clone(),
        corpus_id: output_row.corpus_id.clone(),
        asset_profile_id: output_row.asset_profile_id.clone(),
        output_proof_surface,
        output_proof_path,
        proof_tool_id,
        runtime_path_proof_surface: runtime_path_proof.runtime_path_proof_surface.to_string(),
        runtime_path_proof_path: runtime_path_proof.runtime_path_proof_path.clone(),
        declared_raw_output_ids: declared_raw,
        observed_raw_output_ids,
        declared_normalized_metric_ids: declared_norm,
        observed_normalized_metric_ids,
        declared_index_output_ids: declared_index,
        observed_index_output_ids,
        observed_output_paths,
        stage_undeclared_output_ids,
        declared_logs: output_row.logs.clone(),
        observed_logs,
        declared_manifest: output_row.manifest.clone(),
        observed_manifest,
        raw_outputs_matched,
        normalized_metrics_matched,
        index_outputs_matched,
        logs_matched,
        manifest_matched,
        no_undeclared_outputs,
        independent_execution_proven,
        passed,
        reason,
    })
}

fn expected_runtime_path_proof_for_row(
    row: &super::expected_benchmark_results::ExpectedBenchmarkResultRow,
) -> RuntimePathProof {
    let output_ids = row.expected_output_artifact_ids.iter().cloned().collect::<BTreeSet<_>>();
    RuntimePathProof {
        runtime_path_proof_surface: RUNTIME_PROOF_SURFACE_EXPECTED_RESULT_PATHS,
        runtime_path_proof_path: row.stage_result_manifest_path.clone(),
        stdout_path: row.stdout_path.clone(),
        stderr_path: row.stderr_path.clone(),
        stage_result_path: row.stage_result_manifest_path.clone(),
        output_ids,
    }
}

fn fallback_runtime_path_proof_for_row(row: &AllDomainOutputDeclarationRow) -> RuntimePathProof {
    let output_ids = row
        .raw_outputs
        .iter()
        .chain(row.normalized_metrics.iter())
        .chain(row.index_outputs.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let (stdout_path, stderr_path) = declared_log_paths(row);
    RuntimePathProof {
        runtime_path_proof_surface: RUNTIME_PROOF_SURFACE_DECLARED_VCF_PATHS,
        runtime_path_proof_path: row.manifest.clone(),
        stdout_path,
        stderr_path,
        stage_result_path: row.manifest.clone(),
        output_ids,
    }
}

fn declared_log_paths(row: &AllDomainOutputDeclarationRow) -> (String, String) {
    let stdout_path = row
        .logs
        .iter()
        .find_map(|entry| entry.strip_prefix("stdout=").map(ToString::to_string))
        .unwrap_or_default();
    let stderr_path = row
        .logs
        .iter()
        .find_map(|entry| entry.strip_prefix("stderr=").map(ToString::to_string))
        .unwrap_or_default();
    (stdout_path, stderr_path)
}

fn filter_declared_ids(observed_ids: &BTreeSet<String>, declared_ids: &[String]) -> Vec<String> {
    declared_ids
        .iter()
        .filter(|artifact_id| observed_ids.contains(*artifact_id))
        .cloned()
        .collect::<Vec<_>>()
}

fn string_set(values: &[String]) -> BTreeSet<String> {
    values.iter().cloned().collect()
}

fn local_vcf_stage_result_manifest_path(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
) -> Result<Option<String>> {
    Ok(match stage_id {
        "vcf.admixture" => {
            Some(run_local_vcf_admixture_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.call" => {
            Some(run_local_vcf_call_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.call_diploid" => {
            Some(run_local_vcf_call_diploid_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.call_gl" => {
            Some(run_local_vcf_call_gl_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.call_pseudohaploid" => Some(
            run_local_vcf_call_pseudohaploid_smoke(repo_root, tool_id)?.stage_result_manifest_path,
        ),
        "vcf.damage_filter" => {
            Some(run_local_vcf_damage_filter_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.demography" => {
            Some(run_local_vcf_demography_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.filter" => {
            Some(run_local_vcf_filter_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.gl_propagation" => {
            Some(run_local_vcf_gl_propagation_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.ibd" => Some(run_local_vcf_ibd_smoke(repo_root, tool_id)?.stage_result_manifest_path),
        "vcf.imputation_metrics" => Some(
            run_local_vcf_imputation_metrics_smoke(repo_root, tool_id)?.stage_result_manifest_path,
        ),
        "vcf.impute" => {
            Some(run_local_vcf_impute_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.pca" => Some(run_local_vcf_pca_smoke(repo_root, tool_id)?.stage_result_manifest_path),
        "vcf.phasing" => {
            Some(run_local_vcf_phasing_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.population_structure" => Some(
            run_local_vcf_population_structure_smoke(repo_root, tool_id)?
                .stage_result_manifest_path,
        ),
        "vcf.prepare_reference_panel" => Some(
            run_local_vcf_prepare_reference_panel_smoke(repo_root, tool_id)?
                .stage_result_manifest_path,
        ),
        "vcf.qc" => Some(run_local_vcf_qc_smoke(repo_root, tool_id)?.stage_result_manifest_path),
        "vcf.roh" => Some(run_local_vcf_roh_smoke(repo_root, tool_id)?.stage_result_manifest_path),
        "vcf.stats" => {
            Some(run_local_vcf_stats_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
        "vcf.postprocess" => None,
        other => {
            return Err(anyhow!(
                "output contract tests do not support unexpected VCF stage `{other}`"
            ));
        }
    })
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

    use super::{build_output_contract_tests_report, DEFAULT_OUTPUT_CONTRACT_TESTS_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn build_output_contract_tests_report_records_real_and_blocked_rows() {
        let root = repo_root();
        let report = build_output_contract_tests_report(
            &root,
            &root.join(DEFAULT_OUTPUT_CONTRACT_TESTS_PATH),
        )
        .expect("build output contract tests report");

        assert_eq!(report.schema_version, "bijux.bench.readiness.output_contract_tests.v1");
        assert_eq!(report.output_path, DEFAULT_OUTPUT_CONTRACT_TESTS_PATH);
        assert_eq!(report.row_count, 140);
        assert!(
            report.failed_row_count > 0,
            "current governed matrix should still expose blocked plan-only or missing direct-smoke rows"
        );
        assert!(report.passed_row_count > 0, "report should keep real passing rows");
        assert!(
            report.output_proof_surface_counts.contains_key("shared_stage_smoke"),
            "FASTQ/BAM shared-stage smoke proof must be represented"
        );
        assert!(
            report.output_proof_surface_counts.contains_key("direct_tool_smoke"),
            "VCF direct tool smoke proof must be represented"
        );
        assert!(
            report.output_proof_surface_counts.contains_key("local_stage_materialization_blocked"),
            "blocked FASTQ/BAM materialization surfaces must stay visible"
        );

        assert!(
            report
                .rows
                .iter()
                .any(|row| row.output_proof_surface == "local_stage_materialization_blocked"),
            "at least one retained binding should currently expose a blocked local-stage materialization proof"
        );

        let fastq_index_reference = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.index_reference" && row.tool_id == "bowtie2_build")
            .expect("fastq.index_reference row");
        assert_eq!(fastq_index_reference.output_proof_surface, "local_ready_plan_only");
        assert!(!fastq_index_reference.independent_execution_proven);
        assert!(!fastq_index_reference.passed);
        assert!(
            fastq_index_reference.reason.contains("independent execution proof is missing"),
            "plan-only stages must fail with an explicit blocker reason"
        );

        let vcf_postprocess = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.postprocess" && row.tool_id == "bcftools")
            .expect("vcf.postprocess row");
        assert_eq!(vcf_postprocess.output_proof_surface, "missing_direct_tool_smoke");
        assert!(!vcf_postprocess.independent_execution_proven);
        assert!(!vcf_postprocess.passed);
    }
}
