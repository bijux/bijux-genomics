use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    comparable_metric_stage_ids, stage_comparable_metric_specs, VcfComparableMetricDirection,
    VcfDomainStage,
};
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_COMPARABLE_METRICS_PATH: &str =
    "benchmarks/readiness/vcf-comparable-metrics.tsv";
const VCF_COMPARABLE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_comparable_metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfComparableMetricsRow {
    pub(crate) stage_id: String,
    pub(crate) metric_id: String,
    pub(crate) metric_name: String,
    pub(crate) unit: String,
    pub(crate) direction: String,
    pub(crate) required: bool,
    pub(crate) tools_covered: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfComparableMetricsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) multi_tool_stage_count: usize,
    pub(crate) retained_tool_row_count: usize,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<VcfComparableMetricsRow>,
}

pub(crate) fn run_render_vcf_comparable_metrics(
    args: &parse::BenchReadinessRenderVcfComparableMetricsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_comparable_metrics(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_COMPARABLE_METRICS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_comparable_metrics(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfComparableMetricsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_tools = collect_retained_tools_by_stage(repo_root)?;
    let rows = collect_vcf_comparable_metric_rows(&stage_tools)?;
    let multi_tool_stage_count = stage_tools.values().filter(|tools| tools.len() >= 2).count();
    let retained_tool_row_count =
        stage_tools.values().filter(|tools| tools.len() >= 2).map(BTreeSet::len).sum();
    let covered_stages =
        rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    if covered_stages != multi_tool_stage_count {
        return Err(anyhow!(
            "VCF comparable metrics must cover every multi-tool retained stage, covered {covered_stages} of {multi_tool_stage_count}"
        ));
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_comparable_metrics_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(VcfComparableMetricsReport {
        schema_version: VCF_COMPARABLE_METRICS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: covered_stages,
        multi_tool_stage_count,
        retained_tool_row_count,
        row_count: rows.len(),
        rows,
    })
}

fn collect_vcf_comparable_metric_rows(
    stage_tools: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<VcfComparableMetricsRow>> {
    let governed_stage_ids = comparable_metric_stage_ids()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let multi_tool_stage_ids = stage_tools
        .iter()
        .filter(|(_, tools)| tools.len() >= 2)
        .map(|(stage_id, _)| stage_id.clone())
        .collect::<BTreeSet<_>>();

    let mut missing_contract_stage_ids =
        multi_tool_stage_ids.difference(&governed_stage_ids).cloned().collect::<Vec<_>>();
    if !missing_contract_stage_ids.is_empty() {
        missing_contract_stage_ids.sort();
        return Err(anyhow!(
            "VCF comparable metric contract is missing multi-tool stages: {}",
            missing_contract_stage_ids.join(", ")
        ));
    }

    let mut rows = Vec::new();
    for stage_id in multi_tool_stage_ids {
        let stage = VcfDomainStage::try_from(stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
        let tools_covered = stage_tools
            .get(stage_id.as_str())
            .ok_or_else(|| anyhow!("missing retained tool coverage for `{stage_id}`"))?
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let specs = stage_comparable_metric_specs(stage);
        if specs.is_empty() {
            return Err(anyhow!(
                "VCF comparable metric contract for `{stage_id}` must publish at least one governed metric"
            ));
        }
        for spec in specs {
            rows.push(VcfComparableMetricsRow {
                stage_id: stage_id.clone(),
                metric_id: spec.metric_id.to_string(),
                metric_name: spec.metric_name.to_string(),
                unit: spec.unit.to_string(),
                direction: comparable_metric_direction_label(spec.direction).to_string(),
                required: spec.required,
                tools_covered: tools_covered.clone(),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    Ok(rows)
}

fn collect_retained_tools_by_stage(repo_root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let scratch_root = repo_root.join("artifacts/bench-readiness/vcf-comparable-metrics");
    fs::create_dir_all(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;

    let mut stage_tools = BTreeMap::<String, BTreeSet<String>>::new();

    for row in crate::commands::benchmark::readiness::vcf_bcftools_adapter::collect_vcf_bcftools_adapter_rows(repo_root)? {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in
        crate::commands::benchmark::readiness::vcf_angsd_adapter::collect_vcf_angsd_adapter_rows(
            repo_root,
        )?
    {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in crate::commands::benchmark::readiness::vcf_plink_family_adapter::render_vcf_plink_family_adapter(
        repo_root,
        "plink",
        scratch_root.join("plink.vcf.json"),
    )?
    .rows
    {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in crate::commands::benchmark::readiness::vcf_plink_family_adapter::render_vcf_plink_family_adapter(
        repo_root,
        "plink2",
        scratch_root.join("plink2.vcf.json"),
    )?
    .rows
    {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in
        crate::commands::benchmark::readiness::vcf_eigensoft_adapter::render_vcf_eigensoft_adapter(
            repo_root,
            scratch_root.join("eigensoft.vcf.json"),
        )?
        .rows
    {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for tool_id in ["shapeit5", "eagle", "beagle"] {
        for row in crate::commands::benchmark::readiness::vcf_phasing_family_adapter::render_vcf_phasing_family_adapter(
            repo_root,
            tool_id,
            scratch_root.join(format!("{tool_id}.phasing.vcf.json")),
        )?
        .rows
        {
            register_stage_tool(
                &mut stage_tools,
                &row.stage_id,
                &row.tool_id,
                row.argv_validation_passed,
                row.missing_input_test_passed,
            );
        }
    }
    for row in crate::commands::benchmark::readiness::vcf_imputation_family_adapter::render_vcf_imputation_family_adapter(
        repo_root,
        scratch_root.join("imputation-family.vcf.json"),
    )?
    .rows
    {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }
    for row in crate::commands::benchmark::readiness::vcf_descent_family_adapter::render_vcf_descent_family_adapter(
        repo_root,
        scratch_root.join("descent-family.vcf.json"),
    )?
    .rows
    {
        register_stage_tool(
            &mut stage_tools,
            &row.stage_id,
            &row.tool_id,
            row.argv_validation_passed,
            row.missing_input_test_passed,
        );
    }

    Ok(stage_tools)
}

fn register_stage_tool(
    stage_tools: &mut BTreeMap<String, BTreeSet<String>>,
    stage_id: &str,
    tool_id: &str,
    argv_validation_passed: bool,
    missing_input_test_passed: bool,
) {
    if argv_validation_passed && missing_input_test_passed {
        stage_tools.entry(stage_id.to_string()).or_default().insert(tool_id.to_string());
    }
}

fn render_vcf_comparable_metrics_tsv(rows: &[VcfComparableMetricsRow]) -> String {
    let mut rendered = String::from(
        "stage_id\tmetric_id\tmetric_name\tunit\tdirection\trequired\ttools_covered\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.metric_id),
            sanitize_tsv(&row.metric_name),
            sanitize_tsv(&row.unit),
            sanitize_tsv(&row.direction),
            row.required,
            sanitize_tsv(&row.tools_covered.join(",")),
        ));
    }
    rendered
}

fn comparable_metric_direction_label(direction: VcfComparableMetricDirection) -> &'static str {
    match direction {
        VcfComparableMetricDirection::ExactMatchPreferred => "exact_match_preferred",
        VcfComparableMetricDirection::HigherIsBetter => "higher_is_better",
        VcfComparableMetricDirection::LowerIsBetter => "lower_is_better",
    }
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        comparable_metric_direction_label, render_vcf_comparable_metrics,
        DEFAULT_VCF_COMPARABLE_METRICS_PATH, VCF_COMPARABLE_METRICS_SCHEMA_VERSION,
    };
    use bijux_dna_domain_vcf::VcfComparableMetricDirection;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_vcf_comparable_metrics_reports_governed_metric_rows() {
        let root = repo_root();
        let report = render_vcf_comparable_metrics(
            &root,
            PathBuf::from(DEFAULT_VCF_COMPARABLE_METRICS_PATH),
        )
        .expect("render VCF comparable metrics");

        assert_eq!(report.schema_version, VCF_COMPARABLE_METRICS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_COMPARABLE_METRICS_PATH);
        assert_eq!(report.stage_count, 12);
        assert_eq!(report.multi_tool_stage_count, 12);
        assert_eq!(report.retained_tool_row_count, 30);
        assert_eq!(report.row_count, 33);

        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call_gl"
                && row.metric_id == "sites_with_likelihoods"
                && row.metric_name == "sites with likelihoods"
                && row.unit == "sites"
                && row.direction == "higher_is_better"
                && row.required
                && row.tools_covered == vec!["angsd".to_string(), "bcftools".to_string()]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.qc"
                && row.metric_id == "concordance"
                && row.unit == "fraction"
                && row.direction == "higher_is_better"
                && row.tools_covered == vec!["plink".to_string(), "plink2".to_string()]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.phasing"
                && row.metric_id == "switch_error_proxy"
                && row.direction == "lower_is_better"
                && row.tools_covered
                    == vec!["beagle".to_string(), "eagle".to_string(), "shapeit5".to_string()]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.impute"
                && row.metric_id == "masked_truth_match_count"
                && row.direction == "higher_is_better"
                && row.tools_covered
                    == vec![
                        "beagle".to_string(),
                        "glimpse".to_string(),
                        "impute5".to_string(),
                        "minimac4".to_string(),
                    ]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.ibd"
                && row.metric_id == "pair_count"
                && row.direction == "exact_match_preferred"
                && row.tools_covered
                    == vec!["germline".to_string(), "ibdhap".to_string(), "ibdseq".to_string()]
        }));
    }

    #[test]
    fn comparable_metric_direction_labels_are_stable() {
        assert_eq!(
            comparable_metric_direction_label(VcfComparableMetricDirection::ExactMatchPreferred),
            "exact_match_preferred"
        );
        assert_eq!(
            comparable_metric_direction_label(VcfComparableMetricDirection::HigherIsBetter),
            "higher_is_better"
        );
        assert_eq!(
            comparable_metric_direction_label(VcfComparableMetricDirection::LowerIsBetter),
            "lower_is_better"
        );
    }
}
