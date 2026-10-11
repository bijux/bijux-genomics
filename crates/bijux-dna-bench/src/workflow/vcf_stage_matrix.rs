//! Owner: bijux-dna-bench
//! Admission and persistence of the governed local VCF stage-matrix declaration.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_bench_model::{
    validate_local_vcf_stage_matrix_config, validate_local_vcf_stage_matrix_report,
    validate_vcf_stage_matrix_validation_report, LocalVcfStageMatrixConfig,
    LocalVcfStageMatrixReport, ValidateVcfStageMatrixReport, VcfStageMatrixRow,
    LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1,
    VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1,
};

use super::vcf_stage_catalog::build_vcf_stage_catalog_rows;

pub const DEFAULT_VCF_STAGE_MATRIX_PATH: &str = "configs/bench/local/vcf-stage-matrix.toml";
const DECLARED_VCF_STAGE_MATRIX: &str =
    include_str!("../../../../configs/bench/local/vcf-stage-matrix.toml");
const LOCAL_VCF_STAGE_MATRIX_HEADER: &str = "# schema_version = 1\n\
# owner = bijux-dna-bench\n\
# purpose = Governed VCF local benchmark stage matrix.\n\
# authority = bijux-dna-bench\n\
# stability = evolving\n\
# last_updated = 2026-06-25\n\n";

/// Render the governed local VCF stage matrix.
///
/// # Errors
/// Returns an error when owned contracts drift or the matrix cannot be persisted.
pub fn render_vcf_stage_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfStageMatrixReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = build_vcf_stage_matrix_rows()?;
    let config = LocalVcfStageMatrixConfig {
        schema_version: LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1.to_string(),
        rows: rows.clone(),
    };
    validate_local_vcf_stage_matrix_config(&config)?;
    let body = toml::to_string_pretty(&config).context("serialize VCF stage matrix TOML")?;
    let rendered = format!("{LOCAL_VCF_STAGE_MATRIX_HEADER}{body}");
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let catalog_rows = build_vcf_stage_catalog_rows()?;
    let supported_stage_count =
        catalog_rows.iter().filter(|row| row.support_status == "supported").count();
    let planned_stage_count =
        catalog_rows.iter().filter(|row| row.support_status == "planned").count();
    let report = LocalVcfStageMatrixReport {
        schema_version: LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1.to_string(),
        config_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count,
        supported_stage_count,
        planned_stage_count,
        rows,
    };
    validate_local_vcf_stage_matrix_report(&report)?;
    Ok(report)
}

/// Validate a persisted local VCF stage matrix against owned contracts and registries.
///
/// # Errors
/// Returns an error for schema, row, coverage, tool-registry, or strict declaration drift.
pub fn validate_vcf_stage_matrix(
    repo_root: &Path,
    matrix_path: PathBuf,
    strict: bool,
) -> Result<ValidateVcfStageMatrixReport> {
    let matrix_path = repo_relative_path(repo_root, &matrix_path);
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let parsed: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).context("parse VCF stage matrix TOML")?;
    if parsed.schema_version != LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1 {
        bail!(
            "VCF stage matrix schema drift: expected `{LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1}`, found `{}`",
            parsed.schema_version
        );
    }
    validate_local_vcf_stage_matrix_config(&parsed)?;
    let expected_rows = build_vcf_stage_matrix_rows()?;
    if strict && parsed.rows != expected_rows {
        let drift = first_row_drift(&parsed.rows, &expected_rows);
        bail!("VCF stage matrix drifted from owned contracts; {drift}");
    }
    let required_tools = load_required_tool_ids(repo_root)?;
    let registry_tools = load_registry_tool_ids(repo_root)?;
    validate_matrix_rows(&parsed.rows, &required_tools, &registry_tools)?;
    validate_stage_coverage(&parsed.rows, &expected_rows)?;
    let report = ValidateVcfStageMatrixReport {
        schema_version: VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1.to_string(),
        domain: "vcf".to_string(),
        matrix_path: path_relative_to_repo(repo_root, &matrix_path),
        strict,
        row_count: parsed.rows.len(),
        stage_count: parsed
            .rows
            .iter()
            .map(|row| row.stage_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        required_tool_count: required_tools.len(),
        registry_tool_count: registry_tools.len(),
        rows: parsed.rows,
    };
    validate_vcf_stage_matrix_validation_report(&report)?;
    Ok(report)
}

