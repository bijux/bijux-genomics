use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_DAMAGE_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.damage.local_smoke.report.v1";
const DAMAGE_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.damage.stage_metrics.v1";
const DAMAGE_PARSER_OUTPUT_SCHEMA_VERSION: &str = "bijux.bam.damage.parser_output.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalDamageSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    tools_seen: Vec<String>,
    terminal_c_to_t_5p: f64,
    terminal_g_to_a_3p: f64,
    short_fragment_fraction: f64,
    damage_signal: String,
    strict_profile_upgraded: bool,
    damage_report: String,
    terminal_position_metrics: String,
    parser_output: String,
    damage_profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    damage_plot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    damage_clusters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    damage_parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pmd_scores: Option<String>,
    advisory_boundary: String,
    udg_regime: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize)]
struct ParsedDamageToolOutput {
    tool_id: String,
    metrics: bijux_dna_domain_bam::metrics::DamageMetricsV1,
}

struct LocalDamageOutputPaths {
    summary: PathBuf,
    terminal_position_metrics: PathBuf,
    parser_output: PathBuf,
    damage_profile: PathBuf,
    damage_plot: Option<PathBuf>,
    damage_clusters: Option<PathBuf>,
    damage_parameters: Option<PathBuf>,
    pmd_scores: Option<PathBuf>,
    advisory_boundary: PathBuf,
    udg_regime: PathBuf,
    stage_metrics: PathBuf,
}

struct DamageExpectationDeltas {
    expectation_matched: bool,
    terminal_c_to_t_5p_delta: f64,
    terminal_g_to_a_3p_delta: f64,
    short_fragment_fraction_delta: f64,
}

/// Materialize the governed local-smoke `bam.damage` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.damage/damage.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_damage_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_damage_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.damage expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.damage");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_damage_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("damage.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable typed `bam.damage` postprocess artifacts beside BAM stage outputs.
///
/// # Errors
/// Returns an error if stage damage metrics are missing or the summary artifacts cannot be written.
pub(crate) fn write_stage_damage_artifacts(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let input_bam = resolve_stage_input_path(&input_bam);
    let strict_profile = strict_profile_from_plan(plan);
    let mut measurements = read_stage_damage_measurements(stage_dir)?;
    prioritize_damage_measurements(&mut measurements, plan.tool_id.as_str());
    let unified_path = write_stage_damage_unified_from_measurements(stage_dir, &measurements)?;
    let parser_output_path = write_stage_damage_parser_output(stage_dir, &measurements)?;
    let unified = read_damage_unified(&unified_path)?;
    let canonical = parse_canonical_damage_metrics(&unified)?;
    let tools_seen = parse_tools_seen(&unified);

    let summary = bijux_dna_domain_bam::summarize_tiny_bam_damage_evidence(
        &input_bam,
        &canonical,
        strict_profile,
    )?;
    let summary_path = stage_dir.join("damage.summary.json");
    let advisory_boundary_path = stage_dir.join("advisory_boundary.json");
    let stage_metrics_path = stage_dir.join("stage.metrics.json");

    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    bijux_dna_infra::atomic_write_json(&advisory_boundary_path, &summary.advisory_boundary)
        .with_context(|| format!("write {}", advisory_boundary_path.display()))?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": DAMAGE_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.damage",
            "tool_id": plan.tool_id.as_str(),
            "tools_seen": tools_seen,
            "terminal_c_to_t_5p": summary.terminal_c_to_t_5p,
            "terminal_g_to_a_3p": summary.terminal_g_to_a_3p,
            "short_fragment_fraction": summary.short_fragment_fraction,
            "damage_signal": summary.damage_signal,
            "strict_profile_upgraded": summary.strict_profile_upgraded,
        }),
    )
    .with_context(|| format!("write {}", stage_metrics_path.display()))?;
    debug_assert!(parser_output_path.exists());
    Ok(summary_path)
}

