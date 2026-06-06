use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::vcf_tool_serving_map::{render_vcf_tool_serving_map, DEFAULT_VCF_TOOL_SERVING_MAP_PATH};
use crate::commands::benchmark::local_vcf_stage_catalog::build_vcf_stage_catalog_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_UNDERCOVERED_STAGES_PATH: &str =
    "target/bench-readiness/vcf-undercovered-stages.tsv";
const VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_undercovered_stages.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfUndercoveredStageRow {
    pub(crate) stage_id: String,
    pub(crate) valid_tool_classes: Vec<String>,
    pub(crate) registered_tools: Vec<String>,
    pub(crate) missing_tools: Vec<String>,
    pub(crate) decision: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfUndercoveredStagesReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) undercovered_stage_count: usize,
    pub(crate) decision_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfUndercoveredStageRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfRegistryToolRecord {
    tool_id: String,
    stage_ids: BTreeSet<String>,
    statuses: BTreeSet<String>,
}

pub(crate) fn run_render_vcf_undercovered_stages(
    args: &parse::BenchReadinessRenderVcfUndercoveredStagesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_undercovered_stages(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_undercovered_stages(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfUndercoveredStagesReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_count = build_vcf_stage_catalog_rows()?.len();
    let rows = collect_vcf_undercovered_stage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_undercovered_stages_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut decision_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *decision_counts.entry(row.decision.clone()).or_default() += 1;
    }

    Ok(VcfUndercoveredStagesReport {
        schema_version: VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        undercovered_stage_count: rows.len(),
        decision_counts,
        rows,
    })
}

fn collect_vcf_undercovered_stage_rows(repo_root: &Path) -> Result<Vec<VcfUndercoveredStageRow>> {
    let registry_records = load_vcf_registry_tool_records(repo_root)?;
    let registry_by_tool = registry_records
        .iter()
        .cloned()
        .map(|record| (record.tool_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let mut valid_tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for record in &registry_records {
        for stage_id in &record.stage_ids {
            valid_tools_by_stage
                .entry(stage_id.clone())
                .or_default()
                .insert(record.tool_id.clone());
        }
    }

    let serving_map =
        render_vcf_tool_serving_map(repo_root, PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH))?;
    let mut registered_tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for row in serving_map.rows {
        registered_tools_by_stage.entry(row.stage_id).or_default().insert(row.tool_id);
    }

    let mut rows = Vec::new();
    for (stage_id, valid_tools) in valid_tools_by_stage {
        let registered_tools = registered_tools_by_stage
            .get(stage_id.as_str())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        if valid_tools.len() <= 1 || registered_tools.len() != 1 {
            continue;
        }

        let missing_tools = valid_tools
            .iter()
            .filter(|tool_id| !registered_tools.contains(tool_id))
            .cloned()
            .collect::<Vec<_>>();
        if missing_tools.is_empty() {
            continue;
        }

        let valid_tool_classes = valid_tools
            .iter()
            .map(|tool_id| vcf_tool_class_label(tool_id))
            .collect::<Result<BTreeSet<_>>>()?
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let decision = classify_undercoverage_decision(&missing_tools, &registry_by_tool)?;

        rows.push(VcfUndercoveredStageRow {
            stage_id,
            valid_tool_classes,
            registered_tools,
            missing_tools,
            decision: decision.to_string(),
        });
    }

    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_undercovered_stage_contract(&rows)?;
    Ok(rows)
}

fn classify_undercoverage_decision<'a>(
    missing_tools: &[String],
    registry_by_tool: &'a BTreeMap<String, VcfRegistryToolRecord>,
) -> Result<&'a str> {
    for tool_id in missing_tools {
        let record = registry_by_tool.get(tool_id.as_str()).ok_or_else(|| {
            anyhow!("VCF undercovered-stage detector is missing registry data for `{tool_id}`")
        })?;
        if record.statuses.contains("production") {
            return Ok("limit_to_specialized_tool");
        }
    }
    Ok("future_not_benchmark_ready")
}

fn load_vcf_registry_tool_records(repo_root: &Path) -> Result<Vec<VcfRegistryToolRecord>> {
    let mut records = BTreeMap::<String, VcfRegistryToolRecord>::new();
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
            let tool_id = entry
                .get("id")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| anyhow!("tool entry in {relative_path} is missing id"))?
                .to_string();
            let stage_ids = entry
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| anyhow!("tool `{tool_id}` in {relative_path} is missing stage_ids"))?
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            let status = entry
                .get("status")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| anyhow!("tool `{tool_id}` in {relative_path} is missing status"))?;

            let record = records.entry(tool_id.clone()).or_insert_with(|| VcfRegistryToolRecord {
                tool_id,
                stage_ids: BTreeSet::new(),
                statuses: BTreeSet::new(),
            });
            record.stage_ids.extend(stage_ids);
            record.statuses.insert(status.to_string());
        }
    }
    Ok(records.into_values().collect())
}

fn vcf_tool_class_label(tool_id: &str) -> Result<&'static str> {
    let label = match tool_id {
        "angsd" => "genotype_likelihood_calling",
        "bcftools" => "variant_processing",
        "beagle" => "phasing",
        "eagle" => "phasing",
        "eigensoft" => "population_structure",
        "germline" => "relatedness",
        "glimpse" => "imputation",
        "ibdhap" => "relatedness",
        "ibdne" => "demography",
        "ibdseq" => "relatedness",
        "impute5" => "imputation",
        "minimac4" => "imputation",
        "plink" => "cohort_analysis",
        "plink2" => "cohort_analysis",
        "shapeit" => "phasing",
        "shapeit5" => "phasing",
        other => {
            return Err(anyhow!(
                "VCF undercovered-stage detector encountered unknown tool `{other}`"
            ))
        }
    };
    Ok(label)
}

