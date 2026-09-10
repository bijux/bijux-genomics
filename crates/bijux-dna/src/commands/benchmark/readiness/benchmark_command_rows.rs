use bijux_dna_planner_fastq::decode_local_benchmark_stage_parameters as fastq_stage_params_from_plan;
use bijux_dna_planner_fastq::project_local_benchmark_tool_parameters as project_fastq_benchmark_params_for_tool;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ArtifactRole;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::StagePlanV1;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use crate::commands::benchmark::local_stage_commands::rendered_stage_materialize_argv;
use crate::commands::benchmark::local_stage_commands::{
    collect_local_stage_plan_bundles, local_stage_plans,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};

pub(crate) const DEFAULT_STAGE_TOOL_COMMAND_ROOT: &str =
    "artifacts/benchmark-smoke/readiness/stage-tool-commands";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BenchmarkCommandRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone)]
struct CanonicalStagePlanCandidates {
    readiness_kind: LocalStageReadinessKind,
    plans: Vec<StagePlanV1>,
}

pub(crate) fn collect_benchmark_command_rows(repo_root: &Path) -> Result<Vec<BenchmarkCommandRow>> {
    let fastq_base_plans = canonical_stage_plan_map(repo_root, BenchLocalDomain::Fastq)?;
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;

    let mut rows = Vec::new();

    for row in fastq_rows
        .into_iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
    {
        let base = fastq_base_plans.get(&row.stage_id).ok_or_else(|| {
            anyhow!(
                "missing canonical FASTQ local stage plan for benchmark-ready row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        let argv = render_fastq_stage_tool_argv_from_candidates(
            repo_root,
            &row.stage_id,
            &row.tool_id,
            &base.plans,
        )?;
        rows.push(BenchmarkCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: readiness_kind_label(base.readiness_kind).to_string(),
            argv,
        });
    }

    rows.extend(collect_bam_benchmark_command_rows(repo_root)?);

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_rows(&rows)?;
    Ok(rows)
}

pub(crate) fn collect_bam_benchmark_command_rows(
    repo_root: &Path,
) -> Result<Vec<BenchmarkCommandRow>> {
    let bam_readiness_by_stage = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind))
        .collect::<BTreeMap<_, _>>();
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let mut rows = Vec::new();

    for row in bam_rows
        .into_iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
    {
        let (readiness_kind, argv) = if stage_uses_materialize_stage_fallback(&row.stage_id) {
            (
                bam_readiness_by_stage
                    .get(&row.stage_id)
                    .copied()
                    .unwrap_or(LocalStageReadinessKind::Smoke),
                rendered_stage_materialize_argv(&row.stage_id),
            )
        } else {
            let base = local_stage_plans(repo_root, &row.stage_id)?.into_iter().next().ok_or_else(
                || {
                    anyhow!(
                        "local benchmark BAM stage `{}` did not yield any governed plans",
                        row.stage_id
                    )
                },
            )?;
            let readiness_kind = bam_readiness_by_stage.get(&row.stage_id).copied().ok_or_else(
                || {
                    anyhow!(
                        "missing canonical BAM local stage inventory row for benchmark-ready row `{}` / `{}`",
                        row.stage_id,
                        row.tool_id
                    )
                },
            )?;
            (
                readiness_kind,
                render_bam_stage_tool_argv(repo_root, &row.stage_id, &row.tool_id, &base)?,
            )
        };
        rows.push(BenchmarkCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: readiness_kind_label(readiness_kind).to_string(),
            argv,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_rows(&rows)?;
    Ok(rows)
}

pub(crate) fn collect_selected_fastq_command_rows(
    repo_root: &Path,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BenchmarkCommandRow>> {
    collect_selected_domain_command_rows(repo_root, BenchLocalDomain::Fastq, stage_ids)
}

pub(crate) fn collect_selected_bam_command_rows(
    repo_root: &Path,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BenchmarkCommandRow>> {
    collect_selected_domain_command_rows(repo_root, BenchLocalDomain::Bam, stage_ids)
}

pub(crate) fn render_shell_command(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
            {
                arg.clone()
            } else {
                let escaped = arg.replace('\'', "'\"'\"'");
                format!("'{escaped}'")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn canonical_stage_plan_map(
    repo_root: &Path,
    domain: BenchLocalDomain,
) -> Result<BTreeMap<String, CanonicalStagePlanCandidates>> {
    collect_local_stage_plan_bundles(repo_root, Some(domain))?
        .into_iter()
        .map(|bundle| {
            if bundle.plans.is_empty() {
                return Err(anyhow!(
                    "local stage bundle `{}` did not include any governed plans",
                    bundle.stage_id
                ));
            }
            Ok((
                bundle.stage_id,
                CanonicalStagePlanCandidates {
                    readiness_kind: bundle.readiness_kind,
                    plans: bundle.plans,
                },
            ))
        })
        .collect()
}

fn render_fastq_stage_tool_argv_from_candidates(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    candidates: &[StagePlanV1],
) -> Result<Vec<String>> {
    let mut failures = Vec::new();
    for candidate in candidates {
        match render_fastq_stage_tool_argv(repo_root, stage_id, tool_id, candidate) {
            Ok(argv) => return Ok(argv),
            Err(error) => failures.push(error.to_string()),
        }
    }
    Err(anyhow!(
        "no governed `{stage_id}` case is compatible with tool `{tool_id}`: {}",
        failures.join("; ")
    ))
}

fn collect_selected_domain_command_rows(
    repo_root: &Path,
    domain: BenchLocalDomain,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BenchmarkCommandRow>> {
    let readiness_by_stage = load_local_stage_inventory(repo_root, domain)?
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind))
        .collect::<BTreeMap<_, _>>();
    let mut rows = BTreeMap::new();

    for stage_id in stage_ids {
        let Some(readiness_kind) = readiness_by_stage.get(stage_id).copied() else {
            return Err(anyhow!(
                "missing canonical {} local stage inventory row for `{stage_id}`",
                domain_label(domain)
            ));
        };
        let plan = local_stage_plans(repo_root, stage_id)?.into_iter().next().ok_or_else(|| {
            anyhow!(
                "local benchmark stage `{stage_id}` did not yield any governed {} plans",
                domain_label(domain)
            )
        })?;
        let tool_id = plan.tool_id.as_str().to_string();
        let argv = match domain {
            BenchLocalDomain::Fastq => {
                render_fastq_stage_tool_argv(repo_root, stage_id, &tool_id, &plan)?
            }
            BenchLocalDomain::Bam => {
                render_bam_stage_tool_argv(repo_root, stage_id, &tool_id, &plan)?
            }
            BenchLocalDomain::Vcf => {
                return Err(anyhow!(
                    "benchmark command rows do not render VCF plans through the FASTQ/BAM command adapter path"
                ));
            }
        };
        rows.insert(
            stage_id.clone(),
            BenchmarkCommandRow {
                stage_id: stage_id.clone(),
                tool_id,
                readiness_kind: readiness_kind_label(readiness_kind).to_string(),
                argv,
            },
        );
    }

    Ok(rows)
}

fn stage_uses_materialize_stage_fallback(stage_id: &str) -> bool {
    matches!(stage_id, "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship")
}

fn render_fastq_stage_tool_argv(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    base_plan: &StagePlanV1,
) -> Result<Vec<String>> {
    let stage_id_value = StageId::new(stage_id.to_string());
    let tool_id_value = ToolId::new(tool_id.to_string());
    let tool = bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_execution_spec(
        repo_root,
        &stage_id_value,
        &tool_id_value,
    )
    .with_context(|| format!("load FASTQ execution spec for `{stage_id}` / `{tool_id}`"))?;
    if stage_id == "fastq.report_qc" {
        let qc_inputs = base_plan
            .io
            .inputs
            .iter()
            .map(|artifact| {
                let mut artifact = artifact.clone();
                artifact.path = resolve_repo_input_path(repo_root, &artifact.path);
                artifact
            })
            .collect::<Vec<_>>();
        let paired_mode = if find_fastq_input(base_plan, "reads_r2").is_some() {
            bijux_dna_domain_fastq::params::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::params::PairedMode::SingleEnd
        };
        let report_qc_params = match fastq_stage_params_from_plan(stage_id, base_plan)? {
            Some(bijux_dna_planner_fastq::FastqStageParameters::ReportQc(params)) => params,
            _ => bijux_dna_domain_fastq::params::qc_post::QcPostEffectiveParams {
                schema_version: bijux_dna_domain_fastq::params::qc_post::REPORT_QC_SCHEMA_VERSION
                    .to_string(),
                paired_mode,
                aggregation_engine:
                    bijux_dna_domain_fastq::params::qc_post::QcAggregationEngine::Multiqc,
                aggregation_scope:
                    bijux_dna_domain_fastq::params::qc_post::QcAggregationScope::GovernedQcArtifacts,
            },
        };
        let out_dir = benchmark_command_out_dir("fastq", stage_id, tool_id).with_context(|| {
            format!("build benchmark command output dir for `{stage_id}` / `{tool_id}`")
        })?;
        let plan = {
            bijux_dna_planner_fastq::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
                &tool,
                &qc_inputs,
                &out_dir,
                std::collections::BTreeMap::new(),
                report_qc_params.paired_mode,
                report_qc_params.aggregation_engine,
                report_qc_params.aggregation_scope,
                find_fastq_input(base_plan, "reads_r1")
                    .map(|path| resolve_repo_input_path(repo_root, path))
                    .as_deref(),
                find_fastq_input(base_plan, "reads_r2")
                    .map(|path| resolve_repo_input_path(repo_root, path))
                    .as_deref(),
            )
        }
        .with_context(|| format!("plan FASTQ benchmark command row `{stage_id}` / `{tool_id}`"))?;
        return Ok(plan.command.template);
    }
    let params = project_fastq_benchmark_params_for_tool(
        stage_id,
        tool_id,
        fastq_stage_params_from_plan(stage_id, base_plan)?,
    );
    let explicit_inputs = base_plan
        .io
        .inputs
        .iter()
        .map(|artifact| {
            let mut artifact = artifact.clone();
            artifact.path = resolve_repo_input_path(repo_root, &artifact.path);
            bijux_dna_planner_fastq::FastqStageExplicitInput {
                input_id: artifact.name.as_str().to_string(),
                source_tool_id: explicit_input_source_tool_id(stage_id, tool_id, &artifact),
                artifact,
            }
        })
        .collect::<Vec<_>>();
    let fallback_input = base_plan.io.inputs.first().ok_or_else(|| {
        anyhow!("FASTQ benchmark-ready row `{stage_id}` / `{tool_id}` has no canonical inputs")
    })?;
    let fallback_r1 = resolve_repo_input_path(repo_root, &fallback_input.path);
    let fallback_r2 = find_fastq_input(base_plan, "reads_r2")
        .map(|path| resolve_repo_input_path(repo_root, path));
    let reference_fasta =
        find_reference_fasta(base_plan).map(|path| resolve_repo_input_path(repo_root, path));
    let adapter_bank = bijux_dna_domain_fastq::banks::adapter_bank_context_at(
        repo_root,
        None,
        None,
        None,
        &[],
        &[],
    )
    .context("load default FASTQ adapter bank context")?;
    let polyx_bank = bijux_dna_domain_fastq::banks::polyx_bank_context_at(repo_root, None)
        .context("load default FASTQ polyx bank context")?;
    let contaminant_bank =
        bijux_dna_domain_fastq::banks::contaminant_bank_context_at(repo_root, None)
            .context("load default FASTQ contaminant bank context")?;
    let out_dir = benchmark_command_out_dir("fastq", stage_id, tool_id).with_context(|| {
        format!("build benchmark command output dir for `{stage_id}` / `{tool_id}`")
    })?;
    let plan = {
        bijux_dna_planner_fastq::plan_fastq_stage_binding_with_explicit_inputs(
            bijux_dna_planner_fastq::FastqStageBinding {
                stage_id: stage_id.to_string(),
                stage_instance_id: None,
                tool,
                reason: None,
                params,
            },
            &std::collections::BTreeMap::new(),
            adapter_bank.as_ref(),
            polyx_bank.as_ref(),
            contaminant_bank.as_ref(),
            false,
            &fallback_r1,
            fallback_r2.as_deref(),
            reference_fasta.as_deref(),
            &explicit_inputs,
            &out_dir,
        )
    }
    .with_context(|| format!("plan FASTQ benchmark command row `{stage_id}` / `{tool_id}`"))?;
    Ok(plan.command.template)
}

fn render_bam_stage_tool_argv(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    base_plan: &StagePlanV1,
) -> Result<Vec<String>> {
    let stage_id_value = StageId::new(stage_id.to_string());
    let tool_id_value = ToolId::new(tool_id.to_string());
    let tool = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_execution_spec(
        repo_root,
        &stage_id_value,
        &tool_id_value,
    )
    .with_context(|| format!("load BAM execution spec for `{stage_id}` / `{tool_id}`"))?;
    let (r1, r2) = if stage_id == "bam.align" {
        (
            Some(
                find_first_input(base_plan, &["reads_r1", "fastq_r1"])
                    .map(|path| resolve_repo_input_path(repo_root, path))
                    .context("resolve align reads_r1")?,
            ),
            find_first_input(base_plan, &["reads_r2", "fastq_r2"])
                .map(|path| resolve_repo_input_path(repo_root, path)),
        )
    } else {
        (None, None)
    };
    let bam = if stage_id == "bam.align" {
        None
    } else {
        find_bam_input(base_plan).map(|path| resolve_repo_input_path(repo_root, path))
    };
    let bam_index = if stage_id == "bam.align" {
        None
    } else {
        find_bam_index_input(base_plan).map(|path| resolve_repo_input_path(repo_root, path))
    };
    let derived_reference = find_bam_corpus_reference(repo_root, base_plan);
    let reference = find_reference_fasta(base_plan)
        .map(|path| resolve_repo_input_path(repo_root, path))
        .or(derived_reference);
    let params = project_bam_benchmark_params_for_tool(stage_id, tool_id, base_plan);
    let params_ref = params.as_ref();
    let sample_id = params_ref
        .and_then(|value| value.get("sample_id"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| base_plan.params.get("sample_id").and_then(serde_json::Value::as_str));
    let out_dir = benchmark_command_out_dir("bam", stage_id, tool_id).with_context(|| {
        format!("build benchmark command output dir for `{stage_id}` / `{tool_id}`")
    })?;
    let plan = {
        bijux_dna_planner_bam::stage_api::plan_stage_at(
            repo_root,
            bijux_dna_planner_bam::StagePlanRequest {
                stage_id,
                tool: &tool,
                out_dir: &out_dir,
                bam: bam.as_deref(),
                bam_index: bam_index.as_deref(),
                r1: r1.as_deref(),
                r2: r2.as_deref(),
                reference: reference.as_deref(),
                sample_id,
                params: params_ref,
            },
        )
    }
    .with_context(|| format!("plan BAM benchmark command row `{stage_id}` / `{tool_id}`"))?;
    Ok(plan.command.template)
}

fn benchmark_command_out_dir(domain: &str, stage_id: &str, tool_id: &str) -> Result<PathBuf> {
    let stage_path = stage_id.replace('.', "/");
    Ok(PathBuf::from(DEFAULT_STAGE_TOOL_COMMAND_ROOT).join(domain).join(stage_path).join(tool_id))
}

fn resolve_repo_input_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn domain_label(domain: BenchLocalDomain) -> &'static str {
    match domain {
        BenchLocalDomain::Fastq => "FASTQ",
        BenchLocalDomain::Bam => "BAM",
        BenchLocalDomain::Vcf => "VCF",
    }
}

fn project_bam_benchmark_params_for_tool(
    stage_id: &str,
    tool_id: &str,
    plan: &StagePlanV1,
) -> Option<serde_json::Value> {
    let params = if !plan.effective_params.is_null() {
        Some(plan.effective_params.clone())
    } else if !plan.params.is_null() {
        Some(plan.params.clone())
    } else {
        None
    }?;

    match stage_id {
        "bam.contamination" => project_bam_contamination_params_for_tool(tool_id, params),
        _ => Some(params),
    }
}

fn project_bam_contamination_params_for_tool(
    tool_id: &str,
    mut params: serde_json::Value,
) -> Option<serde_json::Value> {
    let object = params.as_object_mut()?;
    match tool_id {
        "schmutzi" => {
            object.insert("scope".to_string(), serde_json::Value::String("mito".to_string()));
        }
        "verifybamid2" | "contammix" => {
            object.insert("scope".to_string(), serde_json::Value::String("nuclear".to_string()));
        }
        _ => {}
    }
    Some(params)
}

fn find_fastq_input<'a>(plan: &'a StagePlanV1, artifact_name: &str) -> Option<&'a Path> {
    find_first_input(plan, &[artifact_name])
}

fn find_first_input<'a>(plan: &'a StagePlanV1, artifact_names: &[&str]) -> Option<&'a Path> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact_names.iter().any(|name| artifact.name.as_str() == *name))
        .map(|artifact| artifact.path.as_path())
}