fn write_stage_damage_unified_from_measurements(
    stage_dir: &Path,
    measurements: &[(String, bijux_dna_domain_bam::metrics::DamageMetricsV1)],
) -> Result<PathBuf> {
    let canonical = measurements
        .first()
        .map_or_else(bijux_dna_domain_bam::metrics::DamageMetricsV1::empty, |(_, metric)| {
            metric.clone()
        });
    let comparison = if measurements.len() >= 2 {
        Some(bijux_dna_domain_bam::metrics::compare_damage_metrics(
            measurements[0].0.as_str(),
            &measurements[0].1,
            measurements[1].0.as_str(),
            &measurements[1].1,
            0.05,
        ))
    } else {
        None
    };
    let path = stage_dir.join("damage.unified_metrics.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "canonical": canonical,
            "tools_seen": measurements
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            "comparison": comparison,
        }),
    )
    .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn write_stage_damage_parser_output(
    stage_dir: &Path,
    measurements: &[(String, bijux_dna_domain_bam::metrics::DamageMetricsV1)],
) -> Result<PathBuf> {
    let path = stage_dir.join("damage.parser_output.json");
    let parsed_tools = measurements
        .iter()
        .map(|(tool_id, metrics)| ParsedDamageToolOutput {
            tool_id: tool_id.clone(),
            metrics: metrics.clone(),
        })
        .collect::<Vec<_>>();
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "schema_version": DAMAGE_PARSER_OUTPUT_SCHEMA_VERSION,
            "stage_id": "bam.damage",
            "parsed_tools": parsed_tools,
        }),
    )
    .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn materialize_local_damage_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalDamageSmokeCasePlan,
) -> Result<LocalDamageSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let input_bam = repo_root.join(&case.bam);
    let damage_mapdamage2_path = case_out_dir.join("damage.mapdamage2.txt");
    write_local_damage_primary_artifact(&case_out_dir, case)?;
    bijux_dna_infra::atomic_write_bytes(
        &damage_mapdamage2_path,
        render_mapdamage2_misincorporation(
            case.expected_terminal_c_to_t_5p,
            case.expected_terminal_g_to_a_3p,
        )
        .as_bytes(),
    )?;
    write_udg_regime(&case_out_dir, &case.plan)?;
    let output_paths = resolve_local_damage_output_paths(repo_root, &case.plan, &case_out_dir)?;

    let summary: bijux_dna_domain_bam::BamDamageEvidenceV1 = serde_json::from_str(
        &std::fs::read_to_string(&output_paths.summary)
            .with_context(|| format!("read {}", output_paths.summary.display()))?,
    )
    .with_context(|| format!("parse {}", output_paths.summary.display()))?;
    let unified = read_damage_unified(&output_paths.terminal_position_metrics)?;
    let tools_seen = parse_tools_seen(&unified);
    let deltas = local_damage_expectation(case, &summary);

    write_local_damage_extra_artifacts(
        &case.plan,
        &summary,
        &tools_seen,
        &output_paths.damage_profile,
        output_paths.damage_plot.as_deref(),
        output_paths.damage_clusters.as_deref(),
        output_paths.damage_parameters.as_deref(),
        output_paths.pmd_scores.as_deref(),
    )?;
    write_local_damage_stage_metrics(
        &output_paths.stage_metrics,
        case,
        &summary,
        &tools_seen,
        &deltas,
    )?;
    build_local_damage_smoke_report(
        repo_root,
        case,
        &input_bam,
        &summary,
        &tools_seen,
        &output_paths,
        &deltas,
    )
}

fn resolve_local_damage_output_paths(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    case_out_dir: &Path,
) -> Result<LocalDamageOutputPaths> {
    Ok(LocalDamageOutputPaths {
        summary: write_stage_damage_artifacts(case_out_dir, plan)?,
        terminal_position_metrics: resolve_output_path(
            repo_root,
            plan,
            "terminal_position_metrics",
        )?,
        parser_output: resolve_output_path(repo_root, plan, "parser_output")?,
        damage_profile: resolve_output_path(repo_root, plan, "damage_profile")?,
        damage_plot: resolve_optional_output_path(repo_root, plan, "damage_plot"),
        damage_clusters: resolve_optional_output_path(repo_root, plan, "damage_clusters"),
        damage_parameters: resolve_optional_output_path(repo_root, plan, "damage_parameters"),
        pmd_scores: resolve_optional_output_path(repo_root, plan, "pmd_scores"),
        advisory_boundary: case_out_dir.join("advisory_boundary.json"),
        udg_regime: case_out_dir.join("udg_regime.json"),
        stage_metrics: resolve_output_path(repo_root, plan, "stage_metrics")?,
    })
}

