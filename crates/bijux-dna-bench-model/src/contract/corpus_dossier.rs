//! Owner: bijux-dna-bench-model
//! Semantic validation for persisted corpus benchmark dossier summaries.

use std::collections::BTreeSet;

use crate::{
    validate_utc_timestamp, BenchError, BenchmarkCorpusDossierHeadline,
    BenchmarkCorpusDossierSummary, BenchmarkCorpusDossierToolSummary,
    BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
};

/// Validate the identity, coverage, metrics, and headline of one corpus dossier summary.
///
/// # Errors
/// Returns an error when the persisted summary is incomplete or internally inconsistent.
pub fn validate_benchmark_corpus_dossier_summary(
    summary: &BenchmarkCorpusDossierSummary,
) -> Result<(), BenchError> {
    if summary.schema_version != BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1 {
        return invalid("unsupported corpus dossier summary schema");
    }
    for (value, field) in [
        (&summary.corpus_id, "corpus_id"),
        (&summary.stage_id, "stage_id"),
        (&summary.scenario_id, "scenario_id"),
        (&summary.platform, "platform"),
        (&summary.corpus_root, "corpus_root"),
        (&summary.run_root, "run_root"),
    ] {
        require_text(value, field)?;
    }
    validate_utc_timestamp(&summary.generated_at_utc, "generated_at_utc")?;
    if !matches!(summary.sample_scope.as_str(), "full" | "paired") {
        return invalid("corpus dossier summary has unsupported sample_scope");
    }
    if summary.platform == "missing" {
        return invalid("corpus dossier summary is missing its platform identity");
    }
    if summary.samples_total == 0 || summary.samples_failed > summary.samples_total {
        return invalid("corpus dossier summary has invalid sample counters");
    }
    let allowed_cohorts: &[&str] = if summary.sample_scope == "paired" {
        &["ancient_pe", "modern_pe"]
    } else {
        &["ancient_pe", "ancient_se", "modern_pe", "modern_se"]
    };
    validate_count_partition(
        &summary.cohort_counts,
        summary.samples_total,
        "cohort_counts",
        allowed_cohorts,
    )?;
    validate_count_partition(
        &summary.era_counts,
        summary.samples_total,
        "era_counts",
        &["ancient", "modern"],
    )?;
    validate_count_partition(
        &summary.layout_counts,
        summary.samples_total,
        "layout_counts",
        &["pe", "se"],
    )?;
    validate_tools(summary)?;
    validate_headline(&summary.headline, &summary.tool_summary)
}

fn validate_tools(summary: &BenchmarkCorpusDossierSummary) -> Result<(), BenchError> {
    if summary.tools.is_empty()
        || summary.tools.iter().any(|tool| tool.trim().is_empty())
        || summary.tools.iter().collect::<BTreeSet<_>>().len() != summary.tools.len()
    {
        return invalid("corpus dossier summary has an invalid tool roster");
    }
    if summary.tool_summary.len() != summary.tools.len() {
        return invalid("corpus dossier tool summary does not cover the tool roster");
    }
    let mut observed = BTreeSet::new();
    for tool in &summary.tool_summary {
        validate_tool_summary(tool, summary.samples_total)?;
        if !observed.insert(tool.tool.as_str()) {
            return invalid("corpus dossier tool summary repeats a tool");
        }
    }
    if observed != summary.tools.iter().map(String::as_str).collect() {
        return invalid("corpus dossier tool summary drifts from the tool roster");
    }
    Ok(())
}

fn validate_tool_summary(
    tool: &BenchmarkCorpusDossierToolSummary,
    samples_total: usize,
) -> Result<(), BenchError> {
    require_text(&tool.tool, "tool_summary[].tool")?;
    if tool.records != samples_total {
        return invalid(format!(
            "corpus dossier tool `{}` record count does not match samples_total",
            tool.tool
        ));
    }
    validate_optional_rate(tool.pass_rate, "tool_summary[].pass_rate")?;
    for (value, field) in [
        (tool.median_runtime_s, "tool_summary[].median_runtime_s"),
        (tool.mean_runtime_s, "tool_summary[].mean_runtime_s"),
        (tool.max_runtime_s, "tool_summary[].max_runtime_s"),
    ] {
        validate_optional_nonnegative(value, field)?;
    }
    if let (Some(median), Some(maximum)) = (tool.median_runtime_s, tool.max_runtime_s) {
        if median > maximum {
            return invalid(format!(
                "corpus dossier tool `{}` median runtime exceeds maximum",
                tool.tool
            ));
        }
    }
    Ok(())
}

fn validate_headline(
    headline: &BenchmarkCorpusDossierHeadline,
    tools: &[BenchmarkCorpusDossierToolSummary],
) -> Result<(), BenchError> {
    validate_paired_headline_fields(
        headline.fastest_tool.as_deref(),
        headline.fastest_runtime_s,
        "fastest runtime",
    )?;
    validate_paired_headline_fields(
        headline.highest_pass_rate_tool.as_deref(),
        headline.highest_pass_rate,
        "highest pass rate",
    )?;
    validate_optional_nonnegative(headline.fastest_runtime_s, "headline.fastest_runtime_s")?;
    validate_optional_rate(headline.highest_pass_rate, "headline.highest_pass_rate")?;

    let expected_fastest = tools
        .iter()
        .filter_map(|tool| tool.median_runtime_s.map(|runtime| (tool.tool.as_str(), runtime)))
        .min_by(|left, right| left.1.total_cmp(&right.1));
    if expected_fastest != headline.fastest_tool.as_deref().zip(headline.fastest_runtime_s) {
        return invalid("corpus dossier fastest-tool headline contradicts tool summaries");
    }
    let expected_pass_rate = tools
        .iter()
        .filter_map(|tool| tool.pass_rate.map(|rate| (tool.tool.as_str(), rate)))
        .max_by(|left, right| left.1.total_cmp(&right.1));
    if expected_pass_rate
        != headline.highest_pass_rate_tool.as_deref().zip(headline.highest_pass_rate)
    {
        return invalid("corpus dossier pass-rate headline contradicts tool summaries");
    }
    Ok(())
}

fn validate_paired_headline_fields(
    tool: Option<&str>,
    value: Option<f64>,
    label: &str,
) -> Result<(), BenchError> {
    if tool.is_some() != value.is_some() || tool.is_some_and(|value| value.trim().is_empty()) {
        return invalid(format!("corpus dossier {label} headline is incomplete"));
    }
    Ok(())
}

fn validate_count_partition(
    counts: &std::collections::BTreeMap<String, usize>,
    total: usize,
    field: &str,
    allowed_keys: &[&str],
) -> Result<(), BenchError> {
    let observed_total = counts.values().try_fold(0usize, |sum, value| sum.checked_add(*value));
    if counts.is_empty()
        || counts.keys().any(|key| key.trim().is_empty())
        || counts.keys().any(|key| !allowed_keys.contains(&key.as_str()))
        || observed_total != Some(total)
    {
        return invalid(format!("corpus dossier {field} does not partition samples_total"));
    }
    Ok(())
}

fn validate_optional_rate(value: Option<f64>, field: &str) -> Result<(), BenchError> {
    if value.is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value)) {
        return invalid(format!("corpus dossier {field} is not a finite rate"));
    }
    Ok(())
}

fn validate_optional_nonnegative(value: Option<f64>, field: &str) -> Result<(), BenchError> {
    if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
        return invalid(format!("corpus dossier {field} is not a non-negative finite value"));
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("corpus dossier summary requires non-empty {field}"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