fn find_bam_input(plan: &StagePlanV1) -> Option<&Path> {
    plan.io.inputs.iter().find_map(|artifact| {
        if matches!(artifact.role, ArtifactRole::Bam | ArtifactRole::DedupBam) {
            Some(artifact.path.as_path())
        } else {
            None
        }
    })
}

fn find_bam_index_input(plan: &StagePlanV1) -> Option<&Path> {
    plan.io.inputs.iter().find_map(|artifact| {
        if artifact.role == ArtifactRole::Index
            && (artifact.name.as_str().contains("bam") || artifact.name.as_str().contains("bai"))
        {
            Some(artifact.path.as_path())
        } else {
            None
        }
    })
}

fn find_reference_fasta(plan: &StagePlanV1) -> Option<&Path> {
    plan.io.inputs.iter().find_map(|artifact| {
        if artifact.role == ArtifactRole::Reference {
            Some(artifact.path.as_path())
        } else {
            None
        }
    })
}

fn find_bam_corpus_reference(repo_root: &Path, plan: &StagePlanV1) -> Option<PathBuf> {
    #[derive(serde::Deserialize)]
    struct BamCorpusManifestReference {
        reference_fasta: String,
    }

    let bam_path = find_bam_input(plan)?;
    let bam_abs =
        if bam_path.is_absolute() { bam_path.to_path_buf() } else { repo_root.join(bam_path) };
    let corpus_root =
        bam_abs.ancestors().find(|ancestor| ancestor.join("manifest.toml").is_file())?;
    let manifest_path = corpus_root.join("manifest.toml");
    let raw = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest: BamCorpusManifestReference = toml::from_str(&raw).ok()?;
    Some(corpus_root.join(manifest.reference_fasta))
}

