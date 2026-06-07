use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::run::VcfBenchCorpusId;
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{
    vcf_domain_stage_adapter_id, vcf_domain_stage_expected_output_ids, vcf_domain_stage_parser_id,
};
use serde::{Deserialize, Serialize};

use super::local_vcf_stage_catalog::{build_vcf_stage_catalog_rows, VcfStageCatalogRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_STAGE_MATRIX_PATH: &str =
    "benchmarks/configs/local/vcf-stage-matrix.toml";
const LOCAL_VCF_STAGE_MATRIX_SCHEMA_VERSION: &str = "bijux.bench.vcf.local_stage_matrix.v1";
const LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_stage_matrix.v1";
const VCF_STAGE_MATRIX_VALIDATION_SCHEMA_VERSION: &str = "bijux.bench.validate_matrix.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalVcfStageMatrixConfig {
    pub(crate) schema_version: String,
    pub(crate) rows: Vec<VcfStageMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcfStageMatrixRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfStageMatrixReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) supported_stage_count: usize,
    pub(crate) planned_stage_count: usize,
    pub(crate) rows: Vec<VcfStageMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ValidateMatrixReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) matrix_path: String,
    pub(crate) strict: bool,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) required_tool_count: usize,
    pub(crate) registry_tool_count: usize,
    pub(crate) rows: Vec<VcfStageMatrixRow>,
}

pub(crate) fn run_render_vcf_stage_matrix(
    args: &parse::BenchLocalRenderVcfStageMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_stage_matrix(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn run_validate_vcf_stage_matrix(
    repo_root: &Path,
    args: &parse::BenchValidateMatrixArgs,
) -> Result<()> {
    let report = validate_vcf_stage_matrix(
        repo_root,
        args.matrix.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH)),
        args.strict,
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{} {} rows validated", report.matrix_path, report.row_count);
    }
    Ok(())
}

pub(crate) fn render_vcf_stage_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfStageMatrixReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = build_vcf_stage_matrix_rows()?;
    let config = LocalVcfStageMatrixConfig {
        schema_version: LOCAL_VCF_STAGE_MATRIX_SCHEMA_VERSION.to_string(),
        rows: rows.clone(),
    };
    let rendered = toml::to_string_pretty(&config).context("serialize VCF stage matrix TOML")?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let catalog_rows = build_vcf_stage_catalog_rows()?;
    let supported_stage_count =
        catalog_rows.iter().filter(|row| row_support_status(row) == "supported").count();
    let planned_stage_count =
        catalog_rows.iter().filter(|row| row_support_status(row) == "planned").count();

    Ok(LocalVcfStageMatrixReport {
        schema_version: LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count,
        supported_stage_count,
        planned_stage_count,
        rows,
    })
}

pub(crate) fn validate_vcf_stage_matrix(
    repo_root: &Path,
    matrix_path: PathBuf,
    strict: bool,
) -> Result<ValidateMatrixReport> {
    let matrix_path = repo_relative_path(repo_root, &matrix_path);
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let parsed: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).context("parse VCF stage matrix TOML")?;
    if parsed.schema_version != LOCAL_VCF_STAGE_MATRIX_SCHEMA_VERSION {
        bail!(
            "VCF stage matrix schema drift: expected `{LOCAL_VCF_STAGE_MATRIX_SCHEMA_VERSION}`, found `{}`",
            parsed.schema_version
        );
    }

    let expected_rows = build_vcf_stage_matrix_rows()?;
    if strict && parsed.rows != expected_rows {
        let drift = first_row_drift(&parsed.rows, &expected_rows);
        bail!(
            "VCF stage matrix drifted from owned contracts; {drift}; rerun `bijux-dna bench local render-vcf-stage-matrix`"
        );
    }

    let required_tools = load_required_tool_ids(repo_root)?;
    let registry_tools = load_registry_tool_ids(repo_root)?;
    validate_matrix_rows(&parsed.rows, &required_tools, &registry_tools)?;
    validate_stage_coverage(&parsed.rows, &expected_rows)?;

    Ok(ValidateMatrixReport {
        schema_version: VCF_STAGE_MATRIX_VALIDATION_SCHEMA_VERSION,
        domain: "vcf",
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
    })
}

pub(crate) fn build_vcf_stage_matrix_rows() -> Result<Vec<VcfStageMatrixRow>> {
    let catalog_rows = build_vcf_stage_catalog_rows()?;
    let corpus_id = VcfBenchCorpusId::ProductionRegression.as_str().to_string();

    catalog_rows.iter().map(|catalog_row| build_matrix_row(catalog_row, &corpus_id)).collect()
}

