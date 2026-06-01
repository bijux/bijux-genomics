use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_AUTHENTICITY_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.authenticity.local_smoke.report.v1";
const AUTHENTICITY_COMPOSITION_SCHEMA_VERSION: &str = "bijux.bam.authenticity.composition.v1";
const AUTHENTICITY_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.authenticity.stage_metrics.v1";
const AUTHENTICITY_METRIC_IDS: [&str; 5] =
    ["damage", "contamination", "complexity", "coverage", "mapping"];

#[derive(Debug, Clone, Serialize)]
struct LocalAuthenticitySmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    score: f64,
    confidence: f64,
    pmd_like_signal_present: bool,
    #[serde(default)]
    contamination_estimate: Option<f64>,
    consumed_metrics: Vec<String>,
    missing_metrics: Vec<String>,
    authenticity_report: String,
    authenticity_summary: String,
    authenticity_composite: String,
    advisory_boundary: String,
    stage_metrics: String,
    damage_unified_metrics: String,
    contamination_summary: String,
    complexity_summary: String,
    coverage_regime: String,
    mapping_summary: String,
}

/// Materialize the governed local-smoke `bam.authenticity` artifacts and top-level report.
///
/// The written report lives at `target/local-smoke/bam.authenticity/authenticity.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_authenticity_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.authenticity expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("target/local-smoke/bam.authenticity");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_authenticity_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("authenticity.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable typed authenticity-composition artifacts beside BAM stage outputs.
///
/// # Errors
/// Returns an error if required damage artifacts are missing or the composed authenticity outputs
/// cannot be written.
pub(crate) fn write_stage_authenticity_artifacts(
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
    let bam_root = stage_dir.parent().ok_or_else(|| {
        anyhow!("bam.authenticity stage path has no BAM root: {}", stage_dir.display())
    })?;

    let damage_unified_path = bam_root.join("damage").join("damage.unified_metrics.json");
    let canonical_damage = read_canonical_damage_metrics(&damage_unified_path)?;
    let advisory = bijux_dna_domain_bam::summarize_tiny_bam_authenticity_advisory(
        &input_bam,
        &canonical_damage,
    )?;
    let summary_path = stage_dir.join("authenticity.summary.json");
    let advisory_boundary_path = stage_dir.join("advisory_boundary.json");
    bijux_dna_infra::atomic_write_json(&summary_path, &advisory)
        .with_context(|| format!("write {}", summary_path.display()))?;
    bijux_dna_infra::atomic_write_json(&advisory_boundary_path, &advisory.advisory_boundary)
        .with_context(|| format!("write {}", advisory_boundary_path.display()))?;

    let contamination_summary_path =
        bam_root.join("contamination").join("contamination.summary.json");
    let complexity_summary_path = bam_root.join("complexity").join("complexity.summary.json");
    let coverage_regime_path = bam_root.join("coverage").join("coverage.regime.json");
    let mapping_summary_path = bam_root.join("mapping_summary").join("mapping_summary.json");

    let contamination = read_optional_contamination_summary(&contamination_summary_path)?;
    let complexity = read_optional_json::<bijux_dna_domain_bam::BamComplexitySummaryV1>(
        &complexity_summary_path,
    )?;
    let coverage =
        read_optional_json::<bijux_dna_domain_bam::BamCoverageRegimeV1>(&coverage_regime_path)?;
    let mapping = if mapping_summary_path.exists() {
        read_optional_json::<bijux_dna_domain_bam::BamMappingSummaryV1>(&mapping_summary_path)?
    } else {
        Some(bijux_dna_domain_bam::summarize_tiny_bam_mapping(&input_bam)?)
    };

    let contamination_cross_check = contamination.as_ref().map(|metric| {
        bijux_dna_domain_bam::metrics::contamination_cross_check(
            canonical_damage.c_to_t_5p.max(canonical_damage.g_to_a_3p),
            metric.estimate,
        )
    });

    let composition = serde_json::json!({
        "schema_version": AUTHENTICITY_COMPOSITION_SCHEMA_VERSION,
        "stage_id": "bam.authenticity",
        "score": advisory.score,
        "confidence": advisory.confidence,
        "pmd_like_signal_present": advisory.pmd_like_signal_present,
        "contamination_cross_check": contamination_cross_check,
        "consumed_metrics": {
            "damage": {
                "available": true,
                "source": "stage_artifact",
                "path": damage_unified_path,
                "terminal_c_to_t_5p": canonical_damage.c_to_t_5p,
                "terminal_g_to_a_3p": canonical_damage.g_to_a_3p,
            },
            "contamination": {
                "available": contamination.is_some(),
                "source": if contamination.is_some() { "stage_artifact" } else { "unavailable" },
                "path": contamination.as_ref().map(|_| contamination_summary_path.clone()),
                "method": contamination.as_ref().map(|metric| metric.method.clone()),
                "estimate": contamination.as_ref().map(|metric| metric.estimate),
                "ci_low": contamination.as_ref().map(|metric| metric.ci_low),
                "ci_high": contamination.as_ref().map(|metric| metric.ci_high),
            },
            "complexity": {
                "available": complexity.is_some(),
                "source": if complexity.is_some() { "stage_artifact" } else { "unavailable" },
                "path": complexity.as_ref().map(|_| complexity_summary_path.clone()),
                "observed_total_reads": complexity.as_ref().map(|metric| metric.observed_total_reads),
                "observed_unique_reads": complexity.as_ref().map(|metric| metric.observed_unique_reads),
                "estimated_unique_reads": complexity.as_ref().and_then(|metric| metric.estimated_unique_reads),
                "insufficient_data_reason": complexity.as_ref().and_then(|metric| metric.insufficient_data_reason.clone()),
            },
            "coverage": {
                "available": coverage.is_some(),
                "source": if coverage.is_some() { "stage_artifact" } else { "unavailable" },
                "path": coverage.as_ref().map(|_| coverage_regime_path.clone()),
                "regime_id": coverage.as_ref().map(|metric| metric.regime_id.clone()),
                "mean_depth": coverage.as_ref().map(|metric| metric.mean_depth),
                "breadth_1x": coverage.as_ref().map(|metric| metric.breadth_1x),
            },
            "mapping": {
                "available": mapping.is_some(),
                "source": if mapping_summary_path.exists() {
                    "stage_artifact"
                } else if mapping.is_some() {
                    "derived_from_input_bam"
                } else {
                    "unavailable"
                },
                "path": if mapping_summary_path.exists() {
                    Some(mapping_summary_path.clone())
                } else {
                    None::<PathBuf>
                },
                "total_reads": mapping.as_ref().and_then(|metric| metric.flagstat.total_reads),
                "mapped_reads": mapping.as_ref().and_then(|metric| metric.flagstat.mapped_reads),
                "mapped_fraction": mapping.as_ref().and_then(|metric| metric.flagstat.mapped_fraction),
                "mean_mapq": mapping.as_ref().and_then(|metric| metric.mapq_regime.as_ref().map(|regime| regime.mean)),
            },
        },
    });

    let stage_metrics_path = stage_dir.join("stage.metrics.json");
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": AUTHENTICITY_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.authenticity",
            "tool_id": plan.tool_id.as_str(),
            "score": advisory.score,
            "confidence": advisory.confidence,
            "pmd_like_signal_present": advisory.pmd_like_signal_present,
            "contamination_estimate": contamination.as_ref().map(|metric| metric.estimate),
            "consumed_metric_ids": available_metric_ids(&composition),
            "missing_metric_ids": missing_metric_ids(&composition),
        }),
    )
    .with_context(|| format!("write {}", stage_metrics_path.display()))?;

    let composition_path = stage_dir.join("authenticity_composite.json");
    bijux_dna_infra::atomic_write_json(&composition_path, &composition)
        .with_context(|| format!("write {}", composition_path.display()))?;

    let report_path = stage_dir.join("authenticity.json");
    bijux_dna_infra::atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.bam.authenticity.v1",
            "summary": advisory,
            "composition": composition,
        }),
    )
    .with_context(|| format!("write {}", report_path.display()))?;

    Ok(summary_path)
}