fn local_damage_expectation(
    case: &bijux_dna_planner_bam::stage_api::LocalDamageSmokeCasePlan,
    summary: &bijux_dna_domain_bam::BamDamageEvidenceV1,
) -> DamageExpectationDeltas {
    DamageExpectationDeltas {
        expectation_matched: float_matches(
            summary.terminal_c_to_t_5p,
            case.expected_terminal_c_to_t_5p,
        ) && float_matches(
            summary.terminal_g_to_a_3p,
            case.expected_terminal_g_to_a_3p,
        ) && float_matches(
            summary.short_fragment_fraction,
            case.expected_short_fragment_fraction,
        ) && summary.damage_signal == case.expected_damage_signal
            && summary.strict_profile_upgraded == case.expected_strict_profile_upgraded,
        terminal_c_to_t_5p_delta: summary.terminal_c_to_t_5p - case.expected_terminal_c_to_t_5p,
        terminal_g_to_a_3p_delta: summary.terminal_g_to_a_3p - case.expected_terminal_g_to_a_3p,
        short_fragment_fraction_delta: summary.short_fragment_fraction
            - case.expected_short_fragment_fraction,
    }
}

fn write_local_damage_stage_metrics(
    path: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalDamageSmokeCasePlan,
    summary: &bijux_dna_domain_bam::BamDamageEvidenceV1,
    tools_seen: &[String],
    deltas: &DamageExpectationDeltas,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "schema_version": DAMAGE_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.damage",
            "sample_id": case.sample_id,
            "tool_id": case.plan.tool_id.as_str(),
            "tools_seen": tools_seen,
            "expected_terminal_c_to_t_5p": case.expected_terminal_c_to_t_5p,
            "terminal_c_to_t_5p": summary.terminal_c_to_t_5p,
            "terminal_c_to_t_5p_delta": deltas.terminal_c_to_t_5p_delta,
            "expected_terminal_g_to_a_3p": case.expected_terminal_g_to_a_3p,
            "terminal_g_to_a_3p": summary.terminal_g_to_a_3p,
            "terminal_g_to_a_3p_delta": deltas.terminal_g_to_a_3p_delta,
            "expected_short_fragment_fraction": case.expected_short_fragment_fraction,
            "short_fragment_fraction": summary.short_fragment_fraction,
            "short_fragment_fraction_delta": deltas.short_fragment_fraction_delta,
            "expected_damage_signal": case.expected_damage_signal,
            "damage_signal": summary.damage_signal,
            "expected_strict_profile_upgraded": case.expected_strict_profile_upgraded,
            "strict_profile_upgraded": summary.strict_profile_upgraded,
            "expectation_matched": deltas.expectation_matched,
        }),
    )
}

fn build_local_damage_smoke_report(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalDamageSmokeCasePlan,
    input_bam: &Path,
    summary: &bijux_dna_domain_bam::BamDamageEvidenceV1,
    tools_seen: &[String],
    output_paths: &LocalDamageOutputPaths,
    deltas: &DamageExpectationDeltas,
) -> Result<LocalDamageSmokeReport> {
    Ok(LocalDamageSmokeReport {
        schema_version: LOCAL_DAMAGE_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.damage".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched: deltas.expectation_matched,
        input_bam: path_relative_to_repo(repo_root, input_bam),
        method: case.plan.tool_id.as_str().to_string(),
        tools_seen: tools_seen.to_vec(),
        terminal_c_to_t_5p: summary.terminal_c_to_t_5p,
        terminal_g_to_a_3p: summary.terminal_g_to_a_3p,
        short_fragment_fraction: summary.short_fragment_fraction,
        damage_signal: summary.damage_signal.clone(),
        strict_profile_upgraded: summary.strict_profile_upgraded,
        damage_report: path_relative_to_repo(repo_root, &output_paths.summary),
        terminal_position_metrics: path_relative_to_repo(
            repo_root,
            &output_paths.terminal_position_metrics,
        ),
        parser_output: path_relative_to_repo(repo_root, &output_paths.parser_output),
        damage_profile: path_relative_to_repo(repo_root, &output_paths.damage_profile),
        damage_plot: output_paths
            .damage_plot
            .as_deref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        damage_clusters: output_paths
            .damage_clusters
            .as_deref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        damage_parameters: output_paths
            .damage_parameters
            .as_deref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        pmd_scores: output_paths
            .pmd_scores
            .as_deref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        advisory_boundary: path_relative_to_repo(repo_root, &output_paths.advisory_boundary),
        udg_regime: path_relative_to_repo(repo_root, &output_paths.udg_regime),
        stage_metrics: path_relative_to_repo(repo_root, &output_paths.stage_metrics),
    })
}