fn explicit_input_source_tool_id(
    stage_id: &str,
    tool_id: &str,
    artifact: &bijux_dna_stage_contract::ArtifactRef,
) -> Option<String> {
    if artifact.role == ArtifactRole::Index && artifact.name.as_str() == "reference_index" {
        let compatible_backends = bijux_dna_domain_fastq::reference_index_backends_for_tool(
            &ToolId::new(tool_id.to_string()),
        );
        if compatible_backends.len() == 1 {
            return Some(compatible_backends[0].as_str().to_string());
        }
        if matches!(stage_id, "fastq.deplete_host" | "fastq.deplete_reference_contaminants") {
            return compatible_backends.first().map(|tool_id| tool_id.as_str().to_string());
        }
    }
    None
}

fn readiness_kind_label(readiness_kind: LocalStageReadinessKind) -> &'static str {
    match readiness_kind {
        LocalStageReadinessKind::DryRun => "dry_run",
        LocalStageReadinessKind::Smoke => "smoke",
        LocalStageReadinessKind::DryOrSmoke => "dry_or_smoke",
    }
}

fn ensure_unique_rows(rows: &[BenchmarkCommandRow]) -> Result<()> {
    let mut seen = BTreeSet::<(String, String)>::new();
    for row in rows {
        let pair = (row.stage_id.clone(), row.tool_id.clone());
        if !seen.insert(pair.clone()) {
            return Err(anyhow!(
                "benchmark command rows repeat stage/tool pair `{}` / `{}`",
                pair.0,
                pair.1
            ));
        }
    }
    Ok(())
}