fn materialize_local_authenticity_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalAuthenticitySmokeCasePlan,
) -> Result<LocalAuthenticitySmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;
    let case_root = case_out_dir.parent().ok_or_else(|| {
        anyhow!(
            "bam.authenticity local-smoke output has no sample root: {}",
            case_out_dir.display()
        )
    })?;

    let damage_dir = case_root.join("damage");
    let contamination_dir = case_root.join("contamination");
    let complexity_dir = case_root.join("complexity");
    let coverage_dir = case_root.join("coverage");
    let mapping_dir = case_root.join("mapping_summary");
    for directory in [&damage_dir, &contamination_dir, &complexity_dir, &coverage_dir, &mapping_dir]
    {
        bijux_dna_infra::ensure_dir(directory)?;
    }

    let input_bam = repo_root.join(&case.bam);
    let damage_metrics = bijux_dna_domain_bam::metrics::DamageMetricsV1 {
        c_to_t_5p: case.damage_terminal_c_to_t_5p,
        g_to_a_3p: case.damage_terminal_g_to_a_3p,
        pmd_score_histogram: Vec::new(),
    };
    let damage_unified_path = damage_dir.join("damage.unified_metrics.json");
    bijux_dna_infra::atomic_write_json(
        &damage_unified_path,
        &serde_json::json!({
            "canonical": damage_metrics,
            "tools_seen": ["pydamage", "mapdamage2"],
        }),
    )?;

    let contamination_summary_path = contamination_dir.join("contamination.summary.json");
    bijux_dna_infra::atomic_write_json(
        &contamination_summary_path,
        &serde_json::json!({
            "method": case.contamination_method,
            "estimate": case.contamination_estimate,
            "ci_low": case.contamination_ci_low,
            "ci_high": case.contamination_ci_high,
            "assumptions": [
                "local authenticity smoke composes contamination evidence from governed inputs"
            ],
        }),
    )?;

    let complexity_summary = bijux_dna_domain_bam::summarize_tiny_bam_complexity(
        &input_bam,
        "preseq",
        case.complexity_min_reads,
        &case.complexity_projection_points,
    )?;
    let complexity_summary_path = complexity_dir.join("complexity.summary.json");
    bijux_dna_infra::atomic_write_json(&complexity_summary_path, &complexity_summary)?;

    let coverage_summary = bijux_dna_domain_bam::summarize_tiny_bam_coverage(
        &input_bam,
        &case.coverage_depth_thresholds,
    )?;
    let coverage_regime = coverage_summary.regime.clone().ok_or_else(|| {
        anyhow!(
            "bam.authenticity local-smoke could not derive governed coverage regime from {}",
            input_bam.display()
        )
    })?;
    let coverage_regime_path = coverage_dir.join("coverage.regime.json");
    bijux_dna_infra::atomic_write_json(&coverage_regime_path, &coverage_regime)?;

    let mapping_summary = bijux_dna_domain_bam::summarize_tiny_bam_mapping(&input_bam)?;
    let mapping_summary_path = mapping_dir.join("mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&mapping_summary_path, &mapping_summary)?;

    let _summary_path = write_stage_authenticity_artifacts(&case_out_dir, &case.plan)?;
    let authenticity_report_path =
        resolve_output_path(repo_root, &case.plan, "authenticity_report")?;
    let authenticity_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let authenticity_composite_path = case_out_dir.join("authenticity_composite.json");
    let advisory_boundary_path = case_out_dir.join("advisory_boundary.json");

    let authenticity_summary: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1 =
        read_required_json(&authenticity_summary_path)?;
    let composition: serde_json::Value = read_required_json(&authenticity_composite_path)?;
    let consumed_metrics = available_metric_ids(&composition);
    let missing_metrics = missing_metric_ids(&composition);
    let expectation_matched = float_matches(authenticity_summary.score, case.expected_score)
        && float_matches(authenticity_summary.confidence, case.expected_confidence)
        && authenticity_summary.pmd_like_signal_present == case.expected_pmd_like_signal_present
        && consumed_metrics == case.expected_consumed_metrics;

    let contamination_estimate = composition
        .get("consumed_metrics")
        .and_then(|value| value.get("contamination"))
        .and_then(|value| value.get("estimate"))
        .and_then(serde_json::Value::as_f64);

    Ok(LocalAuthenticitySmokeReport {
        schema_version: LOCAL_AUTHENTICITY_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.authenticity".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        method: case.plan.tool_id.as_str().to_string(),
        score: authenticity_summary.score,
        confidence: authenticity_summary.confidence,
        pmd_like_signal_present: authenticity_summary.pmd_like_signal_present,
        contamination_estimate,
        consumed_metrics,
        missing_metrics,
        authenticity_report: path_relative_to_repo(repo_root, &authenticity_report_path),
        authenticity_summary: path_relative_to_repo(repo_root, &authenticity_summary_path),
        authenticity_composite: path_relative_to_repo(repo_root, &authenticity_composite_path),
        advisory_boundary: path_relative_to_repo(repo_root, &advisory_boundary_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
        damage_unified_metrics: path_relative_to_repo(repo_root, &damage_unified_path),
        contamination_summary: path_relative_to_repo(repo_root, &contamination_summary_path),
        complexity_summary: path_relative_to_repo(repo_root, &complexity_summary_path),
        coverage_regime: path_relative_to_repo(repo_root, &coverage_regime_path),
        mapping_summary: path_relative_to_repo(repo_root, &mapping_summary_path),
    })
}

fn available_metric_ids(composition: &serde_json::Value) -> Vec<String> {
    AUTHENTICITY_METRIC_IDS
        .iter()
        .filter_map(|metric_id| {
            composition
                .get("consumed_metrics")
                .and_then(|value| value.get(metric_id))
                .and_then(|value| value.get("available"))
                .and_then(serde_json::Value::as_bool)
                .filter(|available| *available)
                .map(|_| (*metric_id).to_string())
        })
        .collect()
}

fn missing_metric_ids(composition: &serde_json::Value) -> Vec<String> {
    AUTHENTICITY_METRIC_IDS
        .iter()
        .filter_map(|metric_id| {
            composition
                .get("consumed_metrics")
                .and_then(|value| value.get(metric_id))
                .and_then(|value| value.get("available"))
                .and_then(serde_json::Value::as_bool)
                .filter(|available| !available)
                .map(|_| (*metric_id).to_string())
        })
        .collect()
}

fn read_canonical_damage_metrics(
    path: &Path,
) -> Result<bijux_dna_domain_bam::metrics::DamageMetricsV1> {
    let value: serde_json::Value = read_required_json(path)?;
    serde_json::from_value(
        value
            .get("canonical")
            .cloned()
            .ok_or_else(|| anyhow!("damage unified metrics missing canonical payload"))?,
    )
    .with_context(|| format!("parse canonical damage metrics from {}", path.display()))
}

fn read_optional_contamination_summary(
    path: &Path,
) -> Result<Option<bijux_dna_domain_bam::metrics::ContaminationMetricsV1>> {
    if path.exists() {
        Ok(Some(
            bijux_dna_domain_bam::metrics::parse_contamination_json(path)
                .with_context(|| format!("parse {}", path.display()))?,
        ))
    } else {
        Ok(None)
    }
}

fn read_optional_json<T>(path: &Path) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    if path.exists() {
        Ok(Some(read_required_json(path)?))
    } else {
        Ok(None)
    }
}

fn read_required_json<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
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
            anyhow!("bam.authenticity local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
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