fn build_matrix_row(
    catalog_row: &VcfStageCatalogRow,
    corpus_id: &str,
) -> Result<VcfStageMatrixRow> {
    let stage = resolve_stage(&catalog_row.stage_id)?;
    let adapter_id = vcf_domain_stage_adapter_id(stage).ok_or_else(|| {
        anyhow!("VCF stage `{}` is missing an adapter contract id", stage.as_str())
    })?;
    let parser_id = vcf_domain_stage_parser_id(stage)
        .ok_or_else(|| anyhow!("VCF stage `{}` is missing a parser contract id", stage.as_str()))?;
    let expected_outputs = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF stage `{}` is missing expected output ids", stage.as_str()))?;

    Ok(VcfStageMatrixRow {
        stage_id: catalog_row.stage_id.clone(),
        tool_id: catalog_row.default_tool_id.clone(),
        corpus_id: corpus_id.to_string(),
        asset_profile_id: catalog_row.local_smoke_mode.clone(),
        adapter_id: adapter_id.to_string(),
        parser_id: parser_id.to_string(),
        expected_outputs: expected_outputs.iter().map(|value| (*value).to_string()).collect(),
    })
}

fn validate_matrix_rows(
    rows: &[VcfStageMatrixRow],
    required_tools: &BTreeSet<String>,
    registry_tools: &BTreeSet<String>,
) -> Result<()> {
    let mut seen_stage_ids = BTreeSet::<&str>::new();
    for row in rows {
        if !seen_stage_ids.insert(row.stage_id.as_str()) {
            bail!("VCF stage matrix contains duplicate stage row `{}`", row.stage_id);
        }
        if row.tool_id.trim().is_empty() {
            bail!("VCF stage matrix row `{}` is missing tool_id", row.stage_id);
        }
        if row.corpus_id.trim().is_empty() {
            bail!("VCF stage matrix row `{}` is missing corpus_id", row.stage_id);
        }
        if row.asset_profile_id.trim().is_empty() {
            bail!("VCF stage matrix row `{}` is missing asset_profile_id", row.stage_id);
        }
        if row.adapter_id.trim().is_empty() {
            bail!("VCF stage matrix row `{}` is missing adapter_id", row.stage_id);
        }
        if row.parser_id.trim().is_empty() {
            bail!("VCF stage matrix row `{}` is missing parser_id", row.stage_id);
        }
        if row.expected_outputs.is_empty() {
            bail!("VCF stage matrix row `{}` is missing expected_outputs", row.stage_id);
        }
        if !required_tools.contains(&row.tool_id) {
            bail!(
                "VCF stage matrix row `{}` uses tool `{}` missing from required_tools_vcf(_downstream).toml",
                row.stage_id,
                row.tool_id
            );
        }
        if !registry_tools.contains(&row.tool_id) {
            bail!(
                "VCF stage matrix row `{}` uses tool `{}` missing from tool_registry_vcf(_downstream).toml",
                row.stage_id,
                row.tool_id
            );
        }
    }
    Ok(())
}

fn validate_stage_coverage(
    rows: &[VcfStageMatrixRow],
    expected_rows: &[VcfStageMatrixRow],
) -> Result<()> {
    let observed = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let expected = expected_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed != expected {
        let missing = expected.difference(&observed).copied().collect::<Vec<_>>();
        let stale = observed.difference(&expected).copied().collect::<Vec<_>>();
        bail!("VCF stage matrix stage coverage drifted; missing={missing:?} stale={stale:?}");
    }
    Ok(())
}

fn first_row_drift(actual: &[VcfStageMatrixRow], expected: &[VcfStageMatrixRow]) -> String {
    let max_len = actual.len().max(expected.len());
    for index in 0..max_len {
        match (actual.get(index), expected.get(index)) {
            (Some(left), Some(right)) if left != right => {
                return format!("row {} drifted for stage `{}`", index, right.stage_id);
            }
            (Some(left), None) => {
                return format!("unexpected extra row {} for stage `{}`", index, left.stage_id);
            }
            (None, Some(right)) => {
                return format!("missing row {} for stage `{}`", index, right.stage_id);
            }
            _ => {}
        }
    }
    "row content changed".to_string()
}