/// Admit ordered VCF stage-matrix rows from the checked declaration.
///
/// # Errors
/// Returns an error when the declaration violates its model-owned schema or stage catalog.
pub fn build_vcf_stage_matrix_rows() -> Result<Vec<VcfStageMatrixRow>> {
    let config: LocalVcfStageMatrixConfig =
        toml::from_str(DECLARED_VCF_STAGE_MATRIX).context("parse declared VCF stage matrix")?;
    validate_local_vcf_stage_matrix_config(&config)?;

    let catalog_rows = build_vcf_stage_catalog_rows()?;
    let declared_stages =
        config.rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let catalog_stages =
        catalog_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if declared_stages != catalog_stages {
        let missing = catalog_stages.difference(&declared_stages).copied().collect::<Vec<_>>();
        let stale = declared_stages.difference(&catalog_stages).copied().collect::<Vec<_>>();
        bail!("VCF stage matrix catalog coverage drifted; missing={missing:?} stale={stale:?}");
    }
    let catalog_by_stage =
        catalog_rows.iter().map(|row| (row.stage_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    for row in &config.rows {
        let catalog = catalog_by_stage
            .get(row.stage_id.as_str())
            .ok_or_else(|| anyhow!("VCF stage matrix has unknown stage `{}`", row.stage_id))?;
        if row.tool_id != catalog.default_tool_id {
            bail!(
                "VCF stage matrix tool drift for `{}`: expected `{}`, found `{}`",
                row.stage_id,
                catalog.default_tool_id,
                row.tool_id
            );
        }
        if row.asset_profile_id != catalog.local_smoke_mode {
            bail!(
                "VCF stage matrix asset-profile drift for `{}`: expected `{}`, found `{}`",
                row.stage_id,
                catalog.local_smoke_mode,
                row.asset_profile_id
            );
        }
    }
    Ok(config.rows)
}

fn validate_matrix_rows(
    rows: &[VcfStageMatrixRow],
    required_tools: &BTreeSet<String>,
    registry_tools: &BTreeSet<String>,
) -> Result<()> {
    for row in rows {
        if !required_tools.contains(&row.tool_id) {
            bail!("VCF stage matrix row `{}` uses unrequired tool `{}`", row.stage_id, row.tool_id);
        }
        if !registry_tools.contains(&row.tool_id) {
            bail!(
                "VCF stage matrix row `{}` uses unregistered tool `{}`",
                row.stage_id,
                row.tool_id
            );
        }
    }
    Ok(())
}

fn validate_stage_coverage(
    rows: &[VcfStageMatrixRow],
    expected: &[VcfStageMatrixRow],
) -> Result<()> {
    let observed = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let expected = expected.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed != expected {
        let missing = expected.difference(&observed).copied().collect::<Vec<_>>();
        let stale = observed.difference(&expected).copied().collect::<Vec<_>>();
        bail!("VCF stage matrix stage coverage drifted; missing={missing:?} stale={stale:?}");
    }
    Ok(())
}

fn first_row_drift(actual: &[VcfStageMatrixRow], expected: &[VcfStageMatrixRow]) -> String {
    for index in 0..actual.len().max(expected.len()) {
        match (actual.get(index), expected.get(index)) {
            (Some(left), Some(right)) if left != right => {
                return format!("row {index} drifted for stage `{}`", right.stage_id);
            }
            (Some(left), None) => return format!("unexpected row {index} for `{}`", left.stage_id),
            (None, Some(right)) => return format!("missing row {index} for `{}`", right.stage_id),
            _ => {}
        }
    }
    "row content changed".to_string()
}

fn load_required_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    load_tool_ids(
        repo_root,
        &[
            "configs/ci/tools/required_tools_vcf.toml",
            "configs/ci/tools/required_tools_vcf_downstream.toml",
        ],
        "required_tools",
    )
}

fn load_registry_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for relative in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(repo_root.join(relative))
            .with_context(|| format!("read {}", repo_root.join(relative).display()))?;
        let parsed: toml::Value =
            toml::from_str(&raw).with_context(|| format!("parse {relative}"))?;
        let entries = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {relative}"))?;
        for entry in entries {
            if let Some(id) = entry.get("id").and_then(toml::Value::as_str) {
                ids.insert(id.to_string());
            }
        }
    }
    Ok(ids)
}

fn load_tool_ids(repo_root: &Path, paths: &[&str], key: &str) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for relative in paths {
        let raw = fs::read_to_string(repo_root.join(relative))
            .with_context(|| format!("read {}", repo_root.join(relative).display()))?;
        let parsed: toml::Value =
            toml::from_str(&raw).with_context(|| format!("parse {relative}"))?;
        let entries = parsed
            .get(key)
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing {key} in {relative}"))?;
        ids.extend(entries.iter().filter_map(toml::Value::as_str).map(str::to_string));
    }
    Ok(ids)
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
