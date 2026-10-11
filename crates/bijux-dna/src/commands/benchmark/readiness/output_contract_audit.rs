use bijux_dna_api::v1::api::bench::{
    build_output_contract_tests_report, evaluate_output_contract_row, load_vcf_execution_blocker,
    OutputContractProof, OutputContractRuntimePaths,
};
pub(crate) use bijux_dna_api::v1::api::bench::{OutputContractTestRow, OutputContractTestsReport};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
};
use super::catalog::load_external_artifact_tool_contracts;
use super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use crate::commands::benchmark::local_stage_commands::{
    local_stage_plans, materialize_declared_output,
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
use crate::commands::benchmark::local_vcf_postprocess_smoke::run_local_vcf_postprocess_smoke;
use crate::commands::benchmark::local_vcf_prepare_reference_panel_smoke::run_local_vcf_prepare_reference_panel_smoke;
use crate::commands::benchmark::local_vcf_qc_smoke::run_local_vcf_qc_smoke;
use crate::commands::benchmark::local_vcf_roh_smoke::run_local_vcf_roh_smoke;
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::benchmark::local_vcf_stats_smoke::run_local_vcf_stats_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_OUTPUT_CONTRACT_TESTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/tools/output-contract-tests.json";
const PROOF_SURFACE_SHARED_STAGE_SMOKE: &str = "shared_stage_smoke";
const PROOF_SURFACE_DIRECT_TOOL_SMOKE: &str = "direct_tool_smoke";
const PROOF_SURFACE_DIRECT_DECLARED_OUTPUT_PROOF: &str = "direct_declared_output_proof";
const PROOF_SURFACE_EXTERNAL_ARTIFACT_ADMISSION: &str = "external_artifact_admission";
const PROOF_SURFACE_TOOL_CONTRACT_BLOCKED: &str = "tool_contract_planning_refusal";
const PROOF_SURFACE_PLAN_ONLY: &str = "local_ready_plan_only";
const PROOF_SURFACE_MATERIALIZATION_BLOCKED: &str = "local_stage_materialization_blocked";
const PROOF_SURFACE_MISSING_DIRECT_SMOKE: &str = "missing_direct_tool_smoke";
const PROOF_SURFACE_MISSING_TOOL_STAGE_SMOKE: &str = "missing_tool_stage_smoke";
const RUNTIME_PROOF_SURFACE_EXPECTED_RESULT_PATHS: &str = "expected_benchmark_result_paths";
const RUNTIME_PROOF_SURFACE_DECLARED_VCF_PATHS: &str = "declared_vcf_result_paths";
const DIRECT_OUTPUT_CONTRACT_PROOF_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/tools/output-contract-tests";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FastqBamStageKey {
    domain: String,
    stage_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FastqBamToolKey {
    domain: String,
    stage_id: String,
    tool_id: String,
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
    execution_blocker: Option<String>,
}

#[derive(Debug, Clone)]
struct VcfToolProof {
    proof_surface: &'static str,
    proof_path: String,
    proof_tool_id: Option<String>,
    observed_ids: BTreeSet<String>,
    observed_paths_by_id: BTreeMap<String, Vec<String>>,
    independent_execution_proven: bool,
    execution_blocker: Option<String>,
}

#[derive(Debug, Clone)]
struct RuntimePathProof {
    runtime_path_proof_surface: &'static str,
    runtime_path_proof_path: String,
    stdout_path: String,
    stderr_path: String,
    stage_result_path: String,
}

#[derive(Debug, Clone)]
struct FastqBamDeclaredOutputBundle {
    declared_ids: BTreeSet<String>,
    result_ids: Vec<String>,
}

pub(crate) fn run_render_output_contract_audit(
    args: &parse::BenchReadinessRenderOutputContractTestsArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_output_contract_audit(
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

pub(crate) fn render_output_contract_audit(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<OutputContractTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_output_contract_audit_report(repo_root, &output_path)?;
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

fn build_output_contract_audit_report(
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

    let declared_fastq_bam_output_ids = collect_declared_fastq_bam_output_ids(&output_rows);
    let fastq_bam_proofs =
        collect_fastq_bam_stage_proofs(repo_root, &declared_fastq_bam_output_ids)?;
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
    Ok(build_output_contract_tests_report(path_relative_to_repo(repo_root, output_path), rows))
}

fn collect_declared_fastq_bam_output_ids(
    rows: &[AllDomainOutputDeclarationRow],
) -> BTreeMap<FastqBamToolKey, FastqBamDeclaredOutputBundle> {
    let mut declared = BTreeMap::<FastqBamToolKey, FastqBamDeclaredOutputBundle>::new();
    for row in rows {
        if !matches!(row.domain.as_str(), "fastq" | "bam") {
            continue;
        }
        let tool_key = FastqBamToolKey {
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
        };
        let bundle = declared.entry(tool_key).or_insert_with(|| FastqBamDeclaredOutputBundle {
            declared_ids: BTreeSet::new(),
            result_ids: Vec::new(),
        });
        bundle.result_ids.push(row.result_id.clone());
        for artifact_id in row
            .raw_outputs
            .iter()
            .chain(row.normalized_metrics.iter())
            .chain(row.index_outputs.iter())
        {
            bundle.declared_ids.insert(artifact_id.clone());
        }
    }
    for bundle in declared.values_mut() {
        bundle.result_ids.sort();
        bundle.result_ids.dedup();
    }
    declared
}

fn collect_fastq_bam_stage_proofs(
    repo_root: &Path,
    declared_output_ids: &BTreeMap<FastqBamToolKey, FastqBamDeclaredOutputBundle>,
) -> Result<BTreeMap<FastqBamToolKey, LocalStageProof>> {
    let mut proofs = BTreeMap::<FastqBamToolKey, LocalStageProof>::new();
    let external_artifact_bindings = load_external_artifact_tool_contracts(repo_root)?
        .into_iter()
        .flat_map(|contract| {
            contract
                .stage_ids
                .into_iter()
                .filter(|stage_id| stage_id.starts_with("fastq.") || stage_id.starts_with("bam."))
                .map(move |stage_id| (stage_id, contract.tool_id.clone()))
        })
        .collect::<BTreeSet<_>>();
    let stage_keys = declared_output_ids
        .keys()
        .map(|key| FastqBamStageKey { domain: key.domain.clone(), stage_id: key.stage_id.clone() })
        .collect::<BTreeSet<_>>();

    for stage_key in stage_keys {
        let plans = match local_stage_output_contract_proof_plans(repo_root, &stage_key.stage_id) {
            Ok(plans) => plans,
            Err(err) => {
                for tool_key in declared_output_ids.keys().filter(|key| {
                    key.domain == stage_key.domain && key.stage_id == stage_key.stage_id
                }) {
                    if external_artifact_bindings
                        .contains(&(tool_key.stage_id.clone(), tool_key.tool_id.clone()))
                    {
                        proofs.insert(
                            tool_key.clone(),
                            external_artifact_local_stage_proof(tool_key),
                        );
                        continue;
                    }
                    let declared_bundle = declared_output_ids
                        .get(tool_key)
                        .expect("declared fastq/bam ids exist for blocked proof fallback");
                    if supports_direct_declared_output_proof(&tool_key.stage_id) {
                        proofs.insert(
                            tool_key.clone(),
                            collect_direct_declared_output_proof(
                                repo_root,
                                &tool_key.stage_id,
                                &tool_key.tool_id,
                                declared_bundle,
                            )?,
                        );
                        continue;
                    }
                    proofs.insert(
                        tool_key.clone(),
                        LocalStageProof {
                            proof_surface: PROOF_SURFACE_MATERIALIZATION_BLOCKED,
                            proof_path: format!(
                                "materialize {} / {} blocked: {:#}",
                                tool_key.stage_id, tool_key.tool_id, err
                            ),
                            proof_tool_id: Some(tool_key.tool_id.clone()),
                            observed_ids: BTreeSet::new(),
                            observed_paths_by_id: BTreeMap::new(),
                            undeclared_stage_output_ids: Vec::new(),
                            independent_execution_proven: false,
                            execution_blocker: None,
                        },
                    );
                }
                continue;
            }
        };

        let mut plans_by_tool =
            BTreeMap::<String, Vec<bijux_dna_stage_contract::StagePlanV1>>::new();
        for plan in plans {
            plans_by_tool.entry(plan.tool_id.to_string()).or_default().push(plan);
        }

        for tool_key in declared_output_ids
            .keys()
            .filter(|key| key.domain == stage_key.domain && key.stage_id == stage_key.stage_id)
        {
            if external_artifact_bindings
                .contains(&(tool_key.stage_id.clone(), tool_key.tool_id.clone()))
            {
                proofs.insert(tool_key.clone(), external_artifact_local_stage_proof(tool_key));
                continue;
            }
            let declared_bundle = declared_output_ids
                .get(tool_key)
                .expect("declared fastq/bam ids exist for collected proof");
            let declared_ids = &declared_bundle.declared_ids;
            if supports_direct_declared_output_proof(&tool_key.stage_id) {
                proofs.insert(
                    tool_key.clone(),
                    collect_direct_declared_output_proof(
                        repo_root,
                        &tool_key.stage_id,
                        &tool_key.tool_id,
                        declared_bundle,
                    )?,
                );
                continue;
            }
            if let Some(tool_plans) = plans_by_tool.get(&tool_key.tool_id) {
                proofs.insert(
                    tool_key.clone(),
                    collect_local_stage_proof_from_plans(
                        repo_root,
                        &tool_key.stage_id,
                        &tool_key.tool_id,
                        declared_ids,
                        tool_plans,
                    )?,
                );
                continue;
            }

            let stage_tool_keys = declared_output_ids
                .keys()
                .filter(|key| key.domain == stage_key.domain && key.stage_id == stage_key.stage_id)
                .cloned()
                .collect::<Vec<_>>();
            let stage_has_uniform_declared_outputs = stage_tool_keys.iter().all(|key| {
                declared_output_ids
                    .get(key)
                    .is_some_and(|candidate| candidate.declared_ids == *declared_ids)
            });
            if stage_has_uniform_declared_outputs {
                if let Some((proof_tool_id, proof_plans)) = plans_by_tool.iter().next() {
                    proofs.insert(
                        tool_key.clone(),
                        collect_local_stage_proof_from_plans(
                            repo_root,
                            &tool_key.stage_id,
                            proof_tool_id,
                            declared_ids,
                            proof_plans,
                        )?,
                    );
                    continue;
                }
            }

            if supports_direct_declared_output_proof(&tool_key.stage_id) {
                proofs.insert(
                    tool_key.clone(),
                    collect_direct_declared_output_proof(
                        repo_root,
                        &tool_key.stage_id,
                        &tool_key.tool_id,
                        declared_bundle,
                    )?,
                );
                continue;
            }

            proofs.insert(
                tool_key.clone(),
                LocalStageProof {
                    proof_surface: PROOF_SURFACE_MISSING_TOOL_STAGE_SMOKE,
                    proof_path: format!(
                        "missing local proof plan for {} / {}",
                        tool_key.stage_id, tool_key.tool_id
                    ),
                    proof_tool_id: Some(tool_key.tool_id.clone()),
                    observed_ids: BTreeSet::new(),
                    observed_paths_by_id: BTreeMap::new(),
                    undeclared_stage_output_ids: Vec::new(),
                    independent_execution_proven: false,
                    execution_blocker: None,
                },
            );
        }
    }
    Ok(proofs)
}

fn external_artifact_local_stage_proof(tool_key: &FastqBamToolKey) -> LocalStageProof {
    LocalStageProof {
        proof_surface: PROOF_SURFACE_EXTERNAL_ARTIFACT_ADMISSION,
        proof_path: "containers/contracts/catalog.toml".to_string(),
        proof_tool_id: Some(tool_key.tool_id.clone()),
        observed_ids: BTreeSet::new(),
        observed_paths_by_id: BTreeMap::new(),
        undeclared_stage_output_ids: Vec::new(),
        independent_execution_proven: false,
        execution_blocker: Some(format!(
            "blocked_external_access: `{}` / `{}` requires a checksum-admitted operator-provided artifact before output execution proof can be collected",
            tool_key.stage_id, tool_key.tool_id
        )),
    }
}

fn supports_direct_declared_output_proof(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship" | "bam.sex"
    )
}

fn local_stage_output_contract_proof_plans(
    repo_root: &Path,
    stage_id: &str,
) -> Result<Vec<bijux_dna_stage_contract::StagePlanV1>> {
    match stage_id {
        "bam.align" => {
            bijux_dna_planner_bam::stage_api::local_align_output_contract_plans(repo_root)
        }
        "bam.authenticity" => Ok(
            bijux_dna_planner_bam::stage_api::local_authenticity_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect(),
        ),
        "bam.contamination" => {
            bijux_dna_planner_bam::stage_api::local_contamination_smoke_plans(repo_root)
        }
        "bam.damage" => {
            Ok(bijux_dna_planner_bam::stage_api::local_damage_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect())
        }
        #[cfg(feature = "bam_downstream")]
        "bam.kinship" => {
            Ok(bijux_dna_planner_bam::stage_api::local_kinship_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect())
        }
        "bam.sex" => {
            Ok(bijux_dna_planner_bam::stage_api::local_sex_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect())
        }
        "fastq.index_reference" => {
            bijux_dna_planner_fastq::stage_api::local_index_reference_output_contract_plans(
                repo_root,
            )
        }
        "fastq.merge_pairs" => Ok(
            bijux_dna_planner_fastq::stage_api::local_merge_pairs_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect(),
        ),
        "fastq.profile_reads" => {
            Ok(bijux_dna_planner_fastq::stage_api::local_profile_reads_output_contract_plans(
                repo_root,
            )?
            .into_iter()
            .map(|case| case.plan)
            .collect())
        }
        "fastq.screen_taxonomy" => {
            bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_output_contract_plans(
                repo_root,
            )
        }
        "fastq.trim_reads" => Ok(
            bijux_dna_planner_fastq::stage_api::local_trim_reads_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect(),
        ),
        "fastq.trim_terminal_damage" => Ok(
            bijux_dna_planner_fastq::stage_api::local_trim_terminal_damage_output_contract_plans(
                repo_root,
            )?
            .into_iter()
            .map(|case| case.plan)
            .collect(),
        ),
        _ => local_stage_plans(repo_root, stage_id),
    }
}

fn collect_local_stage_proof_from_plans(
    repo_root: &Path,
    stage_id: &str,
    proof_tool_id: &str,
    declared_ids: &BTreeSet<String>,
    tool_plans: &[bijux_dna_stage_contract::StagePlanV1],
) -> Result<LocalStageProof> {
    let proof_root = repo_root
        .join(DIRECT_OUTPUT_CONTRACT_PROOF_ROOT)
        .join("shared-stage")
        .join(stage_id)
        .join(proof_tool_id);
    if proof_root.is_dir() {
        std::fs::remove_dir_all(&proof_root)
            .with_context(|| format!("reset proof directory {}", proof_root.display()))?;
    }

    let mut observed_ids = BTreeSet::<String>::new();
    let mut observed_paths_by_id = BTreeMap::<String, Vec<String>>::new();
    let mut missing_required_count = 0usize;
    let mut proof_path = None;
    for plan in tool_plans {
        for artifact in &plan.io.outputs {
            let declared = repo_relative_path(repo_root, &artifact.path);
            let proof_output = local_stage_proof_output_path(repo_root, &proof_root, &declared)?;
            materialize_declared_output(&proof_output)
                .with_context(|| format!("materialize proof output {}", proof_output.display()))?;
            proof_path.get_or_insert_with(|| path_relative_to_repo(repo_root, &proof_output));
            if proof_output.exists() {
                observed_ids.insert(artifact.name.to_string());
                observed_paths_by_id
                    .entry(artifact.name.to_string())
                    .or_default()
                    .push(path_relative_to_repo(repo_root, &proof_output));
            } else if !artifact.optional {
                missing_required_count += 1;
            }
        }
    }
    for paths in observed_paths_by_id.values_mut() {
        paths.sort();
        paths.dedup();
    }
    let undeclared_stage_output_ids =
        observed_ids.difference(declared_ids).cloned().collect::<Vec<_>>();
    let fixture_complete = !observed_ids.is_empty() && missing_required_count == 0;
    Ok(LocalStageProof {
        proof_surface: PROOF_SURFACE_SHARED_STAGE_SMOKE,
        proof_path: proof_path.unwrap_or_else(|| {
            format!("materialize {} / {} produced no outputs", stage_id, proof_tool_id)
        }),
        proof_tool_id: None,
        observed_ids,
        observed_paths_by_id,
        undeclared_stage_output_ids,
        independent_execution_proven: false,
        execution_blocker: fixture_complete.then(|| "synthetic_output_contract_only: declared outputs were materialized without executing the selected backend".to_string()),
    })
}

fn local_stage_proof_output_path(
    repo_root: &Path,
    proof_root: &Path,
    declared_output: &Path,
) -> Result<PathBuf> {
    let relative = declared_output.strip_prefix(repo_root).with_context(|| {
        format!("declared output must remain inside repository root: {}", declared_output.display())
    })?;
    Ok(proof_root.join(relative))
}

fn collect_direct_declared_output_proof(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    declared_bundle: &FastqBamDeclaredOutputBundle,
) -> Result<LocalStageProof> {
    let mut observed_ids = BTreeSet::<String>::new();
    let mut observed_paths_by_id = BTreeMap::<String, Vec<String>>::new();
    let proof_root = repo_root.join(DIRECT_OUTPUT_CONTRACT_PROOF_ROOT).join(stage_id).join(tool_id);
    if proof_root.is_dir() {
        std::fs::remove_dir_all(&proof_root)
            .with_context(|| format!("reset proof directory {}", proof_root.display()))?;
    }
    std::fs::create_dir_all(&proof_root)
        .with_context(|| format!("create {}", proof_root.display()))?;

    for artifact_id in &declared_bundle.declared_ids {
        let artifact_root = proof_root.join(artifact_id);
        std::fs::create_dir_all(&artifact_root)
            .with_context(|| format!("create {}", artifact_root.display()))?;
        let proof_path = artifact_root.join("proof.json");
        materialize_declared_output(&proof_path)
            .with_context(|| format!("materialize proof output {}", proof_path.display()))?;
        observed_ids.insert(artifact_id.clone());
        observed_paths_by_id
            .entry(artifact_id.clone())
            .or_default()
            .push(path_relative_to_repo(repo_root, &proof_path));
    }

    for paths in observed_paths_by_id.values_mut() {
        paths.sort();
        paths.dedup();
    }

    let result_ids = if declared_bundle.result_ids.is_empty() {
        "no result ids".to_string()
    } else {
        declared_bundle.result_ids.join(", ")
    };

    Ok(LocalStageProof {
        proof_surface: PROOF_SURFACE_DIRECT_DECLARED_OUTPUT_PROOF,
        proof_path: format!(
            "{} (result ids: {})",
            path_relative_to_repo(repo_root, &proof_root),
            result_ids
        ),
        proof_tool_id: None,
        observed_ids,
        observed_paths_by_id,
        undeclared_stage_output_ids: Vec::new(),
        independent_execution_proven: false,
        execution_blocker: Some("synthetic_output_contract_only: declared outputs were materialized without executing the selected backend".to_string()),
    })
}

fn collect_vcf_tool_proofs(
    repo_root: &Path,
    rows: &[AllDomainOutputDeclarationRow],
) -> Result<BTreeMap<String, VcfToolProof>> {
    let external_artifact_bindings = load_external_artifact_tool_contracts(repo_root)?
        .into_iter()
        .flat_map(|contract| {
            contract
                .stage_ids
                .into_iter()
                .filter(|stage_id| stage_id.starts_with("vcf."))
                .map(move |stage_id| (stage_id, contract.tool_id.clone()))
        })
        .collect::<BTreeSet<_>>();
    let mut proofs = BTreeMap::<String, VcfToolProof>::new();
    for row in rows {
        if row.domain != "vcf" {
            continue;
        }
        if proofs.contains_key(&row.result_id) {
            continue;
        }
        if external_artifact_bindings.contains(&(row.stage_id.clone(), row.tool_id.clone())) {
            proofs.insert(
                row.result_id.clone(),
                VcfToolProof {
                    proof_surface: PROOF_SURFACE_EXTERNAL_ARTIFACT_ADMISSION,
                    proof_path: "containers/contracts/catalog.toml".to_string(),
                    proof_tool_id: Some(row.tool_id.clone()),
                    observed_ids: BTreeSet::new(),
                    observed_paths_by_id: BTreeMap::new(),
                    independent_execution_proven: false,
                    execution_blocker: Some(format!(
                        "blocked_external_access: `{}` / `{}` requires a checksum-admitted operator-provided artifact before output execution proof can be collected",
                        row.stage_id, row.tool_id
                    )),
                },
            );
            continue;
        }
        let manifest_path = match local_vcf_stage_result_manifest_path(
            repo_root,
            &row.stage_id,
            &row.tool_id,
        ) {
            Ok(Some(manifest_path)) => manifest_path,
            Ok(None) => {
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
                        execution_blocker: Some(format!(
                            "missing_direct_execution: `{}` / `{}` has no tool-specific local smoke; active-default evidence is not reusable",
                            row.stage_id, row.tool_id
                        )),
                    },
                );
                continue;
            }
            Err(error) => {
                let refusal = format!("{error:#}");
                if !is_vcf_tool_contract_planning_refusal(&refusal) {
                    return Err(error).with_context(|| {
                        format!(
                            "materialize direct VCF proof for `{}` / `{}`",
                            row.stage_id, row.tool_id
                        )
                    });
                }
                proofs.insert(
                    row.result_id.clone(),
                    VcfToolProof {
                        proof_surface: PROOF_SURFACE_TOOL_CONTRACT_BLOCKED,
                        proof_path: format!(
                            "planning refused {} / {}: {refusal}",
                            row.stage_id, row.tool_id
                        ),
                        proof_tool_id: Some(row.tool_id.clone()),
                        observed_ids: BTreeSet::new(),
                        observed_paths_by_id: BTreeMap::new(),
                        independent_execution_proven: false,
                        execution_blocker: Some(format!(
                            "invalid_tool_contract: `{}` / `{}` planning refused before execution: {refusal}",
                            row.stage_id, row.tool_id
                        )),
                    },
                );
                continue;
            }
        };
        let manifest = load_validated_stage_result_manifest_path(&repo_root.join(&manifest_path))
            .with_context(|| format!("load `{manifest_path}`"))?;
        proofs.insert(
            row.result_id.clone(),
            vcf_tool_proof_from_manifest(repo_root, &manifest_path, manifest)?,
        );
    }
    Ok(proofs)
}

fn is_vcf_tool_contract_planning_refusal(message: &str) -> bool {
    message.contains("not compatible with tool") || message.contains("has no canonical binding")
}

fn vcf_tool_proof_from_manifest(
    repo_root: &Path,
    manifest_path: &str,
    manifest: BenchStageResultManifestV1,
) -> Result<VcfToolProof> {
    let execution_blocker = load_vcf_execution_blocker(repo_root, &manifest)?;
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
    Ok(VcfToolProof {
        proof_surface: PROOF_SURFACE_DIRECT_TOOL_SMOKE,
        proof_path: manifest_path.to_string(),
        proof_tool_id: Some(manifest.tool.id),
        observed_ids,
        observed_paths_by_id,
        independent_execution_proven: execution_blocker.is_none(),
        execution_blocker,
    })
}

fn build_row(
    output_row: &AllDomainOutputDeclarationRow,
    runtime_path_proof: &RuntimePathProof,
    fastq_bam_proofs: &BTreeMap<FastqBamToolKey, LocalStageProof>,
    vcf_proofs: &BTreeMap<String, VcfToolProof>,
) -> Result<OutputContractTestRow> {
    let tool_key = FastqBamToolKey {
        domain: output_row.domain.clone(),
        stage_id: output_row.stage_id.clone(),
        tool_id: output_row.tool_id.clone(),
    };

    let (
        output_proof_surface,
        output_proof_path,
        proof_tool_id,
        observed_ids,
        observed_paths_by_id,
        stage_undeclared_output_ids,
        independent_execution_proven,
        execution_blocker,
    ) = match output_row.domain.as_str() {
        "fastq" | "bam" => {
            let proof = fastq_bam_proofs.get(&tool_key).ok_or_else(|| {
                anyhow!(
                    "output contract tests are missing local stage proof for `{}` / `{}`",
                    output_row.stage_id,
                    output_row.tool_id,
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
                proof.execution_blocker.clone(),
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
                proof.execution_blocker.clone(),
            )
        }
        other => {
            return Err(anyhow!(
                "output contract tests do not support unexpected domain `{other}`"
            ));
        }
    };

    Ok(evaluate_output_contract_row(
        output_row,
        &OutputContractRuntimePaths {
            runtime_path_proof_surface: runtime_path_proof.runtime_path_proof_surface.to_string(),
            runtime_path_proof_path: runtime_path_proof.runtime_path_proof_path.clone(),
            stdout_path: runtime_path_proof.stdout_path.clone(),
            stderr_path: runtime_path_proof.stderr_path.clone(),
            stage_result_path: runtime_path_proof.stage_result_path.clone(),
        },
        OutputContractProof {
            output_proof_surface,
            output_proof_path,
            proof_tool_id,
            observed_ids,
            observed_paths_by_id,
            stage_undeclared_output_ids,
            independent_execution_proven,
            execution_blocker,
        },
    ))
}

fn expected_runtime_path_proof_for_row(
    row: &super::expected_benchmark_results::ExpectedBenchmarkResultRow,
) -> RuntimePathProof {
    RuntimePathProof {
        runtime_path_proof_surface: RUNTIME_PROOF_SURFACE_EXPECTED_RESULT_PATHS,
        runtime_path_proof_path: row.stage_result_manifest_path.clone(),
        stdout_path: row.stdout_path.clone(),
        stderr_path: row.stderr_path.clone(),
        stage_result_path: row.stage_result_manifest_path.clone(),
    }
}

fn fallback_runtime_path_proof_for_row(row: &AllDomainOutputDeclarationRow) -> RuntimePathProof {
    let (stdout_path, stderr_path) = declared_log_paths(row);
    RuntimePathProof {
        runtime_path_proof_surface: RUNTIME_PROOF_SURFACE_DECLARED_VCF_PATHS,
        runtime_path_proof_path: row.manifest.clone(),
        stdout_path,
        stderr_path,
        stage_result_path: row.manifest.clone(),
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

fn local_vcf_stage_result_manifest_path(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
) -> Result<Option<String>> {
    let default_tool_id = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == stage_id)
        .map(|row| row.tool_id)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{stage_id}`"))?;
    let stage_worker_owns_tool_selection = matches!(
        stage_id,
        "vcf.admixture"
            | "vcf.ibd"
            | "vcf.imputation_metrics"
            | "vcf.impute"
            | "vcf.pca"
            | "vcf.phasing"
            | "vcf.population_structure"
            | "vcf.qc"
    );
    if tool_id != default_tool_id && !stage_worker_owns_tool_selection {
        return Ok(None);
    }
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
        "vcf.postprocess" => {
            Some(run_local_vcf_postprocess_smoke(repo_root, tool_id)?.stage_result_manifest_path)
        }
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
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        build_output_contract_audit_report, is_vcf_tool_contract_planning_refusal,
        load_validated_stage_result_manifest_path, local_stage_proof_output_path,
        local_vcf_stage_result_manifest_path, DEFAULT_OUTPUT_CONTRACT_TESTS_PATH,
    };
    use crate::commands::benchmark::repo_locking::acquire_cwd_lock;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    fn benchmark_readiness_test_lock(root: &Path) -> bijux_dna_infra::FileLock {
        bijux_dna_infra::FileLock::try_acquire(
            &root.join("artifacts/test-locks/benchmark-readiness-mutators.lock"),
        )
        .expect("acquire benchmark readiness test lock")
    }

    #[test]
    fn local_stage_proof_output_path_isolated_from_plan_evidence() {
        let root = tempfile::tempdir().expect("tempdir");
        let stage_root =
            root.path().join("artifacts/benchmark-smoke/readiness/local-ready/bam.contamination");
        fs::create_dir_all(&stage_root).expect("create stage root");
        let plan_path = stage_root.join("plan.json");
        let output_path = stage_root.join("summary.json");
        fs::write(&plan_path, b"governed plan").expect("write plan");
        fs::write(&output_path, b"stale output").expect("write output");
        let proof_root = root
            .path()
            .join("artifacts/benchmark-smoke/simulated-runs/readiness-probes/output-contracts");

        let proof_output = local_stage_proof_output_path(root.path(), &proof_root, &output_path)
            .expect("derive isolated proof output");
        fs::create_dir_all(proof_output.parent().expect("proof output parent"))
            .expect("create proof output parent");
        fs::write(&proof_output, b"proof output").expect("materialize proof output");

        assert!(plan_path.is_file(), "sibling plan evidence must remain untouched");
        assert_eq!(fs::read(&output_path).expect("read declared output"), b"stale output");
        assert!(proof_output.starts_with(proof_root));
    }

    #[test]
    fn vcf_contract_refusal_classifier_does_not_hide_execution_failures() {
        assert!(is_vcf_tool_contract_planning_refusal(
            "map governed-map not compatible with tool beagle"
        ));
        assert!(is_vcf_tool_contract_planning_refusal(
            "VCF panel workflow smoke has no canonical binding for vcf.phasing / unknown"
        ));
        assert!(!is_vcf_tool_contract_planning_refusal(
            "permission denied while materializing direct VCF proof"
        ));
        assert!(!is_vcf_tool_contract_planning_refusal("stage worker exited 70"));
    }

    #[test]
    fn vcf_admixture_stage_worker_proves_each_governed_tool_independently() {
        let root = repo_root();
        let proof = local_vcf_stage_result_manifest_path(&root, "vcf.admixture", "plink")
            .expect("resolve non-default VCF tool proof")
            .expect("VCF admixture worker must own the governed PLINK backend");
        let manifest = load_validated_stage_result_manifest_path(&root.join(proof))
            .expect("load PLINK VCF admixture stage result");

        assert_eq!(manifest.stage_id, "vcf.admixture");
        assert_eq!(manifest.tool.id, "plink");
    }

    #[test]
    fn vcf_qc_stage_worker_proves_each_governed_tool_independently() {
        let root = tempfile::tempdir().expect("tempdir");
        for tool_id in ["bcftools", "plink", "plink2"] {
            let proof = local_vcf_stage_result_manifest_path(root.path(), "vcf.qc", tool_id)
                .expect("resolve VCF QC tool proof")
                .expect("VCF QC worker must own every governed tool");
            let manifest = load_validated_stage_result_manifest_path(&root.path().join(proof))
                .expect("load VCF QC stage result");
            assert_eq!(manifest.stage_id, "vcf.qc");
            assert_eq!(manifest.tool.id, tool_id);
        }
    }

    #[test]
    fn vcf_ibd_and_pca_workers_preserve_retained_tool_selection() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _lock = benchmark_readiness_test_lock(&root);
        for (stage_id, tool_id) in
            [("vcf.ibd", "ibdseq"), ("vcf.ibd", "ibdhap"), ("vcf.pca", "eigensoft")]
        {
            let proof = local_vcf_stage_result_manifest_path(&root, stage_id, tool_id)
                .expect("resolve retained VCF tool proof")
                .expect("stage worker must own retained tool selection");
            let manifest = load_validated_stage_result_manifest_path(&root.join(proof))
                .expect("load retained VCF tool stage result");
            assert_eq!(manifest.stage_id, stage_id);
            assert_eq!(manifest.tool.id, tool_id);
        }
    }

    #[test]
    fn vcf_phasing_worker_executes_compatible_backends_and_refuses_invalid_maps() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _lock = benchmark_readiness_test_lock(&root);
        for tool_id in ["shapeit", "shapeit5"] {
            let proof = local_vcf_stage_result_manifest_path(&root, "vcf.phasing", tool_id)
                .expect("resolve retained phasing proof")
                .expect("phasing worker must own compatible retained backend");
            let manifest = load_validated_stage_result_manifest_path(&root.join(proof))
                .expect("load retained phasing stage result");
            assert_eq!(manifest.stage_id, "vcf.phasing");
            assert_eq!(manifest.tool.id, tool_id);
        }
        for tool_id in ["beagle", "eagle"] {
            let error = local_vcf_stage_result_manifest_path(&root, "vcf.phasing", tool_id)
                .expect_err("incompatible map contract must fail during planning");
            assert!(error.to_string().contains("map hsapiens_grch38_glimpse_chr22"));
            assert!(error.to_string().contains(&format!("not compatible with tool {tool_id}")));
        }
    }

    #[test]
    fn vcf_imputation_workers_preserve_glimpse_and_derived_minimac_panel() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _lock = benchmark_readiness_test_lock(&root);
        for stage_id in ["vcf.impute", "vcf.imputation_metrics"] {
            for tool_id in ["glimpse", "minimac4"] {
                let proof = local_vcf_stage_result_manifest_path(&root, stage_id, tool_id)
                    .expect("resolve retained imputation proof")
                    .expect("imputation worker must own compatible retained backend");
                let manifest = load_validated_stage_result_manifest_path(&root.join(proof))
                    .expect("load retained imputation stage result");
                assert_eq!(manifest.stage_id, stage_id);
                assert_eq!(manifest.tool.id, tool_id);
            }
        }
    }

    #[test]
    fn build_output_contract_audit_report_records_governed_proof_surfaces() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _lock = benchmark_readiness_test_lock(&root);
        let report = build_output_contract_audit_report(
            &root,
            &root.join(DEFAULT_OUTPUT_CONTRACT_TESTS_PATH),
        )
        .expect("build output contract tests report");

        assert_eq!(report.schema_version, "bijux.bench.readiness.output_contract_audit.v1");
        assert_eq!(report.output_path, DEFAULT_OUTPUT_CONTRACT_TESTS_PATH);
        assert_eq!(report.row_count, 159);
        assert!(report
            .rows
            .iter()
            .filter(|row| row.passed)
            .all(|row| { row.independent_execution_proven && row.execution_blocker.is_none() }));
        assert!(
            report.output_proof_surface_counts.contains_key("shared_stage_smoke"),
            "FASTQ/BAM shared-stage smoke proof must be represented"
        );
        assert!(
            report.output_proof_surface_counts.contains_key("direct_tool_smoke"),
            "VCF direct tool smoke proof must be represented"
        );
        assert!(
            report.output_proof_surface_counts.contains_key("direct_declared_output_proof"),
            "BAM downstream bindings must retain direct declared-output proof coverage"
        );

        let fastq_index_reference = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.index_reference" && row.tool_id == "bowtie2_build")
            .expect("fastq.index_reference row");

        let vcf_postprocess = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.postprocess" && row.tool_id == "bcftools")
            .expect("vcf.postprocess row");
        assert_eq!(vcf_postprocess.output_proof_surface, "direct_tool_smoke");
        assert!(!vcf_postprocess.independent_execution_proven);
        assert!(!vcf_postprocess.passed);
        assert!(vcf_postprocess
            .execution_blocker
            .as_deref()
            .is_some_and(|reason| { reason.starts_with("unverified_execution_report:") }));
        for tool_id in ["eigensoft", "plink", "plink2"] {
            let row = report
                .rows
                .iter()
                .find(|row| row.stage_id == "vcf.population_structure" && row.tool_id == tool_id)
                .unwrap_or_else(|| panic!("vcf.population_structure / {tool_id} row"));
            assert_eq!(row.output_proof_surface, "direct_tool_smoke");
            assert_eq!(row.proof_tool_id.as_deref(), Some(tool_id));
            if row.passed {
                assert!(row.independent_execution_proven);
                assert!(row.execution_blocker.is_none());
            } else if row.independent_execution_proven {
                assert!(row.execution_blocker.is_none());
                assert!(
                    !row.raw_outputs_matched
                        || !row.normalized_metrics_matched
                        || !row.index_outputs_matched
                        || !row.logs_matched
                        || !row.manifest_matched
                        || !row.no_undeclared_outputs
                );
            } else {
                assert!(row.execution_blocker.as_deref().is_some_and(|blocker| {
                    blocker.contains(&format!("selected VCF tool `{tool_id}`"))
                        && blocker.contains("population-structure component plan")
                }));
            }
        }

        let bam_genotyping = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.genotyping" && row.tool_id == "bcftools")
            .expect("bam.genotyping row");
        assert_eq!(report.failed_row_count, 0);
        assert_eq!(report.passed_row_count + report.blocked_row_count, report.row_count);
        let expected_blocked_tool_ids = report
            .rows
            .iter()
            .filter(|row| !row.passed && row.execution_blocker.is_some())
            .map(|row| row.tool_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(report.blocked_tool_ids, expected_blocked_tool_ids);
        assert!(report
            .rows
            .iter()
            .filter(|row| !row.passed)
            .all(|row| { !row.independent_execution_proven && row.execution_blocker.is_some() }));
        assert!(report.failed_stage_ids.is_empty());
        assert!(report.failed_tool_ids.is_empty());
        let external_artifact_rows = report
            .rows
            .iter()
            .filter(|row| row.output_proof_surface == "external_artifact_admission")
            .collect::<Vec<_>>();
        assert_eq!(external_artifact_rows.len(), 8);
        assert!(external_artifact_rows.iter().all(|row| {
            !row.passed
                && !row.independent_execution_proven
                && row
                    .execution_blocker
                    .as_deref()
                    .is_some_and(|blocker| blocker.starts_with("blocked_external_access:"))
        }));
        assert!(
            !report.output_proof_surface_counts.contains_key("local_stage_materialization_blocked"),
            "governed proof runs should not leave blocked FASTQ/BAM materialization rows"
        );
        assert_eq!(fastq_index_reference.output_proof_surface, "shared_stage_smoke");
        assert!(!fastq_index_reference.independent_execution_proven);
        assert!(!fastq_index_reference.passed);
        assert_eq!(bam_genotyping.output_proof_surface, "direct_declared_output_proof");
        assert!(!bam_genotyping.independent_execution_proven);
        assert!(!bam_genotyping.passed);
        for synthetic in [fastq_index_reference, bam_genotyping] {
            assert!(synthetic.proof_tool_id.is_none());
            assert!(synthetic
                .execution_blocker
                .as_deref()
                .is_some_and(|reason| { reason.starts_with("synthetic_output_contract_only:") }));
        }
    }
}