fn read_stage_damage_measurements(
    stage_dir: &Path,
) -> Result<Vec<(String, bijux_dna_domain_bam::metrics::DamageMetricsV1)>> {
    let mut measurements = Vec::new();
    let pydamage = stage_dir.join("damage.pydamage.json");
    if pydamage.exists() {
        measurements.push((
            "pydamage".to_string(),
            bijux_dna_domain_bam::metrics::parse_pydamage_json(&pydamage)
                .with_context(|| format!("parse {}", pydamage.display()))?,
        ));
    }
    let profiler = stage_dir.join("damage.profiler.json");
    if profiler.exists() {
        measurements.push((
            "damageprofiler".to_string(),
            bijux_dna_domain_bam::metrics::parse_damageprofiler_json(&profiler)
                .with_context(|| format!("parse {}", profiler.display()))?,
        ));
    }
    let addeam = stage_dir.join("damage.addeam.json");
    if addeam.exists() {
        measurements.push((
            "addeam".to_string(),
            bijux_dna_domain_bam::metrics::parse_addeam_json(&addeam)
                .with_context(|| format!("parse {}", addeam.display()))?,
        ));
    }
    let mapdamage = stage_dir.join("damage.mapdamage2.txt");
    if mapdamage.exists() {
        measurements.push((
            "mapdamage2".to_string(),
            bijux_dna_domain_bam::metrics::parse_mapdamage2_misincorporation(&mapdamage)
                .with_context(|| format!("parse {}", mapdamage.display()))?,
        ));
    }
    let ngsbriggs = stage_dir.join("damage.ngsbriggs.json");
    if ngsbriggs.exists() {
        measurements.push((
            "ngsbriggs".to_string(),
            bijux_dna_domain_bam::metrics::parse_ngsbriggs_json(&ngsbriggs)
                .with_context(|| format!("parse {}", ngsbriggs.display()))?,
        ));
    }
    let pmdtools = stage_dir.join("damage.pmdtools.json");
    if pmdtools.exists() {
        measurements.push((
            "pmdtools".to_string(),
            bijux_dna_domain_bam::metrics::parse_pmdtools_json(&pmdtools)
                .with_context(|| format!("parse {}", pmdtools.display()))?,
        ));
    }
    if measurements.is_empty() {
        return Err(anyhow!(
            "bam.damage hard failure: no readable damage metrics artifacts found in {}",
            stage_dir.display()
        ));
    }
    Ok(measurements)
}

fn prioritize_damage_measurements(
    measurements: &mut Vec<(String, bijux_dna_domain_bam::metrics::DamageMetricsV1)>,
    primary_tool_id: &str,
) {
    if let Some(index) = measurements.iter().position(|(tool_id, _)| tool_id == primary_tool_id) {
        let primary = measurements.remove(index);
        measurements.insert(0, primary);
    }
}