fn ensure_vcf_undercovered_stage_contract(rows: &[VcfUndercoveredStageRow]) -> Result<()> {
    let expected_rows = [
        (
            "vcf.admixture",
            &["cohort_analysis", "variant_processing"][..],
            &["plink2"][..],
            &["bcftools", "plink"][..],
            "limit_to_specialized_tool",
        ),
        (
            "vcf.call_gl",
            &["genotype_likelihood_calling", "variant_processing"][..],
            &["bcftools"][..],
            &["angsd"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.call_pseudohaploid",
            &["genotype_likelihood_calling", "variant_processing"][..],
            &["bcftools"][..],
            &["angsd"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.damage_filter",
            &["genotype_likelihood_calling", "variant_processing"][..],
            &["bcftools"][..],
            &["angsd"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.gl_propagation",
            &["genotype_likelihood_calling", "variant_processing"][..],
            &["bcftools"][..],
            &["angsd"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.ibd",
            &["demography", "relatedness"][..],
            &["germline"][..],
            &["ibdhap", "ibdne", "ibdseq"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.imputation",
            &["imputation", "phasing", "variant_processing"][..],
            &["beagle"][..],
            &["bcftools", "glimpse", "impute5", "minimac4"][..],
            "limit_to_specialized_tool",
        ),
        (
            "vcf.impute",
            &["imputation", "phasing"][..],
            &["beagle"][..],
            &["glimpse", "impute5", "minimac4"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.pca",
            &["cohort_analysis", "population_structure", "variant_processing"][..],
            &["plink2"][..],
            &["bcftools", "eigensoft"][..],
            "limit_to_specialized_tool",
        ),
        (
            "vcf.phasing",
            &["phasing", "variant_processing"][..],
            &["shapeit5"][..],
            &["bcftools", "beagle", "eagle", "shapeit"][..],
            "limit_to_specialized_tool",
        ),
        (
            "vcf.population_structure",
            &["cohort_analysis", "population_structure"][..],
            &["plink2"][..],
            &["eigensoft", "plink"][..],
            "future_not_benchmark_ready",
        ),
        (
            "vcf.qc",
            &["cohort_analysis", "variant_processing"][..],
            &["plink2"][..],
            &["bcftools", "plink"][..],
            "limit_to_specialized_tool",
        ),
    ];

    if rows.len() != expected_rows.len() {
        return Err(anyhow!(
            "VCF undercovered-stage report drifted from the governed stage slice (expected {}, found {})",
            expected_rows.len(),
            rows.len()
        ));
    }

    for (stage_id, valid_tool_classes, registered_tools, missing_tools, decision) in expected_rows {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id)
            .ok_or_else(|| anyhow!("VCF undercovered-stage report is missing `{stage_id}`"))?;
        if row.valid_tool_classes != valid_tool_classes
            || row.registered_tools != registered_tools
            || row.missing_tools != missing_tools
            || row.decision != decision
        {
            bail!(
                "VCF undercovered-stage `{stage_id}` drifted from its governed detector contract"
            );
        }
    }

    for row in rows {
        if row.registered_tools.len() != 1 {
            bail!(
                "VCF undercovered-stage `{}` must retain exactly one registered tool",
                row.stage_id
            );
        }
        if row.missing_tools.is_empty() {
            bail!(
                "VCF undercovered-stage `{}` must retain at least one missing tool",
                row.stage_id
            );
        }
        if row.decision != "future_not_benchmark_ready"
            && row.decision != "limit_to_specialized_tool"
        {
            bail!(
                "VCF undercovered-stage `{}` declared unsupported decision `{}`",
                row.stage_id,
                row.decision
            );
        }
    }

    Ok(())
}

fn render_vcf_undercovered_stages_tsv(rows: &[VcfUndercoveredStageRow]) -> String {
    let mut rendered =
        String::from("stage_id\tvalid_tool_classes\tregistered_tools\tmissing_tools\tdecision\n");
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.valid_tool_classes.join(",")),
            sanitize_tsv(&row.registered_tools.join(",")),
            sanitize_tsv(&row.missing_tools.join(",")),
            sanitize_tsv(&row.decision),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
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
        render_vcf_undercovered_stages, DEFAULT_VCF_UNDERCOVERED_STAGES_PATH,
        VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_undercovered_stages_report_tracks_governed_stage_slice() {
        let root = repo_root();
        let report = render_vcf_undercovered_stages(
            &root,
            PathBuf::from(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH),
        )
        .expect("render VCF undercovered stages");

        assert_eq!(report.schema_version, VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.undercovered_stage_count, 12);
        assert_eq!(report.decision_counts.get("future_not_benchmark_ready").copied(), Some(7));
        assert_eq!(report.decision_counts.get("limit_to_specialized_tool").copied(), Some(5));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.phasing"
                && row.registered_tools == vec!["shapeit5".to_string()]
                && row.decision == "limit_to_specialized_tool"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.impute"
                && row.missing_tools
                    == vec!["glimpse".to_string(), "impute5".to_string(), "minimac4".to_string()]
                && row.decision == "future_not_benchmark_ready"
        }));
    }
}