fn load_required_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    let mut tool_ids = BTreeSet::<String>::new();
    for relative_path in [
        "configs/ci/tools/required_tools_vcf.toml",
        "configs/ci/tools/required_tools_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(repo_root.join(relative_path))
            .with_context(|| format!("read {}", repo_root.join(relative_path).display()))?;
        let parsed: toml::Value = toml::from_str(&raw)
            .with_context(|| format!("parse {}", repo_root.join(relative_path).display()))?;
        let entries = parsed
            .get("required_tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing required_tools in {relative_path}"))?;
        for entry in entries {
            if let Some(tool_id) = entry.as_str() {
                tool_ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(tool_ids)
}

fn load_registry_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    let mut tool_ids = BTreeSet::<String>::new();
    for relative_path in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(repo_root.join(relative_path))
            .with_context(|| format!("read {}", repo_root.join(relative_path).display()))?;
        let parsed: toml::Value = toml::from_str(&raw)
            .with_context(|| format!("parse {}", repo_root.join(relative_path).display()))?;
        let entries = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {relative_path}"))?;
        for entry in entries {
            if let Some(tool_id) = entry.get("id").and_then(toml::Value::as_str) {
                tool_ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(tool_ids)
}

fn resolve_stage(stage_id: &str) -> Result<VcfDomainStage> {
    VcfDomainStage::all()
        .iter()
        .copied()
        .find(|stage| stage.as_str() == stage_id)
        .ok_or_else(|| anyhow!("unknown VCF stage `{stage_id}`"))
}

fn row_support_status(row: &VcfStageCatalogRow) -> &str {
    row.support_status.as_str()
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
        build_vcf_stage_matrix_rows, render_vcf_stage_matrix, validate_vcf_stage_matrix,
        DEFAULT_VCF_STAGE_MATRIX_PATH, LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_VERSION,
        LOCAL_VCF_STAGE_MATRIX_SCHEMA_VERSION, VCF_STAGE_MATRIX_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_stage_matrix_rows_track_catalog_and_contract_ids() {
        let rows = build_vcf_stage_matrix_rows().expect("build VCF stage matrix rows");

        assert_eq!(rows.len(), 20);
        assert_eq!(
            rows.first().map(|row| row.stage_id.as_str()),
            Some("vcf.prepare_reference_panel")
        );
        assert_eq!(rows.last().map(|row| row.stage_id.as_str()), Some("vcf.stats"));

        let prepare_reference_panel = rows
            .iter()
            .find(|row| row.stage_id == "vcf.prepare_reference_panel")
            .expect("prepare reference panel row");
        assert_eq!(prepare_reference_panel.tool_id, "bcftools");
        assert_eq!(prepare_reference_panel.corpus_id, "vcf_production_regression");
        assert_eq!(prepare_reference_panel.asset_profile_id, "vcf_reference_panel");
        assert_eq!(prepare_reference_panel.adapter_id, "vcf.adapter.reference_panel");
        assert_eq!(prepare_reference_panel.parser_id, "vcf.parser.vcf_output");
        assert_eq!(
            prepare_reference_panel.expected_outputs,
            vec!["prepared_panel".to_string(), "chunks_json".to_string()]
        );

        let phasing = rows.iter().find(|row| row.stage_id == "vcf.phasing").expect("phasing row");
        assert_eq!(phasing.tool_id, "shapeit5");
        assert_eq!(phasing.asset_profile_id, "vcf_cohort_with_panel");
        assert_eq!(phasing.adapter_id, "vcf.adapter.panel_workflow");
        assert_eq!(phasing.parser_id, "vcf.parser.vcf_output");
        assert_eq!(phasing.expected_outputs, vec!["phased_vcf".to_string()]);

        let stats = rows.iter().find(|row| row.stage_id == "vcf.stats").expect("stats row");
        assert_eq!(stats.asset_profile_id, "vcf_cohort");
        assert_eq!(stats.adapter_id, "vcf.adapter.quality_control");
        assert_eq!(stats.parser_id, "vcf.parser.stats_report");
        assert_eq!(stats.expected_outputs, vec!["stats_json".to_string()]);
    }

    #[test]
    fn vcf_stage_matrix_report_writes_governed_config() {
        let repo_root = repo_root();
        let report =
            render_vcf_stage_matrix(&repo_root, PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH))
                .expect("render VCF stage matrix");

        assert_eq!(report.schema_version, LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_VERSION);
        assert_eq!(report.config_path, DEFAULT_VCF_STAGE_MATRIX_PATH);
        assert_eq!(report.row_count, 20);
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.supported_stage_count, 8);
        assert_eq!(report.planned_stage_count, 12);
    }

    #[test]
    fn vcf_stage_matrix_validation_accepts_governed_matrix() {
        let repo_root = repo_root();
        render_vcf_stage_matrix(&repo_root, PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH))
            .expect("render VCF stage matrix");

        let report = validate_vcf_stage_matrix(
            &repo_root,
            PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH),
            true,
        )
        .expect("validate VCF stage matrix");

        assert_eq!(report.schema_version, VCF_STAGE_MATRIX_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert_eq!(report.matrix_path, DEFAULT_VCF_STAGE_MATRIX_PATH);
        assert!(report.strict);
        assert_eq!(report.row_count, 20);
        assert_eq!(report.stage_count, 20);
        assert!(report.required_tool_count >= 8);
        assert!(report.registry_tool_count >= 8);

        let parsed = std::fs::read_to_string(repo_root.join(DEFAULT_VCF_STAGE_MATRIX_PATH))
            .expect("read matrix file");
        let config: super::LocalVcfStageMatrixConfig =
            toml::from_str(&parsed).expect("parse matrix file");
        assert_eq!(config.schema_version, LOCAL_VCF_STAGE_MATRIX_SCHEMA_VERSION);
        assert_eq!(config.rows, report.rows);
    }
}