fn write_local_damage_primary_artifact(
    stage_dir: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalDamageSmokeCasePlan,
) -> Result<()> {
    let metrics_json = serde_json::json!({
        "schema_version": "bijux.bam.damage.v1",
        "ct_5p": case.expected_terminal_c_to_t_5p,
        "ga_3p": case.expected_terminal_g_to_a_3p,
        "pmd_score_histogram": [[0, 8], [1, 13], [2, 21]],
    });
    let path = match case.plan.tool_id.as_str() {
        "addeam" => stage_dir.join("damage.addeam.json"),
        "damageprofiler" => stage_dir.join("damage.profiler.json"),
        "ngsbriggs" => stage_dir.join("damage.ngsbriggs.json"),
        "pmdtools" => stage_dir.join("damage.pmdtools.json"),
        "pydamage" => stage_dir.join("damage.pydamage.json"),
        other => {
            return Err(anyhow!(
                "local-smoke bam.damage does not support synthetic artifact materialization for tool `{other}`"
            ));
        }
    };
    bijux_dna_infra::atomic_write_json(&path, &metrics_json)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_damage_unified(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

fn parse_canonical_damage_metrics(
    unified: &serde_json::Value,
) -> Result<bijux_dna_domain_bam::metrics::DamageMetricsV1> {
    serde_json::from_value(
        unified
            .get("canonical")
            .cloned()
            .ok_or_else(|| anyhow!("damage unified metrics missing canonical payload"))?,
    )
    .context("parse canonical damage metrics")
}

fn parse_tools_seen(unified: &serde_json::Value) -> Vec<String> {
    unified
        .get("tools_seen")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn render_mapdamage2_misincorporation(c_to_t_5p: f64, g_to_a_3p: f64) -> String {
    format!("pos C->T G->A\n1 {c_to_t_5p:.4} {g_to_a_3p:.4}\n")
}

fn write_local_damage_extra_artifacts(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    summary: &bijux_dna_domain_bam::BamDamageEvidenceV1,
    tools_seen: &[String],
    damage_profile_path: &Path,
    damage_plot_path: Option<&Path>,
    damage_clusters_path: Option<&Path>,
    damage_parameters_path: Option<&Path>,
    pmd_scores_path: Option<&Path>,
) -> Result<()> {
    let tool_id = plan.tool_id.as_str();
    write_damage_profile_artifact(damage_profile_path, tool_id, summary)?;
    if let Some(damage_plot_path) = damage_plot_path {
        write_damage_plot_artifact(damage_plot_path, tool_id, tools_seen)?;
    }
    if let Some(damage_clusters_path) = damage_clusters_path {
        write_damage_clusters_artifact(damage_clusters_path, tool_id, summary)?;
    }
    if let Some(damage_parameters_path) = damage_parameters_path {
        write_damage_parameters_artifact(damage_parameters_path, plan, tool_id)?;
    }
    if let Some(pmd_scores_path) = pmd_scores_path {
        write_damage_pmd_scores_artifact(pmd_scores_path, tool_id)?;
    }
    Ok(())
}

fn write_damage_profile_artifact(
    path: &Path,
    tool_id: &str,
    summary: &bijux_dna_domain_bam::BamDamageEvidenceV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "artifact_id": "damage_profile",
            "stage_id": "bam.damage",
            "tool_id": tool_id,
            "terminal_c_to_t_5p": summary.terminal_c_to_t_5p,
            "terminal_g_to_a_3p": summary.terminal_g_to_a_3p,
            "damage_signal": summary.damage_signal,
        }),
    )
}

fn write_damage_plot_artifact(path: &Path, tool_id: &str, tools_seen: &[String]) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "artifact_id": "damage_plot",
            "stage_id": "bam.damage",
            "tool_id": tool_id,
            "status": "local_smoke_placeholder",
            "tools_seen": tools_seen,
        }),
    )
}

fn write_damage_clusters_artifact(
    path: &Path,
    tool_id: &str,
    summary: &bijux_dna_domain_bam::BamDamageEvidenceV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "artifact_id": "damage_clusters",
            "stage_id": "bam.damage",
            "tool_id": tool_id,
            "clusters": [
                {
                    "label": summary.damage_signal,
                    "terminal_c_to_t_5p": summary.terminal_c_to_t_5p,
                    "terminal_g_to_a_3p": summary.terminal_g_to_a_3p,
                }
            ],
        }),
    )
}

fn write_damage_parameters_artifact(
    path: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    tool_id: &str,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "artifact_id": "damage_parameters",
            "stage_id": "bam.damage",
            "tool_id": tool_id,
            "damage_tool_profile": plan.params.get("damage_tool_profile").and_then(serde_json::Value::as_str),
            "evidence_only": plan.params.get("evidence_only").and_then(serde_json::Value::as_bool),
            "udg_model": plan.params.get("udg_model").and_then(serde_json::Value::as_str),
        }),
    )
}

fn write_damage_pmd_scores_artifact(path: &Path, tool_id: &str) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "artifact_id": "pmd_scores",
            "stage_id": "bam.damage",
            "tool_id": tool_id,
            "scores": [0, 1, 2, 3],
        }),
    )
}

fn write_udg_regime(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let udg_model = plan.params.get("udg_model").and_then(serde_json::Value::as_str);
    let path = stage_dir.join("udg_regime.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "udg_model": udg_model,
            "stage_id": plan.stage_id.as_str(),
        }),
    )
    .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn strict_profile_from_plan(plan: &bijux_dna_stage_contract::StagePlanV1) -> bool {
    matches!(
        plan.params.get("damage_tool_profile").and_then(serde_json::Value::as_str),
        Some("strict_damage_profile")
    )
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn resolve_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("bam.damage local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
}

fn resolve_optional_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Option<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| resolve_plan_path(repo_root, &artifact.path))
}

fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}

fn resolve_stage_input_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Ok(repo_root) = crate::support::workspace::resolve_repo_root() {
        let candidate = repo_root.join(path);
        if candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}
